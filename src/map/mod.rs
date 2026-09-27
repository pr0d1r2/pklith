//! Guard mode (`src/map/SPEC.md`): from changed paths to the specs that
//! cover them, so a hook can run only those.

use crate::rule::{Rule, template::Plurals};
use std::collections::BTreeSet;

/// Specs to run for a change, and changed files that should have one but
/// do not.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Mapped {
    /// Spec paths, sorted and deduped (V3).
    pub specs: Vec<String>,
    /// `(file, rule, missing spec)` for a changed file a rule selects whose
    /// spec is not tracked (V1).
    pub unmapped: Vec<(String, String, String)>,
}

/// Map `changed` over `rules`: a changed file an `exists` rule selects
/// maps to its companion; a changed file that is some tracked file's
/// companion maps to itself; anything else is ignored (V2). Deleted paths
/// map to nothing.
#[must_use]
pub fn map(changed: &[String], files: &[String], rules: &[Rule], plurals: &Plurals) -> Mapped {
    Mapper::new(files, rules, plurals).run(changed)
}

/// What mapping one changed path needs to know about the tree.
struct Mapper<'a> {
    tracked: BTreeSet<&'a str>,
    companions: BTreeSet<String>,
    rules: &'a [Rule],
    plurals: &'a Plurals,
}

impl<'a> Mapper<'a> {
    fn new(files: &'a [String], rules: &'a [Rule], plurals: &'a Plurals) -> Self {
        let tracked = files.iter().map(String::as_str).collect();
        let companions = companions(files, rules, plurals);
        Self {
            tracked,
            companions,
            rules,
            plurals,
        }
    }

    fn run(&self, changed: &[String]) -> Mapped {
        let (mut specs, mut unmapped) = (BTreeSet::new(), Vec::new());
        for path in changed.iter().filter(|p| self.tracked.contains(p.as_str())) {
            self.follow(path, &mut specs, &mut unmapped);
        }
        Mapped {
            specs: specs.into_iter().collect(),
            unmapped,
        }
    }

    /// Add `path`'s specs, or record the ones missing.
    fn follow(
        &self,
        path: &String,
        specs: &mut BTreeSet<String>,
        unmapped: &mut Vec<(String, String, String)>,
    ) {
        for rule in self.rules {
            match rule.companion(path, self.plurals) {
                Some(spec) if self.tracked.contains(spec.as_str()) => drop(specs.insert(spec)),
                Some(spec) => unmapped.push((path.clone(), rule.id.clone(), spec)),
                None => {}
            }
        }
        if self.companions.contains(path) {
            specs.insert(path.clone());
        }
    }
}

/// Every companion some tracked file's `exists` rules name.
fn companions(files: &[String], rules: &[Rule], plurals: &Plurals) -> BTreeSet<String> {
    let each = |f: &String| {
        rules
            .iter()
            .filter_map(|r| r.companion(f, plurals))
            .collect::<Vec<_>>()
    };
    files.iter().flat_map(each).collect()
}

#[cfg(test)]
mod tests {
    use super::map;
    use crate::registry::Error;

    const RULES: &str = "format 1\n## rules\nid|kind|select|target|except\nspec|exists|app/**/*.rb|spec/{dir\\|strip:app/}/{stem}_spec.rb|-\n";

    fn own(paths: &[&str]) -> Vec<String> {
        paths.iter().map(|p| (*p).to_owned()).collect()
    }

    const TREE: [&str; 5] = [
        "app/models/user.rb",
        "app/models/post.rb",
        "spec/models/user_spec.rb",
        "spec/models/post_spec.rb",
        "README.md",
    ];

    /// V1-V3: a changed impl maps to its spec, a changed spec to itself, a
    /// file no rule selects is ignored; the output is sorted and deduped.
    #[test]
    fn changed_files_map_to_their_specs() -> Result<(), Error> {
        let rules = crate::rule::parse(&crate::registry::parse(RULES)?.rules)?;
        let changed = own(&[
            "README.md",
            "spec/models/post_spec.rb",
            "app/models/user.rb",
            "app/models/post.rb",
        ]);
        let mapped = map(&changed, &own(&TREE), &rules, &[]);
        assert_eq!(
            mapped.specs,
            ["spec/models/post_spec.rb", "spec/models/user_spec.rb"]
        );
        assert!(mapped.unmapped.is_empty());
        Ok(())
    }

    /// V1: a changed impl whose spec is missing is named, with the rule and
    /// the spec it wants; a deleted path maps to nothing.
    #[test]
    fn a_changed_impl_without_its_spec_is_unmapped() -> Result<(), Error> {
        let rules = crate::rule::parse(&crate::registry::parse(RULES)?.rules)?;
        let tree = own(&["app/models/user.rb", "app/models/new.rb"]);
        let changed = own(&["app/models/new.rb", "app/models/gone.rb"]);
        let mapped = map(&changed, &tree, &rules, &[]);
        let (file, rule, spec) = mapped.unmapped.first().cloned().unwrap_or_default();
        assert_eq!(
            (file.as_str(), rule.as_str()),
            ("app/models/new.rb", "spec")
        );
        assert_eq!(
            (spec.as_str(), mapped.unmapped.len()),
            ("spec/models/new_spec.rb", 1)
        );
        assert!(mapped.specs.is_empty());
        Ok(())
    }
}
