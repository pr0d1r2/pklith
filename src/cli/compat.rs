//! `pklith` run as `lefthook-linter-coverage`, `lefthook-linter-coverage-full`
//! or `lefthook-unit-coverage` (legacy T5): the legacy tool's environment,
//! messages and exit codes (legacy V2, §C), so a consumer swaps the
//! package and nothing else. Arguments are ignored, as they were.

use super::{Outcome, exit};
use crate::legacy::compat::{self, BASE, FULL, UNIT};
use crate::legacy::unit_coverage;
use std::path::{Path, PathBuf};

/// Where each compat entry reads its environment.
type Env<'a> = &'a dyn Fn(&str) -> Option<String>;

/// The outcome when `program`'s basename is a legacy tool's name.
pub(super) fn run(program: &str, cwd: &Path, env: Env) -> Option<Outcome> {
    // `${VAR:-default}`: an empty variable is an unset one.
    let var = |k: &str| env(k).filter(|v| !v.is_empty());
    let name = Path::new(program).file_name()?.to_str()?;
    let tool = match name {
        BASE => |cwd: &Path, var: Env| linter(&BASE_TOOL, cwd, var),
        FULL => |cwd: &Path, var: Env| linter(&FULL_TOOL, cwd, var),
        UNIT => unit,
        _ => return None,
    };
    Some(no_root(name, cwd, &var).unwrap_or_else(|| tool(cwd, &var)))
}

/// The legacy tools began with `cd "$ROOT" || exit 1`: a `*_ROOT` naming no
/// directory fails first, as bash's cd did.
fn no_root(name: &str, cwd: &Path, var: Env) -> Option<Outcome> {
    let key = match name {
        FULL => "LEFTHOOK_LINTER_COVERAGE_ROOT",
        UNIT => "LEFTHOOK_UNIT_COVERAGE_ROOT",
        _ => return None,
    };
    let dir = var(key)?;
    let gone = format!("{name}: cd: {dir}: No such file or directory\n");
    (!cwd.join(&dir).is_dir()).then(|| exit(1, gone))
}

/// The root the tool works in, and whether it walks the tree there
/// (a `*_ROOT` variable) rather than listing tracked files.
struct Root {
    dir: PathBuf,
    walk: bool,
}

/// The repository's top level, as `git rev-parse --show-toplevel` in the
/// caller's git environment; `cwd` when there is none, as the legacy
/// `|| pwd` did. A git that cannot run fails again, loudly, in `listed`.
fn root(cwd: &Path, walk: Option<&String>) -> Root {
    let top = || {
        let out = git(cwd, &["rev-parse", "--show-toplevel"]).ok().flatten();
        out.map_or_else(|| cwd.to_path_buf(), |t| PathBuf::from(t.trim_end()))
    };
    Root {
        dir: walk.map_or_else(top, |r| cwd.join(r)),
        walk: walk.is_some(),
    }
}

/// The files the tool saw: `find . -type f ! -path './.git/*'` when
/// walking, else every entry `git ls-files` lists, symlinks and submodules
/// included, and nothing outside a repository.
fn listed(root: &Root) -> Result<Vec<String>, String> {
    if root.walk {
        let mut found = Vec::new();
        find(&root.dir, "", &mut found)?;
        found.sort();
        return Ok(found);
    }
    let text = git(&root.dir, &["ls-files", "-z"])?.unwrap_or_default();
    Ok(text
        .split('\0')
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
        .collect())
}

/// `find . -type f ! -path './.git/*'` under `dir`, paths relative to the
/// root as `prefix`: regular files only (not symlinks, sockets or FIFOs),
/// symlinked directories not entered, and only the top-level `.git/`
/// skipped. A directory that cannot be read is an error, where find
/// carried on with a partial list (root V1).
fn find(dir: &Path, prefix: &str, found: &mut Vec<String>) -> Result<(), String> {
    let failed = |e: std::io::Error| format!("cannot read {}: {e}", dir.display());
    for entry in std::fs::read_dir(dir).map_err(failed)? {
        let entry = entry.map_err(failed)?;
        let path = format!("{prefix}{}", entry.file_name().to_string_lossy());
        let kind = entry.file_type().map_err(failed)?;
        if kind.is_file() {
            found.push(path);
        } else if kind.is_dir() && path != ".git" {
            find(&entry.path(), &format!("{path}/"), found)?;
        }
    }
    Ok(())
}

