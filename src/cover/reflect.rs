//! Cover V9: a fragment the files switch on that `.pklith` does not
//! reflect, because none of its checks is claimed for any type.

use super::Gap;
use crate::catalog::Fragment;
use crate::registry::Registry;

/// A fragment `.pklith` does not reflect, and the rows that would.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unreflected {
    /// Fragment id.
    pub fragment: String,
    /// Rows to add, as `pklith seed --init` would write them.
    pub rows: Vec<String>,
}

/// The `active` fragments `.pklith` does not reflect: one of their
/// triggers matches a file that is a gap, and no type row claims any of
/// their checks, `*` included. A fragment with one check claimed, or whose
/// files all have a type (even an exempt one), was decided on.
#[must_use]
pub fn unreflected<'a>(
    active: &[&'a Fragment],
    registry: &Registry,
    gaps: &[Gap],
) -> Vec<&'a Fragment> {
    let claimed = |id: &String| registry.types.iter().any(|t| t.checks.contains(id));
    let gap = |f: &Fragment| {
        gaps.iter()
            .flat_map(|g| &g.files)
            .any(|p| f.triggers.matches(p))
    };
    active
        .iter()
        .filter(|f| gap(f) && !f.checks.iter().any(claimed))
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::unreflected;
    use crate::registry::{Error, parse};

    /// The unreflected fragments for a gemspec and a first `.rb`, under a
    /// registry of `*|typos` plus `rows`.
    fn unreflected_with(rows: &str) -> Result<Vec<String>, Error> {
        let fragments = crate::catalog::builtin_fragments()?;
        let files = vec!["x.gemspec".to_owned(), "lib/a.rb".to_owned()];
        let active = crate::detect::active(&files, &fragments);
        let text = format!("format 1\n## types\ntype|checks|min|exempt\n*|typos|-|-\n{rows}");
        let registry = parse(&text)?;
        let gaps = crate::cover::judge(&files, &registry).gaps;
        let found = unreflected(&active, &registry, &gaps);
        Ok(found.iter().map(|f| f.id.clone()).collect())
    }

    /// T5: with no type row for either file, rubocop is unreflected;
    /// claiming one of its checks, or giving the files a type at all,
    /// reflects it.
    #[test]
    fn a_gap_its_fragment_would_cover_is_unreflected() -> Result<(), Error> {
        assert_eq!(unreflected_with("")?, ["rubocop"]);
        assert!(unreflected_with("rb|rubocop|-|-\n")?.is_empty());
        assert!(unreflected_with("rb|-|-|not linted\ngemspec|-|-|packaging\n")?.is_empty());
        Ok(())
    }
}
