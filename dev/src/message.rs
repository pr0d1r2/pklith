//! The commit-message rules (root §C, T4, T68), as pure functions over the
//! message text and what git says about the commit.

/// git's scissors line: `git commit -v` appends the diff below it, and
/// cleanup removes everything from here on.
const SCISSORS: &str = "# ------------------------ >8 ------------------------";

const TYPES: [&str; 9] = [
    "feat", "fix", "docs", "test", "chore", "refactor", "perf", "build", "ci",
];

/// The lines git keeps: up to the scissors, `#` comment lines dropped.
#[must_use]
pub fn kept(text: &str) -> Vec<&str> {
    text.lines()
        .take_while(|l| *l != SCISSORS)
        .filter(|l| !l.starts_with('#'))
        .collect()
}

/// The first line of the message git keeps.
#[must_use]
pub fn subject(text: &str) -> &str {
    kept(text).first().copied().unwrap_or_default()
}

/// Subject `type(scope): claim`, at most 72 characters, and a body that
/// says why. Merge, revert, fixup and squash subjects are git's and pass.
///
/// # Errors
///
/// The rule broken, with the fix.
pub fn check(text: &str) -> Result<(), String> {
    let lines = kept(text);
    let subject = lines.first().copied().unwrap_or_default();
    let git = ["Merge ", "Revert ", "fixup! ", "squash! "];
    if git.iter().any(|p| subject.starts_with(p)) {
        return Ok(());
    }
    headline(subject)?;
    let mut body = lines
        .iter()
        .skip(1)
        .filter(|l| !l.starts_with("Co-Authored-By:"));
    if body.all(|l| l.trim().is_empty()) {
        return Err("the body is empty. Say WHY: the diff already shows what changed.".to_owned());
    }
    Ok(())
}

/// The subject's style and length.
fn headline(subject: &str) -> Result<(), String> {
    if !styled(subject) {
        let types = TYPES.join(" ");
        return Err(format!(
            "subject must be 'type(scope): claim', type one of {types}. Got: {subject}"
        ));
    }
    let n = subject.chars().count();
    if n > 72 {
        return Err(format!(
            "subject is {n} characters, the limit is 72. Say less, or move detail to the body."
        ));
    }
    Ok(())
}

/// `type(scope)!: claim`: a known type, an optional lower-case scope, an
/// optional `!`, then `: ` and a claim that does not start with a space.
fn styled(subject: &str) -> bool {
    let Some(rest) = TYPES.iter().find_map(|t| subject.strip_prefix(t)) else {
        return false;
    };
    let rest = match rest.strip_prefix('(') {
        Some(scoped) => match scoped.split_once(')') {
            Some((scope, after)) if !scope.is_empty() && scope.chars().all(scope_char) => after,
            _ => return false,
        },
        None => rest,
    };
    let rest = rest.strip_prefix('!').unwrap_or(rest);
    rest.strip_prefix(": ")
        .is_some_and(|claim| !claim.is_empty() && !claim.starts_with(' '))
}

fn scope_char(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '/' | '.' | '_' | '-')
}

#[cfg(test)]
mod tests;
