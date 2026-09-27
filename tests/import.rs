//! `pklith import` and `check --registry`, run against the built binary.

mod common;

use common::{Result, pklith, repo, temp};

const LEGACY: &str = "| Extension | Linter | Notes |\n|---|---|---|\n| `.rs` | clippy | |\n| `.pkl` | - | hk config |\n";

const IMPORTED: &str =
    "format 1\n\n## types\ntype|checks|min|exempt\nrs|-|-|legacy: clippy\npkl|-|-|hk config\n";

/// `import` prints a .pklith on stdout: it is data (cli V3).
#[test]
fn import_prints_a_registry_on_stdout() -> Result {
    let dir = temp("import")?;
    std::fs::write(dir.join("legacy.md"), LEGACY)?;
    let (code, stdout, stderr) = pklith(&dir, &["import", "legacy.md"])?;
    assert_eq!(
        (code, stdout.as_str(), stderr.as_str()),
        (Some(0), IMPORTED, "")
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// `check --registry` judges a repository without writing into it: the
/// fleet sweep (T70) relies on that.
#[test]
fn a_registry_outside_the_repository_judges_it() -> Result {
    let dir = repo("registry", None)?;
    let registry = std::env::temp_dir().join(format!("pklith-cli-{}-imported", std::process::id()));
    std::fs::write(&registry, IMPORTED)?;
    let (code, _, stderr) = pklith(&dir, &["check", "--registry", &registry.to_string_lossy()])?;
    assert_eq!((code, stderr.as_str()), (Some(0), ""));
    std::fs::remove_file(registry)?;
    Ok(std::fs::remove_dir_all(dir)?)
}

const IMPORT_ERRORS: [(&[&str], &str); 5] = [
    (&["gen", "--bogus"], "usage: pklith"),
    (
        &["import", "no-such.md"],
        "pklith import: cannot read no-such.md: ",
    ),
    (
        &["import", "empty-table.md"],
        "pklith import: parsed nothing: ",
    ),
    (&["import"], "usage: pklith"),
    (&["check", "--root", "a", "--root", "b"], "usage: pklith"),
];

/// Unreadable or empty documents, and malformed arguments, exit 2.
#[test]
fn import_and_option_errors_exit_2() -> Result {
    let dir = temp("import-errors")?;
    std::fs::write(dir.join("empty-table.md"), "| a | b |\n|---|---|\n")?;
    for (args, prefix) in IMPORT_ERRORS {
        let (code, stdout, stderr) = pklith(&dir, args)?;
        assert_eq!((code, stdout.as_str()), (Some(2), ""), "{args:?}");
        assert!(stderr.starts_with(prefix), "{args:?}: {stderr}");
    }
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Root T51: a set-and-setting fragment map, as `nix eval --json` prints
/// it, imports to type rows pklith reads back.
#[test]
fn a_fragment_map_imports_to_type_rows() -> Result {
    let dir = temp("import-map")?;
    let map = r#"{"coveragePerFileClass":{"all":["gitleaks"],"sh":["shellcheck","no-shell-functions"]},"unlintedFileClasses":{"lock":"generated"}}"#;
    std::fs::write(dir.join("map.json"), map)?;
    let (code, stdout, _) = pklith(&dir, &["import", "map.json"])?;
    assert_eq!(code, Some(0));
    assert!(
        stdout.ends_with(
            "type|checks|min|exempt\n*|ripsecrets|-|-\nsh|shellcheck|-|-\nlock|-|-|generated\n"
        ),
        "{stdout}"
    );
    std::fs::write(dir.join("bad.json"), "{")?;
    assert_eq!(pklith(&dir, &["import", "bad.json"])?.0, Some(2));
    Ok(std::fs::remove_dir_all(dir)?)
}
