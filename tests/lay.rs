//! `pkli lay`, run against the built binary.

mod common;

use common::{HK, OK, Result, pkli, repo};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const PLAN: &str = "ci(lint): add the lint check\n";

/// lay V4: `--dry-run` prints the commits lay would make and writes
/// nothing. With `lint` already an hk step, there is nothing to lay.
#[test]
fn dry_run_lists_only_missing_checks() -> Result {
    let dir = repo("lay-plan", Some(OK))?;
    assert_eq!(
        pkli(&dir, &["lay", "--dry-run"])?,
        (Some(0), String::new(), String::new())
    );
    std::fs::write(dir.join("hk.pkl"), HK.replace("\"lint\"", "\"other\""))?;
    assert_eq!(
        pkli(&dir, &["lay", "--dry-run"])?,
        (Some(0), PLAN.to_owned(), String::new())
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Without hk's steps the plan cannot be made: exit 2.
#[test]
fn dry_run_without_hk_config_exits_2() -> Result {
    let dir = repo("lay-no-hk", Some(OK))?;
    std::fs::remove_file(dir.join("hk.pkl"))?;
    let (code, _, stderr) = pkli(&dir, &["lay", "--dry-run"])?;
    assert_eq!(code, Some(2));
    assert!(
        stderr.starts_with("pkli lay: cannot read hk's steps: "),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// hk config that runs whatever `pkli gen` or `pkli lay` generated.
const IMPORTING: &str = "amends \"pkl/Config.pkl\"\nimport \"hk.pklith.pkl\" as generated\nhooks { [\"check\"] { steps = generated.steps } }\n";

/// A generated module before anything is laid.
const EMPTY: &str = "import \"pkl/Config.pkl\"\n\nsteps: Mapping<String, Config.Step> = new {\n}\n\npush: Mapping<String, Config.Step> = new {\n}\n";

const WHO: [(&str, &str); 4] = [
    ("GIT_AUTHOR_NAME", "t"),
    ("GIT_AUTHOR_EMAIL", "t@t"),
    ("GIT_COMMITTER_NAME", "t"),
    ("GIT_COMMITTER_EMAIL", "t@t"),
];

/// `pkli` with a git identity, since lay commits.
fn pkli_as(dir: &Path, args: &[&str]) -> Result<(Option<i32>, String, String)> {
    let out = Command::new(env!("CARGO_BIN_EXE_pkli"))
        .args(args)
        .envs(WHO)
        .current_dir(dir)
        .output()?;
    Ok((
        out.status.code(),
        String::from_utf8(out.stdout)?,
        String::from_utf8(out.stderr)?,
    ))
}

fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let out = pklith::proc::command("git", dir)
        .args(args)
        .envs(WHO)
        .output()?;
    Ok(String::from_utf8(out.stdout)?.trim_end().to_owned())
}

/// A pre-commit hook that exits `code`, installed through core.hooksPath.
fn install_hook(dir: &Path, code: u8) -> Result {
    let hook = dir.join("hooks/pre-commit");
    std::fs::create_dir_all(dir.join("hooks"))?;
    std::fs::write(&hook, format!("#!/bin/sh\nexit {code}\n"))?;
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755))?;
    git(dir, &["config", "core.hooksPath", "hooks"]).map(|_| ())
}

/// A committed repository whose hk.pkl imports the generated module, with
/// a pre-commit hook that exits `hook_exit`.
fn laying_repo(name: &str, hook_exit: u8) -> Result<PathBuf> {
    let dir = repo(name, Some(OK))?;
    std::fs::write(dir.join("hk.pkl"), IMPORTING)?;
    std::fs::write(dir.join("hk.pklith.pkl"), EMPTY)?;
    install_hook(&dir, hook_exit)?;
    git(&dir, &["add", "-A"])?;
    // The fixture commit skips the hook it installs: that hook exists to
    // refuse lay's commits in the rollback test, not this one.
    git(&dir, &["commit", "-q", "--no-verify", "-m", "fixture"])?;
    Ok(dir)
}

/// lay V2, V9: one commit per check, through the hook, reported as
/// `<sha> <subject>`; the check is now an hk step. V5: a second run lays
/// nothing.
#[test]
fn lay_commits_each_missing_check_once() -> Result {
    let dir = laying_repo("lay-ok", 0)?;
    let (code, stdout, stderr) = pkli_as(&dir, &["lay"])?;
    let sha = git(&dir, &["rev-parse", "--short", "HEAD"])?;
    assert_eq!(
        (code, stdout, stderr),
        (Some(0), format!("{sha} {PLAN}"), String::new())
    );
    assert_eq!(git(&dir, &["log", "-1", "--format=%s"])?, PLAN.trim_end());
    assert!(std::fs::read_to_string(dir.join("hk.pklith.pkl"))?.contains("[\"lint\"]"));
    assert_eq!(
        pkli_as(&dir, &["lay"])?,
        (Some(0), String::new(), String::new())
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// lay V3: a hook that refuses the commit rolls everything back: HEAD and
/// the generated file are as they were, exit 1.
#[test]
fn a_refused_commit_rolls_everything_back() -> Result {
    let dir = laying_repo("lay-refused", 1)?;
    let head = git(&dir, &["rev-parse", "HEAD"])?;
    let (code, _, stderr) = pkli_as(&dir, &["lay"])?;
    assert_eq!(code, Some(1));
    assert!(
        stderr.ends_with("; rolled back to where it started\n"),
        "{stderr}"
    );
    assert_eq!(git(&dir, &["rev-parse", "HEAD"])?, head);
    assert_eq!(std::fs::read_to_string(dir.join("hk.pklith.pkl"))?, EMPTY);
    Ok(std::fs::remove_dir_all(dir)?)
}

/// lay V6: with the generated file already staged, lay does not start:
/// exit 2, nothing written.
#[test]
fn lay_refuses_to_start_with_the_file_staged() -> Result {
    let dir = laying_repo("lay-staged", 0)?;
    std::fs::write(dir.join("hk.pklith.pkl"), format!("{EMPTY}// staged\n"))?;
    git(&dir, &["add", "hk.pklith.pkl"])?;
    let (code, _, stderr) = pkli_as(&dir, &["lay"])?;
    assert_eq!((code, stderr.as_str()), (Some(2), STAGED));
    Ok(std::fs::remove_dir_all(dir)?)
}

const STAGED: &str = "pkli lay: hk.pklith.pkl is already staged; commit or unstage it first\n";
const NO_HOOKS: &str = "pkli lay: no git hooks are installed, so nothing would check the commits\n";

/// lay V1: with no hooks installed nothing would check the commits, so
/// lay does not start: exit 2.
#[test]
fn lay_refuses_to_start_without_hooks() -> Result {
    let dir = laying_repo("lay-no-hooks", 0)?;
    git(&dir, &["config", "--unset", "core.hooksPath"])?;
    assert_eq!(
        pkli_as(&dir, &["lay"])?,
        (Some(2), String::new(), NO_HOOKS.to_owned())
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A laid step hk cannot see (hk.pkl does not import the generated module)
/// is not committed: the run rolls back, exit 1.
#[test]
fn a_step_hk_cannot_see_is_not_committed() -> Result {
    let dir = laying_repo("lay-unseen", 0)?;
    std::fs::write(dir.join("hk.pkl"), HK.replace("\"lint\"", "\"hand\""))?;
    std::fs::remove_file(dir.join("hk.pklith.pkl"))?;
    let (code, _, stderr) = pkli_as(&dir, &["lay"])?;
    assert_eq!(code, Some(1));
    assert!(
        stderr.contains("does hk.pkl import hk.pklith.pkl"),
        "{stderr}"
    );
    let created = dir.join("hk.pklith.pkl");
    assert!(!created.exists(), "the file lay created is removed again");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// When the rollback itself cannot finish (here the generated file is
/// read-only, so neither lay nor the restore can write it), lay says so
/// instead of claiming a clean rollback.
#[test]
fn an_incomplete_rollback_is_reported() -> Result {
    let dir = laying_repo("lay-readonly", 0)?;
    let file = dir.join("hk.pklith.pkl");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o444))?;
    let (code, _, stderr) = pkli_as(&dir, &["lay"])?;
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644))?;
    assert_eq!(code, Some(1));
    assert!(stderr.contains("; rollback incomplete: "), "{stderr}");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Without a registry lay cannot even plan: exit 2.
#[test]
fn lay_without_a_registry_exits_2() -> Result {
    let dir = laying_repo("lay-no-registry", 0)?;
    std::fs::remove_file(dir.join(".pklith"))?;
    let (code, _, stderr) = pkli_as(&dir, &["lay"])?;
    assert_eq!(code, Some(2));
    assert!(stderr.starts_with("pkli lay: cannot read "), "{stderr}");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A PATH holding only git and pkl: enough to plan, but no hk.
fn path_without_hk() -> Result<String> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let dir_of = |tool: &str| std::env::split_paths(&path).find(|d| d.join(tool).is_file());
    let dirs: Vec<PathBuf> = ["git", "pkl"].iter().filter_map(|t| dir_of(t)).collect();
    Ok(std::env::join_paths(dirs)?.to_string_lossy().into_owned())
}

/// lay V1: without hk no hook would run, so lay does not start: exit 2.
#[test]
fn lay_refuses_to_start_without_hk() -> Result {
    let dir = laying_repo("lay-no-hk-binary", 0)?;
    let out = Command::new(env!("CARGO_BIN_EXE_pkli"))
        .arg("lay")
        .envs(WHO)
        .env("PATH", path_without_hk()?)
        .current_dir(&dir)
        .output()?;
    let stderr = String::from_utf8(out.stderr)?;
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr.starts_with("pkli lay: hk is not usable, so no hook would run: "),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// lay V8: an unrelated change the operator staged is not swept into a
/// laid commit; it is still staged afterwards.
#[test]
fn the_operators_staged_change_stays_out_of_laid_commits() -> Result {
    let dir = laying_repo("lay-operator", 0)?;
    std::fs::write(dir.join("a.rs"), "// operator's work\n")?;
    git(&dir, &["add", "a.rs"])?;
    assert_eq!(pkli_as(&dir, &["lay"])?.0, Some(0));
    assert_eq!(
        git(&dir, &["show", "--name-only", "--format=", "HEAD"])?,
        "hk.pklith.pkl\nnix/pklith.nix"
    );
    assert_eq!(git(&dir, &["diff", "--cached", "--name-only"])?, "a.rs");
    Ok(std::fs::remove_dir_all(dir)?)
}
