//! `pkli detect`: the fragments a repository's files switch on
//! (src/detect).

use super::{Options, Outcome, USAGE, data, exit, files, load, options, toplevel};
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
    let fragments = fragments(opts.registry, &root)?;
    let files = files(&root, walk)?;
    let active = crate::detect::active(&files, &fragments);
    Ok(active.into_iter().map(|f| f.id.clone()).collect())
}

/// The registry's fragments over the built-in ones. A `--registry` must
/// exist; the default `.pklith` may not yet, which means the built-in
/// fragments alone.
fn fragments(
    named: Option<std::path::PathBuf>,
    root: &Path,
) -> Result<Vec<crate::catalog::Fragment>, String> {
    let default = root.join(".pklith");
    let (rows, catalog) = match named {
        Some(path) => rows(&path)?,
        None if default.exists() => rows(&default)?,
        None => (
            Vec::new(),
            crate::catalog::builtin().map_err(|e| e.to_string())?,
        ),
    };
    crate::catalog::fragment::resolved(&rows, &catalog).map_err(|e| e.to_string())
}

type Rows = (Vec<crate::registry::Row>, Vec<crate::catalog::Check>);

fn rows(path: &Path) -> Result<Rows, String> {
    let (registry, catalog) = load(path)?;
    Ok((registry.fragments, catalog))
}
