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
    let want = ".editorconfig\n.gitattributes\nhk.pklith.pkl\n.githooks/pre-commit\n.githooks/pre-push\n.envrc\n.gitignore\n";
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

/// A repository holding one shell script and nothing else: no gate, no
/// registry.
fn bare(name: &str) -> Result<std::path::PathBuf> {
    let dir = std::env::temp_dir().join(format!("pklith-cli-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("run.sh"), "#!/bin/sh\necho hi\n")?;
    for args in [&["init", "-q"][..], &["add", "-A"]] {
        pklith::proc::command("git", &dir).args(args).output()?;
    }
    Ok(dir)
}

/// The bootstrap path on a repository with no gate at all: `pkli seed
/// --init` writes an hk gate whose hooks are executable and whose hk.pkl
/// evaluates, so `pkli lay` can plan the first commits right away.
#[test]
fn seed_init_bootstraps_a_repository_lay_can_plan() -> Result {
    use std::os::unix::fs::PermissionsExt as _;
    let dir = bare("bootstrap")?;
    assert_eq!(pkli(&dir, &["seed", "--init"])?.0, Some(0));
    let mode = std::fs::metadata(dir.join(".githooks/pre-commit"))?
        .permissions()
        .mode();
    assert_eq!(mode & 0o111, 0o111);
    let (code, plan, stderr) = pkli(&dir, &["lay", "--dry-run"])?;
    assert_eq!(code, Some(0), "{stderr}");
    assert!(plan.starts_with("ci(trailing-whitespace): "), "{plan}");
    assert!(plan.contains("ci(shellcheck): "), "{plan}");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Seeding is a green start: after `seed --init` and `gen` on a fresh
/// repository, `pkli check` passes. The registry types the files seed and
/// gen write (README, hk config, vendored schema, nix/pklith.nix), and the
/// fragments they switch on, not only the files that were there before.
#[test]
fn seed_init_then_gen_passes_check() -> Result {
    let dir = bare("green")?;
    assert_eq!(pkli(&dir, &["seed", "--init"])?.0, Some(0));
    assert_eq!(pkli(&dir, &["gen"])?.0, Some(0));
    pklith::proc::command("git", &dir)
        .args(["add", "-A"])
        .output()?;
    let (code, _, stderr) = pkli(&dir, &["check"])?;
    assert_eq!((code, stderr.as_str()), (Some(0), ""));
    Ok(std::fs::remove_dir_all(dir)?)
}
