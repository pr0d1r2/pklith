//! `pklith-dev agents` against a fixture hk.pkl, evaluated by the real
//! `pkl`, through the built binary.

use std::path::{Path, PathBuf};
use std::process::Command;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const SCHEMA: &str = include_str!("../../pkl/Config.pkl");
const HK: &str = "amends \"pkl/Config.pkl\"\nlocal fast = new Mapping<String, Step> { [\"a\"] { glob = List(\"*.rs\"); check = \"true\"; fix = \"true\" } }\nhooks {\n  [\"pre-commit\"] { steps = fast }\n  [\"pre-push\"] { steps = (fast) { [\"b\"] { check = \"true\" } } }\n}\n";
const AGENTS: &str = "# Agents\n\n<!-- BEGIN steps -->\n<!-- END steps -->\n";

fn fixture(name: &str, agents: &str) -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("pklith-dev-agents-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("pkl"))?;
    std::fs::write(dir.join("pkl/Config.pkl"), SCHEMA)?;
    std::fs::write(dir.join("hk.pkl"), HK)?;
    std::fs::write(dir.join("AGENTS.md"), agents)?;
    Ok(dir)
}

fn agents(dir: &Path, check: bool) -> Result<(Option<i32>, String)> {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_pklith-dev"));
    cmd.arg("agents").current_dir(dir);
    if check {
        cmd.arg("--check");
    }
    let out = cmd.output()?;
    Ok((out.status.code(), String::from_utf8(out.stderr)?))
}

/// The table renders from hk.pkl; a check passes; a hand edit drifts.
#[test]
fn the_table_renders_then_checks() -> Result {
    let dir = fixture("render", AGENTS)?;
    assert_eq!(agents(&dir, false)?, (Some(0), String::new()));
    let text = std::fs::read_to_string(dir.join("AGENTS.md"))?;
    let want = "| `a` | commit | `*.rs` | yes |\n| `b` | push | whole tree | - |\n";
    assert!(
        text.contains("1 steps run on every commit and 2 on push") && text.contains(want),
        "{text}"
    );
    assert_eq!(agents(&dir, true)?.0, Some(0));
    std::fs::write(dir.join("AGENTS.md"), text.replace("| yes |", "| - |"))?;
    assert_eq!(agents(&dir, true)?.0, Some(1));
    Ok(std::fs::remove_dir_all(dir)?)
}

/// An AGENTS.md without the block is exit 2.
#[test]
fn a_missing_block_exits_2() -> Result {
    let dir = fixture("block", "# Agents\n")?;
    let (code, stderr) = agents(&dir, true)?;
    assert!(
        code == Some(2) && stderr.contains("no <!-- BEGIN steps --> block"),
        "{stderr}"
    );
    Ok(std::fs::remove_dir_all(dir)?)
}
