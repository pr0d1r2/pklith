//! `pklith map [--staged | FILE…]`: guard mode (src/map).

use super::{Outcome, exit, files, load, toplevel};
use crate::map::Mapped;
use std::path::Path;

/// The specs covering the change, one per line on stdout (V3); a changed
/// file a rule selects whose spec is missing exits 1 naming it (map V1).
/// FILE paths are relative to the repository root.
pub(super) fn run(args: &[String], cwd: &Path) -> Outcome {
    match mapped(args, cwd) {
        Ok(found) => outcome(&found),
        Err(message) => exit(2, format!("pklith map: {message}\n")),
    }
}

fn mapped(args: &[String], cwd: &Path) -> Result<Mapped, String> {
    let root = toplevel(cwd)?;
    let loaded = load(&root.join(".pklith"))?;
    let changed = paths(args, &root)?;
    let files = files(&root, false)?;
    Ok(crate::map::map(
        &changed,
        &files,
        &loaded.rules,
        &loaded.plurals,
    ))
}

/// `--staged` alone reads the index; otherwise the arguments are paths.
fn paths(args: &[String], root: &Path) -> Result<Vec<String>, String> {
    match args {
        [flag] if flag == "--staged" => super::changed(root, &crate::scan::Diff::Staged),
        paths if paths.iter().any(|p| p.starts_with('-')) => {
            Err("give --staged alone, or paths; a path cannot start with `-`".to_owned())
        }
        paths => Ok(paths.to_vec()),
    }
}

fn outcome(found: &Mapped) -> Outcome {
    let missing = found.unmapped.iter().map(|(file, rule, spec)| {
        format!("map: {file} matches rule `{rule}`, but its spec {spec} is not tracked; add it\n")
    });
    Outcome {
        code: u8::from(!found.unmapped.is_empty()),
        stdout: found.specs.iter().map(|s| s.clone() + "\n").collect(),
        stderr: missing.collect(),
    }
}
