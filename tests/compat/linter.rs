//! The legacy linter coverage bats suites (legacy T1, V2): the base tool
//! against a git repository, `-full` against a walked root. The parser
//! cases (`parse-coverage-doc.bats`) are `legacy::tests` (V1); the
//! `dev.sh` cases test a dev shell pklith does not ship.

use super::{Result, run, scratch, tracked, write};

const BASE: &str = "lefthook-linter-coverage";
const DOC: &str = "docs/linter-coverage.md";
const FIX: &str = "\nFix: add a row to the extension table in";

/// One base-tool case: the repository's files, all tracked, and the
/// whole stderr and exit code the legacy tool gave.
fn base(name: &str, doc: &str, files: &[(&str, &str)], want: (i32, &str)) -> Result {
    let dir = scratch(name, BASE)?;
    let repo = dir.join("repo");
    write(&repo, files)?;
    tracked(&repo)?;
    let got = run(&dir, BASE, &repo, &[("LEFTHOOK_LINTER_COVERAGE_DOC", doc)])?;
    assert_eq!(got, (Some(want.0), want.1.to_owned()), "{name}");
    Ok(std::fs::remove_dir_all(dir)?)
}

#[test]
fn missing_doc_file_fails() -> Result {
    let want = "linter-coverage: docs/linter-coverage.md missing -- cannot verify coverage\n";
    base("missing-doc", DOC, &[("file.txt", "x")], (1, want))
}

#[test]
fn all_extensions_listed_passes() -> Result {
    let doc = "| Extension | Linter |\n|-----------|--------|\n| `.md` | markdownlint |\n| `.txt` | none |\n| `.sh` | shellcheck |\n";
    let files = [
        (DOC, doc),
        ("file.txt", "hello"),
        ("script.sh", "#!/bin/bash"),
    ];
    base("all-listed", DOC, &files, (0, ""))
}

/// The doc itself is `.md` and unlisted too: the legacy tool said so.
#[test]
fn missing_extension_fails() -> Result {
    let doc = "| Extension | Linter |\n|-----------|--------|\n| `.txt` | none |\n";
    let files = [(DOC, doc), ("file.txt", "hello"), ("file.json", "data")];
    let want = format!(
        "linter-coverage: 2 extension(s) not listed in {DOC}:\n  .json\n  .md\n{FIX} {DOC} with an\nassigned linter or an explicit exempt reason.\n"
    );
    base("missing-ext", DOC, &files, (1, &want))
}

#[test]
fn bare_filename_listed_passes() -> Result {
    let doc = "| Extension | Linter |\n|-----------|--------|\n| `.md` | markdownlint |\n| `Makefile` | make lint |\n";
    base("bare", DOC, &[(DOC, doc), ("Makefile", "all:")], (0, ""))
}

#[test]
fn custom_doc_path_via_env_var() -> Result {
    let doc = "| Extension | Linter |\n|-----------|--------|\n| `.md` | markdownlint |\n| `.txt` | none |\n";
    let files = [("custom/linters.md", doc), ("file.txt", "hello")];
    base("custom-doc", "custom/linters.md", &files, (0, ""))
}

const FULL: &str = "lefthook-linter-coverage-full";
const LINTERS: &str = "| Extension | Linter |\n|-----------|--------|\n| `.sh` | shellcheck |\n| `.nix` | statix |\n| `.md` | markdownlint |\n";

/// The `-full` suite's setup.
const SETUP: [(&str, &str); 4] = [
    ("linters.md", LINTERS),
    ("src/main.sh", ""),
    ("src/config.nix", ""),
    ("README.md", ""),
];

/// One `-full` case over the bats setup (`linters.md`, `src/main.sh`,
/// `src/config.nix`, `README.md`) plus `extra`, walking the root.
fn full(name: &str, doc: Option<&str>, extra: &[(&str, &str)], want: (i32, &str)) -> Result {
    let dir = scratch(name, FULL)?;
    let repo = dir.join("repo");
    write(&repo, &SETUP)?;
    write(&repo, extra)?;
    let root = repo.to_str().unwrap_or_default();
    let mut env = vec![("LEFTHOOK_LINTER_COVERAGE_ROOT", root)];
    env.extend(doc.map(|d| ("LEFTHOOK_LINTER_COVERAGE_DOC", d)));
    let got = run(&dir, FULL, &repo, &env)?;
    assert_eq!(got, (Some(want.0), want.1.to_owned()), "{name}");
    Ok(std::fs::remove_dir_all(dir)?)
}

#[test]
fn fails_when_doc_not_set() -> Result {
    let want = "check-linter-coverage: LEFTHOOK_LINTER_COVERAGE_DOC not set\n  Set it to the path of your linter documentation file\n  (relative to repo root), e.g.:\n    export LEFTHOOK_LINTER_COVERAGE_DOC=docs/linters.md\n";
    full("full-unset", None, &[], (1, want))
}

#[test]
fn fails_when_doc_file_missing() -> Result {
    let want = "check-linter-coverage: nonexistent.md missing \u{2014} cannot verify coverage\n";
    full("full-missing", Some("nonexistent.md"), &[], (1, want))
}

#[test]
fn passes_when_all_extensions_covered() -> Result {
    full("full-pass", Some("linters.md"), &[], (0, ""))
}

#[test]
fn fails_when_extension_not_listed() -> Result {
    let want = format!(
        "check-linter-coverage: 1 extension(s) not listed in linters.md:\n  .json\n{FIX} linters.md with an\nassigned linter or an explicit exempt reason.\n"
    );
    full(
        "full-json",
        Some("linters.md"),
        &[("data.json", "")],
        (1, &want),
    )
}

#[test]
fn handles_multiple_extensions_in_one_table_cell() -> Result {
    let doc = "| Extension | Linter |\n|-----------|--------|\n| `.sh`, `.bats` | shellcheck |\n| `.nix` | statix |\n| `.md` | markdownlint |\n";
    let extra = [("linters.md", doc), ("test.bats", "")];
    full("full-cell", Some("linters.md"), &extra, (0, ""))
}

#[test]
fn handles_extensionless_files_like_justfile() -> Result {
    let doc = format!("{LINTERS}| `justfile` | custom |\n");
    let extra = [("linters.md", doc.as_str()), ("justfile", "")];
    full("full-justfile", Some("linters.md"), &extra, (0, ""))
}
