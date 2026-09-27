//! `pkli seed [--init]`: repo-owned seed files from the active fragments
//! (src/seed).

use super::{Loaded, Outcome, data, detecting, exit, files, toplevel};
use std::path::Path;

/// The paths written, one per line, on stdout (V3); an existing file is
/// never touched (seed V1). `--init` also writes `.pklith` when there is
/// none.
pub(super) fn run(cwd: &Path, init: bool) -> Outcome {
    match seeded(cwd, init) {
        Ok(paths) => data(paths.into_iter().map(|p| p + "\n").collect()),
        Err(message) => exit(2, format!("pkli seed: {message}\n")),
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
        let text = crate::seed::init::registry(&files, &active, &catalog);
        seeds.push(crate::seed::Seed::file(".pklith", text));
    }
    let written = crate::seed::write(&root, &seeds)?;
    Ok(written.into_iter().map(str::to_owned).collect())
}

/// The repository's directory name, for the README stub.
pub(super) fn name(root: &Path) -> String {
    root.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}
