//! `pklith`, the pklith command line. Wiring only: `pklith::cli` does the work.
//! Installed under a legacy tool's name it behaves as that tool.

use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut words = std::env::args();
    let program = words.next().unwrap_or_default();
    let args: Vec<String> = words.collect();
    let cwd = std::env::current_dir().unwrap_or_default();
    let outcome = pklith::cli::run_as(&program, &args, &cwd);
    // A closed pipe is not worth a panic; the exit code still says it all.
    let _ = std::io::stdout().write_all(outcome.stdout.as_bytes());
    let _ = std::io::stderr().write_all(outcome.stderr.as_bytes());
    ExitCode::from(outcome.code)
}
