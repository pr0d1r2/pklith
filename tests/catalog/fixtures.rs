//! The planted-violation fixtures root V30 proves every built-in check
//! against: a bad twin its step must fail and a good twin it must pass.

use super::{Entry, File, Link, Staged, file};

fn conflict() -> String {
    let [ours, mid, theirs] = ["<", "=", ">"].map(|c| c.repeat(7));
    format!("{ours} HEAD\na\n{mid}\nb\n{theirs} x\n")
}

fn private_key() -> String {
    let kind = "RSA PRIVATE KEY";
    format!("-----BEGIN {kind}-----\nMIIBOgIBAAJBAKj34GkxFhD9\n-----END {kind}-----\n")
}

const SPEC: &str = "# SPEC\n\n## §G GOAL\n\nSay hello.\n\n## §N NAV\n\nrel|path|lens\nup|-|-\nself|.|-\n\n## §C CONSTRAINTS\n\n- one greeting.\n\n## §V INVARIANTS\n\nV1: greeting ! non-empty\n\n## §T TASKS\n\nid|status|task|cites\nT1|.|greet|V1\n\n## §B BUGS\n\nid|date|cause|fix\n";

fn spec(from: &str, to: &str) -> Entry {
    File("SPEC.md", SPEC.replacen(from, to, 1))
}

fn good_spec() -> Entry {
    file("SPEC.md", SPEC)
}

fn crate_with(lib: &str) -> [Entry; 2] {
    [
        file(
            "Cargo.toml",
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        ),
        file("src/lib.rs", lib),
    ]
}

