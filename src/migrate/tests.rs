//! Migrate's equivalence gate (migrate T2).

use super::{Refusal, plan};
use crate::registry::Error;

fn pairs(commands: &[&str]) -> Vec<(String, String)> {
    commands
        .iter()
        .map(|c| ("pre-commit".to_owned(), (*c).to_owned()))
        .collect()
}

fn own(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

const TREE: [&str; 3] = ["a.sh", "flake.nix", "README.md"];

/// V1: every command maps, renamed ones included, and the registry claims
/// exactly that set: `gitleaks` becomes `ripsecrets` in `*`.
#[test]
fn a_mappable_set_becomes_a_registry_claiming_it() -> Result<(), Error> {
    let catalog = crate::catalog::builtin()?;
    let commands = pairs(&["gitleaks", "shellcheck", "nixfmt"]);
    let text = plan(&commands, &[], &own(&TREE), &catalog).unwrap_or_default();
    assert!(
        text.contains("\n*|ripsecrets|-|-\nnix|nixfmt|-|-\nsh|shellcheck|-|-\n"),
        "{text}"
    );
    assert!(crate::registry::parse(&text).is_ok());
    Ok(())
}

/// V1, V3: an unmapped command, or a check no tracked file reaches, is a
/// refusal naming it; nothing is dropped silently.
#[test]
fn a_lost_check_refuses_by_name() -> Result<(), Error> {
    let catalog = crate::catalog::builtin()?;
    let commands = pairs(&["flake-manifest", "yamllint", "shellcheck"]);
    let refusal = plan(&commands, &[], &own(&TREE), &catalog)
        .err()
        .unwrap_or_default();
    let want = Refusal {
        unmapped: pairs(&["flake-manifest"]),
        unreached: own(&["yamllint"]),
    };
    assert_eq!(refusal, want);
    Ok(())
}

/// `--drop` names a command the operator lets go; the registry records it.
#[test]
fn a_dropped_command_is_recorded_not_lost() -> Result<(), Error> {
    let catalog = crate::catalog::builtin()?;
    let commands = pairs(&["flake-manifest", "shellcheck"]);
    let text =
        plan(&commands, &own(&["flake-manifest"]), &own(&TREE), &catalog).unwrap_or_default();
    assert!(
        text.contains("# Dropped from lefthook on purpose (`--drop`): flake-manifest.\n"),
        "{text}"
    );
    assert!(
        text.ends_with("## types\ntype|checks|min|exempt\nsh|shellcheck|-|-\n"),
        "{text}"
    );
    Ok(())
}

/// Two commands standing for one check claim it once.
#[test]
fn two_names_for_one_check_claim_it_once() -> Result<(), Error> {
    let catalog = crate::catalog::builtin()?;
    let commands = pairs(&["gitleaks", "ripsecrets"]);
    let text = plan(&commands, &[], &own(&TREE), &catalog).unwrap_or_default();
    assert_eq!(text.matches("ripsecrets").count(), 1, "{text}");
    Ok(())
}
