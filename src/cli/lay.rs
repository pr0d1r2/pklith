//! `pklith lay [--dry-run]`: one commit per missing check.

use super::{Loaded, Outcome, data, exit, files, load, toplevel};
use std::path::{Path, PathBuf};

/// What both `lay` and `lay --dry-run` read.
struct Inputs {
    root: PathBuf,
    registry: crate::registry::Registry,
    catalog: Vec<crate::catalog::Check>,
    present: Vec<String>,
    claimed: Vec<String>,
}

fn inputs(cwd: &Path) -> Result<Inputs, String> {
    let root = toplevel(cwd)?;
    let Loaded {
        registry, catalog, ..
    } = load(&root.join(".pklith"))?;
    let files = laid_tree(&root)?;
    let present = present(&root)?;
    let claimed = crate::cover::claimed_somewhere(&files, &registry);
    Ok(Inputs {
        root,
        registry,
        catalog,
        present,
        claimed,
    })
}

/// The tree as lay leaves it: tracked files and the files gen writes, so
/// a check claimed for a generated file lands in this run, not the next
/// (lay V5).
fn laid_tree(root: &Path) -> Result<Vec<String>, String> {
    let mut files = files(root, false)?;
    files.extend(crate::r#gen::files(root).into_iter().map(str::to_owned));
    files.sort();
    files.dedup();
    Ok(files)
}

/// Step ids hk already runs. For planning, an hk.pkl with no steps yet is
/// a repository with nothing laid, not an error (hook V2 guards judging).
pub(super) fn present(root: &Path) -> Result<Vec<String>, String> {
    match crate::hook::steps(root) {
        Ok(steps) => Ok(steps.into_iter().map(|s| s.id).collect()),
        Err(crate::hook::Error::NoSteps) => Ok(Vec::new()),
        Err(e) => Err(format!("cannot read hk's steps: {e}")),
    }
}

/// `lay --dry-run`: the subjects of the commits lay would make, in order,
/// on stdout (data), writing nothing (lay V4).
pub(super) fn plan(cwd: &Path) -> Outcome {
    let subjects = inputs(cwd).map(|i| {
        let plan = crate::lay::plan(&i.registry, &i.catalog, &i.present, &i.claimed);
        plan.iter()
            .map(|c| crate::lay::subject(c) + "\n")
            .collect::<String>()
    });
    match subjects {
        Ok(stdout) => data(stdout),
        Err(message) => exit(2, format!("pklith lay: {message}\n")),
    }
}

fn laying(inputs: &Inputs) -> Result<String, crate::lay::Failure> {
    let plan = crate::lay::plan(
        &inputs.registry,
        &inputs.catalog,
        &inputs.present,
        &inputs.claimed,
    );
    let used = crate::r#gen::used(&inputs.registry, &inputs.catalog);
    let ctx = crate::lay::Context {
        root: &inputs.root,
        used,
        present: inputs.present.clone(),
    };
    crate::lay::lay(&ctx, &plan)
}

/// `lay`: one commit per planned check, through the hooks; one `<sha>
/// <subject>` line per commit on stdout (lay V9). Exit 2 when it cannot
/// start, 1 when a commit was refused and everything was rolled back (V3).
pub(super) fn run(cwd: &Path) -> Outcome {
    let inputs = match inputs(cwd) {
        Ok(inputs) => inputs,
        Err(message) => return exit(2, format!("pklith lay: {message}\n")),
    };
    match laying(&inputs) {
        Ok(stdout) => data(stdout),
        Err(crate::lay::Failure::NotReady(m)) => exit(2, format!("pklith lay: {m}\n")),
        Err(crate::lay::Failure::RolledBack(m)) => exit(1, format!("pklith lay: {m}\n")),
    }
}
