//! `pkli lay`, run against the built binary.

mod common;

use common::{HK, OK, Result, pkli, repo};

const PLAN: &str = "ci(lint): add the lint check\n";

/// lay V4: `--dry-run` prints the commits lay would make and writes
/// nothing. With `lint` already an hk step, there is nothing to lay.
#[test]
fn dry_run_lists_only_missing_checks() -> Result {
    let dir = repo("lay-plan", Some(OK))?;
    assert_eq!(
        pkli(&dir, &["lay", "--dry-run"])?,
        (Some(0), String::new(), String::new())
    );
    std::fs::write(dir.join("hk.pkl"), HK.replace("\"lint\"", "\"other\""))?;
    assert_eq!(
        pkli(&dir, &["lay", "--dry-run"])?,
        (Some(0), PLAN.to_owned(), String::new())
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Without hk's steps the plan cannot be made: exit 2.
#[test]
fn dry_run_without_hk_config_exits_2() -> Result {
    let dir = repo("lay-no-hk", Some(OK))?;
    std::fs::remove_file(dir.join("hk.pkl"))?;
    let (code, _, stderr) = pkli(&dir, &["lay", "--dry-run"])?;
    assert_eq!(code, Some(2));
    assert!(
        stderr.starts_with("pkli lay: cannot read hk's steps: "),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}
