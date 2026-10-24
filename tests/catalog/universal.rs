//! Catalog T11 beyond the macOS fixture: `cargo-universal` skips off
//! macOS and on a library-only crate, and reads `lipo` and `otool` the way
//! it must, on every host. Stubs on PATH stand in for `uname`, `lipo` and
//! `otool`, so Linux CI proves the decisions the real build reaches only
//! on a Mac.

use super::{Entry, Result, command, file, fixture, run};
use std::path::Path;
use std::process::Command;

/// The row run as hk runs it, or with `args` (a binary to judge).
fn cmd(dir: &Path, args: &[&str]) -> Result<Command> {
    let check = pklith::catalog::builtin()?
        .into_iter()
        .find(|c| c.id == "cargo-universal")
        .ok_or("no built-in check `cargo-universal`")?;
    if args.is_empty() {
        return Ok(command(&check, dir, ""));
    }
    let mut cmd = Command::new("cargo-universal");
    cmd.args(args).current_dir(dir);
    Ok(cmd)
}

/// `cargo-universal` in a fixture of `entries`, behind `stubs`.
fn universal(entries: &[Entry], stubs: &[(&str, &str)], args: &[&str]) -> Result<(bool, String)> {
    let (dir, _) = fixture("universal", entries)?;
    let bin = dir.join("stubs");
    std::fs::create_dir_all(&bin)?;
    stubs
        .iter()
        .try_for_each(|(tool, body)| stub(&bin, tool, body))?;
    let mut cmd = cmd(&dir, args)?;
    let rest = std::env::var_os("PATH").unwrap_or_default();
    let dirs = std::iter::once(bin).chain(std::env::split_paths(&rest));
    cmd.env("PATH", std::env::join_paths(dirs)?);
    let ran = run(cmd);
    std::fs::remove_dir_all(dir)?;
    ran
}

fn stub(dir: &Path, tool: &str, body: &str) -> Result {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(tool);
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n"))?;
    let mode = std::fs::Permissions::from_mode(0o755);
    Ok(std::fs::set_permissions(path, mode)?)
}

/// `otool -arch A -l` output whose `LC_BUILD_VERSION` says `minos` for
/// `arm64` and for `x86_64`; `-` leaves the load command out.
fn otool(arm64: &str, x86_64: &str) -> String {
    let load = |minos: &str| match minos {
        "-" => "      cmd LC_VERSION_MIN_MACOSX\n  version 10.12".to_owned(),
        _ => format!(
            "      cmd LC_BUILD_VERSION\n  cmdsize 32\n platform 1\n    minos {minos}\n      sdk 14.4"
        ),
    };
    let (a, x) = (load(arm64), load(x86_64));
    format!(
        "case $2 in arm64) cat <<'A'\nbin:\n{a}\nA\n;; x86_64) cat <<'X'\nbin:\n{x}\nX\n;; esac"
    )
}

const DARWIN: (&str, &str) = ("uname", "echo Darwin");

/// A bin crate whose build would fail: nothing may build off macOS.
#[test]
fn off_macos_it_passes_with_a_notice() -> Result {
    let entries = [
        file("Cargo.toml", "[package]\nname = \"x\"\n"),
        file("src/main.rs", "nope\n"),
    ];
    let (passed, output) = universal(&entries, &[("uname", "echo Linux")], &[])?;
    assert!(passed, "{output}");
    let notice =
        "cargo-universal: skipped: lipo joins Mac slices on macOS only, and this host is Linux";
    assert!(output.contains(notice), "{output}");
    Ok(())
}

/// A library ships no binary, so nothing is built or judged.
#[test]
fn a_library_crate_passes_with_a_notice() -> Result {
    let toml = "[package]\nname = \"lib\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";
    let entries = [
        file("Cargo.toml", toml),
        file("src/lib.rs", "pub fn f() {}\n"),
    ];
    let (passed, output) = universal(&entries, &[DARWIN, ("lipo", "exit 1")], &[])?;
    assert!(passed, "{output}");
    assert!(
        output.contains("skipped: the workspace has no bin target"),
        "{output}"
    );
    Ok(())
}

/// What `lipo -archs` and each slice's `otool` print, whether the binary
/// passes, and what the step says.
const SLICES: [(&str, [&str; 2], bool, &str); 4] = [
    (
        "arm64",
        ["14.0", "14.0"],
        false,
        "bin holds [arm64], expected [arm64 x86_64]",
    ),
    (
        "x86_64 arm64",
        ["14.0", "26.0"],
        false,
        "bin x86_64 slice has minos 26.0, expected 14.0",
    ),
    (
        "x86_64 arm64",
        ["-", "14.0"],
        false,
        "bin arm64 slice has minos none, expected 14.0",
    ),
    (
        "x86_64 arm64",
        ["14.0", "14.0"],
        true,
        "bin holds arm64 x86_64, each minos 14.0",
    ),
];

/// Given a binary it judges the slices `lipo` and `otool` report: a
/// missing slice, a raised floor and no `LC_BUILD_VERSION` each fail,
/// naming what was found; both slices at 14.0 pass.
#[test]
fn it_reads_the_slices_lipo_and_otool_report() -> Result {
    for (archs, [arm64, x86_64], want, says) in SLICES {
        let (lipo, otool) = (format!("echo {archs}"), otool(arm64, x86_64));
        let stubs = [DARWIN, ("lipo", &lipo), ("otool", &otool)];
        let (passed, output) = universal(&[file("bin", "")], &stubs, &["bin"])?;
        assert!(passed == want && output.contains(says), "{output}");
    }
    Ok(())
}
