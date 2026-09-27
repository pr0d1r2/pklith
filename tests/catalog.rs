//! Root V30 (T66): every built-in catalog check, run the way its hk step
//! runs it, fails on a planted violation and passes on a clean twin. The
//! fixtures are written from here rather than tracked, so pklith's own gate
//! never sees the violations; the few strings its gate would catch are
//! assembled at run time.

mod common;

use common::Result;
use std::path::Path;
use std::process::Command;

/// One fixture path: a file with its bytes, or a symlink to a target.
enum Entry {
    File(&'static str, String),
    Link(&'static str, &'static str),
    /// A path staged in the index only, with the blob of the file named
    /// second: how a case twin exists on a case-insensitive filesystem.
    Staged(&'static str, &'static str),
}

use Entry::{File, Link, Staged};

impl Entry {
    fn staged(&self) -> Option<(&'static str, &'static str)> {
        match self {
            Staged(path, like) => Some((path, like)),
            _ => None,
        }
    }
}

fn file(path: &'static str, text: &str) -> Entry {
    File(path, text.to_owned())
}

/// A throwaway git repository holding `entries`, all staged, and the paths
/// its index holds, which is what hk passes as `{{files}}`. Its git runs
/// without the hook's `GIT_DIR` and `GIT_INDEX_FILE`: inside a commit from a
/// linked worktree those would stage the fixture into the real index.
fn fixture(name: &str, entries: &[Entry]) -> Result<(std::path::PathBuf, String)> {
    let dir = std::env::temp_dir().join(format!("pklith-catalog-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;
    entries.iter().try_for_each(|e| write(&dir, e))?;
    git(&dir, &["init", "-q"])?;
    git(&dir, &["add", "-A"])?;
    for (path, like) in entries.iter().filter_map(Entry::staged) {
        stage(&dir, path, like)?;
    }
    let files = git(&dir, &["ls-files"])?
        .lines()
        .collect::<Vec<_>>()
        .join(" ");
    Ok((dir, files))
}

/// Stage `path` in the index with the blob already staged at `like`.
fn stage(dir: &Path, path: &str, like: &str) -> Result {
    let blob = git(dir, &["rev-parse", &format!(":{like}")])?;
    let info = format!("100644,{blob},{path}");
    git(dir, &["update-index", "--add", "--cacheinfo", &info])?;
    Ok(())
}

fn write(dir: &Path, entry: &Entry) -> Result {
    let path = |p: &str| -> Result<std::path::PathBuf> {
        let path = dir.join(p);
        std::fs::create_dir_all(path.parent().unwrap_or(dir))?;
        Ok(path)
    };
    match entry {
        File(p, text) => std::fs::write(path(p)?, text)?,
        Link(p, target) => std::os::unix::fs::symlink(target, path(p)?)?,
        Staged(..) => {}
    }
    Ok(())
}

fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let out = pklith::proc::command("git", dir).args(args).output()?;
    Ok(String::from_utf8(out.stdout)?.trim_end().to_owned())
}

/// The step's command exactly as gen emits it (guard and failure message
/// included), with `{{files}}` naming every staged path, run by
/// `sh` with only PATH, HOME and the check's own env. The real HOME stays,
/// as it does under hk: what a developer's home holds (user gems, cargo
/// config) is exactly what a step must survive. Build output goes to the
/// fixture, not the developer's cargo target.
fn command(check: &pklith::catalog::Check, dir: &Path, files: &str) -> Command {
    let mut cmd = Command::new("sh");
    cmd.arg("-c")
        .arg(pklith::r#gen::explained(check).replace("{{files}}", files))
        .current_dir(dir)
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("HOME", std::env::var_os("HOME").unwrap_or_default())
        .env("CARGO_TARGET_DIR", dir.join("target"));
    for pair in &check.env {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        cmd.env(key, value);
    }
    cmd
}

/// Whether the command passed, and everything it printed.
fn run(mut cmd: Command) -> Result<(bool, String)> {
    let out = cmd.output()?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    Ok((out.status.success(), text))
}

/// The built-in `id` fails on `bad` and passes on `good`.
fn proves(id: &str, bad: &[Entry], good: &[Entry]) -> Result {
    let check = pklith::catalog::builtin()?
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| format!("no built-in check `{id}`"))?;
    for (twin, entries, want) in [("bad", bad, false), ("good", good, true)] {
        let (dir, files) = fixture(&format!("{id}-{twin}"), entries)?;
        let (passed, output) = run(command(&check, &dir, &files))?;
        assert_eq!(passed, want, "`{id}` on its {twin} fixture:\n{output}");
        if !want {
            explains(&check, &output);
        }
        std::fs::remove_dir_all(dir)?;
    }
    Ok(())
}

/// A failure prints the check's message (catalog V1) and does not blame a
/// missing tool (root V1).
fn explains(check: &pklith::catalog::Check, output: &str) {
    let (id, message) = (&check.id, format!("{}: {}", check.id, check.msg));
    assert!(
        output.contains(&message),
        "`{id}` failed without its message:\n{output}"
    );
    assert!(
        !output.contains("MISSING TOOL"),
        "`{id}` blamed a tool:\n{output}"
    );
}

macro_rules! proven {
    ($($name:ident => $id:literal: $bad:expr, $good:expr;)*) => {
        /// Every id with a fixture below.
        const PROVEN: &[&str] = &[$($id),*];
        $(
            #[test]
            fn $name() -> Result {
                proves($id, &$bad, &$good)
            }
        )*
    };
}

/// V30: no built-in check ships without a fixture, and no fixture outlives
/// its check.
#[test]
fn every_builtin_check_has_a_fixture() -> Result {
    let mut ids: Vec<String> = pklith::catalog::builtin()?
        .into_iter()
        .map(|c| c.id)
        .collect();
    let mut proven: Vec<&str> = PROVEN.to_vec();
    ids.sort();
    proven.sort_unstable();
    assert_eq!(ids, proven);
    Ok(())
}

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
    nixfmt => "nixfmt": [file("a.nix", "{a=1;}")], [file("a.nix", "{ a = 1; }\n")];
    statix => "statix": [file("a.nix", "{ a = a; }\n")], [file("a.nix", "{ a = 1; }\n")];
    deadnix => "deadnix": [file("a.nix", "{ x }: 1\n")], [file("a.nix", "{ x }: x\n")];
    markdownlint => "markdownlint": [file("a.md", "# A\n# A\n")], [file("a.md", "# A\n\ntext\n")];
    yamllint => "yamllint": [file("a.yml", "a: 1\na: 2\n")], [file("a.yml", "---\na: 1\n")];
    xmllint => "xmllint": [file("a.xml", "<a><b></a>\n")], [file("a.xml", "<a/>\n")];
    shfmt => "shfmt": [file("a.sh", "if true;then echo;fi\n")], [file("a.sh", "if true; then echo; fi\n")];
    taplo => "taplo": [file("a.toml", "a=1\n")], [file("a.toml", "a = 1\n")];
    rustfmt => "rustfmt": crate_with("pub fn f(){}\n"), crate_with("pub fn f() {}\n");
    typos => "typos": [File("a.txt", format!("the t{}h cat\n", "e"))], [file("a.txt", "the cat\n")];
    shellcheck => "shellcheck": [file("a.sh", "#!/bin/sh\necho $1\n")], [file("a.sh", "#!/bin/sh\necho \"$1\"\n")];
    clippy => "clippy": crate_with("pub fn f(v: &Vec<u8>) -> usize {\n    v.len()\n}\n"), crate_with("pub fn f(v: &[u8]) -> usize {\n    v.len()\n}\n");
    rubocop => "rubocop": [file("a.rb", "puts \"x\"\n")], [file("a.rb", "# frozen_string_literal: true\n\nputs 'x'\n")];
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
