//! `pklith-dev readme` against a fixture repository: every way it can
//! pass, drift or fail, through the built binary.

use std::path::{Path, PathBuf};
use std::process::Command;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const SCHEMA: &str = include_str!("../../pkl/Config.pkl");
const HK: &str = "amends \"pkl/Config.pkl\"\nlocal fast = new Mapping<String, Step> { [\"a\"] { check = \"true\" } }\nhooks {\n  [\"pre-commit\"] { steps = fast }\n  [\"pre-push\"] { steps = (fast) { [\"b\"] { check = \"true\" } } }\n}\n";
const CARGO: &str = "[package]\nname = \"x\"\nedition = \"2024\"\nrust-version = \"1.95\"\nlicense = \"MIT\"\nrepository = \"https://github.com/o/r\"\n\n[dependencies]\nglobset = \"0.4\"\n";
const LOCK: &str = "{\n  \"nodes\": {\n    \"nixpkgs\": {\n      \"locked\": {\n        \"lastModified\": 1790340521,\n        \"rev\": \"f5c082a40f7571c266e74e80ae2e68aadd8a9fc7\"\n      },\n      \"original\": {\n        \"ref\": \"nixos-26.05\"\n      }\n    }\n  }\n}\n";
const CI: &str = "jobs:\n  gate:\n    strategy:\n      matrix:\n        os: [macos-latest]\n";
const DISCLAIMER: &str = "**1 steps on commit, 2 on push**, **1 `§V` invariants**, **1 `§B` bugs**, a floor of 100% here.\n";
const README: &str = "# x\n\n<!-- BEGIN badges -->\n<!-- END badges -->\n\nbody\n";

/// The fixture's files, whose sources agree with each other.
const FILES: [(&str, &str); 9] = [
    ("pkl/Config.pkl", SCHEMA),
    ("hk.pkl", HK),
    ("Cargo.toml", CARGO),
    (".coverage", "lines 100.00\n"),
    ("flake.lock", LOCK),
    (".github/workflows/ci.yml", CI),
    ("docs/LLM-DISCLAIMER.md", DISCLAIMER),
    ("README.md", README),
    ("SPEC.md", "V1: a\nB1|d|c|f\n"),
];

/// A tracked fixture repository holding `FILES`.
fn fixture(name: &str) -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("pklith-dev-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, text) in FILES {
        write(&dir, path, text)?;
    }
    for args in [&["init", "-q"][..], &["add", "-A"]] {
        Command::new("git").args(args).current_dir(&dir).status()?;
    }
    Ok(dir)
}

fn write(dir: &Path, path: &str, text: &str) -> Result {
    let path = dir.join(path);
    std::fs::create_dir_all(path.parent().unwrap_or(dir))?;
    Ok(std::fs::write(path, text)?)
}

/// Run `pklith-dev` in `dir` with `args`: its exit code and stderr.
fn dev(dir: &Path, args: &[&str], path: Option<&str>) -> Result<(Option<i32>, String)> {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_pklith-dev"));
    cmd.args(args).current_dir(dir);
    if let Some(p) = path {
        cmd.env("PATH", p);
    }
    let out = cmd.output()?;
    Ok((out.status.code(), String::from_utf8(out.stderr)?))
}

