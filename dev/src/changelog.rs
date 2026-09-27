//! The changelog rule (root V33, T68): a `feat`, `fix` or `perf` commit
//! that changes product code under `src/` adds an entry under
//! `## [Unreleased]` in CHANGELOG.md. Pure functions over the message and
//! what git says about the commit.

/// Whether a subject names a change consumers see: `feat`, `fix`, `perf`.
#[must_use]
pub fn behaviour(subject: &str) -> bool {
    ["feat", "fix", "perf"].iter().any(|t| {
        subject
            .strip_prefix(t)
            .is_some_and(|rest| rest.starts_with(['(', '!', ':']))
    })
}

/// The staged paths under `src/` that are product code: specs and test
/// files are not.
#[must_use]
pub fn product(staged: &str) -> Vec<&str> {
    let test =
        |p: &str| p.ends_with("/SPEC.md") || p.ends_with("/tests.rs") || p.contains("/tests/");
    staged
        .lines()
        .filter(|p| p.starts_with("src/") && !test(p))
        .collect()
}

/// The `## [Unreleased]` section of a changelog, heading excluded.
#[must_use]
pub fn unreleased(changelog: &str) -> String {
    let body = changelog
        .lines()
        .skip_while(|l| *l != "## [Unreleased]")
        .skip(1)
        .take_while(|l| !l.starts_with("## "));
    body.flat_map(|l| [l, "\n"]).collect()
}

/// The `[Unreleased]` sections the rule compares, and HEAD's subject.
pub struct Sections {
    /// As staged.
    pub staged: String,
    /// At HEAD.
    pub head: String,
    /// At HEAD's parent.
    pub parent: String,
    /// HEAD's subject.
    pub head_subject: String,
}

/// Pass when the change is not a behaviour change, touches no product
/// code, adds an entry, or amends a commit that brought its own entry
/// under the same subject.
///
/// # Errors
///
/// The rule broken, listing the product paths and the fix.
pub fn check(subject: &str, product: &[&str], s: &Sections) -> Result<(), String> {
    let amend = s.head_subject == subject && s.head != s.parent;
    if !behaviour(subject) || product.is_empty() || s.staged != s.head || amend {
        return Ok(());
    }
    let kind = subject.split(['(', '!', ':']).next().unwrap_or_default();
    let paths: String = product.iter().flat_map(|p| ["  ", p, "\n"]).collect();
    Err(format!(
        "a {kind} commit changed product code with no new entry under ## [Unreleased] in CHANGELOG.md (V33):\n{paths}  fix: add a line saying what a consumer will see change, and stage CHANGELOG.md."
    ))
}

#[cfg(test)]
mod tests;
