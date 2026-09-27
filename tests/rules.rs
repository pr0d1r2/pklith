//! Companion rules through `pkli check` (cover V7), against the built
//! binary.

mod common;

use common::{OK, Result, check_in, repo};

/// Every Rust file under sub/ has a `<stem>_test.rs` beside it.
const RULE: &str = "## rules\nid|kind|select|target|except\npaired|exists|sub/*.rs|sub/{stem}_test.rs|sub/*_test.rs\n";

fn git_add(dir: &std::path::Path) -> Result {
    pklith::proc::command("git", dir)
        .args(["add", "-A"])
        .output()?;
    Ok(())
}

/// A missing companion fails the check, naming the rule, the file and
/// what to add; adding it passes.
#[test]
fn a_missing_companion_fails_until_it_is_added() -> Result {
    let dir = repo("rules", Some(&format!("{OK}{RULE}")))?;
    let want =
        "rule: `paired` wants sub/b_test.rs for sub/b.rs; add it, or except the file in the rule\n";
    assert_eq!(check_in(&dir)?, (Some(1), want.to_owned()));
    std::fs::write(dir.join("sub/b_test.rs"), "")?;
    git_add(&dir)?;
    assert_eq!(check_in(&dir)?, (Some(0), String::new()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A rule that does not parse stops the check with its line (rule V1).
#[test]
fn a_malformed_rule_exits_2() -> Result {
    let bad = format!("{OK}{}", RULE.replace("{stem}", "{name}"));
    let dir = repo("rules-bad", Some(&bad))?;
    let (code, stderr) = check_in(&dir)?;
    assert_eq!(code, Some(2));
    assert!(
        stderr.contains("rule `paired`: unknown template variable `name`"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// `## plural` feeds the `plural` filter: `b` pluralizes to `bees`.
#[test]
fn the_plural_table_reaches_rules() -> Result {
    let rules = "## rules\nid|kind|select|target|except\ndoc|exists|sub/*.rs|docs/{stem\\|plural}.md|-\n## plural\nsingular|plural\nb|bees\n";
    let dir = repo("rules-plural", Some(&format!("{OK}{rules}")))?;
    let want =
        "rule: `doc` wants docs/bees.md for sub/b.rs; add it, or except the file in the rule\n";
    assert_eq!(check_in(&dir)?, (Some(1), want.to_owned()));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// `mentions` reads the tracked text: sub/b.rs must be named in a.rs.
#[test]
fn mentions_reads_tracked_files() -> Result {
    let rules =
        "## rules\nid|kind|select|target|except\nnamed|mentions|sub/*.rs|a.rs :: mod {stem}|-\n";
    let dir = repo("rules-mentions", Some(&format!("{OK}{rules}")))?;
    let want = "rule: `named` wants `mod b` in a.rs for sub/b.rs; add it, or except the file in the rule\n";
    assert_eq!(check_in(&dir)?, (Some(1), want.to_owned()));
    std::fs::write(dir.join("a.rs"), "mod b;\n")?;
    assert_eq!(check_in(&dir)?, (Some(0), String::new()));
    Ok(std::fs::remove_dir_all(dir)?)
}
