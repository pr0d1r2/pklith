//! Lay missing checks into a repository, one commit per check
//! (`src/lay/SPEC.md`). This part plans; nothing here writes.

use crate::catalog::Check;
use crate::registry::Registry;

mod apply;
pub use apply::{Context, Failure, lay};

/// The checks to lay, in the order they are laid (lay §C): what the
/// registry uses, minus what hk.pkl already runs, minus what no file claims
/// (root V15). Universal `*` checks first, then category, then catalog
/// order, the order src/gen emits; gate checks last even when universal.
#[must_use]
pub fn plan<'a>(
    registry: &Registry,
    catalog: &'a [Check],
    present: &[String],
    claimed: &[String],
) -> Vec<&'a Check> {
    let universal: Vec<&String> = registry
        .types
        .iter()
        .filter(|t| t.key == "*")
        .flat_map(|t| &t.checks)
        .collect();
    let missing = |c: &&Check| !present.contains(&c.id) && claimed.contains(&c.id);
    let mut plan: Vec<&Check> = crate::r#gen::used(registry, catalog)
        .into_iter()
        .filter(missing)
        .collect();
    let gate = |c: &Check| c.category == crate::catalog::Category::Gate;
    plan.sort_by_key(|c| (gate(c), !universal.contains(&&c.id)));
    plan
}

/// The commit subject for a laid check: `ci(<id>): add the <id> check`,
/// within the 72 characters the commit-msg rule allows.
#[must_use]
pub fn subject(check: &Check) -> String {
    let id = &check.id;
    let full = format!("ci({id}): add the {id} check");
    if full.chars().count() <= 72 {
        full
    } else {
        format!("ci({id}): add this check")
    }
}

/// The planted-violation proof a laid commit cites (root V32): a built-in
/// row run as shipped has a fixture in pklith's own tests (root V30); a
/// local row, or an override that changes what runs, has none yet.
#[must_use]
pub fn proof(check: &Check) -> &'static str {
    let runs_as_shipped = |b: &Check| {
        (&b.id, &b.nix, &b.globs, &b.check, &b.fix, &b.env)
            == (
                &check.id,
                &check.nix,
                &check.globs,
                &check.check,
                &check.fix,
                &check.env,
            )
    };
    if crate::catalog::builtin().is_ok_and(|b| b.iter().any(runs_as_shipped)) {
        return "Planted-violation proof: pklith's tests/catalog.rs fails this built-in\ncheck on a planted violation and passes it on a clean twin (pklith V30).";
    }
    "Planted-violation proof: none yet; this row is not a built-in check as\nshipped, so no catalog fixture covers it (pklith V30)."
}

#[cfg(test)]
mod tests {
    use super::{plan, proof, subject};
    use crate::catalog::Check;
    use crate::registry::{Error, parse};

    const REGISTRY: &str = "format 1\n## checks\nid|category|nix|glob|check|fix|env|msg\nlint|lint|-|*|l|-|-|m\nfmt|format|-|*|f|-|-|m\nws|hygiene|-|*|w|-|-|m\nsecret|secret|-|*|s|-|-|m\nruff|lint|-|*|r|-|-|m\nwhole|gate|-|*|g|-|-|m\n## types\ntype|checks|min|exempt\nrs|lint,fmt|-|-\npy|ruff|-|-\n*|whole,secret,ws|-|-\n";

    fn ids(checks: &[&Check]) -> Vec<String> {
        checks.iter().map(|c| c.id.clone()).collect()
    }

    fn laid(present: &[&str], claimed: &[&str]) -> Result<Vec<String>, Error> {
        let registry = parse(REGISTRY)?;
        let catalog = crate::catalog::parse(&registry.checks)?;
        let own = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        Ok(ids(&plan(
            &registry,
            &catalog,
            &own(present),
            &own(claimed),
        )))
    }

    const ALL: [&str; 5] = ["lint", "fmt", "ws", "secret", "whole"];

    /// Universal checks first (hygiene before secret), then category order,
    /// and gate checks last even when universal: they judge the rest;
    /// what hk.pkl runs is not laid again (V5), nor what no file claims
    /// (root V15: ruff, with no `.py` file).
    #[test]
    fn the_plan_is_what_is_missing_in_lay_order() -> Result<(), Error> {
        assert_eq!(laid(&[], &ALL)?, ["ws", "secret", "fmt", "lint", "whole"]);
        assert_eq!(laid(&["ws", "fmt"], &ALL)?, ["secret", "lint", "whole"]);
        assert!(laid(&ALL, &ALL)?.is_empty());
        Ok(())
    }

    /// The subject names the check and stays within 72 characters.
    #[test]
    fn the_subject_fits_the_commit_rule() -> Result<(), Error> {
        let registry = parse(REGISTRY)?;
        let mut check = crate::catalog::parse(&registry.checks)?.remove(0);
        assert_eq!(subject(&check), "ci(lint): add the lint check");
        check.id = "a-check-id-long-enough-to-break".into();
        assert_eq!(
            subject(&check),
            "ci(a-check-id-long-enough-to-break): add this check"
        );
        Ok(())
    }

    /// V32: a built-in row as shipped cites its fixture; the same id with a
    /// changed command, or a local id, admits it has none.
    #[test]
    fn the_proof_cites_a_fixture_only_for_a_shipped_row() -> Result<(), Error> {
        let mut shipped = crate::catalog::builtin()?;
        shipped.retain(|c| c.id == "shellcheck");
        assert_eq!(shipped.len(), 1);
        for mut shipped in shipped {
            shipped.msg = "a local message changes nothing that runs".into();
            assert!(proof(&shipped).contains("tests/catalog.rs"));
            shipped.check.push_str(" --shell=bash");
            assert!(proof(&shipped).contains("none yet"));
        }
        let registry = parse(REGISTRY)?;
        let local = crate::catalog::parse(&registry.checks)?.remove(0);
        assert!(proof(&local).contains("none yet"));
        Ok(())
    }
}
