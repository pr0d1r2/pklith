//! `pklith-dev`: tooling that maintains this repository and ships to
//! nobody (`dev/SPEC.md`). Reads the repository in the current directory;
//! every rule lives in a pure function over text.
//!
//!   pklith-dev readme [--check]   the README badge block and the numbers
//!                                  docs/LLM-DISCLAIMER.md quotes
//!
//! Exit 0 clean, 1 drift found by `--check`, 2 usage or I/O.

mod badges;
mod facts;

use std::path::Path;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match words.as_slice() {
        ["readme"] => readme(false),
        ["readme", "--check"] => readme(true),
        _ => Err((2, "usage: pklith-dev readme [--check]".to_owned())),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err((code, message)) => {
            eprintln!("pklith-dev: {message}");
            ExitCode::from(code)
        }
    }
}

type Failed = (u8, String);

fn read(path: &str) -> Result<String, Failed> {
    std::fs::read_to_string(path).map_err(|e| (2, format!("cannot read {path}: {e}")))
}

/// Run `program` and return its stdout; any failure is exit 2.
fn run(program: &str, args: &[&str]) -> Result<String, Failed> {
    let out = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| (2, format!("cannot run {program}: {e}")))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err((
            2,
            format!("{program} {} failed: {}", args.join(" "), err.trim()),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

const DRIFT: &str =
    "README.md badges drifted from their sources; run `cargo run -p pklith-dev -- readme`";

fn readme(check: bool) -> Result<(), Failed> {
    let facts = gather()?.checked().map_err(|e| (2, e))?;
    let stale = badges::stale_prose(&read("docs/LLM-DISCLAIMER.md")?, &facts);
    if let Some(want) = stale.first() {
        let why = format!("docs/LLM-DISCLAIMER.md must say {want}, as its sources do");
        return Err((1, why));
    }
    let current = read("README.md")?;
    let wanted = badges::splice(&current, &badges::render(&facts))
        .ok_or((2, "README.md has no <!-- BEGIN badges --> block".to_owned()))?;
    match (wanted == current, check) {
        (true, _) => Ok(()),
        (false, true) => Err((1, DRIFT.to_owned())),
        (false, false) => std::fs::write("README.md", wanted)
            .map_err(|e| (2, format!("cannot write README.md: {e}"))),
    }
}

fn gather() -> Result<badges::Facts, Failed> {
    let cargo = read("Cargo.toml")?;
    let field = |key: &str| facts::quoted(&cargo, key).unwrap_or_default();
    let (floor, nixpkgs, platforms) = files()?;
    Ok(badges::Facts {
        slug: field("repository").replace("https://github.com/", ""),
        license: field("license"),
        edition: field("edition"),
        msrv: field("rust-version"),
        dependencies: facts::dependencies(&cargo),
        floor,
        steps: (steps("pre-commit")?, steps("pre-push")?),
        spec: spec()?,
        nixpkgs,
        platforms,
    })
}

/// The facts `.coverage`, `flake.lock` and CI's workflow own.
fn files() -> Result<(String, facts::Nixpkgs, Vec<facts::Platform>), Failed> {
    let missing = |what: &str| (2, format!("{what} holds no readable value"));
    let floor = facts::floor(&read(".coverage")?).ok_or_else(|| missing(".coverage"))?;
    let nixpkgs =
        facts::nixpkgs(&read("flake.lock")?).ok_or_else(|| missing("flake.lock nixpkgs"))?;
    let platforms = facts::platforms(&read(".github/workflows/ci.yml")?).map_err(|e| (2, e))?;
    Ok((floor, nixpkgs, platforms))
}

/// Steps in one hook, as `pkl` evaluates hk.pkl.
fn steps(hook: &str) -> Result<usize, Failed> {
    let expr = format!("hooks[\"{hook}\"].steps.length");
    let text = run("pkl", &["eval", "-x", &expr, "hk.pkl"])?;
    text.trim()
        .parse()
        .map_err(|_| (2, format!("pkl printed `{}` for {expr}", text.trim())))
}

/// `§V` rows, `§B` rows and nodes, over every tracked SPEC.md.
fn spec() -> Result<(usize, usize, usize), Failed> {
    let listed = run("git", &["ls-files", "-z", "*SPEC.md"])?;
    let texts = listed
        .split('\0')
        .filter(|p| !p.is_empty() && Path::new(p).is_file())
        .map(read)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((
        facts::rows(&texts, 'V', ':'),
        facts::rows(&texts, 'B', '|'),
        texts.len(),
    ))
}
