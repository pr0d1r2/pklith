//! Read the legacy linter coverage document and import it as `.pklith`
//! (`src/legacy/SPEC.md`), so repositories move off the legacy tools with
//! no change in verdict.

use std::fmt::Write as _;

pub mod compat;
pub mod fragment_map;
pub mod json;
pub mod toml;
pub mod unit_coverage;

/// One backtick token from a legacy table, with the rest of its row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// The token, leading `.` stripped, as the legacy awk printed it.
    pub key: String,
    /// Column 2, the linter the row names; `-` meant exempt by convention.
    pub linter: String,
    /// Column 3, notes; the exemption reason when column 2 is `-`.
    pub notes: String,
}

/// Tokens exactly as the legacy awk read them (V1, R3): rows starting with
/// `|`, column 1 only, every backtick-quoted token, leading `.` stripped.
/// Parsed in-process, never through awk (V4).
#[must_use]
pub fn tokens(doc: &str) -> Vec<Token> {
    doc.lines()
        .filter_map(|l| l.strip_prefix('|'))
        .flat_map(row_tokens)
        .collect()
}

/// A row's tokens; one with no `|` closing its first cell has none, as
/// the awk skipped it (`if (end == 0) next`).
fn row_tokens(row: &str) -> Vec<Token> {
    let Some((first, rest)) = row.split_once('|') else {
        return Vec::new();
    };
    let mut cells = rest.split('|').map(str::trim);
    let (linter, notes) = (
        cells.next().unwrap_or_default(),
        cells.next().unwrap_or_default(),
    );
    let make = |t: &str| Token {
        key: t.strip_prefix('.').unwrap_or(t).to_owned(),
        linter: linter.to_owned(),
        notes: notes.to_owned(),
    };
    backticked(first).into_iter().map(make).collect()
}

/// Every token the awk regex `` `[^`]+` `` finds in a cell, leftmost
/// first: an empty pair is no token, and its second backtick may open one
/// (two backticks, a space and a backtick hold a one-space token).
fn backticked(cell: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut rest = cell;
    while let Some((_, after)) = rest.split_once('`') {
        let Some((token, next)) = after.split_once('`') else {
            break;
        };
        if token.is_empty() {
            rest = after;
        } else {
            found.push(token);
            rest = next;
        }
    }
    found
}

/// Why a legacy document could not be imported.
#[derive(Debug, PartialEq, Eq)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

/// A `.pklith` for the legacy document: one type row per token, first wins
/// (the legacy tools accepted a token twice; the registry does not).
///
/// Every row is an exemption, because the legacy tools checked only that a
/// type was LISTED, and that is the verdict to carry over (root V4). Column
/// 2 `-` keeps the notes as its reason; any other linter becomes
/// `legacy: <linter>` until a catalog check replaces it (V3).
///
/// # Errors
///
/// An [`Error`] when the document has table rows but no token (V5): a
/// parser that read nothing must not report every type uncovered, which
/// is what the legacy `-full` tool did (B2).
pub fn import(doc: &str) -> Result<String, Error> {
    Ok(registry(&read(doc)?))
}

/// The document's tokens, refusing one that parses to nothing (V5).
///
/// # Errors
///
/// An [`Error`] when the document has table rows but no token.
pub fn read(doc: &str) -> Result<Vec<Token>, Error> {
    let tokens = tokens(doc);
    if tokens.is_empty() && doc.lines().any(|l| l.starts_with('|')) {
        return Err(Error(
            "parsed nothing: the document has table rows but no backtick tokens in column 1".into(),
        ));
    }
    Ok(tokens)
}

/// One exempt type row per distinct key, first wins.
fn registry(tokens: &[Token]) -> String {
    let mut out = String::from("format 1\n\n## types\ntype|checks|min|exempt\n");
    let mut seen: Vec<&str> = Vec::new();
    for token in tokens {
        if seen.contains(&token.key.as_str()) {
            continue;
        }
        seen.push(&token.key);
        let _ = writeln!(out, "{}|-|-|{}", escape(&token.key), escape(&reason(token)));
    }
    out
}

fn reason(token: &Token) -> String {
    match (token.linter.as_str(), token.notes.as_str()) {
        ("-" | "", "") => "legacy: listed as exempt".to_owned(),
        ("-" | "", notes) => notes.to_owned(),
        (linter, _) => format!("legacy: {linter}"),
    }
}

fn escape(cell: &str) -> String {
    cell.replace('|', "\\|")
}

#[cfg(test)]
mod tests {
    use super::{import, tokens};

    fn keys(doc: &str) -> Vec<String> {
        tokens(doc).into_iter().map(|t| t.key).collect()
    }

    /// The legacy bats cases (V1): dotted extensions lose their dot, bare
    /// names stay, several tokens share a cell, the header and prose lines
    /// are skipped, an empty doc has no tokens.
    #[test]
    fn tokens_match_the_legacy_parser() {
        let doc = "# Linters\n\nprose with `.nope`\n| Extension | Linter | Notes |\n|---|---|---|\n| `.rb` | RuboCop | x |\n| `.sh`, `.bats` | shellcheck | |\n| `justfile` | custom | bare |\n";
        assert_eq!(keys(doc), ["rb", "sh", "bats", "justfile"]);
        assert!(keys("").is_empty());
    }

    /// The awk regex needs both backticks and one character between them,
    /// matching leftmost: after an empty pair, its second backtick opens
    /// the next token. A row whose first cell has no closing `|` is
    /// skipped, as the awk skipped it.
    #[test]
    fn backticks_and_pipes_are_read_as_the_awk_read_them() {
        assert_eq!(keys("| `a` `` `b |\n"), ["a", " "]);
        assert_eq!(keys("| `a` `b\n"), Vec::<String>::new());
        assert_eq!(keys("| `.rb`\n| `sh` |\n"), ["sh"]);
    }

    const DOC: &str = "| Extension | Linter | Notes |\n|---|---|---|\n| `.lock` | - | Nix flake lock |\n| `LICENSE` | - | |\n| `.rb`, `.lock` | RuboCop | x |\n| `.md` | markdownlint \\| typos | |\n";

    const IMPORTED: &str = "format 1\n\n## types\ntype|checks|min|exempt\nlock|-|-|Nix flake lock\nLICENSE|-|-|legacy: listed as exempt\nrb|-|-|legacy: RuboCop\nmd|-|-|legacy: markdownlint \\\n";

    /// V3: `-` keeps the notes as the reason, a linter becomes `legacy:
    /// <linter>`, a repeated token keeps its first row. Every row is exempt
    /// because the legacy verdict was "listed" (root V4).
    #[test]
    fn import_carries_the_listed_verdict_over() -> Result<(), Box<dyn std::error::Error>> {
        let text = import(DOC)?;
        assert_eq!(text, IMPORTED);
        assert!(crate::registry::parse(&text).is_ok());
        Ok(())
    }

    /// V5, from legacy -full B2: a doc with table rows and no token is an
    /// error, never an import that marks every type uncovered.
    #[test]
    fn a_doc_that_parses_to_nothing_is_an_error() {
        let err = import("| Extension | Linter |\n|---|---|\n")
            .err()
            .map(|e| e.to_string());
        assert_eq!(
            err.as_deref(),
            Some("parsed nothing: the document has table rows but no backtick tokens in column 1")
        );
        assert!(import("no table here\n").is_ok());
    }
}
