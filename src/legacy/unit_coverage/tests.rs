use super::{ALLOWLIST, Config, Rule, allowed, config};

fn rule(glob: &str, dirs: &[&str], test_dir: &str) -> Rule {
    Rule {
        glob: glob.to_owned(),
        dirs: dirs.iter().map(|d| (*d).to_owned()).collect(),
        test_dir: test_dir.to_owned(),
        pattern: "mirror".to_owned(),
        ..Rule::default()
    }
}

fn owned(files: &[&str]) -> Vec<String> {
    files.iter().map(|f| (*f).to_owned()).collect()
}

/// `files`' missing lines under `rule`, with `present` the files on disk.
fn missing(rule: &Rule, files: &[&str], present: &[&str]) -> Result<Vec<String>, String> {
    rule.missing(&owned(files), &[], |s| present.contains(&s))
}

const TOML: &str = "allowlist = \".mine\"\n[[rules]]\nglob = \"*.rb\"\ndirs = [\"app\"]\nexclude = [\"app/x\"]\ntest_dir = \"spec\"\ntest_ext = \"rbx\"\ntest_suffix = \"_spec\"\nstrip = \"app\"\nnormalize = true\n[[rules]]\ntest_dir = \"t\"\n[[rules]]\nglob = \"*.sh\"\n";

/// Every key is read; a rule without `glob` ends the rules as the legacy
/// loop ended; defaults are the legacy ones.
#[test]
fn the_config_is_read_as_the_legacy_tool_read_it() -> Result<(), String> {
    let want = Rule {
        exclude: vec!["app/x".into()],
        test_ext: "rbx".into(),
        test_suffix: "_spec".into(),
        strip: "app".into(),
        normalize: true,
        ..rule("*.rb", &["app"], "spec")
    };
    let got = config(TOML)?;
    assert_eq!((got.allowlist.as_str(), got.rules), (".mine", vec![want]));
    Ok(())
}

/// No config keys at all: the default allowlist and no rules; text outside
/// the subset is an error.
#[test]
fn an_empty_config_has_no_rules() -> Result<(), String> {
    let bare = Config {
        allowlist: ALLOWLIST.into(),
        rules: Vec::new(),
    };
    assert_eq!(config("")?, bare);
    assert!(config("a = {b = 1}").is_err());
    Ok(())
}

/// A string where a list belongs, and a list where a string belongs, read
/// as absent, as `taplo get` failing did.
#[test]
fn a_wrong_shape_reads_as_absent() -> Result<(), String> {
    let cfg = config("[[rules]]\nglob = \"*.sh\"\ndirs = \"a\"\nstrip = [\"b\"]\n")?;
    assert_eq!(cfg.rules, [rule("*.sh", &[], "")]);
    Ok(())
}

/// Blank lines and `#` comments are skipped; every other line is kept
/// verbatim.
#[test]
fn the_allowlist_skips_comments_and_blanks() {
    assert_eq!(allowed("# c\n\na.sh\n b.sh\r\n"), ["a.sh", " b.sh\r"]);
}

/// mirror keeps the directory; strip drops its prefix.
#[test]
fn mirror_keeps_the_directory() -> Result<(), String> {
    let bats = Rule {
        test_ext: "bats".into(),
        ..rule("*.sh", &["scripts"], "tests/unit")
    };
    let want = ("tests/unit/scripts/a/b.bats".to_owned(), None);
    assert_eq!(bats.specs("scripts/a/b.sh")?, want);
    let stripped = Rule {
        strip: "app".into(),
        test_suffix: "_spec".into(),
        ..rule("*.rb", &["app"], "spec")
    };
    assert_eq!(stripped.specs("app/m/u.rb")?.0, "spec/m/u_spec.rb");
    assert_eq!(stripped.specs("app/u.rb")?.0, "spec//u_spec.rb");
    Ok(())
}

/// flat drops the implementation's directory altogether.
#[test]
fn flat_drops_the_directory() -> Result<(), String> {
    let flat = Rule {
        pattern: "flat".into(),
        ..rule("*.rb", &["lib"], "test")
    };
    assert_eq!(flat.specs("lib/x/c.rb")?.0, "test/c.rb");
    Ok(())
}

