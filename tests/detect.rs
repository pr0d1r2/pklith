//! `pklith detect`, run against the built binary.

mod common;

use common::{Result, pklith, repo, temp};

/// A repository holding the common fixture (a.rs, sub/b.rs, hk.pkl,
/// pkl/Config.pkl) plus a shell script and a flake, all tracked.
fn scripted(name: &str, pklith: Option<&str>) -> Result<std::path::PathBuf> {
    let dir = repo(name, pklith)?;
    std::fs::write(dir.join("run.sh"), "")?;
    std::fs::write(dir.join("flake.nix"), "")?;
    pklith::proc::command("git", &dir)
        .args(["add", "-A"])
        .output()?;
    Ok(dir)
}

const FOUND: &str = "base\nnix\nshell\nrust\n";

/// The active fragments, one per line in catalog order, without a
/// `.pklith`: detection is how a repository gets its first one.
#[test]
fn detect_lists_the_fragments_the_files_switch_on() -> Result {
    let dir = scripted("detect", None)?;
    assert_eq!(
        pklith(&dir, &["detect"])?,
        (Some(0), FOUND.to_owned(), String::new())
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Detect V3 (T2): git's tracked files and a `--root` walk of the same
/// tree give the same fragments.
#[test]
fn git_and_a_root_walk_agree() -> Result {
    let dir = scripted("detect-walk", None)?;
    let root = dir.to_string_lossy().into_owned();
    assert_eq!(pklith(&dir, &["detect", "--root", &root])?.1, FOUND);
    Ok(std::fs::remove_dir_all(dir)?)
}

const LOCAL: &str =
    "format 1\n## fragments\nfragment|triggers|checks|seed\nshell|**/*.zsh|shellcheck|-\n";

/// A `.pklith` fragment replaces the built-in one with its id: here shell
/// no longer switches on for `.sh`.
#[test]
fn a_local_fragment_replaces_the_builtin_one() -> Result {
    let dir = scripted("detect-local", Some(LOCAL))?;
    assert_eq!(pklith(&dir, &["detect"])?.1, "base\nnix\nrust\n");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Outside a repository detect exits 2 naming the cause.
#[test]
fn detect_outside_a_repository_exits_2() -> Result {
    let outside = temp("detect-outside")?;
    let want = "pklith detect: not inside a git repository; pass --root DIR\n";
    assert_eq!(
        pklith(&outside, &["detect"])?,
        (Some(2), String::new(), want.to_owned())
    );
    Ok(std::fs::remove_dir_all(outside)?)
}

/// A registry that names an unknown check, or a `--registry` that is not
/// there, exits 2: only the default `.pklith` may be missing.
#[test]
fn detect_with_a_bad_or_missing_registry_exits_2() -> Result {
    let bad = LOCAL.replace("|shellcheck|", "|nope|");
    let dir = scripted("detect-bad", Some(&bad))?;
    let (_, _, stderr) = pklith(&dir, &["detect"])?;
    assert_eq!(
        stderr,
        "pklith detect: .pklith:4: fragment `shell` names unknown check `nope`\n"
    );
    let (code, _, stderr) = pklith(&dir, &["detect", "--registry", "absent"])?;
    assert_eq!(code, Some(2));
    assert!(
        stderr.starts_with("pklith detect: cannot read absent: "),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// An unknown flag is a usage error.
#[test]
fn detect_with_an_unknown_flag_is_a_usage_error() -> Result {
    let dir = scripted("detect-usage", None)?;
    let (code, _, stderr) = pklith(&dir, &["detect", "--bogus"])?;
    assert_eq!(code, Some(2));
    assert!(stderr.starts_with("usage: pklith"), "{stderr}");
    Ok(std::fs::remove_dir_all(dir)?)
}
