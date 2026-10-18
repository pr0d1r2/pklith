//! Catalog T9 beyond the fixtures: `bats-changed` reaches a spec through
//! the registry's rule templates alone, prints only what failed, and
//! refuses a changed script with no spec; gen puts both bats rows at push.

use super::fixtures::{BATS_RULE, bats_suite};
use super::{Result, command, file, fixture, run};

fn builtin(id: &str) -> Result<pklith::catalog::Check> {
    let found = pklith::catalog::builtin()?.into_iter().find(|c| c.id == id);
    Ok(found.ok_or_else(|| format!("no built-in check `{id}`"))?)
}

/// `bats-changed` run as hk runs it, `{{files}}` being `files`.
fn changed(name: &str, entries: &[super::Entry], files: &str) -> Result<(bool, String)> {
    let (dir, _) = fixture(name, entries)?;
    let ran = run(command(&builtin("bats-changed")?, &dir, files));
    std::fs::remove_dir_all(dir)?;
    ran
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
