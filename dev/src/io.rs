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
    let out = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| (2, format!("cannot run {program}: {e}")))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let why = format!("{program} {} failed: {}", args.join(" "), err.trim());
        return Err((2, why));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
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
