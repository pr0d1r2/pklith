//! `pklith map`, run against the built binary.

mod common;

use common::{OK, Result, pklith, repo};
use std::path::Path;

/// Each file under sub/ has its spec under spec/.
const RULE: &str =
    "## rules\nid|kind|select|target|except\nspec|exists|sub/*.rs|spec/{stem}_spec.rs|-\n";

fn git(dir: &Path, args: &[&str]) -> Result {
    let who = ["-c", "user.name=t", "-c", "user.email=t@t"];
    pklith::proc::command("git", dir)
        .args(who)
        .args(args)
        .output()?;
    Ok(())
}

/// The common fixture plus `spec/b_spec.rs`, committed.
fn specced(name: &str) -> Result<std::path::PathBuf> {
    let dir = repo(name, Some(&format!("{OK}{RULE}")))?;
    std::fs::create_dir_all(dir.join("spec"))?;
    std::fs::write(dir.join("spec/b_spec.rs"), "")?;
    git(&dir, &["add", "-A"])?;
    git(&dir, &["commit", "-qm", "base"])?;
    Ok(dir)
}

/// Map V2, V3: named paths map to their specs; a path no rule selects is
/// ignored; `--staged` reads the index.
#[test]
fn changed_files_print_their_specs() -> Result {
    let dir = specced("map")?;
    let want = (Some(0), "spec/b_spec.rs\n".to_owned(), String::new());
    assert_eq!(pklith(&dir, &["map", "sub/b.rs", "a.rs"])?, want);
    std::fs::write(dir.join("sub/b.rs"), "// edited\n")?;
    git(&dir, &["add", "sub/b.rs"])?;
    assert_eq!(pklith(&dir, &["map", "--staged"])?, want);
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Map V1: a changed file a rule selects whose spec is missing exits 1
/// naming it; a malformed call exits 2.
#[test]
fn a_missing_spec_fails_and_bad_arguments_are_refused() -> Result {
    let dir = specced("map-missing")?;
    std::fs::write(dir.join("sub/c.rs"), "")?;
    git(&dir, &["add", "sub/c.rs"])?;
    let want =
        "map: sub/c.rs matches rule `spec`, but its spec spec/c_spec.rs is not tracked; add it\n";
    assert_eq!(
        pklith(&dir, &["map", "sub/c.rs"])?,
        (Some(1), String::new(), want.to_owned())
    );
    let (code, _, stderr) = pklith(&dir, &["map", "--staged", "sub/c.rs"])?;
    assert_eq!(code, Some(2));
    assert!(stderr.contains("give --staged alone"), "{stderr}");
    Ok(std::fs::remove_dir_all(dir)?)
}
