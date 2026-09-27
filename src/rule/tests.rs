//! Rule evaluation tests (rule T2–T5).

use super::{Failure, Input, evaluate, parse};
use crate::registry::Error;

const HEADER: &str = "format 1\n## rules\nid|kind|select|target|except\n";

fn rules(rows: &str) -> Result<Vec<super::Rule>, Error> {
    parse(&crate::registry::parse(&format!("{HEADER}{rows}"))?.rules)
}

fn owned(paths: &[&str]) -> Vec<String> {
    paths.iter().map(|p| (*p).to_owned()).collect()
}

/// The failures as `rule source -> missing`, for short asserts.
fn run(
    rows: &str,
    files: &[&str],
    diff: Option<&[&str]>,
    text: &str,
) -> Result<Vec<String>, Error> {
    let (files, diff) = (owned(files), diff.map(owned));
    let read = |path: &str| (path == "spec/factories/users.rb").then(|| text.to_owned());
    let input = Input {
        files: &files,
        diff: diff.as_deref(),
        read: &read,
        plurals: &[],
    };
    let show = |f: Failure| format!("{} {} -> {}", f.rule, f.source, f.missing);
    Ok(evaluate(&rules(rows)?, &input)
        .into_iter()
        .map(show)
        .collect())
}

const SPEC: &str = "spec|exists|app/**/*.rb|spec/{dir\\|strip:app/}/{stem}_spec.rb|app/assets/**\n";

/// T2: every app file has its spec; an excluded one needs none.
#[test]
fn exists_names_the_missing_companion() -> Result<(), Error> {
    let files = [
        "app/models/user.rb",
        "app/models/post.rb",
        "spec/models/user_spec.rb",
        "app/assets/x.rb",
    ];
    assert_eq!(
        run(SPEC, &files, None, "")?,
        ["spec app/models/post.rb -> spec/models/post_spec.rb"]
    );
    Ok(())
}

const ORPHAN: &str =
    "lonely|orphan|spec/**/*_spec.rb|app/{dir\\|strip:spec/}/{stem\\|chop:_spec}.rb|-\n";

/// T2: a spec whose implementation is gone is an orphan.
#[test]
fn orphan_names_the_missing_source() -> Result<(), Error> {
    let files = [
        "spec/models/user_spec.rb",
        "spec/models/gone_spec.rb",
        "app/models/user.rb",
    ];
    assert_eq!(
        run(ORPHAN, &files, None, "")?,
        ["lonely spec/models/gone_spec.rb -> app/models/gone.rb"]
    );
    Ok(())
}

const FACTORY: &str =
    "factory|mentions|app/models/*.rb|spec/factories/** :: factory :{stem\\|snake}|-\n";

/// T3, T5: `mentions` reads factories literally; editing that text flips
/// its result, and only its result (V5).
#[test]
fn mentions_reads_the_text_and_nothing_else_does() -> Result<(), Error> {
    let files = [
        "app/models/user.rb",
        "spec/factories/users.rb",
        "spec/models/user_spec.rb",
    ];
    let rows = format!("{FACTORY}{}", SPEC.replace("|app/assets/**", "|-"));
    assert!(run(&rows, &files, None, "factory :user do\nend\n")?.is_empty());
    let want = ["factory app/models/user.rb -> `factory :user` in spec/factories/**"];
    assert_eq!(run(&rows, &files, None, "factory :admin do\nend\n")?, want);
    Ok(())
}

const CHANGED: &str = "doc|changed|src/*.rs|docs/{stem}.md|-\n";

/// T4, V3: `changed` judges only with a diff; with one, a source changed
/// without its doc fails.
#[test]
fn changed_runs_only_with_a_diff() -> Result<(), Error> {
    let files = ["src/a.rs", "src/b.rs", "docs/a.md", "docs/b.md"];
    assert!(run(CHANGED, &files, None, "")?.is_empty());
    let diff = ["src/a.rs", "docs/a.md", "src/b.rs"];
    assert_eq!(
        run(CHANGED, &files, Some(&diff), "")?,
        ["doc src/b.rs -> docs/b.md"]
    );
    Ok(())
}

/// V1 and the row shape: a bad kind, template or mentions target is
/// refused by line.
#[test]
fn a_malformed_rule_is_refused_by_line() {
    for (row, want) in [
        ("r|nope|*|x|-", ".pklith:4: rule `r`: unknown kind `nope`"),
        (
            "r|exists|*|{bad}|-",
            ".pklith:4: rule `r`: unknown template variable `bad`",
        ),
        (
            "r|mentions|*|x|-",
            ".pklith:4: rule `r`: a mentions target is `glob :: pattern`",
        ),
        ("r|exists|src/[x|y|-", ".pklith:4: rule `r`: "),
    ] {
        let got = rules(row).err().map(|e| e.to_string()).unwrap_or_default();
        assert!(got.starts_with(want), "{row}: {got}");
    }
}
