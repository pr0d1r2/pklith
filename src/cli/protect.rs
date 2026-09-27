//! `pkli protect [--dry-run] [--repo OWNER/NAME] [--branch B]
//! [--accept-removals]`: branch protection's required contexts from CI job
//! names (src/protect). pklith's one network verb, opt-in by name (root
//! §C): it reaches GitHub through `gh api` and nothing else.

use super::{Outcome, USAGE, data, exit, files, toplevel};
use std::path::Path;

/// What the flags ask for.
#[derive(Default)]
struct Ask {
    dry_run: bool,
    accept_removals: bool,
    repo: Option<String>,
    branch: Option<String>,
}

/// `--dry-run`: the payload on stdout and no call at all (protect V1).
/// Otherwise the change is shown on stderr before it is applied, and a
/// context that would stop being required is refused unless accepted
/// (V2).
pub(super) fn run(args: &[String], cwd: &Path) -> Outcome {
    let Some(ask) = ask(args) else {
        return exit(2, USAGE);
    };
    let derived = toplevel(cwd).and_then(|root| derived(&root).map(|c| (root, c)));
    match derived {
        Ok((_, contexts)) if ask.dry_run => data(crate::protect::payload(&contexts)),
        Ok((root, contexts)) => applied(&root, &ask, &contexts),
        Err(message) => exit(2, format!("pkli protect: {message}\n")),
    }
}

fn ask(args: &[String]) -> Option<Ask> {
    let mut ask = Ask::default();
    let mut words = args.iter();
    while let Some(word) = words.next() {
        match word.as_str() {
            "--dry-run" => ask.dry_run = true,
            "--accept-removals" => ask.accept_removals = true,
            "--repo" => ask.repo = Some(words.next()?.clone()),
            "--branch" => ask.branch = Some(words.next()?.clone()),
            _ => return None,
        }
    }
    Some(ask)
}

/// The contexts the tracked workflows report.
fn derived(root: &Path) -> Result<Vec<String>, String> {
    let patterns = [".github/workflows/*.yml", ".github/workflows/*.yaml"].map(str::to_owned);
    let globs = crate::scan::Globs::new(&patterns).map_err(|e| e.to_string())?;
    let read = |f: &String| std::fs::read_to_string(root.join(f)).map(|t| (f.clone(), t));
    let tracked = files(root, false)?;
    let workflows = tracked
        .iter()
        .filter(|f| globs.matches(f))
        .map(read)
        .collect::<Result<Vec<_>, _>>();
    crate::protect::contexts(&workflows.map_err(|e| format!("cannot read a workflow: {e}"))?)
}

fn applied(root: &Path, ask: &Ask, contexts: &[String]) -> Outcome {
    match apply(root, ask, contexts) {
        Ok(shown) => exit(0, shown),
        Err((code, message)) => exit(code, message),
    }
}

/// Show the diff, refuse unaccepted removals, then PATCH.
fn apply(root: &Path, ask: &Ask, contexts: &[String]) -> Result<String, (u8, String)> {
    let failed = |e: String| (2, format!("pkli protect: {e}\n"));
    let path = endpoint(root, ask).map_err(failed)?;
    let current: Vec<String> = gh(root, &["api", &path, "--jq", ".contexts[]"], None)
        .map_err(failed)?
        .lines()
        .map(str::to_owned)
        .collect();
    let (added, removed) = crate::protect::diff(&current, contexts);
    let shown = shown(&added, &removed);
    if !removed.is_empty() && !ask.accept_removals {
        let why = "pkli protect: applying would stop requiring a context (a renamed or removed job); pass --accept-removals once that is meant\n";
        return Err((1, shown + why));
    }
    patch(root, &path, contexts).map_err(failed)?;
    Ok(shown)
}

/// Set `contexts` as the required checks at `path`.
fn patch(root: &Path, path: &str, contexts: &[String]) -> Result<String, String> {
    let body = crate::protect::payload(contexts);
    gh(
        root,
        &["api", "-X", "PATCH", path, "--input", "-"],
        Some(&body),
    )
}

/// The required-status-checks endpoint of the repository and branch.
fn endpoint(root: &Path, ask: &Ask) -> Result<String, String> {
    let view = [
        "repo",
        "view",
        "--json",
        "nameWithOwner",
        "--jq",
        ".nameWithOwner",
    ];
    let repo = ask.repo.clone().map_or_else(|| gh(root, &view, None), Ok)?;
    let branch = ask.branch.as_deref().unwrap_or("main");
    Ok(format!(
        "repos/{}/branches/{branch}/protection/required_status_checks",
        repo.trim()
    ))
}

fn shown(added: &[String], removed: &[String]) -> String {
    let plus = added.iter().map(|c| format!("protect: + {c}\n"));
    plus.chain(removed.iter().map(|c| format!("protect: - {c}\n")))
        .collect()
}

/// Run `gh` in `root`, with `input` on stdin when given; its stdout.
fn gh(root: &Path, args: &[&str], input: Option<&str>) -> Result<String, String> {
    let mut cmd = crate::proc::command("gh", root);
    cmd.args(args);
    let out = match input {
        Some(body) => crate::proc::output_with(&mut cmd, body.as_bytes()),
        None => crate::proc::output(&mut cmd),
    };
    out.map(|o| String::from_utf8_lossy(&o).into_owned())
        .map_err(|e| e.to_string())
}
