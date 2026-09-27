//! Lay the plan: one commit per check, through the repository's real hooks
//! (lay V2, root V12), undone as a whole when any step fails (V3).

use crate::catalog::Check;
use crate::proc::{command, output};
use std::path::Path;

/// What lay needs besides the plan: the repository, every check the
/// registry uses in gen order, and the steps hk already runs.
pub struct Context<'a> {
    /// Repository root.
    pub root: &'a Path,
    /// `gen::used`: what the registry uses, in emit order.
    pub used: Vec<&'a Check>,
    /// Step ids hk already runs.
    pub present: Vec<String>,
}

/// Why lay stopped.
#[derive(Debug, PartialEq, Eq)]
pub enum Failure {
    /// It could not start, and wrote nothing (V1, V6).
    NotReady(String),
    /// A step failed; the repository is back where the run started (V3).
    RolledBack(String),
}

/// Lay every planned check; one `<sha> <subject>` line per commit (V9).
///
/// # Errors
///
/// [`Failure::NotReady`] when hk or the hooks are missing (V1) or the file
/// is already staged (V6); [`Failure::RolledBack`] when a step fails, after
/// the repository is back at the HEAD it started from (V3).
pub fn lay(ctx: &Context, plan: &[&Check]) -> Result<String, Failure> {
    preconditions(ctx.root).map_err(Failure::NotReady)?;
    let start = git(ctx.root, &["rev-parse", "HEAD"]).map_err(Failure::NotReady)?;
    let before = crate::r#gen::FILES.map(|p| std::fs::read_to_string(ctx.root.join(p)).ok());
    lay_all(ctx, plan).map_err(|e| Failure::RolledBack(rollback(ctx.root, &start, &before, &e)))
}

fn lay_all(ctx: &Context, plan: &[&Check]) -> Result<String, String> {
    let mut laid: Vec<String> = Vec::new();
    let mut out = String::new();
    for check in plan {
        laid.push(check.id.clone());
        out.push_str(&lay_one(ctx, check, &laid)?);
    }
    Ok(out)
}

/// V1: hk on PATH and hooks installed; V6: no generated file staged.
fn preconditions(root: &Path) -> Result<(), String> {
    output(command("hk", root).arg("--version"))
        .map_err(|e| format!("hk is not usable, so no hook would run: {e}"))?;
    let hooks = git(root, &["rev-parse", "--git-path", "hooks/pre-commit"])?;
    let configured = git(root, &["config", "core.hooksPath"]).is_ok();
    if !configured && !root.join(&hooks).is_file() {
        return Err("no git hooks are installed, so nothing would check the commits".into());
    }
    let staged = git(root, &["diff", "--cached", "--name-only"])?;
    match staged.lines().find(|l| crate::r#gen::FILES.contains(l)) {
        Some(file) => Err(format!(
            "{file} is already staged; commit or unstage it first"
        )),
        None => Ok(()),
    }
}

/// Write the generated module with every check laid so far, prove hk now
/// runs the new step, and commit that file alone (V8).
fn lay_one(ctx: &Context, check: &Check, laid: &[String]) -> Result<String, String> {
    write(ctx, laid)?;
    seen(ctx.root, check)?;
    commit(ctx.root, check)
}

/// Commit only the generated file (V8), through the hooks (root V12).
fn commit(root: &Path, check: &Check) -> Result<String, String> {
    let (subject, body) = (super::subject(check), body(check));
    let files = crate::r#gen::FILES;
    git(root, &[&["add", "--"][..], &files].concat())?;
    let args = [
        &["commit", "-q", "-m", &subject, "-m", &body, "--"][..],
        &files,
    ]
    .concat();
    git(root, &args)?;
    let sha = git(root, &["rev-parse", "--short", "HEAD"])?;
    Ok(format!("{sha} {subject}\n"))
}

fn write(ctx: &Context, laid: &[String]) -> Result<(), String> {
    let keep = |c: &&&Check| ctx.present.contains(&c.id) || laid.contains(&c.id);
    let steps: Vec<&Check> = ctx.used.iter().filter(keep).copied().collect();
    crate::r#gen::write(ctx.root, &steps).map(drop)
}

/// The laid step must reach hk: if hk.pkl does not import the generated
/// module, laying would commit a file nothing runs.
fn seen(root: &Path, check: &Check) -> Result<(), String> {
    if crate::hook::steps(root).is_ok_and(|steps| steps.iter().any(|s| s.id == check.id)) {
        return Ok(());
    }
    let (id, file) = (&check.id, crate::r#gen::FILE);
    Err(format!(
        "hk does not see `{id}` after it was written; does hk.pkl import {file}, and does it still evaluate?"
    ))
}

/// The commit body: what the check guards, where it came from, and the
/// planted-violation proof V32 asks for.
fn body(check: &Check) -> String {
    format!(
        "{}: {}\n\nLaid by `pkli lay` from .pklith: one check, one commit, through the\nrepository's own hooks.\n\n{}",
        check.id,
        check.msg,
        super::proof(check)
    )
}

/// V3: back to the HEAD the run started from, keeping the operator's work
/// (`--mixed`, never `--hard`), and each generated file as it was.
fn rollback(root: &Path, start: &str, before: &[Option<String>], why: &str) -> String {
    let reset = git(root, &["reset", "-q", "--mixed", start]).err();
    let restored = crate::r#gen::FILES
        .iter()
        .zip(before)
        .filter_map(|(p, b)| restore(root, p, b.as_deref()));
    let trouble: Vec<String> = reset.into_iter().chain(restored).collect();
    let tail = if trouble.is_empty() {
        "rolled back to where it started".to_owned()
    } else {
        format!("rollback incomplete: {}", trouble.join("; "))
    };
    format!("{why}; {tail}")
}

/// Put one generated file back; one lay created is removed. The problem,
/// if any.
fn restore(root: &Path, path: &str, before: Option<&str>) -> Option<String> {
    let file = root.join(path);
    let done = match before {
        Some(text) => std::fs::write(&file, text),
        None if file.exists() => std::fs::remove_file(&file),
        None => Ok(()),
    };
    done.err().map(|e| format!("{path}: {e}"))
}

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = output(command("git", root).args(args)).map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&out).trim_end().to_owned())
}
