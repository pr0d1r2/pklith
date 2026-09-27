//! `pklith-dev notices` against a fixture crate named `pklith`, locked
//! offline from the local registry cache: every way it can pass, drift or
//! fail, through the built binary.

use std::path::{Path, PathBuf};
use std::process::Command;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const CARGO: &str = "[package]\nname = \"pklith\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\nmemchr = \"2\"\n";
const DOC: &str = "# Notices\n\nships **1 direct dependency**, which pulls in 1 crates.\n\n<!-- BEGIN crates -->\n<!-- END crates -->\n\nMIT.\n";

/// A locked crate with one dependency, and its notices doc.
fn fixture(name: &str, cargo: &str) -> Result<PathBuf> {
    let dir =
        std::env::temp_dir().join(format!("pklith-dev-notices-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src"))?;
    std::fs::create_dir_all(dir.join("docs"))?;
    std::fs::write(dir.join("Cargo.toml"), cargo)?;
    std::fs::write(dir.join("src/lib.rs"), "")?;
    std::fs::write(dir.join("docs/THIRD-PARTY-NOTICES.md"), DOC)?;
    let lock = Command::new("cargo")
        .args(["generate-lockfile", "--offline", "-q"])
        .current_dir(&dir)
        .status()?;
    assert!(lock.success(), "the registry cache lacks memchr");
    Ok(dir)
}

fn notices(dir: &Path, check: bool) -> Result<(Option<i32>, String)> {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_pklith-dev"));
    cmd.arg("notices").current_dir(dir);
    if check {
        cmd.arg("--check");
    }
    let out = cmd.output()?;
    Ok((out.status.code(), String::from_utf8(out.stderr)?))
}

fn doc(dir: &Path) -> Result<String> {
    Ok(std::fs::read_to_string(
        dir.join("docs/THIRD-PARTY-NOTICES.md"),
    )?)
}

/// The table renders from the lock; a check then passes; an edited row
/// drifts.
#[test]
fn the_table_renders_then_checks() -> Result {
    let dir = fixture("render", CARGO)?;
    assert_eq!(notices(&dir, false)?, (Some(0), String::new()));
    assert!(doc(&dir)?.contains("| memchr | 2."), "{}", doc(&dir)?);
    assert_eq!(notices(&dir, true)?.0, Some(0));
    std::fs::write(
        dir.join("docs/THIRD-PARTY-NOTICES.md"),
        doc(&dir)?.replace("| memchr | 2.", "| memchr | 1."),
    )?;
    let (code, stderr) = notices(&dir, true)?;
    assert!(
        code == Some(1) && stderr.contains("drifted from its sources"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

fn set_doc(dir: &Path, text: &str) -> Result {
    Ok(std::fs::write(
        dir.join("docs/THIRD-PARTY-NOTICES.md"),
        text,
    )?)
}

/// A count the prose gets wrong is exit 1, before anything is written.
#[test]
fn a_wrong_count_in_the_prose_fails() -> Result {
    let dir = fixture("count", CARGO)?;
    set_doc(&dir, &DOC.replace("1 crates", "3 crates"))?;
    let (code, stderr) = notices(&dir, false)?;
    assert!(
        code == Some(1) && stderr.contains("must say **1 direct dependency**"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A doc without the block, and a directory cargo cannot read, are 2.
#[test]
fn a_missing_block_or_manifest_exits_2() -> Result {
    let dir = fixture("block", CARGO)?;
    set_doc(&dir, "**1 direct dependency**, which pulls in 1 crates\n")?;
    let (code, stderr) = notices(&dir, true)?;
    assert!(
        code == Some(2) && stderr.contains("no <!-- BEGIN crates --> block"),
        "{stderr}"
    );
    std::fs::remove_file(dir.join("Cargo.toml"))?;
    assert_eq!(notices(&dir, true)?.0, Some(2));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A crate with no dependencies has no table to render: exit 2.
#[test]
fn no_dependencies_exits_2() -> Result {
    let bare = fixture(
        "bare",
        "[package]\nname = \"pklith\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )?;
    let (code, stderr) = notices(&bare, true)?;
    assert!(
        code == Some(2) && stderr.contains("listed no dependencies"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(bare)?)
}
