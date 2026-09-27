//! Companion rules (`src/rule/SPEC.md`): for every file a selector picks,
//! something must exist, be mentioned, or change with it.

use crate::registry::{Error, Row};
use crate::scan::Globs;
use std::collections::HashSet;

pub mod template;

use template::{Plurals, Template};

/// What a rule demands of each selected file.
#[derive(Debug, Clone)]
pub enum Kind {
    /// The rendered path is tracked.
    Exists,
    /// As `exists`, read from the target's side: a target whose source is
    /// gone is an orphan.
    Orphan,
    /// Some tracked file matching `within` contains the rendered text,
    /// literally (V2).
    Mentions { within: Globs, glob: String },
    /// The rendered path changes whenever the source does; judged only
    /// with a diff (V3).
    Changed,
}

/// One `## rules` row.
#[derive(Debug, Clone)]
pub struct Rule {
    /// Rule id.
    pub id: String,
    kind: Kind,
    select: Globs,
    except: Globs,
    target: Template,
}

/// A selected file whose companion is missing (cover V7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    /// Rule id.
    pub rule: String,
    /// The file the rule selected.
    pub source: String,
    /// What should have been there.
    pub missing: String,
}

/// What rules are judged against.
pub struct Input<'a> {
    /// Tracked files.
    pub files: &'a [String],
    /// Changed paths, when a diff was given (`--staged` or a range).
    pub diff: Option<&'a [String]>,
    /// A tracked file's text; only `mentions` reads it (V5).
    pub read: &'a dyn Fn(&str) -> Option<String>,
    /// Irregular plurals from `## plural`.
    pub plurals: &'a Plurals,
}

/// Typed rules from `## rules` rows.
///
/// # Errors
///
/// An [`Error`] naming the line: unknown kind, a selector or `except` glob
/// that does not compile, a bad template (V1), or a `mentions` target
/// without its `glob :: pattern` shape.
pub fn parse(rows: &[Row]) -> Result<Vec<Rule>, Error> {
    rows.iter().map(rule).collect()
}

fn rule(row: &Row) -> Result<Rule, Error> {
    let [id, kind, select, target, except] =
        <[String; 5]>::try_from(row.cells.clone()).unwrap_or_default();
    let typed =
        kind_of(&kind, &target).and_then(|(k, t)| Ok((k, t, globs(&select)?, globs(&except)?)));
    let (kind, target, select, except) = typed.map_err(|what| Error {
        line: row.line,
        message: format!("rule `{id}`: {what}"),
    })?;
    Ok(Rule {
        id,
        kind,
        select,
        except,
        target,
    })
}

fn kind_of(kind: &str, target: &str) -> Result<(Kind, Template), String> {
    let kind = match kind {
        "exists" => Kind::Exists,
        "orphan" => Kind::Orphan,
        "changed" => Kind::Changed,
        "mentions" => return mentions(target),
        other => return Err(format!("unknown kind `{other}`")),
    };
    Ok((kind, Template::parse(target)?))
}

fn mentions(target: &str) -> Result<(Kind, Template), String> {
    let (glob, pattern) = target
        .split_once(" :: ")
        .ok_or("a mentions target is `glob :: pattern`")?;
    let within = globs(glob)?;
    Ok((
        Kind::Mentions {
            within,
            glob: glob.to_owned(),
        },
        Template::parse(pattern)?,
    ))
}

fn globs(cell: &str) -> Result<Globs, String> {
    let list: Vec<String> = cell
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    Globs::new(&list).map_err(|e| e.to_string())
}

/// Every failure of `rules` over `input`, rule by rule, sources in the
/// order given. `changed` rules run only when there is a diff (V3).
#[must_use]
pub fn evaluate(rules: &[Rule], input: &Input<'_>) -> Vec<Failure> {
    let tracked: HashSet<&str> = input.files.iter().map(String::as_str).collect();
    rules
        .iter()
        .flat_map(|r| failures(r, input, &tracked))
        .collect()
}

fn failures(rule: &Rule, input: &Input<'_>, tracked: &HashSet<&str>) -> Vec<Failure> {
    let pool = match (&rule.kind, input.diff) {
        (Kind::Changed, None) => return Vec::new(),
        (Kind::Changed, Some(diff)) => diff,
        _ => input.files,
    };
    let selected = pool
        .iter()
        .filter(|f| rule.select.matches(f) && !rule.except.matches(f));
    selected
        .filter_map(|source| missing(rule, source, input, tracked))
        .collect()
}

/// What `source` lacks under `rule`, if anything.
fn missing(
    rule: &Rule,
    source: &str,
    input: &Input<'_>,
    tracked: &HashSet<&str>,
) -> Option<Failure> {
    let rendered = rule.target.render(source, input.plurals);
    let found = match &rule.kind {
        Kind::Exists | Kind::Orphan => tracked.contains(rendered.as_str()),
        Kind::Changed => input.diff.is_some_and(|d| d.contains(&rendered)),
        Kind::Mentions { within, .. } => mentioned(within, &rendered, input),
    };
    let missing = match &rule.kind {
        Kind::Mentions { glob, .. } => format!("`{rendered}` in {glob}"),
        _ => rendered,
    };
    (!found).then(|| Failure {
        rule: rule.id.clone(),
        source: source.to_owned(),
        missing,
    })
}

/// Whether some tracked file `within` holds `text`, literally (V2).
fn mentioned(within: &Globs, text: &str, input: &Input<'_>) -> bool {
    input
        .files
        .iter()
        .filter(|f| within.matches(f))
        .any(|f| (input.read)(f).is_some_and(|body| body.contains(text)))
}

#[cfg(test)]
mod tests;
