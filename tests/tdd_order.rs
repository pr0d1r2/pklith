//! `tdd-order` rules through `pklith check --range`, `--pushed` and
//! `--staged` (rule V6-V8, scan V4, V6, V7), against the built binary.

mod common;

use common::commits::{RULE, based, commit, git, git_raw, range, want};
use common::{OK, Result, pklith, repo};

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

/// Before the first commit nothing is pushed, and `--staged` judges the
/// index alone.
#[test]
fn an_unborn_repository_judges_only_the_index() -> Result {
    let dir = repo("tdd-unborn", Some(&format!("{OK}{RULE}")))?;
    assert_eq!(pklith(&dir, &["check", "--pushed"])?.0, Some(0));
    let (code, _, stderr) = pklith(&dir, &["check", "--staged"])?;
    let line = want("tdd", "sub/b.rs", "sub/b_test.rs", "the staged commit");
    assert_eq!((code, stderr), (Some(1), line));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Scan V4: a path that is not UTF-8 in a judged commit stops the check,
/// even when a later commit removed it from the tree.
#[test]
fn a_path_not_utf8_in_the_history_exits_2() -> Result {
    let dir = based("tdd-utf8", &format!("{OK}{RULE}"))?;
    let blob = git(&dir, &["hash-object", "-w", "--stdin"])?;
    let entry = [format!("100644,{blob},").as_bytes(), b"bad\xff"].concat();
    git_raw(&dir, &["update-index", "--add", "--cacheinfo"], &entry)?;
    git(&dir, &["commit", "-qm", "a bad name"])?;
    git_raw(&dir, &["update-index", "--force-remove"], b"bad\xff")?;
    git(&dir, &["commit", "-qm", "drop it"])?;
    let (code, stderr) = range(&dir)?;
    assert_eq!(code, Some(2));
    assert!(
        stderr.starts_with("pklith check: path is not UTF-8: bad"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}
