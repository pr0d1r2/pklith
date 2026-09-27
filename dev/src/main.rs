//! `pklith-dev`: tooling that maintains this repository and ships to
//! nobody (`dev/SPEC.md`). Reads the repository in the current directory;
//! every rule lives in a pure function over text, and `io` alone touches
//! the repository.
//!
//! ```text
//! pklith-dev readme [--check]      README badges and the disclaimer's numbers
//! pklith-dev notices [--check]     the third-party notices' crate table
//! pklith-dev agents [--check]      the step table in AGENTS.md
//! pklith-dev catalog [--check]     the tables in docs/CATALOG.md
//! pklith-dev cli [--check]         the usage block in docs/CLI.md
//! pklith-dev commit-msg [FILE]     the commit message style rule
//! pklith-dev changelog [FILE]      a behaviour change adds a CHANGELOG entry
//! ```
//!
//! Exit 0 clean, 1 drift or a broken rule, 2 usage or I/O.

mod agents;
mod badges;
mod block;
mod catalog;
mod changelog;
mod cli;
mod facts;
mod hooks;
mod io;
mod message;
mod notices;
mod readme;

use std::process::ExitCode;

const USAGE: &str = "usage: pklith-dev <readme|notices|agents|catalog|cli> [--check] | <commit-msg|changelog> [FILE]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    dispatch(&args).map_or_else(
        |(code, message)| {
            eprintln!("pklith-dev: {message}");
            ExitCode::from(code)
        },
        |()| ExitCode::SUCCESS,
    )
}

fn dispatch(args: &[String]) -> Result<(), io::Failed> {
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    match words.as_slice() {
        ["commit-msg", rest @ ..] if rest.len() < 2 => hooks::commit_msg(rest.first().copied()),
        ["changelog", rest @ ..] if rest.len() < 2 => hooks::changelog(rest.first().copied()),
        [verb] => generated(verb, false),
        [verb, "--check"] => generated(verb, true),
        _ => Err((2, USAGE.to_owned())),
    }
}

/// A generated file, written or checked.
fn generated(verb: &str, check: bool) -> Result<(), io::Failed> {
    match verb {
        "readme" => readme::run_verb(check),
        "notices" => notices::run_verb(check),
        "agents" => agents::run_verb(check),
        "catalog" => catalog::run_verb(check),
        "cli" => cli::run_verb(check),
        _ => Err((2, USAGE.to_owned())),
    }
}
