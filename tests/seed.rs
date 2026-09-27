//! `pkli seed [--init]`, run against the built binary.

mod common;

use common::{Result, pkli, repo, temp};

/// The common fixture (rust sources, hk.pkl) plus a flake and a README the
/// repository already wrote.
fn grown(name: &str) -> Result<std::path::PathBuf> {
    let dir = repo(name, None)?;
    std::fs::write(dir.join("flake.nix"), "{ }\n")?;
    std::fs::write(dir.join("README.md"), "mine\n")?;
    pklith::proc::command("git", &dir)
        .args(["add", "-A"])
        .output()?;
    Ok(dir)
}

/// Seed V1, V2: the fragments' files are written except the README the
/// repository has, `.gitignore` gathers nix then rust, and a second run
/// writes nothing.
#[test]
fn seed_writes_only_what_is_missing() -> Result {
    let dir = grown("seed")?;
    let want = ".editorconfig\n.gitattributes\n.envrc\n.gitignore\n";
    assert_eq!(
        pkli(&dir, &["seed"])?,
        (Some(0), want.to_owned(), String::new())
    );
    assert_eq!(std::fs::read_to_string(dir.join("README.md"))?, "mine\n");
    let ignore = std::fs::read_to_string(dir.join(".gitignore"))?;
    assert_eq!(ignore, ".direnv/\nresult\nresult-*\n/target\n");
    assert_eq!(pkli(&dir, &["seed"])?.1, "");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Seed T3: `--init` adds a first `.pklith`, which pkli then loads; one
/// already there is left alone.
#[test]
fn seed_init_writes_a_registry_pkli_loads() -> Result {
    let dir = grown("seed-init")?;
    let (code, stdout, _) = pkli(&dir, &["seed", "--init"])?;
    assert_eq!((code, stdout.lines().last()), (Some(0), Some(".pklith")));
    let text = std::fs::read_to_string(dir.join(".pklith"))?;
    assert!(text.contains("\nrs|rustfmt, clippy|-|-\n"), "{text}");
    assert_eq!(pkli(&dir, &["detect"])?.0, Some(0));
    assert_eq!(pkli(&dir, &["seed", "--init"])?.1, "");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Outside a repository seed exits 2 and writes nothing.
#[test]
fn seed_outside_a_repository_exits_2() -> Result {
    let outside = temp("seed-outside")?;
    let (code, stdout, stderr) = pkli(&outside, &["seed", "--init"])?;
    assert_eq!((code, stdout.as_str()), (Some(2), ""));
    assert!(
        stderr.starts_with("pkli seed: not inside a git repository"),
        "{stderr}"
    );
    assert!(!outside.join(".pklith").exists());
    Ok(std::fs::remove_dir_all(outside)?)
}
