//! `pklith seed [--init]`, run against the built binary.

mod common;

use common::{Result, pklith, repo, temp};

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
        pklith(&dir, &["seed"])?,
        (Some(0), want.to_owned(), String::new())
    );
    assert_eq!(std::fs::read_to_string(dir.join("README.md"))?, "mine\n");
    let ignore = std::fs::read_to_string(dir.join(".gitignore"))?;
    assert_eq!(ignore, ".direnv/\nresult\nresult-*\n/target\n");
    assert_eq!(pklith(&dir, &["seed"])?.1, "");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Seed T3: `--init` adds a first `.pklith`, which pklith then loads; one
/// already there is left alone.
#[test]
fn seed_init_writes_a_registry_pklith_loads() -> Result {
    let dir = grown("seed-init")?;
    let (code, stdout, _) = pklith(&dir, &["seed", "--init"])?;
    assert_eq!((code, stdout.lines().last()), (Some(0), Some(".pklith")));
    let text = std::fs::read_to_string(dir.join(".pklith"))?;
    assert!(text.contains("\nrs|rustfmt, clippy|-|-\n"), "{text}");
    assert_eq!(pklith(&dir, &["detect"])?.0, Some(0));
    assert_eq!(pklith(&dir, &["seed", "--init"])?.1, "");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Outside a repository seed exits 2 and writes nothing.
#[test]
fn seed_outside_a_repository_exits_2() -> Result {
    let outside = temp("seed-outside")?;
    let (code, stdout, stderr) = pklith(&outside, &["seed", "--init"])?;
    assert_eq!((code, stdout.as_str()), (Some(2), ""));
    assert!(
        stderr.starts_with("pklith seed: not inside a git repository"),
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

/// The bootstrap path on a repository with no gate at all: `pklith seed
/// --init` writes an hk gate whose hooks are executable and whose hk.pkl
/// evaluates, so `pklith lay` can plan the first commits right away.
#[test]
fn seed_init_bootstraps_a_repository_lay_can_plan() -> Result {
    use std::os::unix::fs::PermissionsExt as _;
    let dir = bare("bootstrap")?;
    assert_eq!(pklith(&dir, &["seed", "--init"])?.0, Some(0));
    let mode = std::fs::metadata(dir.join(".githooks/pre-commit"))?
        .permissions()
        .mode();
    assert_eq!(mode & 0o111, 0o111);
    let (code, plan, stderr) = pklith(&dir, &["lay", "--dry-run"])?;
    assert_eq!(code, Some(0), "{stderr}");
    assert!(plan.starts_with("ci(trailing-whitespace): "), "{plan}");
    assert!(plan.contains("ci(shellcheck): "), "{plan}");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Seeding is a green start: after `seed --init` and `gen` on a fresh
/// repository, `pklith check` passes. The registry types the files seed and
/// gen write (README, hk config, vendored schema, nix/pklith.nix), and the
/// fragments they switch on, not only the files that were there before.
#[test]
fn seed_init_then_gen_passes_check() -> Result {
    let dir = bare("green")?;
    assert_eq!(pklith(&dir, &["seed", "--init"])?.0, Some(0));
    assert_eq!(pklith(&dir, &["gen"])?.0, Some(0));
    pklith::proc::command("git", &dir)
        .args(["add", "-A"])
        .output()?;
    let (code, _, stderr) = pklith(&dir, &["check"])?;
    assert_eq!((code, stderr.as_str()), (Some(0), ""));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A file the repository already had is its own, even where seed would
/// have written one: its type is not exempted as written by pklith.
#[test]
fn seed_init_leaves_existing_files_to_the_repository() -> Result {
    let dir = bare("own-files")?;
    std::fs::write(dir.join(".editorconfig"), "root = true\n")?;
    pklith::proc::command("git", &dir)
        .args(["add", "-A"])
        .output()?;
    assert_eq!(pklith(&dir, &["seed", "--init"])?.0, Some(0));
    let registry = std::fs::read_to_string(dir.join(".pklith"))?;
    assert!(
        !registry.contains("editorconfig|-|-|written by pklith"),
        "{registry}"
    );
    assert!(registry.contains("pkl|-|-|written by pklith"), "{registry}");
    Ok(std::fs::remove_dir_all(dir)?)
}
