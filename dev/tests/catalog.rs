//! `pklith-dev catalog` against a fixture catalog, through the built
//! binary.

use std::path::{Path, PathBuf};
use std::process::Command;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const CATALOG: &str = "format 1\n## checks\nid|category|nix|glob|check|fix|env|msg\nws|hygiene|-|**/*|c|f|-|blanks\n## fragments\nfragment|triggers|checks|seed\nbase|always|ws|-\n";
const DOC: &str = "# Catalog\n\n<!-- BEGIN catalog -->\n<!-- END catalog -->\n";

fn fixture(name: &str, catalog: Option<&str>, doc: &str) -> Result<PathBuf> {
    let dir =
        std::env::temp_dir().join(format!("pklith-dev-catalog-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/catalog"))?;
    std::fs::create_dir_all(dir.join("docs"))?;
    if let Some(text) = catalog {
        std::fs::write(dir.join("src/catalog/builtin.pklith"), text)?;
    }
    std::fs::write(dir.join("docs/CATALOG.md"), doc)?;
    Ok(dir)
}

fn catalog(dir: &Path, check: bool) -> Result<(Option<i32>, String)> {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_pklith-dev"));
    cmd.arg("catalog").current_dir(dir);
    if check {
        cmd.arg("--check");
    }
    let out = cmd.output()?;
    Ok((out.status.code(), String::from_utf8(out.stderr)?))
}

/// The tables render from the catalog; a check passes; a hand edit drifts.
#[test]
fn the_tables_render_then_check() -> Result {
    let dir = fixture("render", Some(CATALOG), DOC)?;
    assert_eq!(catalog(&dir, false)?, (Some(0), String::new()));
    let text = std::fs::read_to_string(dir.join("docs/CATALOG.md"))?;
    assert!(
        text.contains("| `ws` | hygiene | hk built-in | `**/*` | yes | blanks |"),
        "{text}"
    );
    assert_eq!(catalog(&dir, true)?.0, Some(0));
    std::fs::write(dir.join("docs/CATALOG.md"), text.replace("blanks", "gaps"))?;
    assert_eq!(catalog(&dir, true)?.0, Some(1));
    Ok(std::fs::remove_dir_all(dir)?)
}

const INVALID: &str =
    "format 1\n## checks\nid|category|nix|glob|check|fix|env|msg\nws|bogus|-|*|c|-|-|m\n";

/// A catalog source, a doc, and the error they give.
const BROKEN: [(Option<&str>, &str, &str); 3] = [
    (Some(INVALID), DOC, "src/catalog/builtin.pklith: "),
    (None, DOC, "cannot read src/catalog/builtin.pklith"),
    (
        Some(CATALOG),
        "# no block\n",
        "no <!-- BEGIN catalog --> block",
    ),
];

/// An invalid or missing catalog, and a doc without the block, are exit 2.
#[test]
fn broken_inputs_exit_2() -> Result {
    for (i, (source, doc, want)) in BROKEN.into_iter().enumerate() {
        let dir = fixture(&format!("broken-{i}"), source, doc)?;
        let (code, stderr) = catalog(&dir, true)?;
        assert!(code == Some(2) && stderr.contains(want), "{want}: {stderr}");
        std::fs::remove_dir_all(dir)?;
    }
    Ok(())
}
