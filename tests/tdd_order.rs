//! `tdd-order` rules through `pklith check --range`, `--pushed` and
//! `--staged` (rule V6-V8, scan V6-V7), against the built binary.

mod common;

use common::{OK, Result, pklith, repo};
use std::path::{Path, PathBuf};

/// Every `sub/*.rs` changes its `sub/<stem>_test.rs` first or with it;
/// tests and `sub/skip.rs` are excepted.
const RULE: &str = "## rules\nid|kind|select|target|except\ntdd|tdd-order|sub/*.rs|sub/{stem}_test.rs|sub/*_test.rs, sub/skip.rs\n";

/// git in `dir` with a throwaway identity; its stdout, trimmed.
fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let who = ["-c", "user.name=t", "-c", "user.email=t@t"];
    let out = pklith::proc::command("git", dir)
        .args(who)
        .args(args)
        .output()?;
    assert!(out.status.success(), "git {args:?}: {out:?}");
    Ok(String::from_utf8(out.stdout)?.trim_end().to_owned())
}

/// Write `files` (path, text) and commit them as `subject`; the short id.
fn commit(dir: &Path, files: &[(&str, &str)], subject: &str) -> Result<String> {
    for (path, text) in files {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().ok_or("no parent")?)?;
        std::fs::write(path, text)?;
    }
    git(dir, &["add", "-A"])?;
    git(dir, &["commit", "-qm", subject])?;
    git(dir, &["rev-parse", "--short", "HEAD"])
}

/// A repository under `registry` with its first commit made: the base
/// every range starts from, so `sub/b.rs` without a test is history.
fn based(name: &str, registry: &str) -> Result<PathBuf> {
    let dir = repo(name, Some(registry))?;
    git(&dir, &["commit", "-qm", "base"])?;
    git(&dir, &["tag", "base"])?;
    Ok(dir)
}

fn range(dir: &Path) -> Result<(Option<i32>, String)> {
    let (code, stdout, stderr) = pklith(dir, &["check", "--range", "base..HEAD"])?;
    assert_eq!(stdout, "", "stdout carries data only");
    Ok((code, stderr))
}

fn want(id: &str, source: &str, target: &str, commit: &str) -> String {
    format!(
        "rule: `{id}` wants {target} changed in or before commit {commit} for {source}; add it, or except the file in the rule\n"
    )
}

