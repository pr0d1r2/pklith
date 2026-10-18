//! `tdd-order` over a history (V6-V8): every commit that changes a
//! selected file changes its target in that commit or an earlier one.

use super::template::Plurals;
use super::{Failure, Rule};
use crate::scan::Commit;
use std::collections::HashSet;

/// Every failure of the `tdd-order` rules among `rules` over `history`,
/// oldest commit first (V6). Only the history is judged, so a commit
/// before it is never re-judged (V7); deletions demand nothing and a
/// deleted target vouches for nothing (V8).
#[must_use]
pub fn order(rules: &[Rule], history: &[Commit], plurals: &Plurals) -> Vec<Failure> {
    rules
        .iter()
        .filter(|r| r.ordered)
        .flat_map(|r| judged(r, history, plurals))
        .collect()
}

fn judged<'h>(rule: &Rule, history: &'h [Commit], plurals: &Plurals) -> Vec<Failure> {
    let mut seen: HashSet<&'h str> = HashSet::new();
    let mut failed = Vec::new();
    for commit in history {
        seen.extend(commit.changed.iter().map(String::as_str));
        failed.extend(late(rule, commit, &seen, plurals));
    }
    failed
}

/// The sources `commit` changes whose targets have not changed by then:
/// not in `seen`, which already holds the commit's own paths.
fn late(rule: &Rule, commit: &Commit, seen: &HashSet<&str>, plurals: &Plurals) -> Vec<Failure> {
    let sources = commit.changed.iter().filter(|s| rule.selects(s));
    sources
        .map(|s| (s, rule.target.render(s, plurals)))
        .filter(|(_, t)| !seen.contains(t.as_str()))
        .map(|(s, t)| failure(rule, s, &t, commit))
        .collect()
}

/// `source`, changed in `commit`, before its `target` changed.
fn failure(rule: &Rule, source: &str, target: &str, commit: &Commit) -> Failure {
    Failure {
        rule: rule.id.clone(),
        source: source.to_owned(),
        missing: format!("{target} changed in or before commit {}", commit.name),
    }
}
