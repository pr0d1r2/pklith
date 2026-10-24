//! Catalog V5 (#108): on darwin the catalog's swiftlint finds the host
//! toolchain `SourceKitten` needs, and with none it fails as a missing tool
//! naming the framework, not as an abort a step would read as a finding.

/// `TOOLCHAIN_DIR` naming a directory with no `sourcekitdInProc.framework`
/// stands in for a host with neither Xcode nor the Command Line Tools.
#[cfg(target_os = "macos")]
#[test]
fn swiftlint_without_a_host_toolchain_is_a_missing_tool() -> super::Result {
    let entries = [super::file("a.swift", "let answer = 1\n")];
    let (dir, files) = super::fixture("swiftlint-no-toolchain", &entries)?;
    let out = std::process::Command::new("swiftlint")
        .args(["lint", "--quiet", &files])
        .current_dir(&dir)
        .env("TOOLCHAIN_DIR", dir.join("none"))
        .output()?;
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "{stderr}");
    assert!(stderr.contains("MISSING TOOL"), "{stderr}");
    assert!(stderr.contains("sourcekitdInProc"), "{stderr}");
    assert!(!stderr.contains("Fatal error"), "{stderr}");
    Ok(std::fs::remove_dir_all(dir)?)
}
