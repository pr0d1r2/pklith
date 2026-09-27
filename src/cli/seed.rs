//! `pklith seed [--init]`: repo-owned seed files from the active fragments
//! (src/seed).

use super::{Loaded, Outcome, data, detecting, exit, files, toplevel};
use std::path::Path;

/// The paths written, one per line, on stdout (V3); an existing file is
/// never touched (seed V1). `--init` also writes `.pklith` when there is
/// none.
pub(super) fn run(cwd: &Path, init: bool) -> Outcome {
    match seeded(cwd, init) {
        Ok(paths) => data(paths.into_iter().map(|p| p + "\n").collect()),
        Err(message) => exit(2, format!("pklith seed: {message}\n")),
    }
}

fn seeded(cwd: &Path, init: bool) -> Result<Vec<String>, String> {
    let root = toplevel(cwd)?;
    let Loaded {
        catalog, fragments, ..
    } = detecting(None, &root)?;
    let files = files(&root, false)?;
    let active = crate::detect::active(&files, &fragments);
    let mut seeds = crate::seed::files(&active, &name(&root))?;
    if init {
        let text = first_registry(&root, &files, &seeds, (&fragments, &catalog));
        seeds.push(crate::seed::Seed::file(".pklith", text));
    }
    let written = crate::seed::write(&root, &seeds)?;
    Ok(written.into_iter().map(str::to_owned).collect())
}

/// The first `.pklith`, for the tree as it will be once seeded and
/// generated: the seed files and what `pklith gen` writes are files the
/// registry must type too, and they switch fragments on (a README is
/// markdown). Types only pklith's own files have are exempt, so `seed
/// --init`, `gen`, `check` is green on a fresh repository.
fn first_registry(
    root: &Path,
    files: &[String],
    seeds: &[crate::seed::Seed],
    (fragments, catalog): (&[crate::catalog::Fragment], &[crate::catalog::Check]),
) -> String {
    // A seed the repository already has is skipped, so the file is its
    // own, not pklith's.
    let written = seeds.iter().filter(|s| !root.join(s.path).exists());
    let mut own: Vec<&str> = written.map(|s| s.path).collect();
    own.extend(crate::r#gen::files(root));
    own.push(".pklith");
    let mut after = files.to_vec();
    after.extend(own.iter().map(|p| (*p).to_owned()));
    after.sort();
    after.dedup();
    let active = crate::detect::active(&after, fragments);
    crate::seed::init::registry(&after, &active, catalog, &own)
}

/// The repository's directory name, for the README stub.
pub(super) fn name(root: &Path) -> String {
    root.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}
