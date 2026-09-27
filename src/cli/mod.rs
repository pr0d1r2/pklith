//! Argument dispatch, usage and exit codes (`src/cli/SPEC.md`). Every
//! verb's logic lives in its own node; this only wires them, one file per
//! verb, sharing the pieces below.

use std::path::{Path, PathBuf};

mod check;
mod compat;
mod confirm;
mod detect;
mod generate;
mod import;
mod lay;
mod map;
mod migrate;
mod protect;
mod report;
mod seed;

/// Printed on stderr for a usage error (V2).
pub const USAGE: &str = "usage: pkli <command> | --version\n
  check [--root DIR] [--registry FILE] [--staged | --range A..B]
                                         every tracked file has a type in .pklith with its checks;
                                         --staged: only what the commit changes; a diff feeds `changed` rules
  confirm                                accept the gate: complete, faithful, runnable, deterministic
  detect [--root DIR] [--registry FILE]  the fragments the tracked files switch on, one per line
  gen [--check]                          write hk.pklith.pkl from .pklith; --check: fail when it is stale
  lay [--dry-run]                        one commit per missing check, through the hooks; --dry-run: list them
  map [--staged | FILE...]               the specs covering a change, one per line; exit 1 when one is missing
  migrate [--drop ID,...]                a lefthook repo onto hk, only if every check survives
  protect [--dry-run] [--repo O/N] [--branch B] [--accept-removals]
                                         branch protection's required contexts from CI jobs, through gh (network)
  report [--format text|json|md]         the coverage matrix and findings, always printed; takes --root, --registry
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

/// Run as `program`: a legacy tool's name runs its compat entry with
/// the process environment (`src/legacy` T5), anything else is `pkli`.
#[must_use]
pub fn run_as(program: &str, args: &[String], cwd: &Path) -> Outcome {
    let env = |k: &str| std::env::var(k).ok();
    compat::run(program, cwd, &env).unwrap_or_else(|| run(args, cwd))
}

/// Run `pkli` with `args` (without the program name) from `cwd`.
#[must_use]
pub fn run(args: &[String], cwd: &Path) -> Outcome {
    let Some((verb, rest)) = args.split_first() else {
        return exit(2, USAGE);
    };
    match (verb.as_str(), rest) {
        ("--version", []) => data(version()),
        ("confirm", []) => confirm::run(cwd),
        ("import", [doc]) => import::run(Path::new(doc)),
        ("gen" | "lay" | "seed", [] | [_]) => flagged(verb, rest.first(), cwd),
        _ => with_arguments(verb, rest, cwd),
    }
}

/// The verbs that parse their own arguments.
fn with_arguments(verb: &str, rest: &[String], cwd: &Path) -> Outcome {
    match verb {
        "check" => check::run(rest, cwd),
        "detect" => detect::run(rest, cwd),
        "map" => map::run(rest, cwd),
        "migrate" => migrate::run(rest, cwd),
        "protect" => protect::run(rest, cwd),
        "report" => report::run(rest, cwd),
        _ => exit(2, USAGE),
    }
}

/// cli V6: the crate version and the built-in catalog's fingerprint.
fn version() -> String {
    let catalog = crate::catalog::fingerprint();
    format!("pkli {} (catalog {catalog})\n", env!("CARGO_PKG_VERSION"))
}

/// `gen`, `lay` and `seed`, each with its one optional flag: `--check`,
/// `--dry-run`, `--init`.
fn flagged(verb: &str, flag: Option<&String>, cwd: &Path) -> Outcome {
    match (verb, flag.map(String::as_str)) {
        ("gen", None) => generate::run(cwd, false),
        ("gen", Some("--check")) => generate::run(cwd, true),
        ("lay", None) => lay::run(cwd),
        ("lay", Some("--dry-run")) => lay::plan(cwd),
        ("seed", None) => seed::run(cwd, false),
        ("seed", Some("--init")) => seed::run(cwd, true),
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

/// The paths `diff` changes under `root`.
fn changed(root: &Path, diff: &crate::scan::Diff) -> Result<Vec<String>, String> {
    crate::scan::changed(root, diff).map_err(|e| e.to_string())
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

/// A registry with everything it names resolved: the built-in catalog and
/// fragments with its own rows laid over them.
struct Loaded {
    registry: crate::registry::Registry,
    catalog: Vec<crate::catalog::Check>,
    fragments: Vec<crate::catalog::Fragment>,
    rules: Vec<crate::rule::Rule>,
    plurals: Vec<(String, String)>,
}

/// Read and resolve the registry at `path`. Root V11: no `.pklith` is an
/// error, never a pass.
fn load(path: &Path) -> Result<Loaded, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    parsed(&text)
}

/// A registry that adds nothing: the built-in catalog and fragments alone.
const BUILTIN_ONLY: &str = "format 1\n";

/// Registry `text`, parsed and resolved.
fn parsed(text: &str) -> Result<Loaded, String> {
    crate::registry::parse(text)
        .and_then(resolve)
        .map_err(|e| e.to_string())
}

/// Every type row and fragment naming a known check (registry V2, root
/// V17) and every rule parsed (rule V1), so every verb refuses the same
/// malformed registry.
fn resolve(registry: crate::registry::Registry) -> Result<Loaded, crate::registry::Error> {
    let local = crate::catalog::parse(&registry.checks)?;
    // The built-in catalog cannot fail to parse: its own tests keep it so.
    let catalog = crate::catalog::merge(crate::catalog::builtin()?, local)?;
    crate::catalog::known(&registry, &catalog)?;
    let fragments = crate::catalog::fragment::resolved(&registry.fragments, &catalog)?;
    let rules = crate::rule::parse(&registry.rules)?;
    let plurals = crate::rule::plurals(&registry.plural);
    Ok(Loaded {
        registry,
        catalog,
        fragments,
        rules,
        plurals,
    })
}

/// What detection works from. A named registry must exist; the default
/// `.pklith` may not yet, which means the built-in catalog alone, since
/// detection is how a repository gets its first.
fn detecting(named: Option<PathBuf>, root: &Path) -> Result<Loaded, String> {
    let default = root.join(".pklith");
    match named {
        Some(path) => load(&path),
        None if default.exists() => load(&default),
        None => parsed(BUILTIN_ONLY),
    }
}
