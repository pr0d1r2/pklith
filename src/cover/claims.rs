//! Claim against runner (cover V3): every check the registry claims for a
//! file must be an hk step that has a check command and whose globs reach
//! that file. A claim no runner backs is the silent failure: the registry
//! says a file is checked while the step never sees it.

use super::{classes, resolve};
use crate::hook::Step;
use crate::registry::{Registry, TypeRow};
use crate::scan::Globs;

/// Files a claimed check does not reach, grouped by check and reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unbacked {
    /// The claimed check.
    pub check: String,
    /// Why its step does not run on these files.
    pub problem: String,
    /// The files, in scan order.
    pub files: Vec<String>,
}

/// Whether any type claims a check, so the steps must be read at all.
#[must_use]
pub fn claims_any(registry: &Registry) -> bool {
    registry.types.iter().any(|t| !t.checks.is_empty())
}

/// Every claim no step backs. A file's claims are its type's checks plus
/// the universal `*` checks, unless the type is exempt: an exempt type
/// claims nothing. Gaps are reported by `judge`, not here.
#[must_use]
pub fn unbacked(files: &[String], registry: &Registry, steps: &[Step]) -> Vec<Unbacked> {
    let declared: Vec<&TypeRow> = registry.types.iter().filter(|t| t.key != "*").collect();
    let (universal, classes) = (universal(registry), classes(&declared));
    let mut found: Vec<Unbacked> = Vec::new();
    for file in files {
        let row = resolve(file, &declared, &classes).and_then(|i| declared.get(i));
        for check in row.map(|r| claimed(r, &universal)).unwrap_or_default() {
            if let Some(problem) = problem(check, file, steps) {
                record(&mut found, check, problem, file);
            }
        }
    }
    found
}

/// Checks at least one file claims, each once, in first-claimed order. lay
/// lays only these (root V15): a check no file claims would be stale at
/// birth.
#[must_use]
pub fn claimed_somewhere(files: &[String], registry: &Registry) -> Vec<String> {
    let declared: Vec<&TypeRow> = registry.types.iter().filter(|t| t.key != "*").collect();
    let (universal, classes) = (universal(registry), classes(&declared));
    let mut seen: Vec<String> = Vec::new();
    for file in files {
        let row = resolve(file, &declared, &classes).and_then(|i| declared.get(i));
        for check in row.map(|r| claimed(r, &universal)).unwrap_or_default() {
            if !seen.contains(check) {
                seen.push(check.clone());
            }
        }
    }
    seen
}

fn universal(registry: &Registry) -> Vec<&String> {
    registry
        .types
        .iter()
        .filter(|t| t.key == "*")
        .flat_map(|t| &t.checks)
        .collect()
}

