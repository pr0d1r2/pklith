//! `pklith protect` against a fake `gh`, so no test touches the network.

mod common;

use common::{Result, repo};
use std::path::{Path, PathBuf};

const CI: &str = "on: pull_request\njobs:\n  lint:\n    runs-on: x\n  test:\n    runs-on: x\n";

/// A `gh` that logs its arguments, answers `repo view`, prints the
/// current contexts, and keeps a PATCH body.
const GH: &str = "#!/bin/sh\necho \"$*\" >> \"$PWD/gh.log\"\n[ -n \"$FAKE_GH_FAIL\" ] && { echo \"gh: $FAKE_GH_FAIL\" >&2; exit 1; }\ncase \"$*\" in\n  \"repo view\"*) echo acme/app ;;\n  *\"--jq .contexts[]\"*) printf '%b' \"$FAKE_GH_CURRENT\" ;;\n  *PATCH*) cat > \"$PWD/gh.body\" ;;\nesac\n";

fn protected(name: &str) -> Result<PathBuf> {
    use std::os::unix::fs::PermissionsExt as _;
    let dir = repo(name, None)?;
    std::fs::create_dir_all(dir.join(".github/workflows"))?;
    std::fs::write(dir.join(".github/workflows/ci.yml"), CI)?;
    std::fs::create_dir_all(dir.join("bin"))?;
    std::fs::write(dir.join("bin/gh"), GH)?;
    std::fs::set_permissions(dir.join("bin/gh"), std::fs::Permissions::from_mode(0o755))?;
    pklith::proc::command("git", &dir)
        .args(["add", ".github"])
        .output()?;
    Ok(dir)
}

/// `pklith protect` with the fake gh first on PATH and `current` as the
/// contexts GitHub requires now.
fn protect(dir: &Path, args: &[&str], current: &str) -> Result<(Option<i32>, String, String)> {
    let path = format!("{}:{}", dir.join("bin").display(), std::env::var("PATH")?);
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_pklith"));
    cmd.arg("protect").args(args).current_dir(dir);
    let out = cmd
        .env("PATH", path)
        .env("FAKE_GH_CURRENT", current)
        .output()?;
    let text = |b: Vec<u8>| String::from_utf8(b);
    Ok((out.status.code(), text(out.stdout)?, text(out.stderr)?))
}

/// Protect V1: a dry run prints the payload and calls gh not at all.
#[test]
fn a_dry_run_calls_nothing() -> Result {
    let dir = protected("protect-dry")?;
    let want = "{\"strict\":true,\"contexts\":[\"lint\",\"test\"]}\n";
    assert_eq!(
        protect(&dir, &["--dry-run"], "")?,
        (Some(0), want.to_owned(), String::new())
    );
    assert!(!dir.join("gh.log").exists());
    Ok(std::fs::remove_dir_all(dir)?)
}

/// An added context is shown and applied with the derived payload.
#[test]
fn an_addition_is_shown_and_applied() -> Result {
    let dir = protected("protect-add")?;
    assert_eq!(
        protect(&dir, &[], "lint\\n")?,
        (Some(0), String::new(), "protect: + test\n".to_owned())
    );
    let body = std::fs::read_to_string(dir.join("gh.body"))?;
    assert_eq!(body, "{\"strict\":true,\"contexts\":[\"lint\",\"test\"]}\n");
    let log = std::fs::read_to_string(dir.join("gh.log"))?;
    assert!(
        log.contains("repos/acme/app/branches/main/protection/required_status_checks"),
        "{log}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

const CURRENT: &str = "lint\\nold\\ntest\\n";

/// Protect V2: a context that would stop being required is refused, and
/// nothing is sent.
#[test]
fn a_removal_is_refused() -> Result {
    let dir = protected("protect-remove")?;
    let (code, _, stderr) = protect(&dir, &["--branch", "trunk"], CURRENT)?;
    assert_eq!(code, Some(1));
    let want = "protect: - old\npklith protect: applying would stop requiring";
    assert!(stderr.starts_with(want), "{stderr}");
    assert!(!dir.join("gh.body").exists());
    Ok(std::fs::remove_dir_all(dir)?)
}

/// An accepted removal is applied.
#[test]
fn an_accepted_removal_is_applied() -> Result {
    let dir = protected("protect-accept")?;
    let args = ["--accept-removals", "--repo", "o/n"];
    assert_eq!(protect(&dir, &args, CURRENT)?.0, Some(0));
    let log = std::fs::read_to_string(dir.join("gh.log"))?;
    assert!(log.contains("repos/o/n/branches/main/"), "{log}");
    assert!(dir.join("gh.body").exists());
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A flag protect does not know is a usage error.
#[test]
fn an_unknown_flag_is_a_usage_error() -> Result {
    let dir = protected("protect-usage")?;
    assert_eq!(protect(&dir, &["--bogus"], "")?.0, Some(2));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Outside a repository protect exits 2 naming the cause.
#[test]
fn protect_outside_a_repository_exits_2() -> Result {
    let outside = common::temp("protect-outside")?;
    let (code, _, stderr) = protect(&outside, &["--dry-run"], "")?;
    assert_eq!(
        (code, stderr.as_str()),
        (
            Some(2),
            "pklith protect: not inside a git repository; pass --root DIR\n"
        )
    );
    Ok(std::fs::remove_dir_all(outside)?)
}

/// When gh fails, protect exits 2 with gh's own words.
#[test]
fn a_failing_gh_is_named() -> Result {
    let dir = protected("protect-gh-fails")?;
    let path = format!("{}:{}", dir.join("bin").display(), std::env::var("PATH")?);
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_pklith"));
    cmd.arg("protect").current_dir(&dir).env("PATH", path);
    let out = cmd.env("FAKE_GH_FAIL", "boom").output()?;
    let stderr = String::from_utf8(out.stderr)?;
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr.starts_with("pklith protect: `gh repo view") && stderr.contains("gh: boom"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A tracked workflow that cannot be read stops protect: exit 2.
#[test]
fn an_unreadable_workflow_is_an_error() -> Result {
    let dir = protected("protect-unreadable")?;
    std::fs::remove_file(dir.join(".github/workflows/ci.yml"))?;
    let (code, _, stderr) = protect(&dir, &["--dry-run"], "")?;
    assert_eq!(code, Some(2));
    assert!(
        stderr.starts_with("pklith protect: cannot read a workflow: "),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}
