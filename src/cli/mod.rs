//! Argument dispatch, usage and exit codes (`src/cli/SPEC.md`). Every
//! verb's logic lives in its own node; this only wires them.

use std::path::{Path, PathBuf};

/// Printed on stderr for a usage error (V2).
pub const USAGE: &str = "usage: pkli <command>\n
  check [--root DIR] [--registry FILE]  every tracked file has a type in .pklith with its checks
  gen [--check]                         write hk.pklith.pkl from .pklith; --check: fail when it is stale
  lay [--dry-run]                       one commit per missing check, through the hooks; --dry-run: list them
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
    let Some((verb, rest)) = args.split_first() else {
        return exit(2, USAGE);
    };
    match (verb.as_str(), rest) {
        ("check", _) => check(rest, cwd),
        ("gen", []) => generate(cwd, false),
        ("lay", []) => lay_run(cwd),
        ("import", [doc]) => import(Path::new(doc)),
        ("gen" | "lay", [flag]) => flagged(verb, flag, cwd),
        _ => exit(2, USAGE),
    }
}

/// `gen --check` and `lay --dry-run`: each verb's one flag.
fn flagged(verb: &str, flag: &str, cwd: &Path) -> Outcome {
    match (verb, flag) {
        ("gen", "--check") => generate(cwd, true),
        ("lay", "--dry-run") => lay_plan(cwd),
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

/// The registry and its typed checks, with every type row naming a known
/// check (registry V2). Root V11: no `.pklith` is an error, never a pass.
fn load(path: &Path) -> Result<(crate::registry::Registry, Vec<crate::catalog::Check>), String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let registry = crate::registry::parse(&text).map_err(|e| e.to_string())?;
    let catalog = crate::catalog::parse(&registry.checks).map_err(|e| e.to_string())?;
    crate::catalog::known(&registry, &catalog).map_err(|e| e.to_string())?;
    Ok((registry, catalog))
}

/// `gen [--check]`: the repository's `hk.pklith.pkl` from its `.pklith`.
/// Writes only on change (gen V2); `--check` writes nothing and fails when
/// the file is stale or missing (gen V3).
fn generate(cwd: &Path, check: bool) -> Outcome {
    let generated = toplevel(cwd).and_then(|root| Ok((pkl_for(&root)?, root)));
    match (generated, check) {
        (Err(message), _) => exit(2, format!("pkli gen: {message}\n")),
        (Ok((text, root)), true) => freshness(&root, &text),
        (Ok((text, root)), false) => written(&root, &text),
    }
}

fn freshness(root: &Path, text: &str) -> Outcome {
    if crate::r#gen::fresh(root, text) {
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

fn pkl_for(root: &Path) -> Result<String, String> {
    let (registry, catalog) = load(&root.join(".pklith"))?;
    Ok(crate::r#gen::pkl(&crate::r#gen::used(&registry, &catalog)))
}

/// What both `lay` and `lay --dry-run` read.
struct LayInputs {
    root: PathBuf,
    registry: crate::registry::Registry,
    catalog: Vec<crate::catalog::Check>,
    present: Vec<String>,
    claimed: Vec<String>,
}

fn lay_inputs(cwd: &Path) -> Result<LayInputs, String> {
    let root = toplevel(cwd)?;
    let (registry, catalog) = load(&root.join(".pklith"))?;
    let files = files(&root, false)?;
    let present = present(&root)?;
    let claimed = crate::cover::claimed_somewhere(&files, &registry);
    Ok(LayInputs {
        root,
        registry,
        catalog,
        present,
        claimed,
    })
}

/// Step ids hk already runs. For planning, an hk.pkl with no steps yet is
/// a repository with nothing laid, not an error (hook V2 guards judging).
fn present(root: &Path) -> Result<Vec<String>, String> {
    match crate::hook::steps(root) {
        Ok(steps) => Ok(steps.into_iter().map(|s| s.id).collect()),
        Err(crate::hook::Error::NoSteps) => Ok(Vec::new()),
        Err(e) => Err(format!("cannot read hk's steps: {e}")),
    }
}

/// `lay --dry-run`: the subjects of the commits lay would make, in order,
/// on stdout (data), writing nothing (lay V4).
fn lay_plan(cwd: &Path) -> Outcome {
    let subjects = lay_inputs(cwd).map(|i| {
        let plan = crate::lay::plan(&i.registry, &i.catalog, &i.present, &i.claimed);
        plan.iter()
            .map(|c| crate::lay::subject(c) + "\n")
            .collect::<String>()
    });
    match subjects {
        Ok(stdout) => Outcome {
            code: 0,
            stdout,
            stderr: String::new(),
        },
        Err(message) => exit(2, format!("pkli lay: {message}\n")),
    }
}

fn laying(inputs: &LayInputs) -> Result<String, crate::lay::Failure> {
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
fn lay_run(cwd: &Path) -> Outcome {
    let inputs = match lay_inputs(cwd) {
        Ok(inputs) => inputs,
        Err(message) => return exit(2, format!("pkli lay: {message}\n")),
    };
    match laying(&inputs) {
        Ok(stdout) => Outcome {
            code: 0,
            stdout,
            stderr: String::new(),
        },
        Err(crate::lay::Failure::NotReady(m)) => exit(2, format!("pkli lay: {m}\n")),
        Err(crate::lay::Failure::RolledBack(m)) => exit(1, format!("pkli lay: {m}\n")),
    }
}
