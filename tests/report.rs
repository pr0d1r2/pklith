//! `pkli report`, run against the built binary.

mod common;

use common::{OK, Result, pkli, repo};

/// The common fixture with an untyped x.py, so there is a gap to report.
fn gapped(name: &str) -> Result<std::path::PathBuf> {
    let ws = "lint|lint|-|*|true|-|-|m\nws|hygiene|-|*|true|-|-|m\n";
    let star = OK.replace("lint|lint|-|*|true|-|-|m\n", ws).replace(
        "type|checks|min|exempt\n",
        "type|checks|min|exempt\n*|ws|-|-\n",
    );
    let dir = repo(name, Some(&star))?;
    std::fs::write(dir.join("x.py"), "")?;
    pklith::proc::command("git", &dir)
        .args(["add", "-A"])
        .output()?;
    Ok(dir)
}

/// Report V3: the text matrix always prints and exits 0, gaps included.
#[test]
fn report_prints_the_matrix_and_findings_and_exits_0() -> Result {
    let dir = gapped("report")?;
    let (code, stdout, stderr) = pkli(&dir, &["report"])?;
    assert_eq!((code, stderr.as_str()), (Some(0), ""));
    assert!(stdout.starts_with("*       -  ws\n"), "{stdout}");
    assert!(stdout.contains("\nrs      2  lint\n"), "{stdout}");
    assert!(
        stdout.contains("gap: `py` has no row in .pklith"),
        "{stdout}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Report V2, V4: JSON is versioned and says the verdict; md is the
/// legacy table.
#[test]
fn report_speaks_json_and_markdown() -> Result {
    let dir = gapped("report-formats")?;
    let json = pkli(&dir, &["report", "--format", "json"])?.1;
    assert!(json.starts_with("{\"version\":1,\"ok\":false,"), "{json}");
    assert!(
        json.contains("\"gaps\":[{\"type\":\"py\",\"files\":[\"x.py\"]}]"),
        "{json}"
    );
    let md = pkli(&dir, &["report", "--format", "md"])?.1;
    assert!(md.contains("| `.rs` | lint | 2 files |\n"), "{md}");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// An unknown format, or one without its value, is a usage error.
#[test]
fn a_bad_format_is_a_usage_error() -> Result {
    let dir = gapped("report-usage")?;
    for args in [&["report", "--format", "xml"][..], &["report", "--format"]] {
        let (code, _, stderr) = pkli(&dir, args)?;
        assert_eq!(code, Some(2));
        assert!(stderr.starts_with("usage: pkli"), "{stderr}");
    }
    Ok(std::fs::remove_dir_all(dir)?)
}

/// Outside a repository report exits 2 naming the cause.
#[test]
fn report_outside_a_repository_exits_2() -> Result {
    let outside = common::temp("report-outside")?;
    let (code, _, stderr) = pkli(&outside, &["report"])?;
    assert_eq!(
        (code, stderr.as_str()),
        (
            Some(2),
            "pkli report: not inside a git repository; pass --root DIR\n"
        )
    );
    Ok(std::fs::remove_dir_all(outside)?)
}
