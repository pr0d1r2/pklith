//! The `pkli` exit-code contract (`src/cli` V1-V5), run against the built
//! binary in throwaway repositories.

use std::path::{Path, PathBuf};
use std::process::Command;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

/// A registry covering the fixture: `rs` linted, `.pklith` itself exempt.
const OK: &str = "format 1\n## checks\nid|category|nix|glob|check|fix|env|msg\nlint|lint|-|*|true|-|-|m\n## types\ntype|checks|min|exempt\nrs|lint|-|-\npklith|-|-|the registry itself\n";

fn pkli(dir: &Path, args: &[&str]) -> Result<(Option<i32>, String, String)> {
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

fn temp(name: &str) -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("pklith-cli-{}-{name}", std::process::id()));
    std::fs::create_dir_all(dir.join("sub"))?;
    std::fs::write(dir.join("a.rs"), "")?;
    std::fs::write(dir.join("sub/b.rs"), "")?;
    Ok(dir)
}

/// A git repository holding `a.rs`, `sub/b.rs` and, if given, `.pklith`.
fn repo(name: &str, pklith: Option<&str>) -> Result<PathBuf> {
    let dir = temp(name)?;
    if let Some(text) = pklith {
        std::fs::write(dir.join(".pklith"), text)?;
    }
    for args in [&["init", "-q"][..], &["add", "-A"]] {
        Command::new("git").args(args).current_dir(&dir).output()?;
    }
    Ok(dir)
}

fn check_in(dir: &Path) -> Result<(Option<i32>, String)> {
    let (code, stdout, stderr) = pkli(dir, &["check"])?;
    assert_eq!(stdout, "", "stdout carries data only");
    Ok((code, stderr))
}

#[test]
fn no_command_is_a_usage_error() -> Result {
    let (code, stdout, stderr) = pkli(Path::new("."), &[])?;
    assert_eq!((code, stdout.as_str()), (Some(2), ""));
    assert!(stderr.starts_with("usage: pkli"));
    Ok(())
}

#[test]
fn an_unknown_flag_is_a_usage_error() -> Result {
    let (code, _, stderr) = pkli(Path::new("."), &["check", "--bogus"])?;
    assert_eq!(code, Some(2));
    assert!(stderr.starts_with("usage: pkli"));
    Ok(())
}

/// A covered tree passes silently, from the root and from a subdirectory.
#[test]
fn a_covered_repository_passes_silently() -> Result {
    let dir = repo("ok", Some(OK))?;
    assert_eq!(check_in(&dir)?, (Some(0), String::new()));
    assert_eq!(check_in(&dir.join("sub"))?, (Some(0), String::new()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A type with no row is a finding: exit 1, the gap named on stderr.
#[test]
fn an_uncovered_type_is_a_finding() -> Result {
    let dir = repo("gap", Some(OK))?;
    std::fs::write(dir.join("x.py"), "")?;
    Command::new("git")
        .args(["add", "x.py"])
        .current_dir(&dir)
        .output()?;
    let want =
        "gap: `py` has no row in .pklith (1 file: x.py); add its checks, or an exemption reason\n";
    assert_eq!(check_in(&dir)?, (Some(1), want.to_owned()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Root V11: no `.pklith` never passes.
#[test]
fn a_missing_registry_is_an_error() -> Result {
    let dir = repo("missing", None)?;
    let (code, stderr) = check_in(&dir)?;
    assert_eq!(code, Some(2));
    assert!(
        stderr.starts_with("pkli check: cannot read ") && stderr.contains(".pklith"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

const BAD: [(&str, &str); 3] = [
    (
        "version 1\n",
        "pkli check: .pklith:1: expected `format 1`, got `version 1`\n",
    ),
    (
        "format 1\n## checks\nid|category|nix|glob|check|fix|env|msg\nx|style|-|*|x|-|-|m\n",
        "pkli check: .pklith:4: `x`: unknown category `style`\n",
    ),
    (
        "format 1\n## types\ntype|checks|min|exempt\nrs|clippy|-|-\n",
        "pkli check: .pklith:4: `rs` names unknown check `clippy`\n",
    ),
];

/// A registry that cannot be read as written stops the check with its line.
#[test]
fn a_broken_registry_is_an_error_naming_its_line() -> Result {
    for (i, (text, want)) in BAD.into_iter().enumerate() {
        let dir = repo(&format!("bad{i}"), Some(text))?;
        assert_eq!(check_in(&dir)?, (Some(2), want.to_owned()));
        std::fs::remove_dir_all(dir)?;
    }
    Ok(())
}

const OUTSIDE: &str = "pkli check: not inside a git repository; pass --root DIR\n";

/// V5: outside a repository, `--root` is required; with it, the tree is
/// walked instead of read from git.
#[test]
fn outside_a_repository_root_is_walked_on_request() -> Result {
    let dir = temp("walk")?;
    std::fs::write(dir.join(".pklith"), OK)?;
    assert_eq!(check_in(&dir)?, (Some(2), OUTSIDE.to_owned()));
    let root = dir.to_string_lossy().into_owned();
    assert_eq!(
        pkli(Path::new("/"), &["check", "--root", &root])?,
        (Some(0), String::new(), String::new())
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A directory the walk cannot read stops the check with its path; files
/// are never silently skipped.
#[test]
fn an_unreadable_directory_is_an_error() -> Result {
    use std::os::unix::fs::PermissionsExt;
    let dir = temp("unreadable")?;
    std::fs::write(dir.join(".pklith"), OK)?;
    std::fs::set_permissions(dir.join("sub"), std::fs::Permissions::from_mode(0o000))?;
    let (code, _, stderr) = pkli(&dir, &["check", "--root", "."])?;
    std::fs::set_permissions(dir.join("sub"), std::fs::Permissions::from_mode(0o755))?;
    assert_eq!(code, Some(2));
    assert!(
        stderr.starts_with("pkli check: ./sub: ") && stderr.contains("ermission denied"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}