fn claimed<'a>(row: &'a TypeRow, universal: &[&'a String]) -> Vec<&'a String> {
    let spread = if row.exempt.is_some() {
        &[][..]
    } else {
        universal
    };
    row.checks.iter().chain(spread.iter().copied()).collect()
}

/// Why the step named `check` does not run on `file`, if it does not.
fn problem(check: &str, file: &str, steps: &[Step]) -> Option<String> {
    match steps.iter().find(|s| s.id == check) {
        None => Some("no hk step has this id".into()),
        Some(step) if !step.checks => Some("its hk step has no check command".into()),
        Some(step) => reach(step, file),
    }
}

/// Whether the runner reaches `file`: no globs means the whole tree.
fn reach(step: &Step, file: &str) -> Option<String> {
    let matches = |globs: &[String]| Globs::new(globs).is_ok_and(|g| g.matches(file));
    if !step.globs.is_empty() && !matches(&step.globs) {
        return Some(format!(
            "its hk step's globs ({}) do not reach them",
            step.globs.join(", ")
        ));
    }
    let excluded = step
        .exclude
        .iter()
        .find(|x| matches(std::slice::from_ref(*x)));
    excluded.map(|glob| format!("its hk step excludes them (`{glob}`)"))
}

fn record(found: &mut Vec<Unbacked>, check: &str, problem: String, file: &str) {
    match found
        .iter_mut()
        .find(|u| u.check == check && u.problem == problem)
    {
        Some(u) => u.files.push(file.to_owned()),
        None => found.push(Unbacked {
            check: check.to_owned(),
            problem,
            files: vec![file.to_owned()],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::{claims_any, unbacked};
    use crate::hook::Step;
    use crate::registry::{Error, parse};

    const REGISTRY: &str = "format 1\n## types\ntype|checks|min|exempt\n*|ws|-|-\nrs|clippy|-|-\nmd|links|-|-\nlock|nope|-|-\ntoml|fixonly|-|-\npath:vendor/**|-|-|vendored verbatim\n";

    const FILES: [&str; 9] = [
        "src/a.rs",
        "gen/g.rs",
        "README.md",
        "docs/x.md",
        "notes/n.md",
        "vendor/v.md",
        "Cargo.lock",
        "x.toml",
        "a.py",
    ];

    fn step(id: &str, globs: &[&str], exclude: &[&str], checks: bool) -> Step {
        let own = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect();
        Step {
            id: id.to_owned(),
            globs: own(globs),
            exclude: own(exclude),
            checks,
        }
    }

    fn steps() -> Vec<Step> {
        vec![
            step("ws", &[], &["gen/**"], true),
            step("clippy", &["*.rs"], &[], true),
            step("links", &["docs/*.md"], &[], true),
            step("fixonly", &["*.toml"], &[], false),
        ]
    }

    /// Every way a claim can lack a runner, each grouped with its files in
    /// scan order: excluded, globs that miss, no such step, a step without a
    /// check command. The exempt path class claims nothing, not even the
    /// universal `ws`; the gap (a.py) is judge's to report.
    #[test]
    fn every_unbacked_claim_is_named_with_its_files() -> Result<(), Error> {
        let files: Vec<String> = FILES.iter().map(|f| (*f).to_owned()).collect();
        let found = unbacked(&files, &parse(REGISTRY)?, &steps());
        let got: Vec<String> = found
            .iter()
            .map(|u| format!("{}: {} {:?}", u.check, u.problem, u.files))
            .collect();
        assert_eq!(
            got,
            [
                r#"ws: its hk step excludes them (`gen/**`) ["gen/g.rs"]"#,
                r#"links: its hk step's globs (docs/*.md) do not reach them ["README.md", "notes/n.md"]"#,
                r#"nope: no hk step has this id ["Cargo.lock"]"#,
                r#"fixonly: its hk step has no check command ["x.toml"]"#,
            ]
        );
        Ok(())
    }

    const IMPORTED: &str = "format 1\n## types\ntype|checks|min|exempt\nrs|-|-|legacy: clippy\n";

    #[test]
    fn steps_are_only_needed_when_something_is_claimed() -> Result<(), Error> {
        assert!(claims_any(&parse(REGISTRY)?));
        assert!(!claims_any(&parse(IMPORTED)?));
        Ok(())
    }
}

#[cfg(test)]
mod claimed_tests {
    use super::claimed_somewhere;
    use crate::registry::{Error, parse};

    const REGISTRY: &str = "format 1\n## types\ntype|checks|min|exempt\n*|ws|-|-\nrs|clippy,fmt|-|-\npy|ruff|-|-\npath:vendor/**|-|-|vendored\n";

    /// Only checks some file claims: no `.py` file means no ruff; the exempt
    /// class claims nothing; the universal check counts once.
    #[test]
    fn only_checks_a_file_claims_are_listed() -> Result<(), Error> {
        let files = ["a.rs", "vendor/b.py", "b.rs"].map(str::to_owned);
        assert_eq!(
            claimed_somewhere(&files, &parse(REGISTRY)?),
            ["clippy", "fmt", "ws"]
        );
        Ok(())
    }
}
