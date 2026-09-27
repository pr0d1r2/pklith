//! `pkli gen [--check]`: the repository's `hk.pklith.pkl` and
//! `nix/pklith.nix` from its `.pklith`.

use super::{Outcome, exit, load, toplevel};
use std::path::Path;

/// Writes only what changed (gen V2); `--check` writes nothing and fails
/// when a generated file is stale for the steps it holds, or missing (gen
/// V3).
pub(super) fn run(cwd: &Path, check: bool) -> Outcome {
    let loaded = toplevel(cwd).and_then(|root| Ok((load(&root.join(".pklith"))?, root)));
    let (loaded, root) = match loaded {
        Ok(found) => found,
        Err(message) => return exit(2, format!("pkli gen: {message}\n")),
    };
    let used = crate::r#gen::used(&loaded.registry, &loaded.catalog);
    if check {
        return freshness(&root, &used);
    }
    match crate::r#gen::write(&root, &used) {
        Ok(_) => exit(0, ""),
        Err(message) => exit(2, format!("pkli gen: {message}\n")),
    }
}

fn freshness(root: &Path, used: &[&crate::catalog::Check]) -> Outcome {
    match crate::r#gen::stale(root, used) {
        None => exit(0, ""),
        Some(file) => exit(
            1,
            format!("pkli gen: {file} is stale; run `pkli gen` and stage it\n"),
        ),
    }
}
