//! `pkli`, the pklith command line. Wiring only: `pklith::cli` does the work.

use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cwd = std::env::current_dir().unwrap_or_default();
    let outcome = pklith::cli::run(&args, &cwd);
    // A closed pipe is not worth a panic; the exit code still says it all.
    let _ = std::io::stdout().write_all(outcome.stdout.as_bytes());
    let _ = std::io::stderr().write_all(outcome.stderr.as_bytes());
    ExitCode::from(outcome.code)
}
