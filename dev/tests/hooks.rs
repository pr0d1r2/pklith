//! `pklith-dev commit-msg` and `pklith-dev changelog` in a fixture
//! repository, through the built binary.

use std::path::{Path, PathBuf};
use std::process::Command;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const GOOD: &str = "feat(src/a): add x\n\nBecause.\n";

fn git(dir: &Path, args: &[&str]) -> Result {
    let ok = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(args)
        .current_dir(dir)
        .status()?;
    assert!(ok.success(), "git {args:?}");
    Ok(())
}

/// A repository with one commit holding src/a.rs and a changelog, and
/// src/a.rs changed and staged.
fn fixture(name: &str) -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("pklith-dev-hooks-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src"))?;
    std::fs::write(dir.join("src/a.rs"), "1\n")?;
    std::fs::write(dir.join("CHANGELOG.md"), "## [Unreleased]\n\n## [0.1.0]\n")?;
    git(&dir, &["init", "-q"])?;
    git(&dir, &["add", "-A"])?;
    git(&dir, &["commit", "-qm", "init"])?;
    std::fs::write(dir.join("src/a.rs"), "2\n")?;
    git(&dir, &["add", "-A"])?;
    Ok(dir)
}

fn dev(dir: &Path, args: &[&str], path: Option<&str>) -> Result<(Option<i32>, String)> {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_pklith-dev"));
    cmd.args(args).current_dir(dir);
    if let Some(p) = path {
        cmd.env("PATH", p);
    }
    let out = cmd.output()?;
    Ok((out.status.code(), String::from_utf8(out.stderr)?))
}

fn message(dir: &Path, text: &str) -> Result<String> {
    let path = dir.join("msg");
    std::fs::write(&path, text)?;
    Ok(path.to_string_lossy().into_owned())
}

/// The style rule passes a styled message and refuses another.
#[test]
fn commit_msg_judges_the_file_named() -> Result {
    let dir = fixture("style")?;
    assert_eq!(
        dev(&dir, &["commit-msg", &message(&dir, GOOD)?], None)?.0,
        Some(0)
    );
    let (code, stderr) = dev(
        &dir,
        &["commit-msg", &message(&dir, "add x\n\nBecause.\n")?],
        None,
    )?;
    assert!(
        code == Some(1) && stderr.contains("subject must be"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Without a file it reads git's `COMMIT_EDITMSG`; a file that is not there
/// is exit 2, never a pass.
#[test]
fn commit_msg_defaults_to_git_and_refuses_no_file() -> Result {
    let dir = fixture("default")?;
    std::fs::write(dir.join(".git/COMMIT_EDITMSG"), GOOD)?;
    assert_eq!(dev(&dir, &["commit-msg"], None)?.0, Some(0));
    let (code, stderr) = dev(&dir, &["commit-msg", "nowhere"], None)?;
    assert!(
        code == Some(2) && stderr.contains("no message file at nowhere"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Stage a changelog with a new `[Unreleased]` entry.
fn add_entry(dir: &Path) -> Result {
    let text = "## [Unreleased]\n\n- x\n\n## [0.1.0]\n";
    std::fs::write(dir.join("CHANGELOG.md"), text)?;
    git(dir, &["add", "CHANGELOG.md"])
}

/// A feat staging src/ without an entry is refused; with one it passes.
#[test]
fn changelog_asks_for_the_entry() -> Result {
    let dir = fixture("changelog")?;
    let (code, stderr) = dev(&dir, &["changelog", &message(&dir, GOOD)?], None)?;
    assert!(
        code == Some(1) && stderr.contains("\n  src/a.rs\n"),
        "{stderr}"
    );
    add_entry(&dir)?;
    assert_eq!(
        dev(&dir, &["changelog", &message(&dir, GOOD)?], None)?.0,
        Some(0)
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Without git on PATH nothing can be judged: exit 2, never a pass.
#[test]
fn changelog_without_git_exits_2() -> Result {
    let dir = fixture("no-git")?;
    let msg = message(&dir, GOOD)?;
    let (code, stderr) = dev(&dir, &["changelog", &msg], Some("/nonexistent"))?;
    assert!(
        code == Some(2) && stderr.contains("cannot run git"),
        "{stderr}"
    );
    assert_eq!(dev(&dir, &["changelog", "a", "b"], None)?.0, Some(2));
    Ok(std::fs::remove_dir_all(dir)?)
}
