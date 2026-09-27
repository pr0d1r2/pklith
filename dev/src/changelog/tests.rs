use super::{Sections, behaviour, check, product, unreleased};

const LOG: &str = "# Changelog\n\n## [Unreleased]\n\n- a\n\n## [0.1.0]\n\n- old\n";

#[test]
fn behaviour_changes_are_feat_fix_and_perf() {
    for s in ["feat: x", "fix(a): x", "perf!: x"] {
        assert!(behaviour(s), "{s}");
    }
    for s in ["test: x", "fixup! x", "feature: x", "docs: x"] {
        assert!(!behaviour(s), "{s}");
    }
}

#[test]
fn product_code_excludes_specs_and_tests() {
    let staged = "src/a/mod.rs\nsrc/a/SPEC.md\nsrc/a/tests.rs\nsrc/a/tests/b.rs\ndev/src/x.rs\n";
    assert_eq!(product(staged), ["src/a/mod.rs"]);
}

#[test]
fn the_unreleased_section_is_read() {
    assert_eq!(unreleased(LOG), "\n- a\n\n");
    assert_eq!(unreleased("# none\n"), "");
}

fn sections(staged: &str, head: &str, parent: &str, subject: &str) -> Sections {
    Sections {
        staged: staged.into(),
        head: head.into(),
        parent: parent.into(),
        head_subject: subject.into(),
    }
}

/// A feat touching src/ with no new entry is refused, naming the paths.
#[test]
fn a_feat_without_an_entry_is_refused() {
    let same = sections("- a\n", "- a\n", "- a\n", "other");
    let err = check("feat(x): y", &["src/a.rs"], &same)
        .err()
        .unwrap_or_default();
    assert!(
        err.starts_with("a feat commit changed product code"),
        "{err}"
    );
    assert!(err.contains("\n  src/a.rs\n"), "{err}");
}

/// A new entry passes; so does an amend keeping the entry its commit
/// brought; tests, specs and non-behaviour commits owe none.
#[test]
fn an_entry_is_asked_for_only_when_owed() {
    let src = ["src/a.rs"];
    let same = sections("- a\n", "- a\n", "- a\n", "other");
    assert_eq!(
        check("feat: y", &src, &sections("- b\n", "- a\n", "", "")),
        Ok(())
    );
    assert_eq!(
        check("feat: y", &src, &sections("- a\n", "- a\n", "", "feat: y")),
        Ok(())
    );
    assert_eq!(check("test: y", &src, &same), Ok(()));
    assert_eq!(check("feat: y", &[], &same), Ok(()));
}
