//! `pklith-dev cli` against a fixture doc, through the built binary.

use std::path::{Path, PathBuf};
use std::process::Command;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn fixture(name: &str, doc: &str) -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("pklith-dev-cli-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("docs"))?;
    std::fs::write(dir.join("docs/CLI.md"), doc)?;
    Ok(dir)
}

fn cli(dir: &Path, check: bool) -> Result<(Option<i32>, String)> {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_pklith-dev"));
    cmd.arg("cli").current_dir(dir);
    if check {
        cmd.arg("--check");
    }
    let out = cmd.output()?;
    Ok((out.status.code(), String::from_utf8(out.stderr)?))
}

/// The usage renders as pkli prints it; a check passes; a hand edit drifts.
#[test]
fn the_usage_renders_then_checks() -> Result {
    let dir = fixture(
        "render",
        "# CLI\n\n<!-- BEGIN usage -->\n<!-- END usage -->\n",
    )?;
    assert_eq!(cli(&dir, false)?, (Some(0), String::new()));
    let text = std::fs::read_to_string(dir.join("docs/CLI.md"))?;
    assert!(text.contains(pklith::cli::USAGE.trim_end()), "{text}");
    assert_eq!(cli(&dir, true)?.0, Some(0));
    std::fs::write(dir.join("docs/CLI.md"), text.replace("--dry-run", "--try"))?;
    assert_eq!(cli(&dir, true)?.0, Some(1));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A doc without the block is exit 2.
#[test]
fn a_missing_block_exits_2() -> Result {
    let dir = fixture("block", "# CLI\n")?;
    let (code, stderr) = cli(&dir, true)?;
    assert!(
        code == Some(2) && stderr.contains("no <!-- BEGIN usage --> block"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}
