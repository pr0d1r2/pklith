//! `pklith migrate [--drop ID,…]`: a lefthook repository onto hk
//! (src/migrate), writing nothing unless the check set survives.

use super::{Outcome, USAGE, data, exit, files, parsed, toplevel};
use crate::migrate::Refusal;
use std::path::Path;

/// The paths written, one per line (V3); a refusal names every lost check
/// on stderr and exits 1 having written nothing (migrate V1); a second
/// run writes nothing (V2).
pub(super) fn run(args: &[String], cwd: &Path) -> Outcome {
    let drop = match args {
        [] => Vec::new(),
        [flag, ids] if flag == "--drop" => ids.split(',').map(|s| s.trim().to_owned()).collect(),
        _ => return exit(2, USAGE),
    };
    match migrated(&drop, cwd) {
        Ok(Ok(paths)) => data(paths.into_iter().map(|p| p.to_owned() + "\n").collect()),
        Ok(Err(refusal)) => exit(1, refused(&refusal)),
        Err(message) => exit(2, format!("pklith migrate: {message}\n")),
    }
}

type Migrated = Result<Vec<&'static str>, Refusal>;

fn migrated(drop: &[String], cwd: &Path) -> Result<Migrated, String> {
    let root = toplevel(cwd)?;
    let main = std::fs::read_to_string(root.join("lefthook.yml"))
        .map_err(|e| format!("cannot read lefthook.yml: {e}"))?;
    let local = std::fs::read_to_string(root.join("lefthook-local.yml")).unwrap_or_default();
    let commands = crate::migrate::lefthook::commands(&main, &local)?;
    let files = files(&root, false)?;
    let catalog = parsed(super::BUILTIN_ONLY)?.catalog;
    match crate::migrate::plan(&commands, drop, &files, &catalog) {
        Ok(text) => write(&root, &files, text).map(Ok),
        Err(refusal) => Ok(Err(refusal)),
    }
}

/// The registry, the seed files it switches on, and the generated gate;
/// an existing registry that differs is left alone and refused.
fn write(root: &Path, files: &[String], text: String) -> Result<Vec<&'static str>, String> {
    let existing = std::fs::read_to_string(root.join(".pklith")).ok();
    if existing.as_ref().is_some_and(|e| *e != text) {
        return Err(
            ".pklith exists and differs from what migrate writes; migrate only where there is none"
                .into(),
        );
    }
    let loaded = parsed(&text)?;
    let active = crate::detect::active(files, &loaded.fragments);
    let mut seeds = crate::seed::files(&active, &super::seed::name(root))?;
    seeds.insert(0, crate::seed::Seed::file(".pklith", text));
    let mut written = crate::seed::write(root, &seeds)?;
    let used = crate::r#gen::used(&loaded.registry, &loaded.catalog);
    written.extend(crate::r#gen::write(root, &used)?);
    Ok(written)
}

fn refused(refusal: &Refusal) -> String {
    let unmapped = refusal.unmapped.iter().map(|(hook, command)| {
        format!("migrate: lefthook `{hook}` runs `{command}`, which no pklith check stands for; give it a check in .pklith by hand, or `--drop {command}`\n")
    });
    let unreached = refusal.unreached.iter().map(|id| {
        format!("migrate: `{id}` reaches no tracked file, so no row could claim it; `--drop` it if it is stale\n")
    });
    unmapped.chain(unreached).collect()
}
