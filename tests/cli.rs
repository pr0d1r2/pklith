//! The exit-code contract of `src/cli` (V1-V3), run against the built binary.

use std::process::Command;

#[test]
fn no_command_is_a_usage_error() -> Result<(), Box<dyn std::error::Error>> {
    let out = Command::new(env!("CARGO_BIN_EXE_pkli")).output()?;
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty(), "stdout carries data only");
    assert!(String::from_utf8(out.stderr)?.starts_with("usage: pkli"));
    Ok(())
}
