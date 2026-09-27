//! Fragments (catalog §C): a group of checks and the files that switch it
//! on. src/detect reads the triggers; the checks and seed files follow.

use super::{Check, error, list};
use crate::registry::{Error, Row};
use crate::scan::Globs;

/// The trigger that makes a fragment active in every repository.
pub const ALWAYS: &str = "always";

/// One `## fragments` row.
#[derive(Debug, Clone)]
pub struct Fragment {
    /// Line it was defined on, for errors.
    pub line: usize,
    /// Fragment name.
    pub id: String,
    /// Whether it is on in every repository ([`ALWAYS`]).
    pub always: bool,
    /// Globs over tracked paths; any match switches the fragment on.
    pub triggers: Globs,
    /// Catalog ids, in lay order within the fragment.
    pub checks: Vec<String>,
    /// Seed file ids (src/seed).
    pub seed: Vec<String>,
}

/// Typed fragments from `## fragments` rows.
///
/// # Errors
///
/// An [`Error`] naming the line: no triggers, a trigger glob that does not
/// compile, no checks (root V18: an active fragment must select steps), or
/// an id defined twice.
pub fn parse(rows: &[Row]) -> Result<Vec<Fragment>, Error> {
    let fragments = rows.iter().map(fragment).collect::<Result<Vec<_>, _>>()?;
    unique(&fragments)?;
    Ok(fragments)
}

fn fragment(row: &Row) -> Result<Fragment, Error> {
    let [id, triggers, checks, seed] =
        <[String; 4]>::try_from(row.cells.clone()).unwrap_or_default();
    let why = |what: &str| error(row.line, format!("fragment `{id}` {what}"));
    let (always, triggers) = switches(&triggers).map_err(|what| why(&what))?;
    let checks = selected(&checks).map_err(|what| why(&what))?;
    let (line, seed) = (row.line, list(&seed));
    Ok(Fragment {
        line,
        id,
        always,
        triggers,
        checks,
        seed,
    })
}

/// The checks a fragment selects; root V18: an active fragment must
/// select some.
fn selected(cell: &str) -> Result<Vec<String>, String> {
    let checks = list(cell);
    if checks.is_empty() {
        return Err("selects no checks (root V18)".into());
    }
    Ok(checks)
}

/// Whether a triggers cell says `always`, and its globs; why it is wrong
/// otherwise.
fn switches(cell: &str) -> Result<(bool, Globs), String> {
    let mut globs = list(cell);
    let always = globs.iter().any(|t| t == ALWAYS);
    globs.retain(|t| t != ALWAYS);
    if !always && globs.is_empty() {
        return Err("has no triggers; list globs, or `always`".into());
    }
    let triggers = Globs::new(&globs).map_err(|e| format!("has a bad trigger: {e}"))?;
    Ok((always, triggers))
}

fn unique(fragments: &[Fragment]) -> Result<(), Error> {
    for (i, f) in fragments.iter().enumerate() {
        if let Some(first) = fragments.iter().take(i).find(|g| g.id == f.id) {
            let message = format!(
                "fragment `{}` is already defined on line {}",
                f.id, first.line
            );
            return Err(error(f.line, message));
        }
    }
    Ok(())
}

/// The built-in fragments with local ones laid over them: a local fragment
/// replaces the built-in one with its id where it stands, so detect order
/// holds; a new id goes at the end.
#[must_use]
pub fn merge(builtin: Vec<Fragment>, mut local: Vec<Fragment>) -> Vec<Fragment> {
    let mut merged: Vec<Fragment> = builtin
        .into_iter()
        .map(|b| match local.iter().position(|l| l.id == b.id) {
            Some(i) => local.remove(i),
            None => b,
        })
        .collect();
    merged.extend(local);
    merged
}

/// The fragments a repository works with: the built-in ones with its
/// `## fragments` rows laid over them, each naming a check `catalog` holds
/// (root V17).
///
/// # Errors
///
/// An [`Error`] naming the line of a malformed local fragment, or of one
/// naming an unknown check.
pub fn resolved(local: &[Row], catalog: &[Check]) -> Result<Vec<Fragment>, Error> {
    let fragments = merge(crate::catalog::builtin_fragments()?, parse(local)?);
    known(&fragments, catalog)?;
    Ok(fragments)
}

