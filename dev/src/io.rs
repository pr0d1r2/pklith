//! The only code that touches the repository: reading files, running
//! `git`, `pkl` and `cargo`, and writing or checking a generated file.

use std::process::Command;

/// An exit code and what went wrong.
pub type Failed = (u8, String);

/// A file's text; unreadable is exit 2.
///
/// # Errors
///
/// The path and the I/O error.
pub fn read(path: &str) -> Result<String, Failed> {
    std::fs::read_to_string(path).map_err(|e| (2, format!("cannot read {path}: {e}")))
}

/// Run `program` and return its stdout; any failure is exit 2.
///
/// # Errors
///
/// A program that cannot start, or exits non-zero, with its stderr.
pub fn run(program: &str, args: &[&str]) -> Result<String, Failed> {
    let out = spawn(program, args)?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let why = format!("{program} {} failed: {}", args.join(" "), err.trim());
        return Err((2, why));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Run `program` to completion; one that cannot start is exit 2.
fn spawn(program: &str, args: &[&str]) -> Result<std::process::Output, Failed> {
    Command::new(program)
        .args(args)
        .output()
        .map_err(|e| (2, format!("cannot run {program}: {e}")))
}

/// Make `path` hold `wanted`: written, or with `check` compared, drift
/// being exit 1 naming the verb that fixes it (dev V6: equality, never a
/// heuristic).
///
/// # Errors
///
/// Drift under `check`, or a write that fails.
pub fn update(path: &str, wanted: &str, check: bool, verb: &str) -> Result<(), Failed> {
    if read(path)? == wanted {
        return Ok(());
    }
    if check {
        let fix =
            format!("{path} drifted from its sources; run `cargo run -p pklith-dev -- {verb}`");
        return Err((1, fix));
    }
    std::fs::write(path, wanted).map_err(|e| (2, format!("cannot write {path}: {e}")))
}

/// Steps in one hook, as `pkl` evaluates hk.pkl.
///
/// # Errors
///
/// `pkl` failing, or printing something other than a number.
pub fn steps(hook: &str) -> Result<usize, Failed> {
    let expr = format!("hooks[\"{hook}\"].steps.length");
    let text = run("pkl", &["eval", "-x", &expr, "hk.pkl"])?;
    text.trim()
        .parse()
        .map_err(|_| (2, format!("pkl printed `{}` for {expr}", text.trim())))
}

/// Steps on commit and on push.
///
/// # Errors
///
/// As [`steps`].
pub fn hooks() -> Result<(usize, usize), Failed> {
    Ok((steps("pre-commit")?, steps("pre-push")?))
}

/// The commit message: the file named, or git's `COMMIT_EDITMSG`. A
/// missing file means nothing can be checked, which fails.
///
/// # Errors
///
/// No message file, or git failing to name one.
pub fn message(file: Option<&str>) -> Result<String, Failed> {
    let path = match file {
        Some(f) => f.to_owned(),
        None => run("git", &["rev-parse", "--git-path", "COMMIT_EDITMSG"])?
            .trim()
            .to_owned(),
    };
    std::fs::read_to_string(&path).map_err(|_| {
        (
            2,
            format!("no message file at {path}, so nothing was checked"),
        )
    })
}

/// `git show REV:PATH`; a revision or path git does not have reads as
/// empty (no HEAD yet, a file not staged).
///
/// # Errors
///
/// git failing to start.
pub fn show(object: &str) -> Result<String, Failed> {
    let out = spawn("git", &["show", object])?;
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    Ok(if out.status.success() {
        text
    } else {
        String::new()
    })
}
