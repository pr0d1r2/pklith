//! Run external programs one way (`src/proc/SPEC.md`): the git hook
//! environment removed, failures named.

use std::fmt;
use std::path::Path;
use std::process::Command;

/// Variables git exports to hooks. A child that inherits them acts on the
/// hook's repository whatever directory it runs in (root V20, set-and-setting
/// B97).
pub const HOOK_VARS: [&str; 3] = ["GIT_DIR", "GIT_INDEX_FILE", "GIT_WORK_TREE"];

/// A command for `program` run in `dir`, with [`HOOK_VARS`] removed (V1).
#[must_use]
pub fn command(program: &str, dir: &Path) -> Command {
    let mut cmd = Command::new(program);
    cmd.current_dir(dir);
    for var in HOOK_VARS {
        cmd.env_remove(var);
    }
    cmd
}

/// Why a program produced no output (V2).
#[derive(Debug)]
pub enum Error {
    /// It could not be started.
    Spawn {
        /// The program asked for.
        program: String,
        /// What the OS said.
        source: std::io::Error,
    },
    /// It ran and exited non-zero.
    Failed {
        /// The program and its arguments, as run.
        command: String,
        /// What it wrote to stderr.
        stderr: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn { program, source } => write!(f, "{program}: could not start: {source}"),
            Self::Failed { command, stderr } => write!(f, "`{command}` failed: {}", stderr.trim()),
        }
    }
}

impl std::error::Error for Error {}

/// Run `cmd` and return its stdout as bytes (V3).
///
/// # Errors
///
/// [`Error::Spawn`] when it cannot start; [`Error::Failed`] when it exits
/// non-zero, carrying its stderr.
pub fn output(cmd: &mut Command) -> Result<Vec<u8>, Error> {
    let program = cmd.get_program().to_string_lossy().into_owned();
    let out = cmd
        .output()
        .map_err(|source| Error::Spawn { program, source })?;
    if out.status.success() {
        return Ok(out.stdout);
    }
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    Err(Error::Failed {
        command: describe(cmd),
        stderr,
    })
}

fn describe(cmd: &Command) -> String {
    let args = cmd.get_args().map(|a| a.to_string_lossy().into_owned());
    std::iter::once(cmd.get_program().to_string_lossy().into_owned())
        .chain(args)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::{Error, HOOK_VARS, command, output};
    use std::ffi::OsStr;
    use std::path::Path;

    fn sh(script: &str) -> std::process::Command {
        let mut cmd = command("sh", Path::new("."));
        cmd.args(["-c", script]);
        cmd
    }

    #[test]
    fn every_hook_variable_is_removed() {
        let cmd = command("git", Path::new("."));
        let removed: Vec<_> = cmd
            .get_envs()
            .filter(|(_, v)| v.is_none())
            .map(|(k, _)| k)
            .collect();
        let expected: Vec<_> = HOOK_VARS.iter().map(OsStr::new).collect();
        assert_eq!(removed, expected);
    }

    #[test]
    fn success_returns_stdout_bytes() -> Result<(), Error> {
        assert_eq!(output(&mut sh("printf 'a\\0b'"))?, b"a\0b");
        Ok(())
    }

    #[test]
    fn a_non_zero_exit_names_the_command_and_its_stderr() {
        let err = output(&mut sh("echo boom >&2; exit 3"))
            .err()
            .map(|e| e.to_string());
        assert_eq!(
            err.as_deref(),
            Some("`sh -c echo boom >&2; exit 3` failed: boom")
        );
    }

    #[test]
    fn a_missing_program_is_a_spawn_error() {
        let err = output(&mut command("pklith-no-such-program", Path::new(".")));
        assert!(matches!(err, Err(Error::Spawn { .. })));
        assert!(err.err().is_some_and(|e| {
            e.to_string()
                .starts_with("pklith-no-such-program: could not start")
        }));
    }
}
