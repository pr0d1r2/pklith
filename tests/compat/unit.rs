//! The `lefthook-unit-coverage` bats suite (legacy T1, V2), case by case,
//! each walking its own root as the suite's `LEFTHOOK_UNIT_COVERAGE_ROOT`
//! did.

use super::{Result, run, scratch, write};

const UNIT: &str = "lefthook-unit-coverage";
const CONFIG: &str = ".unit-coverage.toml";

/// One case: `files` under the root (config included), the config the
/// environment names if any, and the whole stderr and exit code.
fn case(name: &str, files: &[(&str, &str)], config: Option<&str>, want: (i32, &str)) -> Result {
    let dir = scratch(name, UNIT)?;
    let repo = dir.join("repo");
    write(&repo, files)?;
    let mut env = vec![(
        "LEFTHOOK_UNIT_COVERAGE_ROOT",
        repo.to_str().unwrap_or_default(),
    )];
    env.extend(config.map(|c| ("LEFTHOOK_UNIT_COVERAGE_CONFIG", c)));
    let got = run(&dir, UNIT, &dir, &env)?;
    assert_eq!(got, (Some(want.0), want.1.to_owned()), "{name}");
    Ok(std::fs::remove_dir_all(dir)?)
}

/// The legacy report for one failing rule.
fn missing(rule: &str, lines: &[&str]) -> String {
    let n = lines.len();
    let listed: String = lines.iter().flat_map(|l| ["  ", l, "\n"]).collect();
    format!(
        "{UNIT}: {rule}: {n} file(s) missing spec:\n{listed}\nFix: add missing test files or allowlist paths in .coverage-allowlist.\n"
    )
}

const SH: &str = "\n[[rules]]\nglob = \"*.sh\"\ndirs = [\"scripts\"]\ntest_dir = \"tests/unit\"\npattern = \"mirror\"\ntest_ext = \"bats\"\n";
const RB: &str = "\n[[rules]]\nglob = \"*.rb\"\ndirs = [\"app\"]\ntest_dir = \"spec\"\npattern = \"mirror\"\nstrip = \"app\"\ntest_suffix = \"_spec\"\n";
const FLAT: &str = "\n[[rules]]\nglob = \"*.rb\"\ndirs = [\"lib\"]\ntest_dir = \"test\"\npattern = \"flat\"\ntest_suffix = \"_test\"\n";
const NORMALIZE: &str = "\n[[rules]]\nglob = \"*.sh\"\ndirs = [\"nix/dev\"]\ntest_dir = \"tests/unit\"\npattern = \"mirror\"\ntest_ext = \"bats\"\nnormalize = true\n";

#[test]
fn fails_when_config_file_missing() -> Result {
    let want = "lefthook-unit-coverage: .unit-coverage.toml not found\n  Create a .unit-coverage.toml with [[rules]] entries.\n";
    case("missing-config", &[], Some(CONFIG), (1, want))
}

#[test]
fn mirror_passes_when_all_specs_exist() -> Result {
    let files = [
        (CONFIG, SH),
        ("scripts/build.sh", ""),
        ("tests/unit/scripts/build.bats", ""),
    ];
    case("mirror-pass", &files, None, (0, ""))
}

#[test]
fn mirror_fails_when_spec_missing() -> Result {
    let want = missing(
        "rule 0 (*.sh)",
        &["scripts/build.sh -> tests/unit/scripts/build.bats"],
    );
    case(
        "mirror-fail",
        &[(CONFIG, SH), ("scripts/build.sh", "")],
        None,
        (1, &want),
    )
}

#[test]
fn mirror_preserves_nested_directories() -> Result {
    let config = SH.replace("[\"scripts\"]", "[\"scripts/just\"]");
    let files = [
        (CONFIG, config.as_str()),
        ("scripts/just/build/deploy.sh", ""),
        ("tests/unit/scripts/just/build/deploy.bats", ""),
    ];
    case("nested", &files, None, (0, ""))
}

#[test]
fn mirror_with_strip_strips_the_prefix() -> Result {
    let files = [
        (CONFIG, RB),
        ("app/models/user.rb", ""),
        ("spec/models/user_spec.rb", ""),
    ];
    case("strip-pass", &files, None, (0, ""))
}

#[test]
fn mirror_with_strip_fails_when_spec_missing() -> Result {
    let want = missing(
        "rule 0 (*.rb)",
        &["app/models/user.rb -> spec/models/user_spec.rb"],
    );
    case(
        "strip-fail",
        &[(CONFIG, RB), ("app/models/user.rb", "")],
        None,
        (1, &want),
    )
}

#[test]
fn flat_maps_to_the_test_dir_without_path() -> Result {
    let files = [
        (CONFIG, FLAT),
        ("lib/bundix/convert.rb", ""),
        ("test/convert_test.rb", ""),
    ];
    case("flat-pass", &files, None, (0, ""))
}

