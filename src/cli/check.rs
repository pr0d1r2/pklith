//! `pkli check`: the gate verdict.

use super::{Outcome, USAGE, exit, files, load, toplevel};
use std::path::{Path, PathBuf};

/// Where `check` looks: `--root` walks that tree instead of asking git;
/// `--registry` reads a registry other than `<root>/.pklith`, so a
/// repository can be judged without writing into it.
#[derive(Default)]
struct Options {
    root: Option<PathBuf>,
    registry: Option<PathBuf>,
}

fn options(args: &[String]) -> Option<Options> {
    let mut opts = Options::default();
    for pair in args.chunks(2) {
        match pair {
            [flag, v] if flag == "--root" && opts.root.is_none() => opts.root = Some(v.into()),
            [flag, v] if flag == "--registry" && opts.registry.is_none() => {
                opts.registry = Some(v.into());
            }
            _ => return None,
        }
    }
    Some(opts)
}

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
    Ok((crate::cover::judge(&files, &registry), unbacked))
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
