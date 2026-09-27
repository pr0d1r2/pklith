use super::{check, subject};

const BODY: &str = "\n\nSays why.\n\nCo-Authored-By: C <c@x>\n";

#[test]
fn a_styled_subject_with_a_body_passes() {
    for s in [
        "feat(src/cli): add x",
        "fix: y",
        "docs(a.b_c-d)!: z",
        "ci: w",
    ] {
        assert_eq!(check(&format!("{s}{BODY}")), Ok(()), "{s}");
    }
    assert_eq!(check("Merge branch x\n"), Ok(()));
    assert_eq!(check("fixup! feat: a\n"), Ok(()));
}

/// Subjects that break the style, each for a different reason.
const OFF: [&str; 8] = [
    "add x",
    "feat(): x",
    "feat(X): x",
    "feat:x",
    "feat:  x",
    "feat: ",
    "testing: x",
    "feat(a: x",
];

#[test]
fn a_subject_off_the_style_is_refused() {
    for s in OFF {
        let err = check(&format!("{s}{BODY}")).err().unwrap_or_default();
        assert!(
            err.starts_with("subject must be 'type(scope): claim'"),
            "{s}: {err}"
        );
    }
}

#[test]
fn a_long_subject_is_refused_by_characters() {
    let long = format!("feat: {}", "é".repeat(67));
    let err = check(&format!("{long}{BODY}")).err().unwrap_or_default();
    assert!(err.starts_with("subject is 73 characters"), "{err}");
    assert_eq!(check(&format!("feat: {}{BODY}", "é".repeat(66))), Ok(()));
}

/// Comments, trailers and blank lines are no body; neither is the diff
/// `git commit -v` appends below the scissors line.
#[test]
fn an_empty_body_is_refused() {
    let empty = "feat: x\n\n# comment\nCo-Authored-By: C <c@x>\n";
    let want = Err("the body is empty. Say WHY: the diff already shows what changed.".to_owned());
    assert_eq!(check(empty), want);
    let verbose =
        "feat: x\n\n# ------------------------ >8 ------------------------\ndiff --git a/x b/x\n";
    assert_eq!(check(verbose), want);
}

#[test]
fn the_subject_skips_comment_lines() {
    assert_eq!(subject("# c\nfeat: x\n"), "feat: x");
    assert_eq!(subject(""), "");
}
