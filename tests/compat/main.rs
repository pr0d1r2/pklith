//! `pkli` installed under a legacy tool's name (legacy T5): the legacy
//! environment, messages and exit codes. The legacy bats suites are
//! ported in `linter.rs` and `unit.rs` (legacy T1), stderr compared whole.

#[path = "../common/mod.rs"]
mod common;
mod linter;
mod unit;

use common::Result;
use std::path::{Path, PathBuf};

const VARS: [&str; 5] = [
    "LEFTHOOK_LINTER_COVERAGE_DOC",
    "LEFTHOOK_LINTER_COVERAGE_ROOT",
    "LEFTHOOK_UNIT_COVERAGE_CONFIG",
    "LEFTHOOK_UNIT_COVERAGE_ROOT",
    "GIT_DIR",
];

/// A scratch directory with `bin/<tool>` linked to the built `pkli`.
fn scratch(name: &str, tool: &str) -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("pklith-compat-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("bin"))?;
    std::fs::create_dir_all(dir.join("repo"))?;
    std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_pkli"), dir.join("bin").join(tool))?;
    Ok(dir)
}

/// Run `tool` from `cwd` with only `env` of the legacy variables set.
fn run(
    scratch: &Path,
    tool: &str,
    cwd: &Path,
    env: &[(&str, &str)],
) -> Result<(Option<i32>, String)> {
    let mut cmd = std::process::Command::new(scratch.join("bin").join(tool));
    cmd.current_dir(cwd).arg("ignored");
    for var in VARS {
        cmd.env_remove(var);
    }
    let out = cmd.envs(env.iter().copied()).output()?;
    assert!(
        out.stdout.is_empty(),
        "the legacy tools printed on stderr only"
    );
    Ok((out.status.code(), String::from_utf8(out.stderr)?))
}

/// Write `files` (path, text) under `dir`.
fn write(dir: &Path, files: &[(&str, &str)]) -> Result {
    for (path, text) in files {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().unwrap_or(dir))?;
        std::fs::write(path, text)?;
    }
    Ok(())
}

/// `git init` and track everything in `dir`.
fn tracked(dir: &Path) -> Result {
    for args in [&["init", "-q"][..], &["add", "-A"]] {
        pklith::proc::command("git", dir).args(args).output()?;
    }
    Ok(())
}

/// A name that is not a legacy tool's runs `pkli` itself.
#[test]
fn another_name_is_pkli() -> Result {
    let dir = scratch("other", "pkli-renamed")?;
    let out = std::process::Command::new(dir.join("bin/pkli-renamed"))
        .arg("--version")
        .output()?;
    assert!(String::from_utf8(out.stdout)?.starts_with("pkli "));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Legacy V5 (B2): a document that parses to nothing is exit 2, never
/// every extension reported unlisted.
#[test]
fn a_doc_parsing_to_nothing_exits_2() -> Result {
    let dir = scratch("nothing", "lefthook-linter-coverage")?;
    let repo = dir.join("repo");
    let doc = ("docs/linter-coverage.md", "| Extension |\n|---|\n");
    write(&repo, &[doc, ("a.sh", "")])?;
    tracked(&repo)?;
    let (code, stderr) = run(&dir, "lefthook-linter-coverage", &repo, &[])?;
    let want = "linter-coverage: docs/linter-coverage.md: parsed nothing";
    assert!(code == Some(2) && stderr.starts_with(want), "{stderr}");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Outside a repository with no root: git listed nothing, so the base
/// tool passed; pkli run as it passes too.
#[test]
fn outside_a_repository_nothing_is_listed() -> Result {
    let dir = scratch("outside", "lefthook-linter-coverage")?;
    let repo = dir.join("repo");
    write(
        &repo,
        &[("docs/linter-coverage.md", "| `.md` |\n"), ("a.sh", "")],
    )?;
    let (code, stderr) = run(&dir, "lefthook-linter-coverage", &repo, &[])?;
    assert_eq!((code, stderr.as_str()), (Some(0), ""));
    Ok(std::fs::remove_dir_all(dir)?)
}

const FULL: &str = "lefthook-linter-coverage-full";

/// `-full` reading `d.md` and walking `repo`.
const WALKED: [(&str, &str); 2] = [
    ("LEFTHOOK_LINTER_COVERAGE_DOC", "d.md"),
    ("LEFTHOOK_LINTER_COVERAGE_ROOT", "repo"),
];

/// A document that is not text is an I/O error: exit 2.
#[test]
fn an_unreadable_doc_exits_2() -> Result {
    let dir = scratch("unreadable", FULL)?;
    std::fs::write(dir.join("repo/d.md"), [0xff, 0xfe])?;
    let (code, stderr) = run(&dir, FULL, &dir, &WALKED)?;
    assert_eq!(code, Some(2), "{stderr}");
    assert!(
        stderr.starts_with("check-linter-coverage: d.md: "),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A root that cannot be walked is an I/O error: exit 2.
#[test]
fn an_unwalkable_root_exits_2() -> Result {
    use std::os::unix::fs::PermissionsExt as _;
    let dir = scratch("unwalkable", FULL)?;
    let files = [("d.md", "| `.md` |\n"), ("shut/a.md", "")];
    write(&dir.join("repo"), &files)?;
    let shut = dir.join("repo/shut");
    std::fs::set_permissions(&shut, std::fs::Permissions::from_mode(0o000))?;
    let code = run(&dir, FULL, &dir, &WALKED)?.0;
    std::fs::set_permissions(&shut, std::fs::Permissions::from_mode(0o755))?;
    assert_eq!(code, Some(2));
    Ok(std::fs::remove_dir_all(dir)?)
}

const UNIT: &str = "lefthook-unit-coverage";

/// A config outside the subset is exit 2, naming the config and line.
#[test]
fn a_config_outside_the_subset_exits_2() -> Result {
    let dir = scratch("broken", UNIT)?;
    write(&dir.join("repo"), &[(".unit-coverage.toml", "[rules]\n")])?;
    let env = [("LEFTHOOK_UNIT_COVERAGE_ROOT", "repo")];
    let (code, stderr) = run(&dir, UNIT, &dir, &env)?;
    let want = "lefthook-unit-coverage: .unit-coverage.toml: line 1: only `[[array]]` tables are in the subset\n";
    assert_eq!((code, stderr.as_str()), (Some(2), want));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// An allowlist that is not text is exit 2.
#[test]
fn an_unreadable_allowlist_exits_2() -> Result {
    let dir = scratch("allowlist", UNIT)?;
    write(
        &dir.join("repo"),
        &[(".unit-coverage.toml", "allowlist = \"al\"\n")],
    )?;
    std::fs::write(dir.join("repo/al"), [0xff])?;
    let env = [("LEFTHOOK_UNIT_COVERAGE_ROOT", "repo")];
    assert_eq!(run(&dir, UNIT, &dir, &env)?.0, Some(2));
    Ok(std::fs::remove_dir_all(dir)?)
}
