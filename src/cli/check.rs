//! `pkli check`: the gate verdict.

use super::{Options, Outcome, USAGE, exit, files, load, options, toplevel};
use std::path::Path;

pub(super) fn run(args: &[String], cwd: &Path) -> Outcome {
    let Some(opts) = options(args) else {
        return exit(2, USAGE);
    };
    match judge(opts, cwd) {
        Ok((coverage, unbacked)) => {
            let text = crate::report::text(&coverage) + &crate::report::unbacked(&unbacked);
            exit(u8::from(!coverage.ok() || !unbacked.is_empty()), text)
        }
        Err(message) => exit(2, format!("pkli check: {message}\n")),
    }
}

type Verdict = (crate::cover::Coverage, Vec<crate::cover::Unbacked>);

fn judge(opts: Options, cwd: &Path) -> Result<Verdict, String> {
    let walk = opts.root.is_some();
    let root = opts.root.map_or_else(|| toplevel(cwd), Ok)?;
    let (registry, _) = load(&opts.registry.unwrap_or_else(|| root.join(".pklith")))?;
    let files = files(&root, walk)?;
    let unbacked = backing(&root, &files, &registry)?;
    let mut coverage = crate::cover::judge(&files, &registry);
    coverage.failed = rules(&root, &files, &registry)?;
    Ok((coverage, unbacked))
}

/// cover V7: the companion rules the tree fails. Without a diff, `changed`
/// rules do not run (rule V3).
fn rules(
    root: &Path,
    files: &[String],
    registry: &crate::registry::Registry,
) -> Result<Vec<crate::rule::Failure>, String> {
    let rules = crate::rule::parse(&registry.rules).map_err(|e| e.to_string())?;
    let plurals: Vec<(String, String)> = registry.plural.iter().map(pair).collect();
    let read = |path: &str| std::fs::read_to_string(root.join(path)).ok();
    let input = crate::rule::Input {
        files,
        diff: None,
        read: &read,
        plurals: &plurals,
    };
    Ok(crate::rule::evaluate(&rules, &input))
}

/// A `## plural` row, which the registry keeps at its header's two cells.
fn pair(row: &crate::registry::Row) -> (String, String) {
    let [one, many] = <[String; 2]>::try_from(row.cells.clone()).unwrap_or_default();
    (one, many)
}

/// cover V3: claims no hk step backs. hk.pkl is read only when something
/// is claimed; an imported legacy registry claims nothing.
fn backing(
    root: &Path,
    files: &[String],
    registry: &crate::registry::Registry,
) -> Result<Vec<crate::cover::Unbacked>, String> {
    if !crate::cover::claims_any(registry) {
        return Ok(Vec::new());
    }
    let steps = crate::hook::steps(root).map_err(|e| format!("cannot read hk's steps: {e}"))?;
    Ok(crate::cover::unbacked(files, registry, &steps))
}
