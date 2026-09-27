//! `pkli` installed under a legacy tool's name (legacy T5): the legacy
//! environment, messages and exit codes. The legacy bats suites are
//! ported in `linter.rs` and `unit.rs` (legacy T1), stderr compared whole.

#[path = "../common/mod.rs"]
mod common;
mod linter;
mod unit;

use common::Result;
use std::path::{Path, PathBuf};

/// Cleared for every run: the legacy variables, and the git variables a
/// hook running this suite sets, which the compat entries honour.
const VARS: [&str; 7] = [
    "LEFTHOOK_LINTER_COVERAGE_DOC",
    "LEFTHOOK_LINTER_COVERAGE_ROOT",
    "LEFTHOOK_UNIT_COVERAGE_CONFIG",
    "LEFTHOOK_UNIT_COVERAGE_ROOT",
    "GIT_DIR",
    "GIT_INDEX_FILE",
    "GIT_WORK_TREE",
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
    // A temp dir inside some repository must not make this one tracked.
    let ceiling = [("GIT_CEILING_DIRECTORIES", text(&dir))];
    let (code, stderr) = run(&dir, "lefthook-linter-coverage", &repo, &ceiling)?;
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
    write(
        &dir.join("repo"),
        &[(".unit-coverage.toml", "x = {a = 1}\n")],
    )?;
    let env = [("LEFTHOOK_UNIT_COVERAGE_ROOT", "repo")];
    let (code, stderr) = run(&dir, UNIT, &dir, &env)?;
    let want = "lefthook-unit-coverage: .unit-coverage.toml: line 1: inline tables are outside the subset\n";
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

const BASE: &str = "lefthook-linter-coverage";
const LISTED: (&str, &str) = ("docs/linter-coverage.md", "| `.md` |\n");

/// A scratch dir for the base tool whose `repo` holds a listed doc,
/// an unlisted `x.json` and `d/a.md`, all tracked.
fn unlisted(name: &str) -> Result<(PathBuf, PathBuf)> {
    let dir = scratch(name, BASE)?;
    let repo = dir.join("repo");
    write(&repo, &[LISTED, ("x.json", ""), ("d/a.md", "")])?;
    tracked(&repo)?;
    Ok((dir, repo))
}

fn text(path: &Path) -> &str {
    path.to_str().unwrap_or_default()
}

/// A git that cannot run is exit 2, never an empty list that passes.
#[test]
fn a_missing_git_exits_2() -> Result {
    let (dir, repo) = unlisted("no-git")?;
    let empty = dir.join("bin-empty");
    std::fs::create_dir_all(&empty)?;
    let (code, stderr) = run(&dir, BASE, &repo, &[("PATH", text(&empty))])?;
    let want = "linter-coverage: docs/linter-coverage.md: cannot run git";
    assert!(code == Some(2) && stderr.starts_with(want), "{stderr}");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// The caller's `GIT_DIR` and `GIT_WORK_TREE` are honoured, as the legacy
/// tools honoured them: a git dir kept apart still finds the unlisted file.
#[test]
fn a_separate_git_dir_is_honoured() -> Result {
    let (dir, repo) = unlisted("git-dir")?;
    let store = dir.join("store.git");
    std::fs::rename(repo.join(".git"), &store)?;
    let env = [("GIT_DIR", text(&store)), ("GIT_WORK_TREE", text(&repo))];
    let (code, stderr) = run(&dir, BASE, &repo, &env)?;
    assert!(
        code == Some(1) && stderr.contains("\n  .json\n"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Every entry git lists counts, a symlink to a directory too, as it did
/// for `git ls-files`.
#[test]
fn a_tracked_directory_symlink_is_listed() -> Result {
    let (dir, repo) = unlisted("dirlink")?;
    std::os::unix::fs::symlink("d", repo.join("dirlink"))?;
    tracked(&repo)?;
    let (code, stderr) = run(&dir, BASE, &repo, &[])?;
    assert!(
        code == Some(1) && stderr.contains("\n  dirlink\n"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Git failing for any reason but "not a repository" (here a corrupt
/// index) is exit 2 with git's own words.
#[test]
fn a_failing_git_exits_2() -> Result {
    let (dir, repo) = unlisted("bad-index")?;
    std::fs::write(repo.join(".git/index"), "garbage")?;
    let (code, stderr) = run(&dir, BASE, &repo, &[])?;
    assert!(
        code == Some(2) && stderr.contains("git ls-files -z failed: "),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A symlink to a file, a broken one and a FIFO under `dir`.
fn not_regular(dir: &Path) -> Result {
    std::os::unix::fs::symlink("d.md", dir.join("alias.lnk"))?;
    std::os::unix::fs::symlink("gone", dir.join("broken.brk"))?;
    let mut fifo = std::process::Command::new("mkfifo");
    assert!(fifo.arg(dir.join("pipe.fifo")).status()?.success());
    Ok(())
}

/// Walking is `find . -type f ! -path './.git/*'`: symlinks and FIFOs are
/// not regular files, and only the top-level `.git/` is skipped.
#[test]
fn walking_finds_regular_files_as_find_did() -> Result {
    let dir = scratch("find", FULL)?;
    let repo = dir.join("repo");
    let files = [
        ("d.md", "| `.md` |\n"),
        (".git/config", ""),
        ("nested/.git/HEAD", ""),
    ];
    write(&repo, &files)?;
    not_regular(&repo)?;
    let (code, stderr) = run(&dir, FULL, &dir, &WALKED)?;
    let want = "check-linter-coverage: 1 extension(s) not listed in d.md:\n  .HEAD\n";
    assert!(code == Some(1) && stderr.starts_with(want), "{stderr}");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A `*_ROOT` naming no directory fails first, exit 1, as the legacy
/// `cd "$ROOT" || exit 1` did, before the doc or config is looked at.
#[test]
fn a_missing_root_fails_as_cd_did() -> Result {
    let dir = scratch("no-root", UNIT)?;
    let env = [("LEFTHOOK_UNIT_COVERAGE_ROOT", "gone")];
    let (code, stderr) = run(&dir, UNIT, &dir, &env)?;
    let want = "lefthook-unit-coverage: cd: gone: No such file or directory\n";
    assert_eq!((code, stderr.as_str()), (Some(1), want));
    Ok(std::fs::remove_dir_all(dir)?)
}
