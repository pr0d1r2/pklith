//! Root V36 (T73, B13): `scripts/hk-parity.sh` refuses a step with no
//! command in any hook of the evaluated hk.pkl, naming the hook and the
//! step. The fixture is the #91 shape: a hook amends a step id its mapping
//! lacks, which Pkl turns into the mapping's default, a step with no
//! `check`.

mod common;

use common::{Result, SCHEMA};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Steps both hooks hold; `check` lists them as they are.
const FAST: &str =
    "local fast = new Mapping<String, Step> {\n  [\"lint\"] { check = \"true\" }\n}\n";

/// The trap: `test` is not in `fast`, so this amends the default Step.
const PLANTED: &str =
    "[\"pre-commit\"] { steps = (fast) { [\"test\"] { depends = List(\"lint\") } } }";

/// The clean twin: pre-commit runs `fast` as it is.
const CLEAN: &str = "[\"pre-commit\"] { steps = fast }";

fn fixture(name: &str, pre_commit: &str) -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("pklith-hk-parity-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("pkl"))?;
    std::fs::write(dir.join("pkl/Config.pkl"), SCHEMA)?;
    let hooks = format!("hooks {{\n  {pre_commit}\n  [\"check\"] {{ steps = fast }}\n}}\n");
    let hk = format!("amends \"pkl/Config.pkl\"\n{FAST}{hooks}");
    std::fs::write(dir.join("hk.pkl"), hk)?;
    run(&dir, Command::new("git").args(["init", "-q"]))?;
    Ok(dir)
}

/// Runs `cmd` in `dir` outside any hook's git environment (root V20).
fn run(dir: &Path, cmd: &mut Command) -> Result<(bool, String)> {
    let out = cmd
        .current_dir(dir)
        .env_remove("GIT_DIR")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_WORK_TREE")
        .output()?;
    Ok((out.status.success(), String::from_utf8(out.stderr)?))
}

fn parity(dir: &Path) -> Result<(bool, String)> {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/hk-parity.sh");
    run(dir, &mut Command::new(script))
}

#[test]
fn a_step_amended_into_being_without_a_command_is_refused_by_name() -> Result {
    let (ok, err) = parity(&fixture("planted", PLANTED)?)?;
    assert!(!ok, "hk-parity passed a commandless step:\n{err}");
    assert!(
        err.contains("pre-commit: test"),
        "the refusal names neither hook nor step:\n{err}"
    );
    Ok(())
}

#[test]
fn the_clean_twin_passes() -> Result {
    let (ok, err) = parity(&fixture("clean", CLEAN)?)?;
    assert!(
        ok,
        "hk-parity refused a gate whose every step has a command:\n{err}"
    );
    Ok(())
}
