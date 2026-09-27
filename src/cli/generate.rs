//! `pkli gen [--check]`: the repository's `hk.pklith.pkl` from its
//! `.pklith`.

use super::{Outcome, exit, load, toplevel};
use std::path::Path;

/// Writes only on change (gen V2); `--check` writes nothing and fails when
/// a step the file holds is stale, or the file is missing (gen V3).
pub(super) fn run(cwd: &Path, check: bool) -> Outcome {
    let loaded = toplevel(cwd).and_then(|root| Ok((load(&root.join(".pklith"))?, root)));
    let Ok(((registry, catalog), root)) = loaded else {
        return exit(
            2,
            format!("pkli gen: {}\n", loaded.err().unwrap_or_default()),
        );
    };
    let used = crate::r#gen::used(&registry, &catalog);
    if check {
        return freshness(&root, &used);
    }
    written(&root, &crate::r#gen::pkl(&used))
}

fn freshness(root: &Path, used: &[&crate::catalog::Check]) -> Outcome {
    if crate::r#gen::fresh(root, used) {
        return exit(0, "");
    }
    exit(
        1,
        format!(
            "pkli gen: {} is stale; run `pkli gen` and stage it\n",
            crate::r#gen::FILE
        ),
    )
}

fn written(root: &Path, text: &str) -> Outcome {
    match crate::r#gen::write(root, text) {
        Ok(_) => exit(0, ""),
        Err(e) => exit(
            2,
            format!("pkli gen: cannot write {}: {e}\n", crate::r#gen::FILE),
        ),
    }
}
