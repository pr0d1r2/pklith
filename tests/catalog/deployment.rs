//! Catalog T13 beyond the V30 fixture: `cargo-deployment-target`, run as hk
//! runs it, judges `.cargo/config.toml` by its parsed TOML. A missing key,
//! another value and a value without `force` each fail, naming the file
//! and what it must hold; the inline table and the sub-table both pass; a
//! repository without a root `Cargo.toml` is left alone.

use super::{Entry, File, Result, command, file, fixture, run};

const MANIFEST: &str = "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";

/// What the failures name as expected.
const WANT: &str = "[env] MACOSX_DEPLOYMENT_TARGET = { value = \"14.0\", force = true }";

/// The row in a fixture of `entries`: whether it passed, and its output.
fn judged(entries: &[Entry]) -> Result<(bool, String)> {
    let check = pklith::catalog::builtin()?
        .into_iter()
        .find(|c| c.id == "cargo-deployment-target")
        .ok_or("no built-in check `cargo-deployment-target`")?;
    let (dir, files) = fixture("deployment", entries)?;
    let ran = run(command(&check, &dir, &files));
    std::fs::remove_dir_all(dir)?;
    ran
}

/// A crate whose `.cargo/config.toml` holds `config`.
fn crate_with(config: &str) -> [Entry; 2] {
    [
        file("Cargo.toml", MANIFEST),
        File(".cargo/config.toml", config.to_owned()),
    ]
}

/// Each config, and what the failure says.
const REFUSED: [(&str, &str); 4] = [
    (
        "[build]\njobs = 1\n",
        "cargo-deployment-target: .cargo/config.toml sets no [env] MACOSX_DEPLOYMENT_TARGET; it must hold",
    ),
    (
        "[env]\nMACOSX_DEPLOYMENT_TARGET = { value = \"26.0\", force = true }\n",
        "cargo-deployment-target: .cargo/config.toml sets MACOSX_DEPLOYMENT_TARGET to 26.0, expected 14.0",
    ),
    (
        "[env]\nMACOSX_DEPLOYMENT_TARGET = \"14.0\"\n",
        "cargo-deployment-target: .cargo/config.toml sets MACOSX_DEPLOYMENT_TARGET without force = true",
    ),
    (
        "[env\n",
        "cargo-deployment-target: .cargo/config.toml does not parse as TOML",
    ),
];

/// A missing key, the wrong value, a plain string (no `force`, which the
/// dev shell's own `MACOSX_DEPLOYMENT_TARGET` beats) and broken TOML fail.
#[test]
fn a_config_without_the_pinned_floor_fails() -> Result {
    for (config, says) in REFUSED {
        let (passed, output) = judged(&crate_with(config))?;
        assert!(!passed && output.contains(says), "{config}:\n{output}");
    }
    Ok(())
}

/// No `.cargo/config.toml` at all fails, naming the file and the line.
#[test]
fn a_missing_config_fails_naming_it() -> Result {
    let (passed, output) = judged(&[file("Cargo.toml", MANIFEST)])?;
    let says = format!(".cargo/config.toml is missing; it must hold {WANT}");
    assert!(!passed && output.contains(&says), "{output}");
    Ok(())
}

/// The inline table and the `[env.MACOSX_DEPLOYMENT_TARGET]` sub-table are
/// one TOML value, and both pass.
#[test]
fn both_table_forms_pass() -> Result {
    let inline = "[env]\nMACOSX_DEPLOYMENT_TARGET = { value = \"14.0\", force = true }\n";
    let sub = "[env.MACOSX_DEPLOYMENT_TARGET]\nforce = true\nvalue = \"14.0\"\n";
    for config in [inline, sub] {
        let (passed, output) = judged(&crate_with(config))?;
        assert!(passed, "{config}:\n{output}");
    }
    Ok(())
}

/// Without a root `Cargo.toml` there is no workspace to pin: a notice.
#[test]
fn a_repository_without_cargo_passes_with_a_notice() -> Result {
    let (passed, output) = judged(&[file("a.txt", "x\n")])?;
    assert!(passed, "{output}");
    assert!(
        output.contains("cargo-deployment-target: skipped: no Cargo.toml at the repository root"),
        "{output}"
    );
    Ok(())
}
