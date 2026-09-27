//! `pklith-dev readme [--check]`: the README badge block, and the numbers
//! `docs/LLM-DISCLAIMER.md` quotes, from the files that own them.

use crate::io::{Failed, read, run};
use crate::{badges, block, facts};
use std::path::Path;

/// Render or check.
///
/// # Errors
///
/// Stale disclaimer prose or drift (1), an unreadable source (2).
pub fn run_verb(check: bool) -> Result<(), Failed> {
    let facts = gather()?.checked().map_err(|e| (2, e))?;
    let stale = badges::stale_prose(&read("docs/LLM-DISCLAIMER.md")?, &facts);
    if let Some(want) = stale.first() {
        let why = format!("docs/LLM-DISCLAIMER.md must say {want}, as its sources do");
        return Err((1, why));
    }
    let wanted = block::splice(&read("README.md")?, "badges", &badges::render(&facts))
        .ok_or((2, "README.md has no <!-- BEGIN badges --> block".to_owned()))?;
    crate::io::update("README.md", &wanted, check, "readme")
}

fn gather() -> Result<badges::Facts, Failed> {
    let cargo = read("Cargo.toml")?;
    let field = |key: &str| facts::quoted(&cargo, key).unwrap_or_default();
    let (floor, nixpkgs, platforms) = files()?;
    Ok(badges::Facts {
        slug: field("repository").replace("https://github.com/", ""),
        license: field("license"),
        edition: field("edition"),
        msrv: field("rust-version"),
        dependencies: facts::dependencies(&cargo),
        floor,
        steps: crate::io::hooks()?,
        spec: spec()?,
        nixpkgs,
        platforms,
    })
}

/// The facts `.coverage`, `flake.lock` and CI's workflow own.
fn files() -> Result<(String, facts::Nixpkgs, Vec<facts::Platform>), Failed> {
    let missing = |what: &str| (2, format!("{what} holds no readable value"));
    let floor = facts::floor(&read(".coverage")?).ok_or_else(|| missing(".coverage"))?;
    let nixpkgs =
        facts::nixpkgs(&read("flake.lock")?).ok_or_else(|| missing("flake.lock nixpkgs"))?;
    let platforms = facts::platforms(&read(".github/workflows/ci.yml")?).map_err(|e| (2, e))?;
    Ok((floor, nixpkgs, platforms))
}

/// `§V` rows, `§B` rows and nodes, over every tracked SPEC.md.
fn spec() -> Result<(usize, usize, usize), Failed> {
    let listed = run("git", &["ls-files", "-z", "*SPEC.md"])?;
    let texts = listed
        .split('\0')
        .filter(|p| !p.is_empty() && Path::new(p).is_file())
        .map(read)
        .collect::<Result<Vec<_>, _>>()?;
    let rows = |prefix, sep| facts::rows(&texts, prefix, sep);
    Ok((rows('V', ':'), rows('B', '|'), texts.len()))
}
