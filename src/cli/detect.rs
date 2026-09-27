//! `pkli detect`: the fragments a repository's files switch on
//! (src/detect).

use super::{Options, Outcome, USAGE, data, detecting, exit, files, options, toplevel};
use std::path::Path;

/// One fragment id per line, in catalog order, on stdout (V3). `.pklith`
/// is optional here: detection is how a repository without one gets its
/// first (src/seed), so its absence means the built-in catalog alone.
pub(super) fn run(args: &[String], cwd: &Path) -> Outcome {
    let Some(opts) = options(args) else {
        return exit(2, USAGE);
    };
    match detected(opts, cwd) {
        Ok(ids) => data(ids.into_iter().map(|id| id + "\n").collect()),
        Err(message) => exit(2, format!("pkli detect: {message}\n")),
    }
}

fn detected(opts: Options, cwd: &Path) -> Result<Vec<String>, String> {
    let walk = opts.root.is_some();
    let root = opts.root.map_or_else(|| toplevel(cwd), Ok)?;
    let fragments = detecting(opts.registry, &root)?.fragments;
    let files = files(&root, walk)?;
    let active = crate::detect::active(&files, &fragments);
    Ok(active.into_iter().map(|f| f.id.clone()).collect())
}