/// Root V17: every check a fragment names resolves to a catalog row.
///
/// # Errors
///
/// An [`Error`] at the first fragment naming an unknown check.
pub fn known(fragments: &[Fragment], catalog: &[Check]) -> Result<(), Error> {
    for f in fragments {
        if let Some(id) = f
            .checks
            .iter()
            .find(|id| catalog.iter().all(|c| &c.id != *id))
        {
            return Err(error(
                f.line,
                format!("fragment `{}` names unknown check `{id}`", f.id),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{known, parse};
    use crate::registry::Error;

    const HEADER: &str = "format 1\n## fragments\nfragment|triggers|checks|seed\n";

    fn err(rows: &str) -> Option<String> {
        crate::registry::parse(&format!("{HEADER}{rows}"))
            .and_then(|r| parse(&r.fragments))
            .err()
            .map(|e| e.to_string())
    }

    const BAD: [(&str, &str); 4] = [
        (
            "x|-|a|-",
            ".pklith:4: fragment `x` has no triggers; list globs, or `always`",
        ),
        (
            "x|always|-|-",
            ".pklith:4: fragment `x` selects no checks (root V18)",
        ),
        (
            "x|src/[bad|a|-",
            ".pklith:4: fragment `x` has a bad trigger: ",
        ),
        (
            "x|always|a|-\nx|*.sh|b|-",
            ".pklith:5: fragment `x` is already defined on line 4",
        ),
    ];

    /// A fragment switches on by globs or always, selects checks, and has
    /// one definition.
    #[test]
    fn a_malformed_fragment_is_refused_by_line() {
        for (rows, want) in BAD {
            let got = err(rows).unwrap_or_default();
            assert!(got.starts_with(want), "{rows}: {got}");
        }
    }

    /// Root V17: a fragment naming a check the catalog lacks is refused.
    #[test]
    fn a_fragment_naming_an_unknown_check_is_refused() -> Result<(), Error> {
        let registry = crate::registry::parse(&format!("{HEADER}sh|always|shellcheck, nope|-\n"))?;
        let fragments = parse(&registry.fragments)?;
        let catalog = crate::catalog::builtin()?;
        let got = known(&fragments, &catalog).err().map(|e| e.to_string());
        assert_eq!(
            got.as_deref(),
            Some(".pklith:4: fragment `sh` names unknown check `nope`")
        );
        Ok(())
    }

    /// Root V16, V17: the built-in fragments parse, name only built-in
    /// checks, and between them reach every built-in check, so detect can
    /// select any of it.
    #[test]
    fn the_builtin_fragments_reach_every_builtin_check() -> Result<(), Error> {
        let (checks, fragments) = (
            crate::catalog::builtin()?,
            crate::catalog::builtin_fragments()?,
        );
        known(&fragments, &checks)?;
        let reached =
            |c: &crate::catalog::Check| fragments.iter().any(|f| f.checks.contains(&c.id));
        let orphans: Vec<&str> = checks
            .iter()
            .filter(|c| !reached(c))
            .map(|c| c.id.as_str())
            .collect();
        assert!(orphans.is_empty(), "in no fragment: {orphans:?}");
        Ok(())
    }
    /// A local fragment replaces the built-in one with its id where it
    /// stands; a new id goes last; its checks must exist (root V17).
    #[test]
    fn local_fragments_override_in_place_and_extend() -> Result<(), Error> {
        let rows = "shell|**/*.sh|shellcheck|-\nlocal|*.x|typos|-\n";
        let registry = crate::registry::parse(&format!("{HEADER}{rows}"))?;
        let catalog = crate::catalog::builtin()?;
        let merged = super::resolved(&registry.fragments, &catalog)?;
        let ids: Vec<&str> = merged.iter().map(|f| f.id.as_str()).collect();
        let shell = merged
            .iter()
            .find(|f| f.id == "shell")
            .map(|f| f.checks.clone());
        assert_eq!(ids.get(3..5), Some(&["shell", "rubocop"][..]));
        assert_eq!(ids.last(), Some(&"local"));
        assert_eq!(shell, Some(vec!["shellcheck".to_owned()]));
        Ok(())
    }
}
