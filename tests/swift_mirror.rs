//! Issue #88's `SwiftPM` mirror rows (`exists` and `orphan`) on a toy tree
//! with age-plugin-se's fork layout, through `pklith check` against the
//! built binary. The `tdd-order` row is in `tests/tdd_order_swift.rs`.

mod common;

use common::{OK, Result, check_in, repo};
use std::path::{Path, PathBuf};

/// #88's rows, plus its `tdd-order` row with the same select and target:
/// a plain check parses it and leaves it to a history (rule V6).
const ROWS: &str = "swift|-|-|Swift package fixture\n## rules\nid|kind|select|target|except\nswift-test|exists|Sources/*.swift|Tests/Fork/{stem}ForkTests.swift|-\nswift-orphan|orphan|Tests/*Tests.swift|Sources/{stem\\|chop:ForkTests\\|chop:Tests}.swift|Tests/StanzaTests.swift\nswift-tdd|tdd-order|Sources/*.swift|Tests/Fork/{stem}ForkTests.swift|-\n";

/// Sources, upstream tests, fork tests in `Tests/Fork/`, the excepted
/// upstream `StanzaTests.swift`, and helpers not named `*Tests.swift`.
const TREE: [&str; 9] = [
    "Package.swift",
    "Sources/Plugin.swift",
    "Sources/HPKE.swift",
    "Tests/PluginTests.swift",
    "Tests/StanzaTests.swift",
    "Tests/DummyCrypto.swift",
    "Tests/Fork/PluginForkTests.swift",
    "Tests/Fork/HPKEForkTests.swift",
    "Tests/Fork/BuiltBinary.swift",
];

/// The toy tree under `registry`, every file tracked.
fn package(name: &str, registry: &str) -> Result<PathBuf> {
    let dir = repo(name, Some(&format!("{OK}{registry}")))?;
    plant(&dir, &TREE)?;
    Ok(dir)
}

/// Write each of `paths` and track it.
fn plant(dir: &Path, paths: &[&str]) -> Result {
    for path in paths {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().ok_or("no parent")?)?;
        std::fs::write(path, "import XCTest\n")?;
    }
    pklith::proc::command("git", dir)
        .args(["add", "-A"])
        .output()?;
    Ok(())
}

fn want(id: &str, missing: &str, source: &str) -> String {
    format!("rule: `{id}` wants {missing} for {source}; add it, or except the file in the rule\n")
}

/// The full tree passes, so `*` crossing `/` reaches `Tests/Fork/`, both
/// chops map their layout, and no helper is selected.
#[test]
fn the_fork_layout_passes() -> Result {
    let dir = package("swift-mirror", ROWS)?;
    assert_eq!(check_in(&dir)?, (Some(0), String::new()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A helper anywhere under `Tests/` without a source is not an orphan.
#[test]
fn a_helper_is_not_selected() -> Result {
    let dir = package("swift-helper", ROWS)?;
    plant(&dir, &["Tests/Fork/Stub.swift", "Tests/Fixtures.swift"])?;
    assert_eq!(check_in(&dir)?, (Some(0), String::new()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A planted source without its fork test fails `swift-test`.
#[test]
fn a_source_wants_its_fork_test() -> Result {
    let dir = package("swift-untested", ROWS)?;
    plant(&dir, &["Sources/New.swift"])?;
    let line = want(
        "swift-test",
        "Tests/Fork/NewForkTests.swift",
        "Sources/New.swift",
    );
    assert_eq!(check_in(&dir)?, (Some(1), line));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A fork test and an upstream test without a source each fail
/// `swift-orphan`, naming the same missing source.
#[test]
fn a_test_without_a_source_is_an_orphan() -> Result {
    for ghost in ["Tests/Fork/GhostForkTests.swift", "Tests/GhostTests.swift"] {
        let dir = package("swift-ghost", ROWS)?;
        plant(&dir, &[ghost])?;
        let line = want("swift-orphan", "Sources/Ghost.swift", ghost);
        assert_eq!(check_in(&dir)?, (Some(1), line));
        std::fs::remove_dir_all(dir)?;
    }
    Ok(())
}

/// Without its `except`, upstream's `StanzaTests.swift` is an orphan.
#[test]
fn the_except_is_what_spares_stanza_tests() -> Result {
    let rows = ROWS.replace("|Tests/StanzaTests.swift\n", "|-\n");
    let dir = package("swift-stanza", &rows)?;
    let line = want(
        "swift-orphan",
        "Sources/Stanza.swift",
        "Tests/StanzaTests.swift",
    );
    assert_eq!(check_in(&dir)?, (Some(1), line));
    Ok(std::fs::remove_dir_all(dir)?)
}
