//! `pkli check --staged` and `--range` (cover V8, rule V3), against the
//! built binary.

mod common;

use common::{OK, Result, check_in, pkli, repo};
use std::path::Path;

fn git(dir: &Path, args: &[&str]) -> Result {
    let who = ["-c", "user.name=t", "-c", "user.email=t@t"];
    pklith::proc::command("git", dir)
        .args(who)
        .args(args)
        .output()?;
    Ok(())
}

/// The common fixture committed, plus an untyped `old.py` the full check
/// flags but staged mode leaves alone.
fn committed(name: &str, pklith: &str) -> Result<std::path::PathBuf> {
    let dir = repo(name, Some(pklith))?;
    std::fs::write(dir.join("old.py"), "")?;
    git(&dir, &["add", "-A"])?;
    git(&dir, &["commit", "-qm", "base"])?;
    Ok(dir)
}

fn staged(dir: &Path) -> Result<(Option<i32>, String, String)> {
    pkli(dir, &["check", "--staged"])
}

/// T4: staging the deletion of the last `.rs` files flags `rs` stale; the
/// untouched `old.py` gap waits for the full check.
#[test]
fn deleting_the_last_file_of_a_type_is_stale_when_staged() -> Result {
    let dir = committed("staged-stale", OK)?;
    git(&dir, &["rm", "-q", "a.rs", "sub/b.rs"])?;
    let (code, _, stderr) = staged(&dir)?;
    assert_eq!(code, Some(1));
    assert!(stderr.starts_with("stale: `rs`"), "{stderr}");
    assert!(!stderr.contains("py"), "{stderr}");
    assert!(check_in(&dir)?.1.contains("gap: `py`"));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// V8: a staged file with no type is a gap; with nothing staged the
/// staged check passes.
#[test]
fn a_staged_file_without_a_type_is_a_gap() -> Result {
    let dir = committed("staged-gap", OK)?;
    assert_eq!(staged(&dir)?.0, Some(0));
    std::fs::write(dir.join("new.go"), "")?;
    git(&dir, &["add", "new.go"])?;
    let (code, _, stderr) = staged(&dir)?;
    assert_eq!(code, Some(1));
    assert!(stderr.starts_with("gap: `go` has no row"), "{stderr}");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// More types (docs, the fixture's .py) and a rule: sub/*.rs changes
/// with docs/<stem>.md.
const RANGE: &str = "md|-|-|docs\npy|-|-|fixture\n## rules\nid|kind|select|target|except\ndoc|changed|sub/*.rs|docs/{stem}.md|-\n";

/// Commit docs/b.md, then a change to sub/b.rs alone.
fn docs_then_code(dir: &Path) -> Result {
    std::fs::create_dir_all(dir.join("docs"))?;
    std::fs::write(dir.join("docs/b.md"), "")?;
    git(dir, &["add", "-A"])?;
    git(dir, &["commit", "-qm", "docs"])?;
    std::fs::write(dir.join("sub/b.rs"), "// changed\n")?;
    git(dir, &["commit", "-qam", "code only"])
}

/// Rule V3: a `changed` rule runs only with a diff; over a range where
/// sub/b.rs changed without docs/b.md it fails.
#[test]
fn a_range_feeds_changed_rules() -> Result {
    let dir = committed("staged-range", &format!("{OK}{RANGE}"))?;
    docs_then_code(&dir)?;
    assert_eq!(check_in(&dir)?, (Some(0), String::new()));
    let (code, _, stderr) = pkli(&dir, &["check", "--range", "HEAD~1..HEAD"])?;
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains("rule: `doc` wants docs/b.md for sub/b.rs"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// `--staged` with `--range`, or `--range` without its value, is a usage
/// error; so is a range git would read as an option.
#[test]
fn diff_flags_are_one_and_well_formed() -> Result {
    let dir = committed("staged-usage", OK)?;
    for args in [
        &["check", "--staged", "--range", "a..b"][..],
        &["check", "--range"],
    ] {
        let (code, _, stderr) = pkli(&dir, args)?;
        assert_eq!(code, Some(2));
        assert!(stderr.starts_with("usage: pkli"), "{stderr}");
    }
    let (code, _, stderr) = pkli(&dir, &["check", "--range", "--output=x"])?;
    assert_eq!(code, Some(2));
    assert!(stderr.contains("is not A..B"), "{stderr}");
    Ok(std::fs::remove_dir_all(dir)?)
}
