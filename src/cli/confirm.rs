//! `pklith confirm`: accept or reject the materialized gate (src/confirm).

use super::{Outcome, exit, files, load, toplevel};
use std::path::Path;

/// Silent and 0 when accepted; each failed sub-check on stderr and 1
/// otherwise (confirm V3); 2 when the inputs cannot be read.
pub(super) fn run(cwd: &Path) -> Outcome {
    match findings(cwd) {
        Ok(found) => {
            let lines: String = found.iter().map(line).collect();
            exit(u8::from(!found.is_empty()), lines)
        }
        Err(message) => exit(2, format!("pklith confirm: {message}\n")),
    }
}

fn findings(cwd: &Path) -> Result<Vec<crate::confirm::Finding>, String> {
    let root = toplevel(cwd)?;
    let loaded = load(&root.join(".pklith"))?;
    let files = files(&root, false)?;
    // An hk.pkl that does not evaluate is the executability finding, not an
    // abort: judge completeness as if nothing were laid.
    let present = super::lay::present(&root).unwrap_or_default();
    let claimed = crate::cover::claimed_somewhere(&files, &loaded.registry);
    let missing = crate::lay::plan(&loaded.registry, &loaded.catalog, &present, &claimed);
    let used = crate::r#gen::used(&loaded.registry, &loaded.catalog);
    let inputs = crate::confirm::Inputs {
        root: &root,
        used: &used,
        missing: &missing,
    };
    Ok(crate::confirm::confirm(&inputs))
}

fn line(f: &crate::confirm::Finding) -> String {
    format!("confirm: {}: {}: {}; {}\n", f.part, f.file, f.what, f.fix)
}
