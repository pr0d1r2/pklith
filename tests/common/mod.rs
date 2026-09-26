//! Helpers shared by the `pkli` integration tests: throwaway repositories
//! and a way to run the built binary. Each test file uses a subset.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

pub const SCHEMA: &str = include_str!("../../pkl/Config.pkl");

/// hk config backing the `lint` claim: its step runs on every `.rs` file.
pub const HK: &str = "amends \"pkl/Config.pkl\"\nhooks { [\"check\"] { steps { [\"lint\"] { glob = \"*.rs\"; check = \"true\" } } } }\n";

/// A registry covering the fixture: `rs` linted, `.pklith` itself exempt.
pub const OK: &str = "format 1\n## checks\nid|category|nix|glob|check|fix|env|msg\nlint|lint|-|*|true|-|-|m\n## types\ntype|checks|min|exempt\nrs|lint|-|-\npklith|-|-|the registry itself\npkl|-|-|hk config\n";

pub fn pkli(dir: &Path, args: &[&str]) -> Result<(Option<i32>, String, String)> {
    let out = Command::new(env!("CARGO_BIN_EXE_pkli"))
        .args(args)
        .current_dir(dir)
        .output()?;
    Ok((
        out.status.code(),
        String::from_utf8(out.stdout)?,
        String::from_utf8(out.stderr)?,
    ))
}

pub fn temp(name: &str) -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("pklith-cli-{}-{name}", std::process::id()));
    std::fs::create_dir_all(dir.join("sub"))?;
    std::fs::create_dir_all(dir.join("pkl"))?;
    std::fs::write(dir.join("pkl/Config.pkl"), SCHEMA)?;
    std::fs::write(dir.join("hk.pkl"), HK)?;
    std::fs::write(dir.join("a.rs"), "")?;
    std::fs::write(dir.join("sub/b.rs"), "")?;
    Ok(dir)
}

/// A git repository holding `a.rs`, `sub/b.rs` and, if given, `.pklith`.
pub fn repo(name: &str, pklith: Option<&str>) -> Result<PathBuf> {
    let dir = temp(name)?;
    if let Some(text) = pklith {
        std::fs::write(dir.join(".pklith"), text)?;
    }
    for args in [&["init", "-q"][..], &["add", "-A"]] {
        Command::new("git").args(args).current_dir(&dir).output()?;
    }
    Ok(dir)
}

pub fn check_in(dir: &Path) -> Result<(Option<i32>, String)> {
    let (code, stdout, stderr) = pkli(dir, &["check"])?;
    assert_eq!(stdout, "", "stdout carries data only");
    Ok((code, stderr))
}
