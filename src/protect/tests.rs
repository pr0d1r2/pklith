//! Context derivation goldens (protect T1).

use super::{contexts, diff, payload};

fn files(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .map(|(p, t)| ((*p).to_owned(), (*t).to_owned()))
        .collect()
}

const CI: &str = "name: ci\non:\n  pull_request:\njobs:\n  lint:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          persist-credentials: false\n      - run: true\n  test:\n    name: \"unit tests\"\n    strategy:\n      fail-fast: false\n      matrix:\n        os: [ubuntu-latest, macos-latest]\n        rust:\n          - stable\n          - '1.95'\n    runs-on: ${{ matrix.os }}\n  gate:\n    uses: ./.github/workflows/guard.yml\n";

const GUARD: &str = "on:\n  workflow_call:\njobs:\n  check:\n    runs-on: ubuntu-latest\n";

/// A job's name or id; a matrix job per combination, keys in declared
/// order; a reusable call as `caller / callee`; the reusable workflow
/// reports nothing of its own. Sorted.
#[test]
fn contexts_follow_github_naming() {
    let found = contexts(&files(&[
        (".github/workflows/ci.yml", CI),
        (".github/workflows/guard.yml", GUARD),
    ]));
    let want = [
        "gate / check",
        "lint",
        "unit tests (macos-latest, 1.95)",
        "unit tests (macos-latest, stable)",
        "unit tests (ubuntu-latest, 1.95)",
        "unit tests (ubuntu-latest, stable)",
    ];
    assert_eq!(found.unwrap_or_default(), want);
}

const UNNAMABLE: [(&str, &str); 3] = [
    (
        "on: push\njobs:\n  t:\n    strategy:\n      matrix:\n        include:\n          - os: x\n",
        "a matrix `include` changes its combinations",
    ),
    (
        "on: push\njobs:\n  t:\n    uses: org/repo/.github/workflows/x.yml@v1\n",
        "job `t` calls org/repo/.github/workflows/x.yml@v1, which pkli cannot read",
    ),
    (
        "on: push\njobs:\n  t:\n    uses: ./.github/workflows/nope.yml\n",
        "job `t` calls .github/workflows/nope.yml, which is not tracked",
    ),
];

/// What cannot be named exactly is refused: a matrix `include`, a remote
/// reusable workflow, a local one that is not there.
#[test]
fn unnamable_contexts_are_refused() {
    for (text, want) in UNNAMABLE {
        let err = contexts(&files(&[("w.yml", text)]))
            .err()
            .unwrap_or_default();
        assert!(err.starts_with("w.yml: ") && err.contains(want), "{err}");
    }
}

/// V2: the diff names what applying would add and remove; the payload is
/// the JSON body gh sends.
#[test]
fn diff_and_payload() {
    let own = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    let (added, removed) = diff(&own(&["lint", "test"]), &own(&["lint", "unit"]));
    assert_eq!((added, removed), (own(&["unit"]), own(&["test"])));
    assert_eq!(
        payload(&own(&["a \"b\"", "c"])),
        "{\"strict\":true,\"contexts\":[\"a \\\"b\\\"\",\"c\"]}\n"
    );
}
