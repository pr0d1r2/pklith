//! Root V30 (T66): every built-in catalog check, run the way its hk step
//! runs it, fails on a planted violation and passes on a clean twin. The
//! fixtures are written from here rather than tracked, so pklith's own gate
//! never sees the violations; the few strings its gate would catch are
//! assembled at run time.

mod common;

use common::Result;
use std::path::Path;
use std::process::Command;

/// One fixture path: a file with its bytes, or a symlink to a target.
enum Entry {
    File(&'static str, String),
    Link(&'static str, &'static str),
    /// A path staged in the index only, with the blob of the file named
    /// second: how a case twin exists on a case-insensitive filesystem.
    Staged(&'static str, &'static str),
}

use Entry::{File, Link, Staged};

impl Entry {
    fn staged(&self) -> Option<(&'static str, &'static str)> {
        match self {
            Staged(path, like) => Some((path, like)),
            _ => None,
        }
    }
}

fn file(path: &'static str, text: &str) -> Entry {
    File(path, text.to_owned())
}

/// A fresh directory name, unique per call: `cargo test` runs every case in
/// one process, and a check proven twice (a crate and a workspace) shares
/// its name.
fn scratch(name: &str) -> std::path::PathBuf {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let pid = std::process::id();
    std::env::temp_dir().join(format!("pklith-catalog-{pid}-{n}-{name}"))
}

/// A throwaway git repository holding `entries`, all staged, and the paths
/// its index holds, which is what hk passes as `{{files}}`. Its git runs
/// without the hook's `GIT_DIR` and `GIT_INDEX_FILE`: inside a commit from a
/// linked worktree those would stage the fixture into the real index.
fn fixture(name: &str, entries: &[Entry]) -> Result<(std::path::PathBuf, String)> {
    let dir = scratch(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;
    entries.iter().try_for_each(|e| write(&dir, e))?;
    git(&dir, &["init", "-q"])?;
    git(&dir, &["add", "-A"])?;
    for (path, like) in entries.iter().filter_map(Entry::staged) {
        stage(&dir, path, like)?;
    }
    let files = git(&dir, &["ls-files"])?
        .lines()
        .collect::<Vec<_>>()
        .join(" ");
    Ok((dir, files))
}

/// Stage `path` in the index with the blob already staged at `like`.
fn stage(dir: &Path, path: &str, like: &str) -> Result {
    let blob = git(dir, &["rev-parse", &format!(":{like}")])?;
    let info = format!("100644,{blob},{path}");
    git(dir, &["update-index", "--add", "--cacheinfo", &info])?;
    Ok(())
}

fn write(dir: &Path, entry: &Entry) -> Result {
    let path = |p: &str| -> Result<std::path::PathBuf> {
        let path = dir.join(p);
        std::fs::create_dir_all(path.parent().unwrap_or(dir))?;
        Ok(path)
    };
    match entry {
        File(p, text) => std::fs::write(path(p)?, text)?,
        Link(p, target) => std::os::unix::fs::symlink(target, path(p)?)?,
        Staged(..) => {}
    }
    Ok(())
}

fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let out = pklith::proc::command("git", dir).args(args).output()?;
    Ok(String::from_utf8(out.stdout)?.trim_end().to_owned())
}

/// The step's command exactly as gen emits it (guard and failure message
/// included), with `{{files}}` naming every staged path, run by
/// `sh` with only PATH, HOME and the check's own env. PATH leads with this
/// build's `pklith`, which a row such as `bats-changed` runs, so it is the
/// binary under test, not the dev shell's. The real HOME stays,
/// as it does under hk: what a developer's home holds (user gems, cargo
/// config) is exactly what a step must survive. Build output goes to the
/// fixture, not the developer's cargo target.
fn command(check: &pklith::catalog::Check, dir: &Path, files: &str) -> Command {
    let mut cmd = Command::new("sh");
    cmd.arg("-c")
        .arg(pklith::r#gen::explained(check).replace("{{files}}", files))
        .current_dir(dir)
        .env_clear()
        .env("PATH", path())
        .env("HOME", std::env::var_os("HOME").unwrap_or_default())
        .env("CARGO_TARGET_DIR", dir.join("target"));
    for pair in &check.env {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        cmd.env(key, value);
    }
    cmd
}

/// PATH with the directory of the built `pklith` first.
fn path() -> std::ffi::OsString {
    let built = Path::new(env!("CARGO_BIN_EXE_pklith")).parent();
    let rest = std::env::var_os("PATH").unwrap_or_default();
    let dirs = built.map(Path::to_path_buf).into_iter();
    std::env::join_paths(dirs.chain(std::env::split_paths(&rest))).unwrap_or(rest)
}

/// Whether the command passed, and everything it printed.
fn run(mut cmd: Command) -> Result<(bool, String)> {
    let out = cmd.output()?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    Ok((out.status.success(), text))
}

/// The built-in `id` fails on `bad` and passes on `good`.
fn proves(id: &str, bad: &[Entry], good: &[Entry]) -> Result {
    let check = pklith::catalog::builtin()?
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| format!("no built-in check `{id}`"))?;
    for (twin, entries, want) in [("bad", bad, false), ("good", good, true)] {
        let (dir, files) = fixture(&format!("{id}-{twin}"), entries)?;
        let (passed, output) = run(command(&check, &dir, &files))?;
        assert_eq!(passed, want, "`{id}` on its {twin} fixture:\n{output}");
        if !want {
            explains(&check, &output);
        }
        std::fs::remove_dir_all(dir)?;
    }
    Ok(())
}

/// A failure prints the check's message (catalog V1) and does not blame a
/// missing tool (root V1).
fn explains(check: &pklith::catalog::Check, output: &str) {
    let (id, message) = (&check.id, format!("{}: {}", check.id, check.msg));
    assert!(
        output.contains(&message),
        "`{id}` failed without its message:\n{output}"
    );
    assert!(
        !output.contains("MISSING TOOL"),
        "`{id}` blamed a tool:\n{output}"
    );
}

macro_rules! proven {
    ($($(#[$attr:meta])* $name:ident => $id:literal: $bad:expr, $good:expr;)*) => {
        /// Every id with a fixture below.
        pub const PROVEN: &[&str] = &[$($id),*];
        $(
            #[test]
            $(#[$attr])*
            fn $name() -> super::Result {
                super::proves($id, &$bad, &$good)
            }
        )*
    };
}

#[path = "catalog/fixtures.rs"]
mod fixtures;

#[path = "catalog/bats.rs"]
mod bats;

#[path = "catalog/universal.rs"]
mod universal;

/// V30: no built-in check ships without a fixture, and no fixture outlives
/// its check.
#[test]
fn every_builtin_check_has_a_fixture() -> Result {
    let mut ids: Vec<String> = pklith::catalog::builtin()?
        .into_iter()
        .map(|c| c.id)
        .collect();
    let mut proven: Vec<&str> = fixtures::PROVEN.to_vec();
    ids.sort();
    proven.sort_unstable();
    // A check may have several fixtures (rustfmt and clippy also prove a
    // workspace member); every check needs at least one.
    proven.dedup();
    assert_eq!(ids, proven);
    Ok(())
}
