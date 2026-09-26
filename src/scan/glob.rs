//! The one glob engine (`src/scan/SPEC.md`, root R19).

use super::Error;

/// Globs matched exactly as hk matches a step's globs (root R19): globset
/// with empty alternates allowed and `*` crossing `/`, so `*.rs` matches
/// `src/a.rs`. The one engine catalog, rule, hook and cover share (`src`
/// §C), so a claim and the runner that backs it see the same files.
#[derive(Debug, Clone)]
pub struct Globs(globset::GlobSet);

impl Globs {
    /// Compile `patterns`.
    ///
    /// # Errors
    ///
    /// [`Error::Glob`] naming the first pattern that does not compile.
    pub fn new(patterns: &[String]) -> Result<Self, Error> {
        let mut set = globset::GlobSetBuilder::new();
        for pattern in patterns {
            let glob = globset::GlobBuilder::new(pattern)
                .empty_alternates(true)
                .build();
            set.add(glob.map_err(Error::from)?);
        }
        set.build().map(Self).map_err(Error::from)
    }

    /// Whether any pattern matches `path`.
    #[must_use]
    pub fn matches(&self, path: &str) -> bool {
        self.0.is_match(path)
    }
}

impl From<globset::Error> for Error {
    fn from(e: globset::Error) -> Self {
        Self::Glob(e.to_string())
    }
}

#[cfg(test)]
mod glob_tests {
    use super::{Error, Globs};

    fn globs(patterns: &[&str]) -> Result<Globs, Error> {
        Globs::new(&patterns.iter().map(|p| (*p).to_owned()).collect::<Vec<_>>())
    }

    /// hk's semantics (R19): `*` crosses `/`, so `*.rs` matches in
    /// subdirectories; `**/` matches at the root too.
    #[test]
    fn star_crosses_directories_as_in_hk() -> Result<(), Error> {
        let rs = globs(&["*.rs"])?;
        assert!(rs.matches("a.rs") && rs.matches("src/scan/mod.rs"));
        assert!(!rs.matches("a.rsx"));
        assert!(globs(&["**/*.md"])?.matches("README.md"));
        Ok(())
    }

    /// Several patterns, alternates, and an empty alternate as hk allows.
    #[test]
    fn alternates_and_sets_match_like_hk() -> Result<(), Error> {
        let ci = globs(&[".github/workflows/*.{yml,yaml}", "flake.lock"])?;
        assert!(ci.matches(".github/workflows/ci.yaml") && ci.matches("flake.lock"));
        assert!(!ci.matches("docs/ci.yml"));
        assert!(globs(&["x{,.bak}"])?.matches("x"));
        assert!(!globs(&[])?.matches("anything"));
        Ok(())
    }

    #[test]
    fn a_broken_pattern_is_an_error() {
        let err = globs(&["ok", "src/[unclosed"]).err().map(|e| e.to_string());
        assert!(
            err.is_some_and(|e| e.contains("src/[unclosed")),
            "the error names the pattern"
        );
    }
}
