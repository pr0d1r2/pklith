//! `pkli` run as `lefthook-linter-coverage`, `lefthook-linter-coverage-full`
//! or `lefthook-unit-coverage` (legacy T5): the legacy tool's environment,
//! messages and exit codes (legacy V2, §C), so a consumer swaps the
//! package and nothing else. Arguments are ignored, as they were.

use super::{Outcome, exit, toplevel};
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
    Some(match name {
        BASE => linter(&BASE_TOOL, cwd, &var),
        FULL => linter(&FULL_TOOL, cwd, &var),
        UNIT => unit(cwd, &var),
        _ => return None,
    })
}

/// The root the tool works in, and whether it walks the tree there
/// (a `*_ROOT` variable) rather than listing tracked files.
struct Root {
    dir: PathBuf,
    walk: bool,
}

fn root(cwd: &Path, walk: Option<&String>) -> Root {
    let dir = walk.map_or_else(
        || toplevel(cwd).unwrap_or_else(|_| cwd.to_path_buf()),
        |r| cwd.join(r),
    );
    Root {
        dir,
        walk: walk.is_some(),
    }
}

/// The files the tool saw: `find . -type f ! -path './.git/*'` when
/// walking, else `git ls-files`, which lists nothing outside a repository.
fn listed(root: &Root) -> Result<Vec<String>, String> {
    if root.walk {
        return crate::scan::walked(&root.dir).map_err(|e| e.to_string());
    }
    Ok(crate::scan::tracked(&root.dir).unwrap_or_default())
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
