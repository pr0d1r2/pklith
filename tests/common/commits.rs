//! Throwaway repositories with commits, for the `tdd-order` tests (rule
//! V6-V8): a fixture rule, git with a throwaway identity, and the check
//! over the commits since the `base` tag.

use super::{Result, pklith, repo};
use std::path::{Path, PathBuf};

/// Every `sub/*.rs` changes its `sub/<stem>_test.rs` first or with it;
/// tests and `sub/skip.rs` are excepted.
pub const RULE: &str = "## rules\nid|kind|select|target|except\ntdd|tdd-order|sub/*.rs|sub/{stem}_test.rs|sub/*_test.rs, sub/skip.rs\n";

/// git in `dir` with a throwaway identity; its stdout, trimmed.
pub fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let who = ["-c", "user.name=t", "-c", "user.email=t@t"];
    let out = pklith::proc::command("git", dir)
        .args(who)
        .args(args)
        .output()?;
    assert!(out.status.success(), "git {args:?}: {out:?}");
    Ok(String::from_utf8(out.stdout)?.trim_end().to_owned())
}

/// Write `files` (path, text) and commit them as `subject`; the short id.
pub fn commit(dir: &Path, files: &[(&str, &str)], subject: &str) -> Result<String> {
    for (path, text) in files {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().ok_or("no parent")?)?;
        std::fs::write(path, text)?;
    }
    git(dir, &["add", "-A"])?;
    git(dir, &["commit", "-qm", subject])?;
    git(dir, &["rev-parse", "--short", "HEAD"])
}

/// A repository under `registry` with its first commit made: the base
/// every range starts from, so `sub/b.rs` without a test is history.
pub fn based(name: &str, registry: &str) -> Result<PathBuf> {
    let dir = repo(name, Some(registry))?;
    git(&dir, &["commit", "-qm", "base"])?;
    git(&dir, &["tag", "base"])?;
    Ok(dir)
}

pub fn range(dir: &Path) -> Result<(Option<i32>, String)> {
    let (code, stdout, stderr) = pklith(dir, &["check", "--range", "base..HEAD"])?;
    assert_eq!(stdout, "", "stdout carries data only");
    Ok((code, stderr))
}

pub fn want(id: &str, source: &str, target: &str, commit: &str) -> String {
    format!(
        "rule: `{id}` wants {target} changed in or before commit {commit} for {source}; add it, or except the file in the rule\n"
    )
}

/// git with one raw (not UTF-8) argument after `args`.
pub fn git_raw(dir: &Path, args: &[&str], raw: &[u8]) -> Result {
    use std::os::unix::ffi::OsStrExt;
    let out = pklith::proc::command("git", dir)
        .args(args)
        .arg(std::ffi::OsStr::from_bytes(raw))
        .output()?;
    assert!(out.status.success(), "git {args:?}: {out:?}");
    Ok(())
}
