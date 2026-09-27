//! Enumerate repository files and classify each basename to exactly one file
//! type (`src/scan/SPEC.md`).

use std::path::Path;

mod diff;
mod glob;
pub use diff::{Diff, changed};
pub use glob::Globs;

/// How a candidate key was derived from a basename (scan §C).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Whole basename without a dot: `justfile`, `Makefile`.
    Bare,
    /// A leading dot and no other: `.envrc`.
    Dotfile,
    /// Whole basename containing dots: `Cargo.lock`.
    Name,
    /// Multi-part suffix: `tar.gz`.
    Compound,
    /// Last suffix: `gz`.
    Ext,
}

/// One way a file could be typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// Registry key, with one leading `.` dropped.
    pub key: String,
    /// How the key was derived.
    pub kind: Kind,
}

/// Candidate types for a path, most specific first (scan V1): the whole
/// basename, then compound suffixes longest first, then the last extension.
///
/// ```
/// let keys: Vec<_> = pklith::scan::candidates("a/foo.tar.gz")
///     .into_iter()
///     .map(|c| c.key)
///     .collect();
/// assert_eq!(keys, ["foo.tar.gz", "tar.gz", "gz"]);
/// ```
#[must_use]
pub fn candidates(path: &str) -> Vec<Candidate> {
    let base = path.rsplit('/').next().unwrap_or(path);
    let name = base.strip_prefix('.').unwrap_or(base);
    let whole = Candidate {
        key: name.to_owned(),
        kind: whole_kind(base, name),
    };
    let parts = suffixes(name).map(|s| Candidate {
        key: s.to_owned(),
        kind: suffix_kind(s),
    });
    std::iter::once(whole).chain(parts).collect()
}

/// The type a path has when no registry says otherwise: its last candidate,
/// the extension or, without one, the whole basename. The key a gap is
/// reported under, and the one `pklith seed --init` writes a row for.
///
/// ```
/// assert_eq!(pklith::scan::key("a/foo.tar.gz"), "gz");
/// assert_eq!(pklith::scan::key("justfile"), "justfile");
/// ```
#[must_use]
pub fn key(path: &str) -> String {
    candidates(path).pop().map(|c| c.key).unwrap_or_default()
}

fn whole_kind(base: &str, name: &str) -> Kind {
    match (name.contains('.'), base.starts_with('.')) {
        (true, _) => Kind::Name,
        (false, true) => Kind::Dotfile,
        (false, false) => Kind::Bare,
    }
}

fn suffix_kind(suffix: &str) -> Kind {
    if suffix.contains('.') {
        Kind::Compound
    } else {
        Kind::Ext
    }
}

/// Every non-empty suffix after a dot, longest first.
fn suffixes(name: &str) -> impl Iterator<Item = &str> {
    name.match_indices('.')
        .filter_map(move |(i, _)| name.get(i + 1..))
        .filter(|s| !s.is_empty())
}

