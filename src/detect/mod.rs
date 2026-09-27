//! The one fragment detector (`src/detect/SPEC.md`, root V16): which
//! fragments a repository's files switch on, in catalog order.

use crate::catalog::Fragment;

/// The fragments `files` switch on: those marked `always`, and those with a
/// trigger matching some path. Catalog order, never discovery order (V2).
#[must_use]
pub fn active<'a>(files: &[String], fragments: &'a [Fragment]) -> Vec<&'a Fragment> {
    let on = |f: &&Fragment| f.always || files.iter().any(|p| f.triggers.matches(p));
    fragments.iter().filter(on).collect()
}

#[cfg(test)]
mod tests {
    use super::active;
    use crate::registry::Error;

    /// Detect the built-in fragments for `files`, as ids.
    fn detected(files: &[&str]) -> Result<Vec<String>, Error> {
        let fragments = crate::catalog::builtin_fragments()?;
        let files: Vec<String> = files.iter().map(|f| (*f).to_owned()).collect();
        Ok(active(&files, &fragments)
            .into_iter()
            .map(|f| f.id.clone())
            .collect())
    }

    /// Cases ported from set-and-setting's `tests/detect-fragments.bats`,
    /// for the fragments pklith ships: its expectations, less the dropped
    /// fragments (ruby, rspec, ascii, …).
    const CASES: [(&str, &[&str], &[&str]); 15] = [
        ("awk detects awk", &["x.awk"], &["base", "awk"]),
        ("xml detects xml", &["a/b.xml"], &["base", "xml"]),
        (
            "markdown-only repo detects markdown",
            &["README.md"],
            &["base", "markdown"],
        ),
        ("base is always on", &[], &["base"]),
        (
            "nix-only repo detects nix",
            &["flake.nix"],
            &["base", "nix"],
        ),
        (
            "shell-only repo detects shell",
            &["a.sh"],
            &["base", "shell"],
        ),
        (".bash is shell", &["a.bash"], &["base", "shell"]),
        (
            ".rubocop.yml detects rubocop",
            &[".rubocop.yml"],
            &["base", "rubocop", "yaml"],
        ),
        (
            "gemspec detects rubocop",
            &["x.gemspec"],
            &["base", "rubocop"],
        ),
        ("ruby source alone does not", &["lib/a.rb"], &["base"]),
        (".toml detects toml", &["a.toml"], &["base", "toml"]),
        (
            "workflows detect actions",
            &[".github/workflows/ci.yml"],
            &["base", "actions", "yaml"],
        ),
        (
            "a yml elsewhere is yaml, not a workflow",
            &["x/ci.yml"],
            &["base", "yaml"],
        ),
        (
            "nested files are detected",
            &["a/b/c.sh", "d/e.nix"],
            &["base", "nix", "shell"],
        ),
        (
            "spec and rust",
            &["SPEC.md", "src/a.rs"],
            &["base", "markdown", "rust", "spec"],
        ),
    ];

    /// V2: each case yields its fragments, in catalog order.
    #[test]
    fn files_switch_on_their_fragments() -> Result<(), Error> {
        for (name, files, want) in CASES {
            assert_eq!(detected(files)?, want, "{name}");
        }
        Ok(())
    }

    /// V2: the order is the catalog's, whatever order the files come in.
    #[test]
    fn the_order_does_not_follow_the_files() -> Result<(), Error> {
        let forward = detected(&["a.toml", "b.sh", "c.nix", ".github/workflows/x.yaml"])?;
        let backward = detected(&[".github/workflows/x.yaml", "c.nix", "b.sh", "a.toml"])?;
        assert_eq!(forward, ["base", "actions", "nix", "shell", "yaml", "toml"]);
        assert_eq!(forward, backward);
        Ok(())
    }
}
