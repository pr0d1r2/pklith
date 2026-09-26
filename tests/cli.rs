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

/// Root T63, V20 (set-and-setting B97): with a hook's git variables
/// pointing at another repository, as git sets them for hooks in worktrees
/// and during a rebase, pkli still judges its own repository. Without the
/// scrub, `git ls-files` would read the other repository's index.
#[test]
fn hook_variables_from_another_repository_do_not_leak() -> Result {
    let b = repo("leak-b", Some(OK))?;
    std::fs::write(b.join("only-in-b.py"), "")?;
    Command::new("git")
        .args(["add", "only-in-b.py"])
        .current_dir(&b)
        .output()?;
    let a = repo("leak-a", None)?;
    let (code, _, stderr) = pkli_with_hook_env(&b, &a)?;
    let want = "gap: `py` has no row in .pklith (1 file: only-in-b.py); add its checks, or an exemption reason\n";
    assert_eq!((code, stderr.as_str()), (Some(1), want));
    std::fs::remove_dir_all(a)?;
    Ok(std::fs::remove_dir_all(b)?)
}

/// `pkli check` in `dir` with `GIT_DIR`, `GIT_INDEX_FILE` and
/// `GIT_WORK_TREE` all pointing at `other`.
fn pkli_with_hook_env(dir: &Path, other: &Path) -> Result<(Option<i32>, String, String)> {
    let git = other.join(".git");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_pkli"));
    cmd.arg("check").current_dir(dir);
    cmd.env("GIT_DIR", &git)
        .env("GIT_INDEX_FILE", git.join("index"))
        .env("GIT_WORK_TREE", other);
    let out = cmd.output()?;
    Ok((
        out.status.code(),
        String::from_utf8(out.stdout)?,
        String::from_utf8(out.stderr)?,
    ))
}

const LEGACY: &str = "| Extension | Linter | Notes |\n|---|---|---|\n| `.rs` | clippy | |\n";

const IMPORTED: &str = "format 1\n\n## types\ntype|checks|min|exempt\nrs|-|-|legacy: clippy\n";

/// `import` prints a .pklith on stdout: it is data (cli V3).
#[test]
fn import_prints_a_registry_on_stdout() -> Result {
    let dir = temp("import")?;
    std::fs::write(dir.join("legacy.md"), LEGACY)?;
    let (code, stdout, stderr) = pkli(&dir, &["import", "legacy.md"])?;
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
    let (code, _, stderr) = pkli(&dir, &["check", "--registry", &registry.to_string_lossy()])?;
    assert_eq!((code, stderr.as_str()), (Some(0), ""));
    std::fs::remove_file(registry)?;
    Ok(std::fs::remove_dir_all(dir)?)
}

const IMPORT_ERRORS: [(&[&str], &str); 4] = [
    (
        &["import", "no-such.md"],
        "pkli import: cannot read no-such.md: ",
    ),
    (
        &["import", "empty-table.md"],
        "pkli import: parsed nothing: ",
    ),
    (&["import"], "usage: pkli"),
    (&["check", "--root", "a", "--root", "b"], "usage: pkli"),
];

/// Unreadable or empty documents, and malformed arguments, exit 2.
#[test]
fn import_and_option_errors_exit_2() -> Result {
    let dir = temp("import-errors")?;
    std::fs::write(dir.join("empty-table.md"), "| a | b |\n|---|---|\n")?;
    for (args, prefix) in IMPORT_ERRORS {
        let (code, stdout, stderr) = pkli(&dir, args)?;
        assert_eq!((code, stdout.as_str()), (Some(2), ""), "{args:?}");
        assert!(stderr.starts_with(prefix), "{args:?}: {stderr}");
    }
    Ok(std::fs::remove_dir_all(dir)?)
}

const GENERATED: &str = "// generated by `pkli gen` from .pklith; do not edit\nimport \"pkl/Config.pkl\"\n\nsteps: Mapping<String, Config.Step> = new {\n  [\"lint\"] {\n    glob = List(\"*\")\n    check = \"true\"\n  }\n}\n";

fn read_only(path: &Path, on: bool) -> Result {
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_readonly(on);
    Ok(std::fs::set_permissions(path, permissions)?)
}

/// gen writes hk.pklith.pkl, and a second run writes nothing (gen V2): the
/// file is read-only by then, so any write would fail the run.
#[test]
fn gen_writes_once_and_then_leaves_the_file_alone() -> Result {
    let dir = repo("gen", Some(OK))?;
    assert_eq!(
        pkli(&dir, &["gen"])?,
        (Some(0), String::new(), String::new())
    );
    let file = dir.join("hk.pklith.pkl");
    assert_eq!(std::fs::read_to_string(&file)?, GENERATED);
    read_only(&file, true)?;
    assert_eq!(pkli(&dir, &["gen"])?.0, Some(0));
    read_only(&file, false)?;
    Ok(std::fs::remove_dir_all(dir)?)
}

/// gen V3: `--check` passes on a fresh file and fails on an edited or
/// missing one, writing nothing.
#[test]
fn gen_check_fails_on_a_stale_or_missing_file() -> Result {
    let dir = repo("gen-check", Some(OK))?;
    let stale = (
        Some(1),
        String::new(),
        "pkli gen: hk.pklith.pkl is stale; run `pkli gen` and stage it\n".to_owned(),
    );
    assert_eq!(pkli(&dir, &["gen", "--check"])?, stale);
    pkli(&dir, &["gen"])?;
    assert_eq!(pkli(&dir, &["gen", "--check"])?.0, Some(0));
    std::fs::write(dir.join("hk.pklith.pkl"), "edited\n")?;
    assert_eq!(pkli(&dir, &["gen", "--check"])?, stale);
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A stale file that cannot be written exits 2 naming it.
#[test]
fn gen_cannot_write_exits_2() -> Result {
    let dir = repo("gen-readonly", Some(OK))?;
    let file = dir.join("hk.pklith.pkl");
    std::fs::write(&file, "old\n")?;
    read_only(&file, true)?;
    let (code, _, stderr) = pkli(&dir, &["gen"])?;
    read_only(&file, false)?;
    assert_eq!(code, Some(2));
    assert!(
        stderr.starts_with("pkli gen: cannot write hk.pklith.pkl: "),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// No registry, or no repository, exits 2 naming the cause.
#[test]
fn gen_without_a_registry_or_repository_exits_2() -> Result {
    let dir = repo("gen-missing", None)?;
    assert!(
        pkli(&dir, &["gen"])?
            .2
            .starts_with("pkli gen: cannot read ")
    );
    let outside = temp("gen-outside")?;
    assert_eq!(
        pkli(&outside, &["gen", "--check"])?.2,
        "pkli gen: not inside a git repository; pass --root DIR\n"
    );
    std::fs::remove_dir_all(outside)?;
    Ok(std::fs::remove_dir_all(dir)?)
}