/// Writing renders the block in place; a check then passes.
#[test]
fn readme_renders_then_checks() -> Result {
    let dir = fixture("render")?;
    assert_eq!(dev(&dir, &["readme"], None)?, (Some(0), String::new()));
    let readme = std::fs::read_to_string(dir.join("README.md"))?;
    assert!(
        readme.contains("[![gate steps 1 commit / 2 push]"),
        "{readme}"
    );
    assert!(
        readme.ends_with("<!-- END badges -->\n\nbody\n"),
        "{readme}"
    );
    assert_eq!(dev(&dir, &["readme", "--check"], None)?.0, Some(0));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A hand-edited badge is drift: `--check` fails, naming the fix.
#[test]
fn an_edited_badge_drifts() -> Result {
    let dir = fixture("drift")?;
    dev(&dir, &["readme"], None)?;
    let readme = std::fs::read_to_string(dir.join("README.md"))?;
    std::fs::write(
        dir.join("README.md"),
        readme.replace("MSRV 1.95", "MSRV 1.0"),
    )?;
    let (code, stderr) = dev(&dir, &["readme", "--check"], None)?;
    assert!(
        code == Some(1) && stderr.contains("README.md badges drifted"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// The disclaimer's numbers must match the sources, or nothing is written.
#[test]
fn stale_disclaimer_prose_fails() -> Result {
    let dir = fixture("prose")?;
    write(
        &dir,
        "docs/LLM-DISCLAIMER.md",
        &DISCLAIMER.replace("**1 `§V`", "**9 `§V`"),
    )?;
    let (code, stderr) = dev(&dir, &["readme"], None)?;
    assert_eq!(code, Some(1));
    assert!(
        stderr.contains("must say **1 `§V` invariants**"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A source replaced by text that holds no fact, and the error it names.
const BROKEN: [(&str, &str, &str); 6] = [
    (
        "README.md",
        "# no block\n",
        "README.md has no <!-- BEGIN badges --> block",
    ),
    (
        ".coverage",
        "nothing\n",
        ".coverage holds no readable value",
    ),
    (
        "flake.lock",
        "{}\n",
        "flake.lock nixpkgs holds no readable value",
    ),
    (
        ".github/workflows/ci.yml",
        "jobs: {}\n",
        "ci.yml has no `os: [...]` matrix",
    ),
    ("Cargo.toml", "[package]\n", "read empty or zero"),
    ("hk.pkl", "not pkl\n", "pkl eval -x"),
];

/// Unreadable sources and missing facts are exit 2, named.
#[test]
fn broken_sources_exit_2() -> Result {
    for (i, (path, text, want)) in BROKEN.iter().enumerate() {
        let dir = fixture(&format!("broken-{i}"))?;
        write(&dir, path, text)?;
        let (code, stderr) = dev(&dir, &["readme", "--check"], None)?;
        assert!(code == Some(2) && stderr.contains(want), "{path}: {stderr}");
        std::fs::remove_dir_all(dir)?;
    }
    Ok(())
}

/// An unknown verb, a missing file, and a program that cannot run.
#[test]
fn usage_and_missing_inputs_exit_2() -> Result {
    let dir = fixture("io")?;
    assert_eq!(dev(&dir, &["bogus"], None)?.0, Some(2));
    let (code, stderr) = dev(&dir, &["readme"], Some("/nonexistent"))?;
    assert!(
        code == Some(2) && stderr.contains("cannot run pkl"),
        "{stderr}"
    );
    std::fs::remove_file(dir.join(".coverage"))?;
    let (code, stderr) = dev(&dir, &["readme"], None)?;
    assert!(
        code == Some(2) && stderr.contains("cannot read .coverage"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A `pkl` that prints no number is exit 2, quoting what it printed.
#[test]
fn a_step_count_that_is_not_a_number_exits_2() -> Result {
    let dir = fixture("not-a-number")?;
    write(&dir, "bin/pkl", "#!/bin/sh\necho x\n")?;
    Command::new("chmod")
        .arg("+x")
        .arg(dir.join("bin/pkl"))
        .status()?;
    let path = format!("{}:{}", dir.join("bin").display(), std::env::var("PATH")?);
    let (code, stderr) = dev(&dir, &["readme"], Some(&path))?;
    assert!(
        code == Some(2) && stderr.contains("pkl printed `x`"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A README that cannot be written is exit 2.
#[test]
fn an_unwritable_readme_exits_2() -> Result {
    use std::os::unix::fs::PermissionsExt as _;
    let dir = fixture("readonly")?;
    std::fs::set_permissions(
        dir.join("README.md"),
        std::fs::Permissions::from_mode(0o444),
    )?;
    let (code, stderr) = dev(&dir, &["readme"], None)?;
    assert!(
        code == Some(2) && stderr.contains("cannot write README.md"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}
