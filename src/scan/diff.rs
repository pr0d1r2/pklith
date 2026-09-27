//! Changed paths, from git (root §I `--staged` and `--range`): what a
//! commit or a range touches, deleted paths included.

use super::Error;
use std::path::Path;

/// Which change to read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diff {
    /// What the index holds against HEAD (all of it before the first
    /// commit).
    Staged,
    /// Between two revisions, `A..B`.
    Range(String),
}

/// Paths `diff` adds, modifies or deletes under `root`, sorted and unique.
/// A rename counts as a deletion and an addition, so both rows are judged.
///
/// # Errors
///
/// [`Error::Git`] when git fails, as for an unknown revision, and a range
/// starting with `-`, which git would read as an option.
pub fn changed(root: &Path, diff: &Diff) -> Result<Vec<String>, Error> {
    let mut cmd = crate::proc::command("git", root);
    cmd.args(["diff", "--name-only", "--no-renames", "-z"]);
    match diff {
        Diff::Staged => cmd.arg("--cached"),
        Diff::Range(range) if range.starts_with('-') => {
            return Err(Error::Malformed(format!("range `{range}` is not A..B")));
        }
        Diff::Range(range) => cmd.arg(range),
    };
    paths(&crate::proc::output(&mut cmd).map_err(Error::Git)?)
}

/// NUL-separated paths, sorted and unique; one that is not UTF-8 is named
/// lossily (scan V4).
fn paths(bytes: &[u8]) -> Result<Vec<String>, Error> {
    let utf8 = |p: &[u8]| {
        String::from_utf8(p.to_vec())
            .map_err(|_| Error::NotUtf8(String::from_utf8_lossy(p).into_owned()))
    };
    let mut paths = bytes
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .map(utf8)
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    paths.dedup();
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::{Error, paths};

    /// NUL-separated paths come back sorted and unique; one that is not
    /// UTF-8 is an error naming it (scan V4).
    #[test]
    fn paths_are_sorted_unique_and_utf8() {
        assert_eq!(
            paths(b"b\0a\0b\0").ok(),
            Some(vec!["a".to_owned(), "b".to_owned()])
        );
        let bad = paths(b"ok\0\xffx\0").err().map(|e| e.to_string());
        assert_eq!(bad.as_deref(), Some("path is not UTF-8: \u{fffd}x"));
        assert!(matches!(paths(b"\xff"), Err(Error::NotUtf8(_))));
    }
}
