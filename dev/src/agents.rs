//! `pklith-dev agents [--check]`: the step table in AGENTS.md, from the
//! steps hk.pkl gives each hook (root V14).

use crate::io::{Failed, read, run};
use std::fmt::Write as _;

/// Every push step as one line: id, `commit` or `push`, globs joined by
/// U+001F, and whether it fixes; tab-separated. Asking `pkl` for text
/// keeps the dev crate free of a JSON parser, as src/hook is.
const ROWS: &str = r#"let (fast = hooks["pre-commit"].steps) hooks["pre-push"].steps.toMap().entries.map((e) -> let (s = e.value) List(e.key, if (fast.containsKey(e.key)) "commit" else "push", (if (s.glob is String) List(s.glob) else s.glob ?? List()).join("\u{1F}"), if (s.fix != null) "yes" else "-").join("\t")).join("\n")"#;

/// The table block, markers included, for `rows` (as `ROWS` prints them)
/// and the number of commit and push steps.
#[must_use]
pub fn table(rows: &str, commit: usize, push: usize) -> String {
    let mut out = format!(
        "<!-- BEGIN steps: generated from hk.pkl by `cargo run -p pklith-dev -- agents`; do not edit -->\n{commit} steps run on every commit and {push} on push and `hk check`; the commit-msg hook checks the message.\n\n| step | runs on | files | fixes |\n|---|---|---|---|\n"
    );
    for line in rows.lines().filter(|l| !l.is_empty()) {
        let mut cells = line.split('\t');
        let mut next = || cells.next().unwrap_or_default();
        let (id, when, globs, fix) = (next(), next(), next(), next());
        let _ = writeln!(out, "| `{id}` | {when} | {} | {fix} |", files(globs));
    }
    out + "<!-- END steps -->\n"
}

/// Globs as code spans, or "whole tree" for a step without one.
fn files(globs: &str) -> String {
    if globs.is_empty() {
        return "whole tree".to_owned();
    }
    let spans: Vec<String> = globs.split('\u{1F}').map(|g| format!("`{g}`")).collect();
    spans.join(" ")
}

/// Render or check.
///
/// # Errors
///
/// Drift (1), `pkl` or a file failing (2).
pub fn run_verb(check: bool) -> Result<(), Failed> {
    let rows = run("pkl", &["eval", "-x", ROWS, "hk.pkl"])?;
    let (commit, push) = crate::io::hooks()?;
    let wanted = crate::block::splice(&read("AGENTS.md")?, "steps", &table(&rows, commit, push))
        .ok_or((2, "AGENTS.md has no <!-- BEGIN steps --> block".to_owned()))?;
    crate::io::update("AGENTS.md", &wanted, check, "agents")
}

#[cfg(test)]
mod tests {
    use super::table;

    #[test]
    fn rows_become_the_table() {
        let rows = "a\tcommit\t**/*.rs\u{1F}Cargo.toml\tyes\nb\tpush\t\t-\n";
        let got = table(rows, 1, 2);
        assert!(
            got.contains("1 steps run on every commit and 2 on push"),
            "{got}"
        );
        assert!(got.contains("| `a` | commit | `**/*.rs` `Cargo.toml` | yes |\n| `b` | push | whole tree | - |\n<!-- END steps -->\n"), "{got}");
    }
}