/// Why a file list could not be produced.
#[derive(Debug)]
pub enum Error {
    /// git itself failed.
    Git(crate::proc::Error),
    /// A path is not UTF-8 (V4); shown lossily so it can be found.
    NotUtf8(String),
    /// A `git ls-files --stage` record without a tab.
    Malformed(String),
    /// A directory could not be read.
    Io(String, std::io::Error),
    /// A glob that does not compile.
    Glob(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Git(e) => write!(f, "{e}"),
            Self::NotUtf8(p) => write!(f, "path is not UTF-8: {p}"),
            Self::Malformed(r) => write!(f, "unexpected git ls-files record: {r}"),
            Self::Io(p, e) => write!(f, "{p}: {e}"),
            Self::Glob(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}

/// Git modes that are not files: a submodule (V3).
const GITLINK: &str = "160000";
/// A symlink; it counts as a file unless it points at a directory (V3).
const SYMLINK: &str = "120000";

/// Tracked and staged paths under `root`, sorted and unique (V5), without
/// submodules or symlinks to directories (V3).
///
/// # Errors
///
/// [`Error::Git`] when git fails, [`Error::NotUtf8`] for a path that is not
/// UTF-8 (V4), [`Error::Malformed`] for output git should never produce.
pub fn tracked(root: &Path) -> Result<Vec<String>, Error> {
    let mut cmd = crate::proc::command("git", root);
    cmd.args(["ls-files", "-z", "--stage"]);
    let bytes = crate::proc::output(&mut cmd).map_err(Error::Git)?;
    let mut paths: Vec<String> = parse_stage(&bytes)?
        .into_iter()
        .filter(|(mode, path)| is_file(root, mode, path))
        .map(|(_, path)| path)
        .collect();
    paths.sort();
    paths.dedup();
    Ok(paths)
}

/// Files under `root` found by walking the filesystem (`--root`), sorted,
/// skipping `.git/` and symlinks to directories (V3). For trees that are
/// not git repositories.
///
/// # Errors
///
/// [`Error::Io`] when a directory cannot be read, [`Error::NotUtf8`] for a
/// path that is not UTF-8 (V4).
pub fn walked(root: &Path) -> Result<Vec<String>, Error> {
    let mut out = Vec::new();
    walk(root, root, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), Error> {
    let failed = |e| io(dir, e);
    for entry in std::fs::read_dir(dir).map_err(failed)? {
        let path = entry.map_err(failed)?.path();
        if path.file_name() == Some(".git".as_ref()) {
            continue;
        }
        if !path.is_dir() {
            out.push(relative(root, &path)?);
        } else if !path.is_symlink() {
            walk(root, &path, out)?;
        }
    }
    Ok(())
}

fn relative(root: &Path, path: &Path) -> Result<String, Error> {
    let rel = path.strip_prefix(root).unwrap_or(path);
    utf8(rel.as_os_str().as_encoded_bytes()).map(str::to_owned)
}

/// The one UTF-8 gate for both sources (V4): the error shows the path lossily.
fn utf8(bytes: &[u8]) -> Result<&str, Error> {
    std::str::from_utf8(bytes)
        .map_err(|_| Error::NotUtf8(String::from_utf8_lossy(bytes).into_owned()))
}

fn io(path: &Path, source: std::io::Error) -> Error {
    Error::Io(path.display().to_string(), source)
}

fn is_file(root: &Path, mode: &str, path: &str) -> bool {
    match mode {
        GITLINK => false,
        SYMLINK => !root.join(path).is_dir(),
        _ => true,
    }
}

/// `(mode, path)` from NUL-separated `<mode> <object> <stage>\t<path>` records.
fn parse_stage(bytes: &[u8]) -> Result<Vec<(String, String)>, Error> {
    bytes
        .split(|b| *b == 0)
        .filter(|r| !r.is_empty())
        .map(parse_record)
        .collect()
}

fn parse_record(record: &[u8]) -> Result<(String, String), Error> {
    let lossy = || String::from_utf8_lossy(record).into_owned();
    let tab = record
        .iter()
        .position(|b| *b == b'\t')
        .ok_or_else(|| Error::Malformed(lossy()))?;
    let (meta, path) = record.split_at(tab);
    let path = utf8(path.get(1..).unwrap_or_default())?;
    let mode = String::from_utf8_lossy(meta)
        .split(' ')
        .next()
        .unwrap_or_default()
        .to_owned();
    Ok((mode, path.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::{Kind, candidates, parse_stage, tracked, walked};
    use std::path::{Path, PathBuf};

    fn temp(name: &str) -> Result<PathBuf, std::io::Error> {
        let dir = std::env::temp_dir().join(format!("pklith-scan-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    fn git(dir: &Path, args: &[&str]) -> Result<Vec<u8>, crate::proc::Error> {
        crate::proc::output(crate::proc::command("git", dir).args(args))
    }

    /// V3 and V5 on a real index: staged files count, a submodule and a
    /// symlink to a directory do not, a symlink to a file does.
    #[test]
    fn tracked_lists_files_only_sorted() -> Result<(), Box<dyn std::error::Error>> {
        let dir = fixture("tracked")?;
        assert_eq!(tracked(&dir)?, ["a.txt", "sub/b.rs", "to-file"]);
        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }

    /// A submodule entry needs no real submodule: an index entry with mode 160000.
    const GITLINK: &str = "160000,0123456789abcdef0123456789abcdef01234567,vendor/mod";

    fn fixture(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let dir = temp(name)?;
        git(&dir, &["init", "-q"])?;
        std::fs::create_dir_all(dir.join("sub"))?;
        std::fs::write(dir.join("sub/b.rs"), "")?;
        std::fs::write(dir.join("a.txt"), "")?;
        std::os::unix::fs::symlink("sub", dir.join("to-dir"))?;
        std::os::unix::fs::symlink("a.txt", dir.join("to-file"))?;
        git(&dir, &["add", "-A"])?;
        git(&dir, &["update-index", "--add", "--cacheinfo", GITLINK])?;
        Ok(dir)
    }

    /// The walk skips `.git/` and agrees with the index on the same tree
    /// (the gitlink is only in the index, so both leave it out).
    #[test]
    fn walking_agrees_with_the_index() -> Result<(), Box<dyn std::error::Error>> {
        let dir = fixture("agrees")?;
        assert_eq!(walked(&dir)?, tracked(&dir)?);
        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }

    /// T3: an empty repository is a valid repository with no files, not an
    /// error, from either source.
    #[test]
    fn an_empty_repository_has_no_files() -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp("empty")?;
        git(&dir, &["init", "-q"])?;
        assert_eq!((tracked(&dir)?, walked(&dir)?), (vec![], vec![]));
        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }

    /// T3: NUL-separated records keep a newline inside a name intact, where
    /// line-based `git ls-files` would split it into two bogus paths.
    #[test]
    fn a_newline_in_a_name_is_one_path() -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp("newline")?;
        git(&dir, &["init", "-q"])?;
        std::fs::write(dir.join("two\nlines.txt"), "")?;
        git(&dir, &["add", "-A"])?;
        assert_eq!(tracked(&dir)?, ["two\nlines.txt"]);
        assert_eq!(
            candidates("two\nlines.txt").last().map(|c| c.key.clone()),
            Some("txt".to_owned())
        );
        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn walking_a_missing_directory_names_it() {
        let err = walked(Path::new("/pklith-no-such-dir"))
            .err()
            .map(|e| e.to_string());
        assert!(err.is_some_and(|e| e.starts_with("/pklith-no-such-dir: ")));
    }

    #[test]
    fn outside_a_repository_git_fails_by_name() -> Result<(), std::io::Error> {
        let dir = temp("no-repo")?;
        let err = tracked(&dir).err().map(|e| e.to_string());
        assert!(err.is_some_and(|e| e.starts_with("`git ls-files -z --stage` failed:")));
        std::fs::remove_dir_all(&dir)
    }

    /// V4: a path that is not UTF-8 is an error naming it, never skipped.
    #[test]
    fn a_non_utf8_path_is_an_error_naming_it() {
        let err = parse_stage(b"100644 abc 0\tok.txt\x00100644 abc 0\tbad\xff.txt\0")
            .err()
            .map(|e| e.to_string());
        assert_eq!(err.as_deref(), Some("path is not UTF-8: bad\u{fffd}.txt"));
    }

    #[test]
    fn a_record_without_a_tab_is_malformed() {
        let err = parse_stage(b"100644 abc 0 no-tab\0")
            .err()
            .map(|e| e.to_string());
        assert_eq!(
            err.as_deref(),
            Some("unexpected git ls-files record: 100644 abc 0 no-tab")
        );
    }

    fn typed(path: &str) -> Vec<(String, Kind)> {
        candidates(path)
            .into_iter()
            .map(|c| (c.key, c.kind))
            .collect()
    }

    fn pair(key: &str, kind: Kind) -> (String, Kind) {
        (key.to_owned(), kind)
    }

    #[test]
    fn every_kind_is_recorded_most_specific_first() {
        assert_eq!(typed("justfile"), [pair("justfile", Kind::Bare)]);
        assert_eq!(typed(".envrc"), [pair("envrc", Kind::Dotfile)]);
        assert_eq!(
            typed("Cargo.lock"),
            [pair("Cargo.lock", Kind::Name), pair("lock", Kind::Ext)]
        );
        let spec = [
            pair("foo.spec.ts", Kind::Name),
            pair("spec.ts", Kind::Compound),
            pair("ts", Kind::Ext),
        ];
        assert_eq!(typed("web/foo.spec.ts"), spec);
    }

    #[test]
    fn a_dotted_dotfile_drops_one_dot_and_keeps_its_suffix() {
        assert_eq!(
            typed(".typos.toml"),
            [pair("typos.toml", Kind::Name), pair("toml", Kind::Ext)]
        );
    }

    #[test]
    fn a_trailing_dot_adds_no_empty_suffix() {
        assert_eq!(typed("notes."), [pair("notes.", Kind::Name)]);
    }

    /// V2: the least specific candidate is what the legacy tools keyed on,
    /// `sed 's/.*\.//'` over the basename (R1, R2).
    #[test]
    fn the_last_candidate_is_the_legacy_key() {
        let legacy = [
            ("x.tar.gz", "gz"),
            (".envrc", "envrc"),
            ("justfile", "justfile"),
            ("a/b/Cargo.lock", "lock"),
            (".typos.toml", "toml"),
        ];
        for (path, key) in legacy {
            assert_eq!(
                candidates(path).last().map(|c| c.key.as_str()),
                Some(key),
                "{path}"
            );
        }
    }
}
