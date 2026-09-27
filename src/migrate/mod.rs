//! Lefthook to hk (`src/migrate/SPEC.md`), without losing a check: the
//! lefthook check set maps onto pklith checks, and the registry written
//! claims exactly that set (V1), or nothing is written.

use crate::catalog::{Check, Fragment};
use crate::scan::Globs;

pub mod lefthook;

/// Why migrate writes nothing (V1, V3).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Refusal {
    /// `(hook, command)` pairs no pklith check stands for.
    pub unmapped: Vec<(String, String)>,
    /// Checks the lefthook set needs that no tracked file reaches, so the
    /// registry could not claim them.
    pub unreached: Vec<String>,
}

/// The `.pklith` for a lefthook check set: every command mapped to its
/// pklith check (renamed per docs/PORT.md), less the ones `drop` names, each
/// claimed for the files its runner reaches.
///
/// # Errors
///
/// A [`Refusal`] naming every unmapped command and every check no file
/// reaches: the set after would differ from the set before (V1).
pub fn plan(
    commands: &[(String, String)],
    drop: &[String],
    files: &[String],
    catalog: &[Check],
) -> Result<String, Refusal> {
    let (ids, unmapped) = mapped(commands, drop, catalog);
    let (star, rows) = claim(&ids, files, catalog);
    let claimed = |i: &String| star.contains(i) || rows.iter().any(|r| row_claims(r, i));
    let unreached: Vec<String> = ids.iter().filter(|i| !claimed(i)).cloned().collect();
    if !unmapped.is_empty() || !unreached.is_empty() {
        return Err(Refusal {
            unmapped,
            unreached,
        });
    }
    Ok(render(&star, &rows, drop))
}

/// The pklith checks the kept commands stand for, each once, and the
/// commands none does.
fn mapped(
    commands: &[(String, String)],
    drop: &[String],
    catalog: &[Check],
) -> (Vec<String>, Vec<(String, String)>) {
    let (mut ids, mut unmapped) = (Vec::new(), Vec::new());
    for (hook, command) in commands.iter().filter(|(_, c)| !drop.contains(c)) {
        match catalog
            .iter()
            .find(|c| c.id == crate::catalog::renamed(command))
        {
            Some(check) if !ids.contains(&check.id) => ids.push(check.id.clone()),
            Some(_) => {}
            None => unmapped.push((hook.clone(), command.clone())),
        }
    }
    (ids, unmapped)
}

/// Seed's row builder over one fragment holding `ids`: the `*` checks and
/// the type rows.
fn claim(ids: &[String], files: &[String], catalog: &[Check]) -> (Vec<String>, Vec<String>) {
    let triggers = Globs::none();
    let all = Fragment {
        line: 0,
        id: "lefthook".into(),
        always: false,
        triggers,
        checks: ids.to_vec(),
        seed: Vec::new(),
    };
    crate::seed::init::rows(files, &[&all], catalog)
}

/// Whether a `type|checks|min|exempt` row lists `id`.
fn row_claims(row: &str, id: &str) -> bool {
    row.split('|')
        .nth(1)
        .is_some_and(|checks| checks.split(", ").any(|c| c == id))
}

fn render(star: &[String], rows: &[String], drop: &[String]) -> String {
    let dropped = dropped(drop);
    let star = if star.is_empty() {
        String::new()
    } else {
        format!("*|{}|-|-\n", star.join(", "))
    };
    format!(
        "format 1\n\n# Migrated from lefthook.yml by `pklith migrate`: the same checks, now hk\n# steps. It is yours now: pklith never rewrites it.\n{dropped}\n## types\ntype|checks|min|exempt\n{star}{}\n",
        rows.join("\n")
    )
}

/// The header line recording what `--drop` let go, if anything.
fn dropped(drop: &[String]) -> String {
    if drop.is_empty() {
        return String::new();
    }
    format!(
        "#\n# Dropped from lefthook on purpose (`--drop`): {}.\n",
        drop.join(", ")
    )
}

#[cfg(test)]
mod tests;
