//! `pkli check`: the gate verdict.

use super::{Options, Outcome, USAGE, exit, files, load, options, toplevel};
use crate::scan::Diff;
use std::path::{Path, PathBuf};

pub(super) fn run(args: &[String], cwd: &Path) -> Outcome {
    let Some((diff, opts)) = diff_flag(args).and_then(|(d, rest)| Some((d, options(&rest)?)))
    else {
        return exit(2, USAGE);
    };
    match judge(opts, diff, cwd) {
        Ok((coverage, unbacked)) => {
            let text = crate::report::text(&coverage) + &crate::report::unbacked(&unbacked);
            exit(u8::from(!coverage.ok() || !unbacked.is_empty()), text)
        }
        Err(message) => exit(2, format!("pkli check: {message}\n")),
    }
}

/// `--staged` or `--range A..B`, at most one, and the other arguments.
fn diff_flag(args: &[String]) -> Option<(Option<Diff>, Vec<String>)> {
    let mut rest = args.to_vec();
    let staged = take_flag(&mut rest, "--staged");
    let range = take_value(&mut rest, "--range").ok()?;
    match (staged, range) {
        (true, Some(_)) => None,
        (true, None) => Some((Some(Diff::Staged), rest)),
        (false, range) => Some((range.map(Diff::Range), rest)),
    }
}

/// Remove `flag` from `args`, saying whether it was there; a repeated flag
/// stays behind for the option parser to refuse.
fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
    let at = args.iter().position(|a| a == flag);
    at.map(|i| args.remove(i)).is_some()
}

/// Remove `flag` and its value from `args`; a flag without its value is an
/// error.
fn take_value(args: &mut Vec<String>, flag: &str) -> Result<Option<String>, String> {
    let Some(i) = args.iter().position(|a| a == flag) else {
        return Ok(None);
    };
    args.remove(i);
    (i < args.len())
        .then(|| args.remove(i))
        .map(Some)
        .ok_or(format!("{flag} needs a value"))
}

/// What one run judges.
struct Scope {
    root: PathBuf,
    registry: crate::registry::Registry,
    catalog: Vec<crate::catalog::Check>,
    files: Vec<String>,
    /// Paths the diff touches, when one was given.
    changed: Option<Vec<String>>,
    /// `--staged`: judge only what the commit touches (cover V8).
    staged: bool,
}

fn scope(opts: Options, diff: Option<Diff>, cwd: &Path) -> Result<Scope, String> {
    let walk = opts.root.is_some();
    let root = opts.root.map_or_else(|| toplevel(cwd), Ok)?;
    let (registry, catalog) = load(&opts.registry.unwrap_or_else(|| root.join(".pklith")))?;
    let files = files(&root, walk)?;
    let staged = diff == Some(Diff::Staged);
    let changed = diff.map(|d| crate::scan::changed(&root, &d));
    let changed = changed.transpose().map_err(|e| e.to_string())?;
    Ok(Scope {
        root,
        registry,
        catalog,
        files,
        changed,
        staged,
    })
}

type Verdict = (crate::cover::Coverage, Vec<crate::cover::Unbacked>);

/// The full verdict, or with `--staged` only what the commit touches
/// (cover V8, root V26). A diff also feeds `changed` rules (rule V3).
fn judge(opts: Options, diff: Option<Diff>, cwd: &Path) -> Result<Verdict, String> {
    let s = scope(opts, diff, cwd)?;
    let unbacked = backing(&s)?;
    let mut coverage = match &s.changed {
        Some(paths) if s.staged => crate::cover::judge_staged(&s.files, paths, &s.registry),
        _ => crate::cover::judge(&s.files, &s.registry),
    };
    coverage.failed = rules(&s)?;
    coverage.unreflected = reflect(&s, &coverage.gaps)?;
    Ok((coverage, unbacked))
}

/// The files judged: the changed ones still in the tree when staged (a
/// deleted file claims nothing), else the whole tree.
fn judged(s: &Scope) -> Vec<String> {
    match &s.changed {
        Some(paths) if s.staged => paths
            .iter()
            .filter(|p| s.files.binary_search(p).is_ok())
            .cloned()
            .collect(),
        _ => s.files.clone(),
    }
}

/// cover V3: claims no hk step backs. hk.pkl is read only when something
/// is claimed; an imported legacy registry claims nothing.
fn backing(s: &Scope) -> Result<Vec<crate::cover::Unbacked>, String> {
    if !crate::cover::claims_any(&s.registry) {
        return Ok(Vec::new());
    }
    let steps = crate::hook::steps(&s.root).map_err(|e| format!("cannot read hk's steps: {e}"))?;
    Ok(crate::cover::unbacked(&judged(s), &s.registry, &steps))
}

/// cover V7: the companion rules the tree fails; `changed` rules run only
/// with a diff (rule V3), and in staged mode only changed files are
/// sources.
fn rules(s: &Scope) -> Result<Vec<crate::rule::Failure>, String> {
    let rules = crate::rule::parse(&s.registry.rules).map_err(|e| e.to_string())?;
    let plurals: Vec<(String, String)> = s.registry.plural.iter().map(pair).collect();
    let read = |path: &str| std::fs::read_to_string(s.root.join(path)).ok();
    let input = crate::rule::Input {
        files: &s.files,
        diff: s.changed.as_deref(),
        read: &read,
        plurals: &plurals,
        only_changed: s.staged,
    };
    Ok(crate::rule::evaluate(&rules, &input))
}

/// Cover V9: fragments the gaps switch on that `.pklith` does not
/// reflect, each with the rows `pkli seed --init` would add for those
/// files. A registry that claims nothing (a legacy import) is not asked.
fn reflect(
    s: &Scope,
    gaps: &[crate::cover::Gap],
) -> Result<Vec<crate::cover::Unreflected>, String> {
    if !crate::cover::claims_any(&s.registry) {
        return Ok(Vec::new());
    }
    let fragments = crate::catalog::fragment::resolved(&s.registry.fragments, &s.catalog);
    let fragments = fragments.map_err(|e| e.to_string())?;
    let active = crate::detect::active(&judged(s), &fragments);
    let found = crate::cover::unreflected(&active, &s.registry, gaps);
    Ok(found
        .into_iter()
        .map(|f| suggest(f, gaps, &s.catalog))
        .collect())
}

fn suggest(
    fragment: &crate::catalog::Fragment,
    gaps: &[crate::cover::Gap],
    catalog: &[crate::catalog::Check],
) -> crate::cover::Unreflected {
    let files: Vec<String> = gaps.iter().flat_map(|g| g.files.clone()).collect();
    let (star, rows) = crate::seed::init::rows(&files, &[fragment], catalog);
    let star = (!star.is_empty()).then_some(format!("*|{}|-|-", star.join(", ")));
    let rows = star
        .into_iter()
        .chain(rows)
        .filter(|r| !r.ends_with("|binary asset"))
        .collect();
    crate::cover::Unreflected {
        fragment: fragment.id.clone(),
        rows,
    }
}

/// A `## plural` row, which the registry keeps at its header's two cells.
fn pair(row: &crate::registry::Row) -> (String, String) {
    let [one, many] = <[String; 2]>::try_from(row.cells.clone()).unwrap_or_default();
    (one, many)
}
