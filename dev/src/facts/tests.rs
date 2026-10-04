use super::{day, dependencies, floor, nixpkgs, platforms, quoted, rows};

const CARGO: &str = "[package]\nname = \"x\"\nlicense = \"MIT\"\n\n[dependencies]\n# a comment\nglobset = { version = \"0.4\" }\nserde.workspace = true\n\n[lints]\nworkspace = true\n";

#[test]
fn cargo_fields_and_dependencies_are_read() {
    assert_eq!(quoted(CARGO, "license").as_deref(), Some("MIT"));
    assert_eq!(quoted(CARGO, "edition"), None);
    assert_eq!(dependencies(CARGO), 2);
    assert_eq!(dependencies("[package]\n"), 0);
}

#[test]
fn the_floor_drops_trailing_zeros() {
    assert_eq!(floor("# c\nlines 100.00\n").as_deref(), Some("100"));
    assert_eq!(floor("lines 92.30").as_deref(), Some("92.3"));
    assert_eq!(floor("lines 97").as_deref(), Some("97"));
    assert_eq!(floor("lines 0.00").as_deref(), Some("0"));
    assert_eq!(floor("lines "), None);
    assert_eq!(floor("nothing"), None);
}

#[test]
fn the_floor_truncates_to_one_decimal() {
    assert_eq!(floor("lines 97.96").as_deref(), Some("97.9"));
}

/// Rows, not distinct ids: two nodes' V1 are two invariants.
#[test]
fn spec_rows_are_counted_per_node() {
    let specs = vec![
        "V1: a\nV2: b\nVx: no\nB1|d|c|f\n".to_owned(),
        "V1: c\nB1|d|c|f\nB2|d|c|f\n".to_owned(),
    ];
    assert_eq!((rows(&specs, 'V', ':'), rows(&specs, 'B', '|')), (3, 3));
    assert_eq!(rows(&["V: none\n".to_owned()], 'V', ':'), 0);
}

/// sherd dev B1: an input entry `"nixpkgs": [` comes first and is skipped;
/// the node is `"nixpkgs": {`.
const LOCK: &str = r#"{
  "nodes": {
    "nix-hk": {
      "inputs": {
        "nixpkgs": [
          "nixpkgs-lock",
          "nixpkgs"
        ]
      },
      "locked": { "rev": "decoydecoy" }
    },
    "nixpkgs": {
      "locked": {
        "lastModified": 1790340521,
        "rev": "f5c082a40f7571c266e74e80ae2e68aadd8a9fc7",
        "type": "github"
      },
      "original": {
        "ref": "nixos-26.05",
        "type": "github"
      }
    },
    "other": { "rev": "later" }
  }
}"#;

#[test]
fn the_nixpkgs_node_is_found_by_its_header() {
    let n = nixpkgs(LOCK);
    let want = ("26.05", "f5c082a", "2026-09-25");
    assert_eq!(
        n.as_ref()
            .map(|n| (n.series.as_str(), n.rev.as_str(), n.day.as_str())),
        Some(want)
    );
    assert_eq!(nixpkgs("{}"), None);
    assert_eq!(nixpkgs(&LOCK.replace("nixos-26.05", "26.05")), None);
}

#[test]
fn days_are_civil_dates() {
    assert_eq!(day(0).as_deref(), Some("1970-01-01"));
    assert_eq!(day(951_782_400).as_deref(), Some("2000-02-29"));
    assert_eq!(day(1_790_340_521).as_deref(), Some("2026-09-25"));
    assert_eq!(day(-1), None);
}

/// dev V3: platforms come from CI's matrix, in badge order.
#[test]
fn platforms_follow_the_ci_matrix() {
    let ci = "jobs:\n  gate:\n    matrix:\n      os: [macos-latest, ubuntu-24.04-arm, ubuntu-latest, macos-13]\n";
    let want = vec![
        ("intel", "linux"),
        ("amd", "linux"),
        ("arm", "linux"),
        ("intel", "macos"),
        ("arm", "macos"),
    ];
    assert_eq!(platforms(ci), Ok(want));
}

/// No matrix, or a runner of unknown platform, is an error naming it.
#[test]
fn an_unknown_matrix_is_refused() {
    assert_eq!(
        platforms("jobs: {}\n"),
        Err("ci.yml has no `os: [...]` matrix".to_owned())
    );
    let odd = platforms("os: [windows-latest]\n");
    assert_eq!(
        odd,
        Err("ci.yml runner `windows-latest` has no known platform".to_owned())
    );
}
