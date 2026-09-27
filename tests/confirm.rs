//! `pklith confirm`, one planted failure per sub-check (confirm T1),
//! against the built binary.

mod common;

use common::{OK, Result, pklith, repo};
use std::path::Path;

/// An hk.pkl whose check hook runs the generated steps.
const HK: &str = "amends \"pkl/Config.pkl\"\nimport \"hk.pklith.pkl\" as generated\nhooks {\n  [\"check\"] { steps = generated.steps }\n}\n";

/// The common fixture with that hk.pkl and a fresh `pklith gen`.
fn gated(name: &str, registry: &str) -> Result<std::path::PathBuf> {
    let dir = repo(name, Some(registry))?;
    std::fs::write(dir.join("hk.pkl"), HK)?;
    assert_eq!(pklith(&dir, &["gen"])?.0, Some(0));
    Ok(dir)
}

fn confirm(dir: &Path) -> Result<(Option<i32>, String)> {
    let (code, _, stderr) = pklith(dir, &["confirm"])?;
    Ok((code, stderr))
}

/// A complete, faithful, runnable gate is accepted silently.
#[test]
fn a_sound_gate_is_accepted() -> Result {
    let dir = gated("confirm-ok", OK)?;
    assert_eq!(confirm(&dir)?, (Some(0), String::new()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Completeness: a check claimed after the gate was generated has no step.
#[test]
fn a_claimed_check_without_a_step_is_incomplete() -> Result {
    let dir = gated("confirm-incomplete", OK)?;
    let more = OK.replace(
        "lint|lint|-|*|true|-|-|m\n",
        "lint|lint|-|*|true|-|-|m\nother|lint|-|*|true|-|-|m\n",
    );
    std::fs::write(
        dir.join(".pklith"),
        more.replace("rs|lint|-|-", "rs|lint, other|-|-"),
    )?;
    let want = "confirm: completeness: hk.pklith.pkl: `other` is claimed but hk runs no step for it; run `pklith lay`\n";
    assert_eq!(confirm(&dir)?, (Some(1), want.to_owned()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Fidelity: a hand-edited step is not what gen writes.
#[test]
fn a_hand_edited_step_is_unfaithful() -> Result {
    let dir = gated("confirm-edited", OK)?;
    let module = std::fs::read_to_string(dir.join("hk.pklith.pkl"))?;
    std::fs::write(
        dir.join("hk.pklith.pkl"),
        module.replace("true ||", "true && true ||"),
    )?;
    let (code, stderr) = confirm(&dir)?;
    assert_eq!(code, Some(1));
    assert!(
        stderr
            .starts_with("confirm: fidelity: hk.pklith.pkl: differs from what `pklith gen` writes"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Coherence: a step runs a program nothing provides.
#[test]
fn a_program_not_on_path_is_incoherent() -> Result {
    let dir = gated(
        "confirm-path",
        &OK.replace("|true|", "|no-such-tool-zz {{files}}|"),
    )?;
    let want = "confirm: coherence: nix/pklith.nix: `lint` runs `no-such-tool-zz`, which is not on PATH; enter the dev shell that imports nix/pklith.nix\n";
    assert_eq!(confirm(&dir)?, (Some(1), want.to_owned()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Executability: an hk.pkl that does not evaluate is named, not an
/// abort (confirm V2).
#[test]
fn an_hk_pkl_that_does_not_evaluate_is_not_executable() -> Result {
    let dir = gated("confirm-broken", OK)?;
    std::fs::write(dir.join("hk.pkl"), "amends \"pkl/Config.pkl\"\nhooks {\n")?;
    let (code, stderr) = confirm(&dir)?;
    assert_eq!(code, Some(1));
    assert!(
        stderr.contains("confirm: executability: hk.pkl: does not evaluate to steps: "),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Coherence reads a repository script by its path: a missing one is
/// named.
#[test]
fn a_missing_script_is_incoherent() -> Result {
    let dir = gated("confirm-script", &OK.replace("|true|", "|scripts/lint.sh|"))?;
    let (code, stderr) = confirm(&dir)?;
    assert_eq!(code, Some(1));
    assert!(
        stderr.contains("`lint` runs `scripts/lint.sh`, which is not on PATH"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Outside a repository confirm exits 2 naming the cause.
#[test]
fn confirm_outside_a_repository_exits_2() -> Result {
    let outside = common::temp("confirm-outside")?;
    let want = "pklith confirm: not inside a git repository; pass --root DIR\n";
    assert_eq!(confirm(&outside)?, (Some(2), want.to_owned()));
    Ok(std::fs::remove_dir_all(outside)?)
}

/// Coherence finds a program on PATH, and an executable script in the
/// repository by its path.
#[test]
fn programs_on_path_and_repository_scripts_are_coherent() -> Result {
    use std::os::unix::fs::PermissionsExt as _;
    let dir = gated(
        "confirm-found",
        &OK.replace("|true|", "|scripts/lint.sh \\| git --version|"),
    )?;
    std::fs::create_dir_all(dir.join("scripts"))?;
    std::fs::write(dir.join("scripts/lint.sh"), "#!/bin/sh\n")?;
    std::fs::set_permissions(
        dir.join("scripts/lint.sh"),
        std::fs::Permissions::from_mode(0o755),
    )?;
    assert_eq!(confirm(&dir)?, (Some(0), String::new()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A PATH directory holding git and pkl, and nothing else.
fn without_hk(dir: &Path) -> Result<std::path::PathBuf> {
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin)?;
    for tool in ["git", "pkl"] {
        let which = format!("command -v {tool}");
        let found = std::process::Command::new("sh")
            .args(["-c", &which])
            .output()?;
        std::os::unix::fs::symlink(String::from_utf8(found.stdout)?.trim(), bin.join(tool))?;
    }
    Ok(bin)
}

/// Confirm V2: without hk, `hk validate` cannot run, and that is a
/// finding, never a skip.
#[test]
fn a_missing_hk_fails_executability() -> Result {
    let dir = gated("confirm-no-hk", OK)?;
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_pklith"));
    let out = cmd
        .arg("confirm")
        .current_dir(&dir)
        .env("PATH", without_hk(&dir)?)
        .output()?;
    let stderr = String::from_utf8(out.stderr)?;
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr.contains("confirm: executability: hk.pkl: hk validate refused it: "),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}