/// A workspace whose root crate is clean and whose member `m` holds `lib`:
/// a check that reads only the root package passes it whatever `lib` says.
fn workspace_with(lib: &str) -> [Entry; 4] {
    [
        file(
            "Cargo.toml",
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n[workspace]\nmembers = [\"m\"]\n",
        ),
        file("src/lib.rs", "pub fn g() {}\n"),
        file(
            "m/Cargo.toml",
            "[package]\nname = \"m\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        ),
        file("m/src/lib.rs", lib),
    ]
}

/// The `.swiftlint.yml` `pklith seed` writes (seed T4): with it a force
/// unwrap fails, which swiftlint's defaults allow, and a short name such as
/// `a` passes, which they refuse.
fn swiftlint_seed() -> Entry {
    let text = match pklith::seed::templates::named("swiftlint") {
        Some(pklith::seed::templates::Template::File(_, text)) => text,
        _ => "",
    };
    file(".swiftlint.yml", text)
}

/// A script, its companion spec under the registry's `exists` rule, and
/// `tests/unit/a.bats` asserting the script prints 3: `prints` is what
/// the script prints, so 2 is the planted bug.
pub fn bats_suite(prints: u8) -> [Entry; 3] {
    [
        file(".pklith", BATS_RULE),
        File("scripts/a.sh", format!("echo {prints}\n")),
        file("tests/unit/a.bats", BATS_SPEC),
    ]
}

pub const BATS_RULE: &str = "format 1\n## rules\nid|kind|select|target|except\nbats|exists|scripts/*.sh|tests/unit/{stem}.bats|-\n";

const BATS_SPEC: &str = "@test \"runs\" {\n  sh \"$BATS_TEST_DIRNAME/../../scripts/a.sh\"\n}\n\n@test \"prints 3\" {\n  [ \"$(sh \"$BATS_TEST_DIRNAME/../../scripts/a.sh\")\" = 3 ]\n}\n";

const EDITORCONFIG: &str = "root = true\n[*]\nindent_style = space\n";

const WORKFLOW: &str = "on: push\npermissions: {}\njobs:\n  a:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hi\n";

fn workflow(text: &str) -> Entry {
    file(".github/workflows/ci.yml", text)
}

proven! {
    trailing_whitespace => "trailing-whitespace": [file("a.txt", "x \n")], [file("a.txt", "x\n")];
    final_newline => "final-newline": [file("a.txt", "x")], [file("a.txt", "x\n")];
    line_endings => "line-endings": [file("a.txt", "a\r\nb\n")], [file("a.txt", "a\nb\n")];
    no_bom => "no-bom": [file("a.txt", "\u{feff}x\n")], [file("a.txt", "x\n")];
    no_merge_conflict => "no-merge-conflict": [File("a.txt", conflict())], [file("a.txt", "a\n")];
    no_case_conflict => "no-case-conflict": [file("a.txt", "x\n"), Staged("A.txt", "a.txt")], [file("a.txt", "x\n")];
    no_broken_symlinks => "no-broken-symlinks": [Link("dead", "nowhere")], [file("a.txt", "x\n"), Link("live", "a.txt")];
    no_private_key => "no-private-key": [File("id_rsa", private_key())], [file("a.txt", "x\n")];
    ripsecrets => "ripsecrets": [File("env", format!("aws_access_key_id = AKIA{}\n", "Z7Q3VXJKL5PNR2WT"))], [file("env", "region = eu\n")];
    no_local_paths => "no-local-paths": [file("a.md", &format!("see {}alice/x\n", "/home/"))], [file("a.md", &format!("see {}alice/x # nolocalpath\nsee ~/x\n", "/home/"))];
    nixfmt => "nixfmt": [file("a.nix", "{a=1;}")], [file("a.nix", "{ a = 1; }\n")];
    statix => "statix": [file("a.nix", "{ a = a; }\n")], [file("a.nix", "{ a = 1; }\n")];
    deadnix => "deadnix": [file("a.nix", "{ x }: 1\n")], [file("a.nix", "{ x }: x\n")];
    markdownlint => "markdownlint": [file("a.md", "# A\n# A\n")], [file("a.md", "# A\n\ntext\n")];
    yamllint => "yamllint": [file("a.yml", "a: 1\na: 2\n")], [file("a.yml", "---\na: 1\n")];
    xmllint => "xmllint": [file("a.xml", "<a><b></a>\n")], [file("a.xml", "<a/>\n")];
    gawk_lint => "gawk-lint": [file("a.awk", "BEGIN { system(\"touch ran\"); x = substr(\"a\") }\n")], [file("a.awk", "BEGIN { print 1 }\n")];
    bats_parse => "bats-parse": [file("a.bats", "@test \"x\" {\n  true\n")], [file("a.bats", "@test \"x\" {\n  true\n}\n")];
    bats_changed => "bats-changed": bats_suite(2), bats_suite(3);
    bats_unit => "bats-unit": bats_suite(2), bats_suite(3);
    bats_unit_none => "bats-unit": [file("tests/AppTests.swift", "import XCTest\n")], bats_suite(3);
    shfmt => "shfmt": [file("a.sh", "if true;then echo;fi\n")], [file("a.sh", "if true; then echo; fi\n")];
    taplo => "taplo": [file("a.toml", "a=1\n")], [file("a.toml", "a = 1\n")];
    rustfmt => "rustfmt": crate_with("pub fn f(){}\n"), crate_with("pub fn f() {}\n");
    swift_format => "swift-format": [file("a.swift", "let answer=1\n")], [file("a.swift", "let answer = 1\n")];
    rustfmt_workspace => "rustfmt": workspace_with("pub fn f(){}\n"), workspace_with("pub fn f() {}\n");
    typos => "typos": [File("a.txt", format!("the t{}h cat\n", "e"))], [file("a.txt", "the cat\n")];
    shellcheck => "shellcheck": [file("a.sh", "#!/bin/sh\necho $1\n")], [file("a.sh", "#!/bin/sh\necho \"$1\"\n")];
    clippy => "clippy": crate_with("pub fn f(v: &Vec<u8>) -> usize {\n    v.len()\n}\n"), crate_with("pub fn f(v: &[u8]) -> usize {\n    v.len()\n}\n");
    clippy_workspace => "clippy": workspace_with("pub fn f(v: &Vec<u8>) -> usize {\n    v.len()\n}\n"), workspace_with("pub fn f(v: &[u8]) -> usize {\n    v.len()\n}\n");
    rubocop => "rubocop": [file("a.rb", "puts \"x\"\n")], [file("a.rb", "# frozen_string_literal: true\n\nputs 'x'\n")];
    swiftlint => "swiftlint": [file("a.swift", "let answer = try! compute()\n")], [file("a.swift", "let answer = try compute()\n")];
    swiftlint_seeded => "swiftlint": [swiftlint_seed(), file("a.swift", "let answer = value!\n")], [swiftlint_seed(), file("a.swift", "let a = value ?? 1\n")];
    actionlint => "actionlint": [workflow(&format!("{WORKFLOW}    bogus: 1\n"))], [workflow(WORKFLOW)];
    zizmor => "zizmor": [workflow(&WORKFLOW.replace("on: push", "on: issues").replace("echo hi", "echo \"${{ github.event.issue.title }}\""))], [workflow(WORKFLOW)];
    editorconfig_checker => "editorconfig-checker": [file(".editorconfig", EDITORCONFIG), file("a.txt", "a\n\tb\n")], [file(".editorconfig", EDITORCONFIG), file("a.txt", "a\n  b\n")];
    mth_fmt => "mth-fmt": [spec("- one greeting.", "- one\n  greeting.")], [good_spec()];
    mth_check => "mth-check": [spec("V1: greeting", "V1: dup\nV1: greeting")], [good_spec()];
    sherd_check => "sherd-check": [spec("T1|.|greet|V1", "T1|.|greet|V9")], [good_spec()];
    sherd_nav => "sherd-nav": [spec("## §N NAV\n\nrel|path|lens\nup|-|-\nself|.|-\n\n", "")], [good_spec()];
    sherd_budget => "sherd-budget": [good_spec(), file(".context-limits", "SPEC.md  10\n")], [good_spec(), file(".context-limits", "SPEC.md  5000\n")];
    itok => "itok": [good_spec(), file(".context-limits", "SPEC.md  10\n")], [good_spec(), file(".context-limits", "SPEC.md  5000\n")];
}
