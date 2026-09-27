//! What the legacy linter coverage tools reported (legacy T5, V2, root R1
//! R2): which file types a repository has that the document does not
//! list, printed the way each tool printed them. Sorted bytewise, as
//! `sort -u` sorts under `LC_ALL=C`.

use std::collections::BTreeSet;
use std::fmt::Write as _;

/// The name the base tool ran as.
pub const BASE: &str = "lefthook-linter-coverage";
/// The name the `-full` tool ran as.
pub const FULL: &str = "lefthook-linter-coverage-full";
/// The name the unit coverage tool ran as.
pub const UNIT: &str = "lefthook-unit-coverage";

fn basenames(files: &[String]) -> impl Iterator<Item = &str> {
    files.iter().filter_map(|f| f.rsplit('/').next())
}

/// The base tool (R1): `.ext` for every extension not listed, then every
/// bare (dotless) basename not listed.
#[must_use]
pub fn uncovered_base(listed: &[String], files: &[String]) -> Vec<String> {
    let unlisted = |k: &&str| !k.is_empty() && !listed.iter().any(|l| l == k);
    let exts: BTreeSet<&str> = basenames(files)
        .filter_map(|b| b.rsplit_once('.').map(|(_, e)| e))
        .filter(unlisted)
        .collect();
    let bare: BTreeSet<&str> = basenames(files)
        .filter(|b| !b.contains('.'))
        .filter(unlisted)
        .collect();
    let dotted = exts.into_iter().map(|e| format!(".{e}"));
    dotted.chain(bare.into_iter().map(str::to_owned)).collect()
}

/// The `-full` tool (R2): the text after the last dot, the whole name
/// when there is none, always printed with a leading `.`.
#[must_use]
pub fn uncovered_full(listed: &[String], files: &[String]) -> Vec<String> {
    let keys: BTreeSet<&str> = basenames(files)
        .filter_map(|b| b.rsplit('.').next())
        .filter(|k| !k.is_empty() && !listed.iter().any(|l| l == k))
        .collect();
    keys.into_iter().map(|k| format!(".{k}")).collect()
}

/// The report both tools printed on stderr before exiting 1.
#[must_use]
pub fn listing(prefix: &str, doc: &str, missing: &[String]) -> String {
    let mut out = format!(
        "{prefix}: {} extension(s) not listed in {doc}:\n",
        missing.len()
    );
    for item in missing {
        let _ = writeln!(out, "  {item}");
    }
    let _ = write!(
        out,
        "\nFix: add a row to the extension table in {doc} with an\nassigned linter or an explicit exempt reason.\n"
    );
    out
}

#[cfg(test)]
mod tests {
    use super::{listing, uncovered_base, uncovered_full};

    fn owned(items: &[&str]) -> Vec<String> {
        items.iter().map(|i| (*i).to_owned()).collect()
    }

    const FILES: [&str; 7] = [
        "a/x.tar.gz",
        "b.rb",
        ".envrc",
        "justfile",
        "c/Makefile",
        "d.",
        "e.rb",
    ];

    /// R1: the last extension, a dotfile's name, bare names apart and
    /// after the extensions, an empty extension skipped.
    #[test]
    fn the_base_tool_splits_extensions_from_bare_names() {
        let got = uncovered_base(&owned(&["rb", "justfile"]), &owned(&FILES));
        assert_eq!(got, [".envrc", ".gz", "Makefile"]);
    }

    /// R2: a bare name is its own key and printed with a dot.
    #[test]
    fn the_full_tool_dots_every_key() {
        let got = uncovered_full(&owned(&["rb", "justfile"]), &owned(&FILES));
        assert_eq!(got, [".Makefile", ".envrc", ".gz"]);
    }

    /// The legacy report, word for word.
    #[test]
    fn the_listing_is_the_legacy_text() {
        let want = "p: 2 extension(s) not listed in d.md:\n  .a\n  b\n\nFix: add a row to the extension table in d.md with an\nassigned linter or an explicit exempt reason.\n";
        assert_eq!(listing("p", "d.md", &owned(&[".a", "b"])), want);
    }
}
