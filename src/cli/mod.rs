//! Argument dispatch, usage and exit codes (`src/cli/SPEC.md`). Every
//! verb's logic lives in its own node; this only wires them, one file per
//! verb, sharing the pieces below.

use std::path::{Path, PathBuf};

mod check;
mod detect;
mod generate;
mod import;
mod lay;
mod seed;

/// Printed on stderr for a usage error (V2).
pub const USAGE: &str = "usage: pkli <command>\n
  check [--root DIR] [--registry FILE]   every tracked file has a type in .pklith with its checks
  detect [--root DIR] [--registry FILE]  the fragments the tracked files switch on, one per line
  gen [--check]                          write hk.pklith.pkl from .pklith; --check: fail when it is stale
  lay [--dry-run]                        one commit per missing check, through the hooks; --dry-run: list them
  seed [--init]                          write the seed files the fragments ask for, never over one; --init: and a first .pklith
  import DOC                             a .pklith from a legacy linter coverage document, on stdout
";

/// What a run produced: the exit code (V1) and the text for each stream (V3).
pub struct Outcome {
    /// 0 ok, 1 finding, 2 usage or I/O error.
    pub code: u8,
    /// Data only.
    pub stdout: String,
    /// Findings and diagnostics.
    pub stderr: String,
}

fn exit(code: u8, stderr: impl Into<String>) -> Outcome {
    Outcome {
        code,
        stdout: String::new(),
        stderr: stderr.into(),
    }
}

/// A successful run whose result is data, on stdout (V3).
fn data(stdout: String) -> Outcome {
    Outcome {
        code: 0,
        stdout,
        stderr: String::new(),
    }
}

/// Run `pkli` with `args` (without the program name) from `cwd`.
#[must_use]
pub fn run(args: &[String], cwd: &Path) -> Outcome {
    let Some((verb, rest)) = args.split_first() else {
        return exit(2, USAGE);
    };
    match (verb.as_str(), rest) {
        ("check", _) => check::run(rest, cwd),
        ("detect", _) => detect::run(rest, cwd),
        ("gen", []) => generate::run(cwd, false),
        ("lay", []) => lay::run(cwd),
        ("seed", []) => seed::run(cwd, false),
        ("import", [doc]) => import::run(Path::new(doc)),
        ("gen" | "lay" | "seed", [flag]) => flagged(verb, flag, cwd),
        _ => exit(2, USAGE),
    }
}

/// `gen --check`, `lay --dry-run` and `seed --init`: each verb's one flag.
fn flagged(verb: &str, flag: &str, cwd: &Path) -> Outcome {
    match (verb, flag) {
        ("gen", "--check") => generate::run(cwd, true),
        ("lay", "--dry-run") => lay::plan(cwd),
        ("seed", "--init") => seed::run(cwd, true),
        _ => exit(2, USAGE),
    }
}

/// Where a verb looks: `--root` walks that tree instead of asking git;
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

fn files(root: &Path, walk: bool) -> Result<Vec<String>, String> {
    let listed = if walk {
        crate::scan::walked(root)
    } else {
        crate::scan::tracked(root)
    };
    listed.map_err(|e| e.to_string())
}

/// V5: the repository root, from any subdirectory; outside a repository
/// without `--root` is an error.
fn toplevel(cwd: &Path) -> Result<PathBuf, String> {
    let mut cmd = crate::proc::command("git", cwd);
    cmd.args(["rev-parse", "--show-toplevel"]);
    let out = crate::proc::output(&mut cmd)
        .map_err(|_| "not inside a git repository; pass --root DIR".to_owned())?;
    Ok(PathBuf::from(String::from_utf8_lossy(&out).trim_end()))
}

/// The registry and its checks: the built-in catalog with the local rows
/// laid over it, every type row naming a known check (registry V2). Root
/// V11: no `.pklith` is an error, never a pass.
fn load(path: &Path) -> Result<(crate::registry::Registry, Vec<crate::catalog::Check>), String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let registry = crate::registry::parse(&text).map_err(|e| e.to_string())?;
    let local = crate::catalog::parse(&registry.checks).map_err(|e| e.to_string())?;
    // The built-in catalog cannot fail to parse: its own tests keep it so.
    let catalog = crate::catalog::builtin()
        .and_then(|builtin| crate::catalog::merge(builtin, local))
        .map_err(|e| e.to_string())?;
    crate::catalog::known(&registry, &catalog).map_err(|e| e.to_string())?;
    Ok((registry, catalog))
}

type Catalog = (Vec<crate::catalog::Fragment>, Vec<crate::catalog::Check>);

/// The fragments to detect with, and the catalog they name: the registry's
/// over the built-in ones. A named registry must exist; the default
/// `.pklith` may not yet, which means the built-in catalog alone, since
/// detection is how a repository gets its first.
fn fragments(named: Option<PathBuf>, root: &Path) -> Result<Catalog, String> {
    let default = root.join(".pklith");
    let loaded = match named {
        Some(path) => Some(rows(&path)?),
        None if default.exists() => Some(rows(&default)?),
        None => None,
    };
    // The built-in catalog cannot fail to parse: its own tests keep it so.
    let catalog = loaded.map_or_else(|| crate::catalog::builtin().map(|c| (Vec::new(), c)), Ok);
    let resolve = |(rows, catalog): (Vec<_>, Vec<_>)| {
        crate::catalog::fragment::resolved(&rows, &catalog).map(|f| (f, catalog))
    };
    catalog.and_then(resolve).map_err(|e| e.to_string())
}

fn rows(path: &Path) -> Result<(Vec<crate::registry::Row>, Vec<crate::catalog::Check>), String> {
    let (registry, catalog) = load(path)?;
    Ok((registry.fragments, catalog))
}