/// normalize: dashes first, the raw stem as the fallback; no fallback
/// when nothing changed.
#[test]
fn normalize_accepts_both_stems() -> Result<(), String> {
    let r = Rule {
        normalize: true,
        ..rule("*.sh", &["d"], "t")
    };
    let want = ("t/d/a-b.sh".into(), Some("t/d/a_b.sh".into()));
    assert_eq!(r.specs("d/a_b.sh")?, want);
    assert_eq!(r.specs("d/ab.sh")?.1, None);
    assert!(missing(&r, &["d/a_b.sh"], &["t/d/a_b.sh"])?.is_empty());
    assert!(missing(&r, &["d/a_b.sh"], &["t/d/a-b.sh"])?.is_empty());
    assert_eq!(missing(&r, &["d/a_b.sh"], &[])?, ["d/a_b.sh -> t/d/a-b.sh"]);
    Ok(())
}

/// Only files of the extension, under a directory, outside every
/// exclusion, and not allowlisted need a test.
#[test]
fn selection_matches_the_legacy_case_patterns() -> Result<(), String> {
    let r = Rule {
        exclude: vec![String::new(), "s/skip".into()],
        ..rule("*.sh", &["", "s"], "t")
    };
    let files = owned(&[
        "s/a.sh",
        "s/skip/b.sh",
        "o/c.sh",
        "s/d.rb",
        "s/e.sh",
        "sx/f.sh",
    ]);
    let got = r.missing(&files, &["s/e.sh"], |_| false)?;
    assert_eq!(got, ["s/a.sh -> t/s/a.sh"]);
    Ok(())
}

/// An unknown pattern is an error once a file needs a test, and only
/// then, as the legacy tool failed.
#[test]
fn an_unknown_pattern_fails_when_used() {
    let r = Rule {
        pattern: "odd".into(),
        ..rule("*.sh", &["s"], "t")
    };
    assert_eq!(missing(&r, &[], &[]), Ok(Vec::new()));
    let err = missing(&r, &["s/a.sh"], &[]).err();
    assert_eq!(err.as_deref(), Some("unknown pattern 'odd'"));
}

/// The legacy stderr: a section per failing rule, then the fix line; a
/// passing run prints nothing.
#[test]
fn the_verdict_is_the_legacy_text() -> Result<(), String> {
    let cfg = config(
        "[[rules]]\nglob = \"*.sh\"\ndirs = [\"s\"]\ntest_dir = \"t\"\n[[rules]]\nglob = \"*.rb\"\ndirs = [\"s\"]\ntest_dir = \"t\"\n",
    )?;
    let files = owned(&["s/a.sh", "s/b.sh", "s/c.rb"]);
    let (text, failed) = super::verdict(&cfg, &files, &["s/b.sh"], |s| s == "t/s/c.rb");
    let want = "lefthook-unit-coverage: rule 0 (*.sh): 1 file(s) missing spec:\n  s/a.sh -> t/s/a.sh\n\nFix: add missing test files or allowlist paths in .coverage-allowlist.\n";
    assert_eq!((text.as_str(), failed), (want, true));
    assert_eq!(
        super::verdict(&cfg, &files, &[], |_| true),
        (String::new(), false)
    );
    Ok(())
}

/// An unknown pattern stops the run after what earlier rules printed.
#[test]
fn an_unknown_pattern_ends_the_verdict() -> Result<(), String> {
    let cfg = config(
        "[[rules]]\nglob = \"*.sh\"\ndirs = [\"s\"]\n[[rules]]\nglob = \"*.sh\"\ndirs = [\"s\"]\npattern = \"odd\"\n",
    )?;
    let (text, failed) = super::verdict(&cfg, &owned(&["s/a.sh"]), &[], |_| false);
    assert!(failed && text.starts_with("lefthook-unit-coverage: rule 0 (*.sh)"));
    assert!(
        text.ends_with(
            "\n  s/a.sh -> /s/a.sh\nlefthook-unit-coverage: unknown pattern 'odd' in rule 1\n"
        ),
        "{text}"
    );
    Ok(())
}
