//! The `pkli check` exit-code contract (`src/cli` V1-V5), run against the
//! built binary in throwaway repositories.

mod common;

use common::{HK, OK, Result, check_in, pkli, repo, temp};
use std::path::Path;
use std::process::Command;

#[test]
fn no_command_is_a_usage_error() -> Result {
    let (code, stdout, stderr) = pkli(Path::new("."), &[])?;
    assert_eq!((code, stdout.as_str()), (Some(2), ""));
    assert!(stderr.starts_with("usage: pkli"));
    Ok(())
}

#[test]
fn an_unknown_flag_is_a_usage_error() -> Result {
    let (code, _, stderr) = pkli(Path::new("."), &["check", "--bogus"])?;
    assert_eq!(code, Some(2));
    assert!(stderr.starts_with("usage: pkli"));
    Ok(())
}

/// A covered tree passes silently, from the root and from a subdirectory.
#[test]
fn a_covered_repository_passes_silently() -> Result {
    let dir = repo("ok", Some(OK))?;
    assert_eq!(check_in(&dir)?, (Some(0), String::new()));
    assert_eq!(check_in(&dir.join("sub"))?, (Some(0), String::new()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A type with no row is a finding: exit 1, the gap named on stderr.
#[test]
fn an_uncovered_type_is_a_finding() -> Result {
    let dir = repo("gap", Some(OK))?;
    std::fs::write(dir.join("x.py"), "")?;
    Command::new("git")
        .args(["add", "x.py"])
        .current_dir(&dir)
        .output()?;
    let want =
        "gap: `py` has no row in .pklith (1 file: x.py); add its checks, or an exemption reason\n";
    assert_eq!(check_in(&dir)?, (Some(1), want.to_owned()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Root V11: no `.pklith` never passes.
#[test]
fn a_missing_registry_is_an_error() -> Result {
    let dir = repo("missing", None)?;
    let (code, stderr) = check_in(&dir)?;
    assert_eq!(code, Some(2));
    assert!(
        stderr.starts_with("pkli check: cannot read ") && stderr.contains(".pklith"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

const BAD: [(&str, &str); 3] = [
    (
        "version 1\n",
        "pkli check: .pklith:1: expected `format 1`, got `version 1`\n",
    ),
    (
        "format 1\n## checks\nid|category|nix|glob|check|fix|env|msg\nx|style|-|*|x|-|-|m\n",
        "pkli check: .pklith:4: `x`: unknown category `style`\n",
    ),
    (
        "format 1\n## types\ntype|checks|min|exempt\nrs|clippy-nightly|-|-\n",
        "pkli check: .pklith:4: `rs` names unknown check `clippy-nightly`\n",
    ),
];

/// A registry that cannot be read as written stops the check with its line.
#[test]
fn a_broken_registry_is_an_error_naming_its_line() -> Result {
    for (i, (text, want)) in BAD.into_iter().enumerate() {
        let dir = repo(&format!("bad{i}"), Some(text))?;
        assert_eq!(check_in(&dir)?, (Some(2), want.to_owned()));
        std::fs::remove_dir_all(dir)?;
    }
    Ok(())
}

const OUTSIDE: &str = "pkli check: not inside a git repository; pass --root DIR\n";

/// V5: outside a repository, `--root` is required; with it, the tree is
/// walked instead of read from git.
#[test]
fn outside_a_repository_root_is_walked_on_request() -> Result {
    let dir = temp("walk")?;
    std::fs::write(dir.join(".pklith"), OK)?;
    assert_eq!(check_in(&dir)?, (Some(2), OUTSIDE.to_owned()));
    let root = dir.to_string_lossy().into_owned();
    assert_eq!(
        pkli(Path::new("/"), &["check", "--root", &root])?,
        (Some(0), String::new(), String::new())
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A directory the walk cannot read stops the check with its path; files
/// are never silently skipped.
#[test]
fn an_unreadable_directory_is_an_error() -> Result {
    use std::os::unix::fs::PermissionsExt;
    let dir = temp("unreadable")?;
    std::fs::write(dir.join(".pklith"), OK)?;
    std::fs::set_permissions(dir.join("sub"), std::fs::Permissions::from_mode(0o000))?;
    let (code, _, stderr) = pkli(&dir, &["check", "--root", "."])?;
    std::fs::set_permissions(dir.join("sub"), std::fs::Permissions::from_mode(0o755))?;
    assert_eq!(code, Some(2));
    assert!(
        stderr.starts_with("pkli check: ./sub: ") && stderr.contains("ermission denied"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Root T63, V20 (set-and-setting B97): with a hook's git variables
/// pointing at another repository, as git sets them for hooks in worktrees
/// and during a rebase, pkli still judges its own repository. Without the
/// scrub, `git ls-files` would read the other repository's index.
#[test]
fn hook_variables_from_another_repository_do_not_leak() -> Result {
    let b = repo("leak-b", Some(OK))?;
    std::fs::write(b.join("only-in-b.py"), "")?;
    Command::new("git")
        .args(["add", "only-in-b.py"])
        .current_dir(&b)
        .output()?;
    let a = repo("leak-a", None)?;
    let (code, _, stderr) = pkli_with_hook_env(&b, &a)?;
    let want = "gap: `py` has no row in .pklith (1 file: only-in-b.py); add its checks, or an exemption reason\n";
    assert_eq!((code, stderr.as_str()), (Some(1), want));
    std::fs::remove_dir_all(a)?;
    Ok(std::fs::remove_dir_all(b)?)
}

/// `pkli check` in `dir` with `GIT_DIR`, `GIT_INDEX_FILE` and
/// `GIT_WORK_TREE` all pointing at `other`.
fn pkli_with_hook_env(dir: &Path, other: &Path) -> Result<(Option<i32>, String, String)> {
    let git = other.join(".git");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_pkli"));
    cmd.arg("check").current_dir(dir);
    cmd.env("GIT_DIR", &git)
        .env("GIT_INDEX_FILE", git.join("index"))
        .env("GIT_WORK_TREE", other);
    let out = cmd.output()?;
    Ok((
        out.status.code(),
        String::from_utf8(out.stdout)?,
        String::from_utf8(out.stderr)?,
    ))
}

/// cover V3: `lint` is claimed for `.rs` files, but the hk step only runs on
/// `.py`: every `.rs` file is an unbacked claim, exit 1.
#[test]
fn a_claim_no_hk_step_reaches_is_a_finding() -> Result {
    let dir = repo("unbacked", Some(OK))?;
    std::fs::write(dir.join("hk.pkl"), HK.replace("*.rs", "*.py"))?;
    let want = "unbacked: `lint` is claimed for 2 files its hk step never checks: its hk step's globs (*.py) do not reach them: a.rs, sub/b.rs; fix the step, or stop claiming it\n";
    assert_eq!(check_in(&dir)?, (Some(1), want.to_owned()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Claims need hk's steps; no readable hk.pkl stops the check (root V1).
#[test]
fn claims_without_an_hk_config_are_an_error() -> Result {
    let dir = repo("no-hk", Some(OK))?;
    std::fs::remove_file(dir.join("hk.pkl"))?;
    let (code, stderr) = check_in(&dir)?;
    assert_eq!(code, Some(2));
    assert!(
        stderr.starts_with("pkli check: cannot read hk's steps: `pkl eval -x "),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}
