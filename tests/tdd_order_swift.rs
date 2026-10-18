//! `tdd-order` on Swift packages (issue #88, age-plugin-se's fork layout)
//! and the hk step that runs it, against the built binary.

mod common;

use common::commits::{RULE, commit, git, range, want};
use common::{OK, Result, pklith, repo};
use std::path::PathBuf;

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

/// The step a consumer adds to run its `tdd-order` rows: category `test`,
/// so gen puts it in `push` (gen V8), whole-tree like `pklith-check`
/// (gen V7), claimed by the registry's own type.
const STEP: &str = "format 1\n## checks\nid|category|nix|glob|check|fix|env|msg\ntdd-order|test|-|-|pklith check --pushed|-|-|a commit changed a file before its test\n## types\ntype|checks|min|exempt\nrs|-|-|fixture\npklith|tdd-order|-|-\npkl|-|-|hk config\n";

/// The wiring: gen emits the `tdd-order` step at push, guarded, and never
/// on commit.
#[test]
fn gen_runs_tdd_order_at_push() -> Result {
    let dir = repo("tdd-gen", Some(&format!("{STEP}{RULE}")))?;
    assert_eq!(pklith(&dir, &["gen"])?.0, Some(0));
    let module = std::fs::read_to_string(dir.join("hk.pklith.pkl"))?;
    let (steps, push) = module.split_once("push:").ok_or("no push mapping")?;
    assert!(!steps.contains("tdd-order"), "{module}");
    let guard = "command -v pklith >/dev/null 2>&1 || { echo 'tdd-order: pklith is not on PATH";
    assert!(push.contains(guard), "{module}");
    assert!(push.contains("pklith check --pushed || {"), "{module}");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// age-plugin-se's fork layout: fork tests live in `Tests/Fork/` as
/// `<Stem>ForkTests.swift`, since swiftc refuses two files with one
/// basename in a module.
const FORK: &str = "swift|-|-|Swift package fixture\n## rules\nid|kind|select|target|except\nswift-tdd|tdd-order|Sources/*.swift|Tests/Fork/{stem}ForkTests.swift|-\n";

/// A package under the fork row, its first commit tagged `base`.
fn fork_package(name: &str) -> Result<PathBuf> {
    let dir = repo(name, Some(&format!("{OK}{FORK}")))?;
    commit(
        &dir,
        &[("Package.swift", "// swift-tools-version:5.9\n")],
        "base",
    )?;
    git(&dir, &["tag", "base"])?;
    Ok(dir)
}

/// The fork target: a fork test committed first passes.
#[test]
fn a_fork_test_first_passes() -> Result {
    let dir = fork_package("tdd-fork")?;
    let red = [("Tests/Fork/NewForkTests.swift", "import XCTest\n")];
    commit(&dir, &red, "red")?;
    commit(&dir, &[("Sources/New.swift", "struct New {}\n")], "green")?;
    assert_eq!(range(&dir)?, (Some(0), String::new()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// The fork target: an upstream-named test does not stand in for the
/// fork test.
#[test]
fn an_upstream_test_is_not_the_fork_test() -> Result {
    let dir = fork_package("tdd-fork-red")?;
    let upstream = [("Tests/OldTests.swift", "import XCTest\n")];
    commit(&dir, &upstream, "upstream")?;
    let id = commit(&dir, &[("Sources/Old.swift", "struct Old {}\n")], "old")?;
    let (source, target) = ("Sources/Old.swift", "Tests/Fork/OldForkTests.swift");
    let line = want("swift-tdd", source, target, &format!("{id} (old)"));
    assert_eq!(range(&dir)?, (Some(1), line));
    Ok(std::fs::remove_dir_all(dir)?)
}
