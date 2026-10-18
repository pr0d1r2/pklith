//! Commits one by one (scan V6, V7): what `tdd-order` rules read, oldest
//! first, each with the paths it changed and the paths it deleted.

use super::Error;
use std::path::Path;

/// Which commits to read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Span {
    /// The commits of `A..B`.
    Range(String),
    /// The commits on no remote-tracking ref: `HEAD --not --remotes`,
    /// what a push is about to send.
    Pushed,
    /// As `Pushed`, then the index as the commit being made.
    Staged,
}

/// One commit of a history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    /// `<short id> (<subject>)`, or `the staged commit` for the index.
    pub name: String,
    /// Paths added, modified, or renamed or copied to.
    pub changed: Vec<String>,
    /// Paths deleted, or renamed from.
    pub deleted: Vec<String>,
}

/// The commits `span` covers under `root`, oldest first, merges skipped
/// (V6); renames detected, each an old path deleted and a new one changed
/// (V7).
///
/// # Errors
///
/// [`Error::Git`] when git fails, as for an unknown revision; a range
/// starting with `-`, which git would read as an option; a path that is
/// not UTF-8.
pub fn history(root: &Path, span: &Span) -> Result<Vec<Commit>, Error> {
    let revs = match span {
        Span::Range(range) if range.starts_with('-') => {
            return Err(Error::Malformed(format!("range `{range}` is not A..B")));
        }
        Span::Range(range) => vec![range.as_str()],
        Span::Pushed | Span::Staged if !born(root) => Vec::new(),
        Span::Pushed | Span::Staged => vec!["HEAD", "--not", "--remotes"],
    };
    let mut commits = listed(root, &revs)?;
    if *span == Span::Staged {
        let diff = ["diff", "--cached", "-M", "--name-status", "-z"];
        commits.push(commit("the staged commit".to_owned(), &git(root, &diff)?)?);
    }
    Ok(commits)
}

/// Whether HEAD names a commit yet.
fn born(root: &Path) -> bool {
    git(root, &["rev-parse", "--verify", "-q", "HEAD"]).is_ok()
}

/// The commits `revs` lists, oldest first, each with its paths.
fn listed(root: &Path, revs: &[&str]) -> Result<Vec<Commit>, Error> {
    if revs.is_empty() {
        return Ok(Vec::new());
    }
    let mut log = vec!["log", "--reverse", "--no-merges", "--format=%h %s"];
    log.extend(revs);
    let text = String::from_utf8_lossy(&git(root, &log)?).into_owned();
    let one = |line: &str| {
        let (id, subject) = line.split_once(' ').unwrap_or((line, ""));
        let tree = ["diff-tree", "--root", "--no-commit-id", "-r", "-M"];
        let bytes = git(root, &[&tree[..], &["--name-status", "-z", id]].concat())?;
        commit(format!("{id} ({subject})"), &bytes)
    };
    text.lines().map(one).collect()
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, Error> {
    let mut cmd = crate::proc::command("git", root);
    cmd.args(args);
    crate::proc::output(&mut cmd).map_err(Error::Git)
}

/// A commit named `name` from `git --name-status -z` output: a status
/// word, then one path, or two for a rename or copy.
fn commit(name: String, bytes: &[u8]) -> Result<Commit, Error> {
    let words = super::diff::words(bytes)?;
    let mut found = Commit {
        name,
        changed: Vec::new(),
        deleted: Vec::new(),
    };
    let mut words = words.into_iter();
    while let (Some(status), Some(path)) = (words.next(), words.next()) {
        file(&mut found, &status, path, &mut words);
    }
    Ok(found)
}

/// Record one `status path [path]` entry in `commit` (V7).
fn file(commit: &mut Commit, status: &str, path: String, rest: &mut impl Iterator<Item = String>) {
    match status.chars().next() {
        Some('R') => {
            commit.deleted.push(path);
            commit.changed.extend(rest.next());
        }
        Some('C') => commit.changed.extend(rest.next()),
        Some('D') => commit.deleted.push(path),
        _ => commit.changed.push(path),
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, Span, commit, history};
    use std::path::Path;

    /// A range git would read as an option is refused before git runs.
    #[test]
    fn a_range_like_an_option_is_refused() {
        let got = history(Path::new("."), &Span::Range("-x".to_owned()));
        assert!(matches!(got, Err(Error::Malformed(_))), "{got:?}");
    }

    type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

    /// A fresh repository with `a` staged and no commit yet.
    fn unborn() -> Result<std::path::PathBuf> {
        let dir = std::env::temp_dir().join(format!("pklith-history-{}", std::process::id()));
        std::fs::create_dir_all(&dir)?;
        std::fs::write(dir.join("a"), "")?;
        for args in [&["init", "-q"][..], &["add", "a"]] {
            crate::proc::command("git", &dir).args(args).output()?;
        }
        Ok(dir)
    }

    /// Before the first commit there is nothing pushed; staged mode still
    /// judges the index.
    #[test]
    fn an_unborn_head_has_only_the_index() -> Result {
        let dir = unborn()?;
        assert_eq!(history(&dir, &Span::Pushed)?, []);
        let staged = history(&dir, &Span::Staged)?;
        let got: Vec<(&str, &[String])> = staged
            .iter()
            .map(|c| (c.name.as_str(), &c.changed[..]))
            .collect();
        assert_eq!(got, [("the staged commit", &["a".to_owned()][..])]);
        Ok(std::fs::remove_dir_all(dir)?)
    }

    /// V7: a rename deletes its old path and changes its new one; a copy
    /// changes only its new path; a deletion changes nothing.
    #[test]
    fn statuses_split_into_changed_and_deleted() {
        let bytes = b"M\0a\0R100\0old\0new\0C75\0src\0dup\0D\0gone\0A\0added\0";
        let got = commit("x".to_owned(), bytes).ok();
        let (changed, deleted) = got.map(|c| (c.changed, c.deleted)).unwrap_or_default();
        assert_eq!(changed, ["a", "new", "dup", "added"]);
        assert_eq!(deleted, ["old", "gone"]);
    }
}
