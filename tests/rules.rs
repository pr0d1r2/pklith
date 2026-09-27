//! Companion rules through `pklith check` (cover V7), against the built
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

/// Cover V9 (T5): the first `.rb` beside a gemspec, with no rows for
/// either, is reported with the rows the rubocop fragment would add.
#[test]
fn an_unreflected_fragment_names_the_rows_to_add() -> Result {
    let dir = repo("reflect", Some(OK))?;
    std::fs::create_dir_all(dir.join("lib"))?;
    std::fs::write(dir.join("lib/c.rb"), "")?;
    std::fs::write(dir.join("x.gemspec"), "")?;
    git_add(&dir)?;
    let (code, stderr) = check_in(&dir)?;
    assert_eq!(code, Some(1));
    let want = "fragment: `rubocop` is on, but .pklith claims none of its checks; add rows such as: gemspec|rubocop|-|-; rb|rubocop|-|-\n";
    assert!(stderr.ends_with(want), "{stderr}");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// A local fragment naming an unknown check stops the check with its
/// line, as a bad check row does.
#[test]
fn a_bad_local_fragment_exits_2() -> Result {
    let bad = format!("{OK}## fragments\nfragment|triggers|checks|seed\nx|always|nope|-\n");
    let dir = repo("reflect-bad", Some(&bad))?;
    let (code, stderr) = check_in(&dir)?;
    assert_eq!(
        (code, stderr.as_str()),
        (
            Some(2),
            "pklith check: .pklith:12: fragment `x` names unknown check `nope`\n"
        )
    );
    Ok(std::fs::remove_dir_all(dir)?)
}
