//! Catalog T9 beyond the fixtures: `bats-changed` reaches a spec through
//! the registry's rule templates alone, prints only what failed, and
//! refuses a changed script with no spec; gen puts both bats rows at push.
//! T10: neither row passes having run 0 tests, and `bats-unit` prints
//! only what failed.

use super::fixtures::{BATS_RULE, bats_suite};
use super::{Result, command, file, fixture, run};

fn builtin(id: &str) -> Result<pklith::catalog::Check> {
    let found = pklith::catalog::builtin()?.into_iter().find(|c| c.id == id);
    Ok(found.ok_or_else(|| format!("no built-in check `{id}`"))?)
}

/// `bats-changed` run as hk runs it, `{{files}}` being `files`.
fn changed(name: &str, entries: &[super::Entry], files: &str) -> Result<(bool, String)> {
    step("bats-changed", name, entries, files)
}

/// The built-in `id` run as hk runs it in a fixture of `entries`.
fn step(id: &str, name: &str, entries: &[super::Entry], files: &str) -> Result<(bool, String)> {
    let (dir, _) = fixture(name, entries)?;
    let ran = run(command(&builtin(id)?, &dir, files));
    std::fs::remove_dir_all(dir)?;
    ran
}

/// `bats-unit` over a whole fixture of `entries`.
fn unit(name: &str, entries: &[super::Entry]) -> Result<(bool, String)> {
    step("bats-unit", name, entries, "")
}

/// Issue #102: bats run on no test prints `1..0` and exits 0. A `SwiftPM`
/// `Tests/` alone is that case where `tests` opens it (a case-insensitive
/// filesystem, the macOS default); on Linux CI it is a missing `tests/`.
/// A `tests/` with no `.bats` reaches `1..0` on every system, so the
/// 0-tests rule it proves is what covers the alias wherever it occurs.
#[test]
fn bats_unit_fails_having_run_no_test() -> Result {
    let swift = "import XCTest\n";
    for (name, path) in [("swift", "Tests/AppTests.swift"), ("none", "README")] {
        let (passed, output) = unit(name, &[file(path, swift)])?;
        assert!(!passed, "{name}:\n{output}");
        assert!(output.contains("bats-unit: "), "{output}");
    }
    let (passed, output) = unit("empty", &[file("tests/AppTests.swift", swift)])?;
    assert!(!passed, "{output}");
    assert!(
        output.contains("no bats tests found under tests/"),
        "{output}"
    );
    Ok(())
}

/// A failing spec under `tests/` prints its `not ok` lines and no `ok`
/// line; a passing suite prints nothing.
#[test]
fn bats_unit_prints_failures_only() -> Result {
    let (passed, output) = unit("unit-red", &bats_suite(2))?;
    assert!(!passed && output.contains("not ok 2 prints 3"), "{output}");
    assert!(!output.lines().any(|l| l.starts_with("ok ")), "{output}");
    let (passed, output) = unit("unit-green", &bats_suite(3))?;
    assert!(passed, "{output}");
    assert_eq!(output, "");
    Ok(())
}

/// A changed spec holding no `@test` runs 0 tests, which fails too.
#[test]
fn bats_changed_fails_on_a_spec_with_no_test() -> Result {
    let entries = [
        file(".pklith", BATS_RULE),
        file("tests/unit/x.bats", "# todo\n"),
    ];
    let (passed, output) = changed("bats-no-test", &entries, "tests/unit/x.bats")?;
    assert!(!passed, "{output}");
    assert!(output.contains("bats-changed: no bats tests"), "{output}");
    Ok(())
}

/// Only the script changed: its spec is found by the `exists` template,
/// run, and of its two tests only the failing one is printed.
#[test]
fn a_changed_script_runs_its_spec_and_prints_failures_only() -> Result {
    let (passed, output) = changed("bats-companion", &bats_suite(2), "scripts/a.sh")?;
    assert!(!passed, "{output}");
    assert!(output.contains("not ok 2 prints 3"), "{output}");
    assert!(!output.lines().any(|l| l.starts_with("ok ")), "{output}");
    let (passed, output) = changed("bats-companion-good", &bats_suite(3), "scripts/a.sh")?;
    assert!(passed, "{output}");
    assert_eq!(output, "");
    Ok(())
}

/// A change no rule selects runs nothing and passes; a changed script whose
/// spec is missing fails, naming the spec it wants (map V1).
#[test]
fn nothing_to_run_passes_and_a_missing_spec_fails() -> Result {
    let (passed, output) = changed("bats-none", &bats_suite(2), ".pklith")?;
    assert!(passed, "{output}");
    let entries = [file(".pklith", BATS_RULE), file("scripts/b.sh", "echo\n")];
    let (passed, output) = changed("bats-missing", &entries, "scripts/b.sh")?;
    assert!(!passed, "{output}");
    assert!(
        output.contains("tests/unit/b.bats is not tracked"),
        "{output}"
    );
    Ok(())
}

/// Both rows are `test` checks: gen emits them in `push`, never in the
/// commit steps (gen V8).
#[test]
fn gen_puts_both_bats_rows_at_push() -> Result {
    let types = "## types\ntype|checks|min|exempt\nsh|bats-changed, bats-unit|-|-\nbats|-|-|specs\npklith|-|-|registry\n";
    let entries = [file(".pklith", &format!("{BATS_RULE}{types}"))];
    let (dir, _) = fixture("bats-gen", &entries)?;
    let module = generated(&dir)?;
    let (steps, push) = module.split_once("push:").ok_or("no push mapping")?;
    for id in ["bats-changed", "bats-unit"] {
        assert!(!steps.contains(id) && push.contains(id), "{module}");
    }
    Ok(std::fs::remove_dir_all(dir)?)
}

/// `hk.pklith.pkl` as the built `pklith gen` writes it in `dir`.
fn generated(dir: &std::path::Path) -> Result<String> {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_pklith"))
        .arg("gen")
        .current_dir(dir)
        .output()?;
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    Ok(std::fs::read_to_string(dir.join("hk.pklith.pkl"))?)
}
