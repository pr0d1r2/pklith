//! `pkli migrate`, run against the built binary.

mod common;

use common::{Result, pkli, repo};
use std::path::Path;

const LEFTHOOK: &str = "pre-commit:\n  commands:\n    gitleaks:\n      run: lefthook-gitleaks {staged_files}\n    shellcheck:\n      run: lefthook-shellcheck {staged_files}\n    flake-manifest:\n      run: lefthook-flake-manifest-wrapper\n";

/// The common fixture, a shell script, and a lefthook.yml; no .pklith.
fn lefthooked(name: &str) -> Result<std::path::PathBuf> {
    let dir = repo(name, None)?;
    std::fs::write(dir.join("run.sh"), "#!/bin/sh\n")?;
    std::fs::write(dir.join("lefthook.yml"), LEFTHOOK)?;
    pklith::proc::command("git", &dir)
        .args(["add", "-A"])
        .output()?;
    Ok(dir)
}

fn migrate(dir: &Path, args: &[&str]) -> Result<(Option<i32>, String, String)> {
    pkli(dir, &[&["migrate"][..], args].concat())
}

/// Migrate V1, V3: a command no pklith check stands for is named, and
/// nothing is written.
#[test]
fn a_lost_check_refuses_and_writes_nothing() -> Result {
    let dir = lefthooked("migrate-refuse")?;
    let (code, stdout, stderr) = migrate(&dir, &[])?;
    assert_eq!((code, stdout.as_str()), (Some(1), ""));
    assert!(
        stderr.contains(
            "lefthook `pre-commit` runs `flake-manifest`, which no pklith check stands for"
        ),
        "{stderr}"
    );
    assert!(!dir.join(".pklith").exists());
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Migrate V1: with the lost check dropped on purpose, the registry, the
/// gate and the generated files are written; the drop is recorded.
#[test]
fn a_kept_set_migrates() -> Result {
    let dir = lefthooked("migrate-ok")?;
    let (code, stdout, stderr) = migrate(&dir, &["--drop", "flake-manifest"])?;
    assert_eq!(code, Some(0), "{stderr}");
    assert!(
        stdout.starts_with(".pklith\n") && stdout.contains("hk.pklith.pkl\n"),
        "{stdout}"
    );
    let registry = std::fs::read_to_string(dir.join(".pklith"))?;
    assert!(registry.contains("\n*|ripsecrets|-|-\n"), "{registry}");
    assert!(registry.contains("\nsh|shellcheck|-|-\n"), "{registry}");
    assert!(
        registry.contains("(`--drop`): flake-manifest."),
        "{registry}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Migrate V2: a second run writes nothing.
#[test]
fn a_second_migration_writes_nothing() -> Result {
    let dir = lefthooked("migrate-twice")?;
    assert_eq!(migrate(&dir, &["--drop", "flake-manifest"])?.0, Some(0));
    let again = migrate(&dir, &["--drop", "flake-manifest"])?;
    assert_eq!(again, (Some(0), String::new(), String::new()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Without a lefthook.yml there is nothing to migrate: exit 2.
#[test]
fn no_lefthook_is_an_error() -> Result {
    let dir = repo("migrate-none", None)?;
    let (code, _, stderr) = migrate(&dir, &[])?;
    assert_eq!(code, Some(2));
    assert!(
        stderr.starts_with("pkli migrate: cannot read lefthook.yml: "),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A registry already there that differs is left alone: exit 2.
#[test]
fn an_existing_registry_is_not_overwritten() -> Result {
    let dir = lefthooked("migrate-exists")?;
    std::fs::write(dir.join(".pklith"), "format 1\n")?;
    let (code, _, stderr) = migrate(&dir, &["--drop", "flake-manifest"])?;
    assert_eq!(code, Some(2));
    assert!(stderr.contains(".pklith exists and differs"), "{stderr}");
    assert_eq!(std::fs::read_to_string(dir.join(".pklith"))?, "format 1\n");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A lefthook check no tracked file reaches cannot be claimed: named.
/// A malformed call is a usage error.
#[test]
fn an_unreached_check_is_named_and_bad_flags_are_refused() -> Result {
    let dir = lefthooked("migrate-unreached")?;
    std::fs::write(
        dir.join("lefthook.yml"),
        "pre-commit:\n  commands:\n    xmllint:\n      run: x\n",
    )?;
    let (code, _, stderr) = migrate(&dir, &[])?;
    assert_eq!(code, Some(1));
    assert!(
        stderr.contains("migrate: `xmllint` reaches no tracked file"),
        "{stderr}"
    );
    assert_eq!(migrate(&dir, &["--bogus"])?.0, Some(2));
    Ok(std::fs::remove_dir_all(dir)?)
}
