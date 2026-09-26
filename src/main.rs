//! `pkli`, the pklith command line.

use std::process::ExitCode;

/// Printed on stderr when no known command is given. Commands arrive with
/// their spec tasks; until then every invocation is a usage error.
const USAGE: &str = "usage: pkli <command>\n\nno commands exist yet; see SPEC.md.\n";

fn main() -> ExitCode {
    eprint!("{USAGE}");
    ExitCode::from(2)
}
