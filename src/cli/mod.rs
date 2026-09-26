//! Argument dispatch, usage and exit codes (`src/cli/SPEC.md`). Every
//! verb's logic lives in its own node; this only wires them.

use std::path::{Path, PathBuf};

/// Printed on stderr for a usage error (V2).
pub const USAGE: &str = "usage: pkli <command>\n
  check [--root DIR] [--registry FILE]  every tracked file has a type in .pklith with its checks
  import DOC                            a .pklith from a legacy linter coverage document, on stdout
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

/// Run `pkli` with `args` (without the program name) from `cwd`.
#[must_use]
pub fn run(args: &[String], cwd: &Path) -> Outcome {
    match args.split_first() {
        Some((verb, rest)) if verb == "check" => check(rest, cwd),
        Some((verb, [doc])) if verb == "import" => import(Path::new(doc)),
        _ => exit(2, USAGE),
    }
}

/// `import DOC`: the `.pklith` text is data, so it goes to stdout (V3).
fn import(doc: &Path) -> Outcome {
    let imported = std::fs::read_to_string(doc)
        .map_err(|e| format!("cannot read {}: {e}", doc.display()))
        .and_then(|text| crate::legacy::import(&text).map_err(|e| e.to_string()));
    match imported {
        Ok(text) => Outcome {
            code: 0,
            stdout: text,
            stderr: String::new(),
        },
        Err(message) => exit(2, format!("pkli import: {message}\n")),
    }
}

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

fn check(args: &[String], cwd: &Path) -> Outcome {
    let Some(opts) = options(args) else {
        return exit(2, USAGE);
    };
    match judge(opts, cwd) {
        Ok(coverage) => exit(u8::from(!coverage.ok()), crate::report::text(&coverage)),
        Err(message) => exit(2, format!("pkli check: {message}\n")),
    }
}

fn judge(opts: Options, cwd: &Path) -> Result<crate::cover::Coverage, String> {
    let walk = opts.root.is_some();
    let root = opts.root.map_or_else(|| toplevel(cwd), Ok)?;
    let registry = load(&opts.registry.unwrap_or_else(|| root.join(".pklith")))?;
    let catalog = crate::catalog::parse(&registry.checks).map_err(|e| e.to_string())?;
    crate::catalog::known(&registry, &catalog).map_err(|e| e.to_string())?;
    Ok(crate::cover::judge(&files(&root, walk)?, &registry))
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

/// Root V11: no `.pklith` is an error, never a pass.
fn load(path: &Path) -> Result<crate::registry::Registry, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    crate::registry::parse(&text).map_err(|e| e.to_string())
}
