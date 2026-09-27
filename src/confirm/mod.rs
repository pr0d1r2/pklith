//! Acceptance of a materialized repository (`src/confirm/SPEC.md`),
//! read-only (V1): is what pklith laid complete, faithful, runnable?

use crate::catalog::Check;
use std::path::Path;

/// One failed sub-check (V3): which, where, what, and what to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// completeness, fidelity, coherence, executability or idempotence.
    pub part: &'static str,
    /// The file at fault.
    pub file: String,
    /// What is wrong.
    pub what: String,
    /// How to fix it.
    pub fix: String,
}

/// What confirm judges.
pub struct Inputs<'a> {
    /// Repository root.
    pub root: &'a Path,
    /// The checks the registry uses, in emit order.
    pub used: &'a [&'a Check],
    /// The checks `pklith lay` would still lay.
    pub missing: &'a [&'a Check],
}

/// Every failed sub-check; empty when the repository is accepted. A
/// sub-check that cannot run fails too (V2).
#[must_use]
pub fn confirm(inputs: &Inputs<'_>) -> Vec<Finding> {
    let mut found = completeness(inputs.missing);
    found.extend(fidelity(inputs.root, inputs.used));
    found.extend(coherence(inputs.root, inputs.used));
    found.extend(executability(inputs.root));
    found.extend(idempotence(&|| crate::r#gen::outputs(inputs.used)));
    found
}

fn finding(part: &'static str, file: &str, what: String, fix: &str) -> Finding {
    Finding {
        part,
        file: file.to_owned(),
        what,
        fix: fix.to_owned(),
    }
}

/// Every check the registry claims has an hk step.
fn completeness(missing: &[&Check]) -> Vec<Finding> {
    let what = |c: &&Check| format!("`{}` is claimed but hk runs no step for it", c.id);
    let one = |c: &&Check| {
        finding(
            "completeness",
            crate::r#gen::FILE,
            what(c),
            "run `pklith lay`",
        )
    };
    missing.iter().map(one).collect()
}

/// The generated files are exactly what `pklith gen` writes.
fn fidelity(root: &Path, used: &[&Check]) -> Option<Finding> {
    let file = crate::r#gen::stale(root, used)?;
    let what = "differs from what `pklith gen` writes".to_owned();
    Some(finding(
        "fidelity",
        file,
        what,
        "run `pklith gen` and commit the result",
    ))
}

/// Every program a step runs is on PATH, or a script in the repository.
fn coherence(root: &Path, used: &[&Check]) -> Vec<Finding> {
    used.iter().flat_map(|check| absent(root, check)).collect()
}

/// A finding per program `check` runs that is not there to run.
fn absent(root: &Path, check: &Check) -> Vec<Finding> {
    let gone = crate::r#gen::programs(&check.check)
        .into_iter()
        .filter(|p| !runnable(root, p));
    let what = |p: String| format!("`{}` runs `{p}`, which is not on PATH", check.id);
    let fix = "enter the dev shell that imports nix/pklith.nix";
    gone.map(|p| finding("coherence", crate::r#gen::NIX, what(p), fix))
        .collect()
}

/// A program on PATH, or a path from the root, that is an executable file.
fn runnable(root: &Path, program: &str) -> bool {
    use std::os::unix::fs::PermissionsExt as _;
    let executable = |p: std::path::PathBuf| {
        p.metadata()
            .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    };
    if program.contains('/') {
        return executable(root.join(program));
    }
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path).any(|dir| executable(dir.join(program)))
}

/// hk.pkl evaluates to steps, and `hk validate` accepts it.
fn executability(root: &Path) -> Option<Finding> {
    let problem = match crate::hook::steps(root) {
        Err(e) => Some(format!("does not evaluate to steps: {e}")),
        Ok(_) => validate(root)
            .err()
            .map(|e| format!("hk validate refused it: {e}")),
    };
    let fix = "make hk.pkl evaluate: `hk validate` names the problem";
    problem.map(|what| finding("executability", "hk.pkl", what, fix))
}

fn validate(root: &Path) -> Result<Vec<u8>, crate::proc::Error> {
    crate::proc::output(crate::proc::command("hk", root).arg("validate"))
}

/// The files gen renders for some checks.
type Rendered = [(&'static str, String); 2];

/// Rendering the same checks twice gives the same bytes (root V3).
/// `render` is gen's renderer; taking it lets a test plant one that is not
/// deterministic.
fn idempotence(render: &dyn Fn() -> Rendered) -> Option<Finding> {
    let (first, second) = (render(), render());
    let (path, _) = first.iter().zip(&second).find(|(a, b)| a != b)?.0;
    let what = "renders differently on a second run".to_owned();
    let fix = "report it: pklith's output must be deterministic";
    Some(finding("idempotence", path, what, fix))
}

#[cfg(test)]
mod tests {
    use super::idempotence;
    use std::cell::Cell;

    /// A renderer whose output changes between runs is caught, naming the
    /// file; gen's own renderer is not.
    #[test]
    fn a_renderer_that_changes_its_mind_is_not_idempotent() {
        let runs = Cell::new(0);
        let drifting = || {
            runs.set(runs.get() + 1);
            [
                ("hk.pklith.pkl", runs.get().to_string()),
                ("nix/pklith.nix", String::new()),
            ]
        };
        let found = idempotence(&drifting).map(|f| (f.part, f.file));
        assert_eq!(found, Some(("idempotence", "hk.pklith.pkl".to_owned())));
        assert!(idempotence(&|| crate::r#gen::outputs(&[])).is_none());
    }
}