/// Run git as the legacy tools did: in the caller's git environment, so
/// `GIT_DIR`, `GIT_WORK_TREE` and a commit's `GIT_INDEX_FILE` are honoured
/// (pklith's own verbs scrub them; a drop-in must not). `None` outside a
/// repository; any other failure, git missing included, is an error, never
/// an empty list that passes (root V1).
fn git(dir: &Path, args: &[&str]) -> Result<Option<String>, String> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    let stderr = String::from_utf8_lossy(&out.stderr);
    if out.status.success() {
        Ok(Some(String::from_utf8_lossy(&out.stdout).into_owned()))
    } else if stderr.contains("not a git repository") {
        Ok(None)
    } else {
        Err(format!("git {} failed: {}", args.join(" "), stderr.trim()))
    }
}

/// How one linter coverage tool was configured and spoke.
struct Linter {
    prefix: &'static str,
    /// The dash in "missing -- cannot verify".
    dash: &'static str,
    /// The default document; `None` when the variable is required.
    default: Option<&'static str>,
    /// The variable that makes the tool walk a root, if it read one.
    root: Option<&'static str>,
    uncovered: fn(&[String], &[String]) -> Vec<String>,
}

const BASE_TOOL: Linter = Linter {
    prefix: "linter-coverage",
    dash: "--",
    default: Some("docs/linter-coverage.md"),
    root: None,
    uncovered: compat::uncovered_base,
};

const FULL_TOOL: Linter = Linter {
    prefix: "check-linter-coverage",
    dash: "\u{2014}",
    default: None,
    root: Some("LEFTHOOK_LINTER_COVERAGE_ROOT"),
    uncovered: compat::uncovered_full,
};

const NOT_SET: &str = "check-linter-coverage: LEFTHOOK_LINTER_COVERAGE_DOC not set\n  Set it to the path of your linter documentation file\n  (relative to repo root), e.g.:\n    export LEFTHOOK_LINTER_COVERAGE_DOC=docs/linters.md\n";

fn linter(tool: &Linter, cwd: &Path, var: Env) -> Outcome {
    let root = root(cwd, tool.root.and_then(var).as_ref());
    let doc = var("LEFTHOOK_LINTER_COVERAGE_DOC").or_else(|| tool.default.map(str::to_owned));
    let Some(doc) = doc else {
        return exit(1, NOT_SET);
    };
    if !root.dir.join(&doc).is_file() {
        let (prefix, dash) = (tool.prefix, tool.dash);
        let missing = format!("{prefix}: {doc} missing {dash} cannot verify coverage\n");
        return exit(1, missing);
    }
    match uncovered(tool, &root, &doc) {
        Ok(missing) if missing.is_empty() => exit(0, ""),
        Ok(missing) => exit(1, compat::listing(tool.prefix, &doc, &missing)),
        Err(e) => exit(2, format!("{}: {doc}: {e}\n", tool.prefix)),
    }
}

/// What the document leaves unlisted. One that parses to nothing is an
/// error, never every type reported unlisted (legacy V5, B2).
fn uncovered(tool: &Linter, root: &Root, doc: &str) -> Result<Vec<String>, String> {
    let text = std::fs::read_to_string(root.dir.join(doc)).map_err(|e| e.to_string())?;
    let tokens = crate::legacy::read(&text).map_err(|e| e.to_string())?;
    let keys: Vec<String> = tokens.into_iter().map(|t| t.key).collect();
    Ok((tool.uncovered)(&keys, &listed(root)?))
}

fn unit(cwd: &Path, var: Env) -> Outcome {
    let root = root(cwd, var("LEFTHOOK_UNIT_COVERAGE_ROOT").as_ref());
    let config = var("LEFTHOOK_UNIT_COVERAGE_CONFIG");
    let config = config.unwrap_or_else(|| ".unit-coverage.toml".to_owned());
    let path = root.dir.join(&config);
    if !path.is_file() {
        let create = "  Create a .unit-coverage.toml with [[rules]] entries.";
        return exit(1, format!("{UNIT}: {config} not found\n{create}\n"));
    }
    match judged(&root, &path) {
        Ok((text, failed)) => exit(u8::from(failed), text),
        Err(e) => exit(2, format!("{UNIT}: {config}: {e}\n")),
    }
}

/// The legacy verdict over the files the tool saw.
fn judged(root: &Root, config: &Path) -> Result<(String, bool), String> {
    let read = |p: &Path| std::fs::read_to_string(p).map_err(|e| e.to_string());
    let config = unit_coverage::config(&read(config)?)?;
    let allowlist = root.dir.join(&config.allowlist);
    let allowed = if allowlist.is_file() {
        read(&allowlist)?
    } else {
        String::new()
    };
    let exists = |spec: &str| root.dir.join(spec).is_file();
    let files = listed(root)?;
    let allow = unit_coverage::allowed(&allowed);
    Ok(unit_coverage::verdict(&config, &files, &allow, exists))
}
