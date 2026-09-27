//! Staged mode (cover V8, root V26): judge what a commit changes, not the
//! whole tree, so the pre-commit hot path costs about what was touched.

use super::{Coverage, Gap, Stale, classes, gap, resolve};
use crate::registry::{Registry, TypeRow};
use std::collections::HashSet;

/// Judge `changed` paths (added, modified or deleted) against `registry`,
/// with `files` the tracked tree after the change: a changed file with no
/// type is a gap, and a row a changed or deleted path belonged to is stale
/// once no file resolves to it. Rows nothing touched are not judged; the
/// full check at push does that.
#[must_use]
pub fn judge_staged(files: &[String], changed: &[String], registry: &Registry) -> Coverage {
    let declared: Vec<&TypeRow> = registry.types.iter().filter(|t| t.key != "*").collect();
    let classes = classes(&declared);
    let row = |path: &str| resolve(path, &declared, &classes);
    let (touched, gaps) = sort_out(files, changed, &row);
    let alive = |i: usize| files.iter().any(|f| row(f) == Some(i));
    let stale = dead(&touched, &declared, &alive);
    Coverage {
        gaps,
        stale,
        ..Coverage::default()
    }
}

/// The touched rows no file resolves to any more.
fn dead(touched: &[usize], declared: &[&TypeRow], alive: &dyn Fn(usize) -> bool) -> Vec<Stale> {
    let gone = touched
        .iter()
        .filter(|i| !alive(**i))
        .filter_map(|i| declared.get(*i));
    gone.map(|t| Stale {
        key: t.key.clone(),
        line: t.line,
    })
    .collect()
}

/// The rows changed paths belong to, in registry order, and the gaps among
/// changed files still in the tree (a deleted file has no type to lack).
fn sort_out(
    files: &[String],
    changed: &[String],
    row: &dyn Fn(&str) -> Option<usize>,
) -> (Vec<usize>, Vec<Gap>) {
    let present: HashSet<&str> = files.iter().map(String::as_str).collect();
    let (mut touched, mut gaps) = (Vec::new(), Vec::new());
    for path in changed {
        match row(path) {
            Some(i) => touched.push(i),
            None if present.contains(path.as_str()) => gap(&mut gaps, path),
            None => {}
        }
    }
    touched.sort_unstable();
    touched.dedup();
    (touched, gaps)
}

#[cfg(test)]
mod tests {
    use super::judge_staged;
    use crate::registry::{Error, parse};

    const REGISTRY: &str =
        "format 1\n## types\ntype|checks|min|exempt\nsh|shellcheck|-|-\nrs|clippy|-|-\n";

    fn own(names: &[&str]) -> Vec<String> {
        names.iter().map(|n| (*n).to_owned()).collect()
    }

    /// T4: deleting the last `.sh` flags its row stale; deleting one of two
    /// `.rs` files does not.
    #[test]
    fn deleting_the_last_file_of_a_type_is_stale() -> Result<(), Error> {
        let registry = parse(REGISTRY)?;
        let after = own(&["a.rs"]);
        let coverage = judge_staged(&after, &own(&["run.sh", "b.rs"]), &registry);
        let stale: Vec<&str> = coverage.stale.iter().map(|s| s.key.as_str()).collect();
        assert_eq!((stale, coverage.gaps.len()), (vec!["sh"], 0));
        Ok(())
    }

    /// V8: a changed file with no type is a gap; one untouched elsewhere
    /// in the tree is left for the full check, and a deleted file with no
    /// type lacks nothing.
    #[test]
    fn only_changed_files_are_gaps() -> Result<(), Error> {
        let registry = parse(REGISTRY)?;
        let tree = own(&["a.rs", "old.py", "new.go"]);
        let changed = own(&["new.go", "gone.txt"]);
        let coverage = judge_staged(&tree, &changed, &registry);
        let gaps: Vec<&str> = coverage.gaps.iter().map(|g| g.key.as_str()).collect();
        assert_eq!((gaps, coverage.stale.len()), (vec!["go"], 0));
        Ok(())
    }
}