/// V6: the test committed first, then the code, passes.
#[test]
fn a_test_first_push_passes() -> Result {
    let dir = based("tdd-first", &format!("{OK}{RULE}"))?;
    commit(&dir, &[("sub/b_test.rs", "// red\n")], "test b")?;
    commit(&dir, &[("sub/b.rs", "// green\n")], "make b pass")?;
    assert_eq!(range(&dir)?, (Some(0), String::new()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// V6: the test in the same commit as the code passes.
#[test]
fn a_test_in_the_same_commit_passes() -> Result {
    let dir = based("tdd-same", &format!("{OK}{RULE}"))?;
    let both = [("sub/b_test.rs", "// t\n"), ("sub/b.rs", "// c\n")];
    commit(&dir, &both, "b and its test")?;
    assert_eq!(range(&dir)?, (Some(0), String::new()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// V6, the planted violation: the code before its test fails, naming the
/// file, the test it wanted and the commit; the later test commit does
/// not excuse it.
#[test]
fn code_before_its_test_fails_naming_file_and_commit() -> Result {
    let dir = based("tdd-late", &format!("{OK}{RULE}"))?;
    let id = commit(&dir, &[("sub/b.rs", "// c\n")], "b first")?;
    commit(&dir, &[("sub/b_test.rs", "// t\n")], "test after")?;
    let commit = format!("{id} (b first)");
    let line = want("tdd", "sub/b.rs", "sub/b_test.rs", &commit);
    assert_eq!(range(&dir)?, (Some(1), line));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// `except` cells are honoured, and the test files themselves are not
/// sources.
#[test]
fn excepted_files_demand_nothing() -> Result {
    let dir = based("tdd-except", &format!("{OK}{RULE}"))?;
    commit(&dir, &[("sub/skip.rs", "// c\n")], "skip has no test")?;
    commit(&dir, &[("sub/c_test.rs", "// t\n")], "a lone test")?;
    assert_eq!(range(&dir)?, (Some(0), String::new()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// V8: deleting a source needs no test change.
#[test]
fn a_deletion_passes() -> Result {
    let dir = based("tdd-delete", &format!("{OK}{RULE}"))?;
    git(&dir, &["rm", "-q", "sub/b.rs"])?;
    git(&dir, &["commit", "-qm", "drop b"])?;
    assert_eq!(range(&dir)?, (Some(0), String::new()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// V8, scan V7: a renamed source follows its new name, so it wants the
/// test of the new name.
#[test]
fn a_rename_follows_the_new_name() -> Result {
    let dir = based("tdd-rename", &format!("{OK}{RULE}"))?;
    commit(&dir, &[("sub/b_test.rs", "// b's test\n")], "t")?;
    git(&dir, &["mv", "sub/b.rs", "sub/d.rs"])?;
    git(&dir, &["commit", "-qm", "rename b"])?;
    let id = git(&dir, &["rev-parse", "--short", "HEAD"])?;
    let line = want(
        "tdd",
        "sub/d.rs",
        "sub/d_test.rs",
        &format!("{id} (rename b)"),
    );
    assert_eq!(range(&dir)?, (Some(1), line));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// V8, scan V7: a test renamed with its source counts at its new path.
#[test]
fn a_rename_of_both_passes() -> Result {
    let dir = based("tdd-rename-both", &format!("{OK}{RULE}"))?;
    commit(&dir, &[("sub/b_test.rs", "// b's test\n")], "t")?;
    git(&dir, &["mv", "sub/b_test.rs", "sub/d_test.rs"])?;
    git(&dir, &["mv", "sub/b.rs", "sub/d.rs"])?;
    git(&dir, &["commit", "-qm", "rename both"])?;
    assert_eq!(range(&dir)?, (Some(0), String::new()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// V7, scan V6: `--pushed` judges only commits on no remote-tracking ref,
/// so code already on the remote is never judged again.
#[test]
fn pushed_judges_only_unpushed_commits() -> Result {
    let dir = based("tdd-pushed", &format!("{OK}{RULE}"))?;
    commit(&dir, &[("sub/b.rs", "// pushed\n")], "on the remote")?;
    git(&dir, &["update-ref", "refs/remotes/origin/main", "HEAD"])?;
    assert_eq!(pklith(&dir, &["check", "--pushed"])?.0, Some(0));
    let id = commit(&dir, &[("sub/b.rs", "// unpushed\n")], "not yet")?;
    let (code, _, stderr) = pklith(&dir, &["check", "--pushed"])?;
    let line = want(
        "tdd",
        "sub/b.rs",
        "sub/b_test.rs",
        &format!("{id} (not yet)"),
    );
    assert_eq!((code, stderr), (Some(1), line));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Scan V6: `--staged` judges the index as the commit being made, after
/// the unpushed commits, so a test committed earlier counts.
#[test]
fn staged_judges_the_commit_being_made() -> Result {
    let dir = based("tdd-staged", &format!("{OK}{RULE}"))?;
    git(&dir, &["update-ref", "refs/remotes/origin/main", "HEAD"])?;
    std::fs::write(dir.join("sub/b.rs"), "// staged\n")?;
    git(&dir, &["add", "sub/b.rs"])?;
    let (code, _, stderr) = pklith(&dir, &["check", "--staged"])?;
    let line = want("tdd", "sub/b.rs", "sub/b_test.rs", "the staged commit");
    assert_eq!((code, stderr), (Some(1), line));
    git(&dir, &["reset", "-q", "sub/b.rs"])?;
    std::fs::write(dir.join("sub/b_test.rs"), "// t\n")?;
    git(&dir, &["add", "sub/b_test.rs"])?;
    git(&dir, &["commit", "-qm", "test first"])?;
    git(&dir, &["add", "sub/b.rs"])?;
    assert_eq!(pklith(&dir, &["check", "--staged"])?.0, Some(0));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Issue #88's rows for a Swift package: `swift-test`'s select, target
/// and except taken unchanged as a `tdd-order` row.
const SWIFT: &str = "swift|-|-|Swift package fixture\n## rules\nid|kind|select|target|except\nswift-tdd|tdd-order|Sources/*.swift|Tests/{stem}Tests.swift|Sources/HPKE.swift, Sources/Stream.swift\n";

/// A toy Swift package with flat targets, its first commit tagged `base`.
fn swift_package(name: &str) -> Result<PathBuf> {
    let dir = repo(name, Some(&format!("{OK}{SWIFT}")))?;
    let tree = [
        ("Package.swift", "// swift-tools-version:5.9\n"),
        ("Sources/Plugin.swift", "struct Plugin {}\n"),
        ("Sources/HPKE.swift", "enum HPKE {}\n"),
        ("Tests/PluginTests.swift", "import XCTest\n"),
    ];
    commit(&dir, &tree, "base")?;
    git(&dir, &["tag", "base"])?;
    Ok(dir)
}

/// The #88 row on a Swift package: a test-first `New.swift` passes (Swift
/// Testing as well as `XCTest`, the rule reads no content), and so does an
/// excepted `HPKE.swift` change.
#[test]
fn a_swift_package_takes_the_issue_88_row() -> Result {
    let dir = swift_package("tdd-swift")?;
    commit(&dir, &[("Tests/NewTests.swift", "import Testing\n")], "red")?;
    commit(&dir, &[("Sources/New.swift", "struct New {}\n")], "green")?;
    commit(&dir, &[("Sources/HPKE.swift", "enum HPKE { }\n")], "hpke")?;
    assert_eq!(range(&dir)?, (Some(0), String::new()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// The planted violation on a Swift package: `Sources/New.swift` without
/// `Tests/NewTests.swift` fails, naming both and the commit.
#[test]
fn a_swift_source_without_its_test_fails() -> Result {
    let dir = swift_package("tdd-swift-red")?;
    let id = commit(
        &dir,
        &[("Sources/New.swift", "struct New {}\n")],
        "untested",
    )?;
    let (source, target) = ("Sources/New.swift", "Tests/NewTests.swift");
    let line = want("swift-tdd", source, target, &format!("{id} (untested)"));
    assert_eq!(range(&dir)?, (Some(1), line));
    Ok(std::fs::remove_dir_all(dir)?)
}
