//! Lay missing checks into a repository, one commit per check
//! (`src/lay/SPEC.md`). This part plans; nothing here writes.

use crate::catalog::Check;
use crate::registry::Registry;

/// The checks to lay, in the order they are laid (lay §C): what the
/// registry uses, minus what hk.pkl already runs, minus what no file claims
/// (root V15). Universal `*` checks first, then category, then catalog
/// order, the order src/gen emits.
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
    plan.sort_by_key(|c| !universal.contains(&&c.id));
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

#[cfg(test)]
mod tests {
    use super::{plan, subject};
    use crate::catalog::Check;
    use crate::registry::{Error, parse};

    const REGISTRY: &str = "format 1\n## checks\nid|category|nix|glob|check|fix|env|msg\nlint|lint|-|*|l|-|-|m\nfmt|format|-|*|f|-|-|m\nws|hygiene|-|*|w|-|-|m\nsecret|secret|-|*|s|-|-|m\nruff|lint|-|*|r|-|-|m\n## types\ntype|checks|min|exempt\nrs|lint,fmt|-|-\npy|ruff|-|-\n*|secret,ws|-|-\n";

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

    const ALL: [&str; 4] = ["lint", "fmt", "ws", "secret"];

    /// Universal checks first (hygiene before secret), then category order;
    /// what hk.pkl runs is not laid again (V5), nor what no file claims
    /// (root V15: ruff, with no `.py` file).
    #[test]
    fn the_plan_is_what_is_missing_in_lay_order() -> Result<(), Error> {
        assert_eq!(laid(&[], &ALL)?, ["ws", "secret", "fmt", "lint"]);
        assert_eq!(laid(&["ws", "fmt"], &ALL)?, ["secret", "lint"]);
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
}
