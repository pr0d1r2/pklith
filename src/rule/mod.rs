//! Companion rules (`src/rule/SPEC.md`): for every file a selector picks,
//! something must exist, be mentioned, change with it, or change before
//! it.

use crate::registry::{Error, Row};
use crate::scan::Globs;
use std::collections::HashSet;

mod order;
pub mod template;

pub use order::order;

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
    /// with a diff (V3). A `tdd-order` row is this kind, judged commit by
    /// commit over a history instead ([`order()`], V6).
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
    /// `tdd-order`: judged by [`order()`] over a history, never by
    /// [`evaluate`].
    ordered: bool,
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
    /// Staged mode (cover V8): only changed files still tracked are
    /// sources, not the whole tree.
    pub only_changed: bool,
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
    let [id, name, select, target, except] =
        <[String; 5]>::try_from(row.cells.clone()).unwrap_or_default();
    let typed = typed(&name, &target, &select, &except);
    let (kind, target, select, except) = typed.map_err(|what| Error {
        line: row.line,
        message: format!("rule `{id}`: {what}"),
    })?;
    Ok(Rule {
        ordered: name == "tdd-order",
        id,
        kind,
        select,
        except,
        target,
    })
}

/// A row's kind, target and globs, typed.
fn typed(
    kind: &str,
    target: &str,
    select: &str,
    except: &str,
) -> Result<(Kind, Template, Globs, Globs), String> {
    let (kind, target) = kind_of(kind, target)?;
    Ok((kind, target, globs(select)?, globs(except)?))
}

fn kind_of(kind: &str, target: &str) -> Result<(Kind, Template), String> {
    let kind = match kind {
        "exists" => Kind::Exists,
        "orphan" => Kind::Orphan,
        "changed" | "tdd-order" => Kind::Changed,
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

/// The irregular plurals of a `## plural` table, which the registry keeps
/// at its header's two cells.
#[must_use]
pub fn plurals(rows: &[Row]) -> Vec<(String, String)> {
    let pair = |row: &Row| <[String; 2]>::try_from(row.cells.clone()).unwrap_or_default();
    rows.iter()
        .map(pair)
        .map(|[one, many]| (one, many))
        .collect()
}

impl Rule {
    /// The companion an `exists` rule wants for `source`, when the rule
    /// selects it; `None` for other kinds and unselected files. What
    /// src/map follows from a changed file to its spec.
    #[must_use]
    pub fn companion(&self, source: &str, plurals: &Plurals) -> Option<String> {
        let exists = matches!(self.kind, Kind::Exists);
        (exists && self.selects(source)).then(|| self.target.render(source, plurals))
    }

    /// Whether the rule picks `path`: selected and not excepted.
    fn selects(&self, path: &str) -> bool {
        self.select.matches(path) && !self.except.matches(path)
    }
}

/// Every failure of `rules` over `input`, rule by rule, sources in the
/// order given. `changed` rules run only when there is a diff (V3);
/// `tdd-order` rules need a history and run in [`order()`] (V6).
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
        _ if rule.ordered => return Vec::new(),
        (Kind::Changed, None) => return Vec::new(),
        (Kind::Changed, Some(diff)) => diff,
        (_, Some(diff)) if input.only_changed => diff,
        _ => input.files,
    };
    let live = |f: &&String| matches!(rule.kind, Kind::Changed) || tracked.contains(f.as_str());
    let picked = |f: &&String| rule.selects(f);
    let selected = pool.iter().filter(live).filter(picked);
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
