//! `pklith-dev commit-msg [FILE]` and `pklith-dev changelog [FILE]`: the
//! commit-msg hook's two rules. The hook passes the message file git gave
//! it (`MERGE_MSG` on a merge); by hand it defaults to `COMMIT_EDITMSG`.

use crate::changelog::{self, Sections};
use crate::io::{Failed, message, run, show};

/// Product paths are staged paths under `src/`, added, changed, renamed or
/// deleted.
const STAGED: [&str; 6] = [
    "diff",
    "--cached",
    "--name-only",
    "--diff-filter=ACMRD",
    "--",
    "src",
];

/// The commit-message style rule.
///
/// # Errors
///
/// The rule broken (1), or no message to judge (2).
pub fn commit_msg(file: Option<&str>) -> Result<(), Failed> {
    crate::message::check(&message(file)?).map_err(|e| (1, e))
}

/// The changelog rule.
///
/// # Errors
///
/// The rule broken (1), or git or the message failing (2).
pub fn changelog(file: Option<&str>) -> Result<(), Failed> {
    let text = message(file)?;
    let subject = crate::message::subject(&text);
    let staged = run("git", &STAGED)?;
    let product = changelog::product(&staged);
    changelog::check(subject, &product, &sections()?).map_err(|e| (1, e))
}

/// The `[Unreleased]` section staged, at HEAD and at its parent, and
/// HEAD's subject; each empty where git has none (a first commit).
fn sections() -> Result<Sections, Failed> {
    let at = |rev: &str| show(&format!("{rev}:CHANGELOG.md")).map(|t| changelog::unreleased(&t));
    let head_subject = run("git", &["log", "-1", "--format=%s"]).unwrap_or_default();
    Ok(Sections {
        staged: at("")?,
        head: at("HEAD")?,
        parent: at("HEAD~1")?,
        head_subject: head_subject.trim_end().to_owned(),
    })
}