#[test]
fn flat_fails_when_spec_missing() -> Result {
    let want = missing(
        "rule 0 (*.rb)",
        &["lib/bundix/convert.rb -> test/convert_test.rb"],
    );
    case(
        "flat-fail",
        &[(CONFIG, FLAT), ("lib/bundix/convert.rb", "")],
        None,
        (1, &want),
    )
}

#[test]
fn normalize_converts_underscores_to_dashes() -> Result {
    let files = [
        (CONFIG, NORMALIZE),
        ("nix/dev/shell_hook.sh", ""),
        ("tests/unit/nix/dev/shell-hook.bats", ""),
    ];
    case("normalize", &files, None, (0, ""))
}

#[test]
fn normalize_also_accepts_the_unnormalized_name() -> Result {
    let files = [
        (CONFIG, NORMALIZE),
        ("nix/dev/shell_hook.sh", ""),
        ("tests/unit/nix/dev/shell_hook.bats", ""),
    ];
    case("normalize-raw", &files, None, (0, ""))
}

#[test]
fn exclude_skips_excluded_directories() -> Result {
    let config = format!("{SH}exclude = [\"scripts/lefthook\"]\n");
    let files = [
        (CONFIG, config.as_str()),
        ("scripts/lefthook/install.sh", ""),
        ("scripts/build/deploy.sh", ""),
        ("tests/unit/scripts/build/deploy.bats", ""),
    ];
    case("exclude", &files, None, (0, ""))
}

#[test]
fn allowlist_skips_allowlisted_files() -> Result {
    let files = [
        (CONFIG, SH),
        ("scripts/legacy.sh", ""),
        (".coverage-allowlist", "scripts/legacy.sh\n"),
    ];
    case("allowlist", &files, None, (0, ""))
}

#[test]
fn allowlist_ignores_comments_and_blank_lines() -> Result {
    let allow = "# This is a comment\n\nscripts/legacy.sh\n";
    let files = [
        (CONFIG, SH),
        ("scripts/legacy.sh", ""),
        (".coverage-allowlist", allow),
    ];
    case("allowlist-comments", &files, None, (0, ""))
}

#[test]
fn custom_allowlist_path_from_config() -> Result {
    let config = format!("allowlist = \".my-allowlist\"{SH}");
    let files = [
        (CONFIG, config.as_str()),
        ("scripts/legacy.sh", ""),
        (".my-allowlist", "scripts/legacy.sh\n"),
    ];
    case("allowlist-custom", &files, None, (0, ""))
}

#[test]
fn multiple_rules_are_all_checked() -> Result {
    let config = format!("{SH}{RB}");
    let files = [
        (CONFIG, config.as_str()),
        ("scripts/build.sh", ""),
        ("tests/unit/scripts/build.bats", ""),
        ("app/models/user.rb", ""),
        ("spec/models/user_spec.rb", ""),
    ];
    case("multi-pass", &files, None, (0, ""))
}

#[test]
fn a_failure_in_one_rule_fails_overall() -> Result {
    let config = format!("{SH}{RB}");
    let files = [
        (CONFIG, config.as_str()),
        ("scripts/build.sh", ""),
        ("tests/unit/scripts/build.bats", ""),
        ("app/models/user.rb", ""),
    ];
    let want = missing(
        "rule 1 (*.rb)",
        &["app/models/user.rb -> spec/models/user_spec.rb"],
    );
    case("multi-fail", &files, None, (1, &want))
}

#[test]
fn files_outside_configured_dirs_are_ignored() -> Result {
    let files = [
        (CONFIG, SH),
        ("scripts/build.sh", ""),
        ("tests/unit/scripts/build.bats", ""),
        ("other/random.sh", ""),
    ];
    case("outside", &files, None, (0, ""))
}

#[test]
fn test_suffix_test_for_minitest() -> Result {
    let config = RB.replace("\"spec\"", "\"test\"").replace("_spec", "_test");
    let files = [
        (CONFIG, config.as_str()),
        ("app/controllers/users_controller.rb", ""),
        ("test/controllers/users_controller_test.rb", ""),
    ];
    case("minitest", &files, None, (0, ""))
}

#[test]
fn unknown_pattern_fails_with_error() -> Result {
    let config = "\n[[rules]]\nglob = \"*.sh\"\ndirs = [\"scripts\"]\ntest_dir = \"tests\"\npattern = \"unknown\"\n";
    let want = "lefthook-unit-coverage: unknown pattern 'unknown' in rule 0\n";
    case(
        "unknown",
        &[(CONFIG, config), ("scripts/build.sh", "")],
        None,
        (1, want),
    )
}

#[test]
fn no_rules_passes_vacuously() -> Result {
    case("no-rules", &[(CONFIG, "\n")], None, (0, ""))
}

#[test]
fn custom_config_path_via_env_var() -> Result {
    let files = [
        ("custom.toml", SH),
        ("scripts/build.sh", ""),
        ("tests/unit/scripts/build.bats", ""),
    ];
    case("custom-config", &files, Some("custom.toml"), (0, ""))
}
