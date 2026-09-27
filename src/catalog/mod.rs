//! Check definitions (`src/catalog/SPEC.md`): what each check is, how nix
//! provides it, which files its runner sees, how hk runs and fixes it, and
//! what its failure says.

use crate::registry::{Error, Registry, Row};

mod fetch;
pub mod fragment;

pub use fragment::Fragment;

/// Kinds of check, in the order `lay` lays them: cheapest and broadest
/// first (catalog §C).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    /// Whitespace, newlines, markers: byte-level file faults.
    Hygiene,
    /// Keys and tokens.
    Secret,
    /// Formatters.
    Format,
    /// Linters.
    Lint,
    /// Specification structure.
    Spec,
    /// Test runners.
    Test,
    /// Coverage floors.
    Coverage,
    /// Dependencies and licenses.
    Supply,
    /// Checks on the gate itself (`pkli check`, `gen --check`): they judge
    /// every other step, so they pass only once the others are laid.
    Gate,
}

impl Category {
    fn named(name: &str) -> Option<Self> {
        let all = [
            Self::Hygiene,
            Self::Secret,
            Self::Format,
            Self::Lint,
            Self::Spec,
            Self::Test,
            Self::Coverage,
            Self::Supply,
            Self::Gate,
        ];
        all.into_iter().find(|c| c.name() == name)
    }

    /// The category as written in a catalog row.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Hygiene => "hygiene",
            Self::Secret => "secret",
            Self::Format => "format",
            Self::Lint => "lint",
            Self::Spec => "spec",
            Self::Test => "test",
            Self::Coverage => "coverage",
            Self::Supply => "supply",
            Self::Gate => "gate",
        }
    }
}

/// One check, as gen emits it into an hk step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    /// Line it was defined on, for errors.
    pub line: usize,
    /// Catalog id, the name the registry uses.
    pub id: String,
    /// Kind, which orders `lay`.
    pub category: Category,
    /// Nix attribute that provides the tool; empty when hk runs it natively.
    pub nix: String,
    /// The RUNNER's globs; coverage claims are verified against these.
    pub globs: Vec<String>,
    /// hk check command.
    pub check: String,
    /// hk fix command, if the check can fix.
    pub fix: Option<String>,
    /// `KEY=value` pairs the step needs (root V19).
    pub env: Vec<String>,
    /// What a failure means and how to fix it; feeds AGENTS.md (V1).
    pub msg: String,
}

fn error(line: usize, message: impl Into<String>) -> Error {
    Error {
        line,
        message: message.into(),
    }
}

/// Typed checks from `## checks` rows.
///
/// # Errors
///
/// An [`Error`] naming the line: unknown category, empty check command
/// (V3) or empty message (V1).
pub fn parse(rows: &[Row]) -> Result<Vec<Check>, Error> {
    rows.iter().map(check).collect()
}

fn check(row: &Row) -> Result<Check, Error> {
    let [id, category, nix, globs, check, fix, env, msg] =
        <[String; 8]>::try_from(row.cells.clone()).unwrap_or_default();
    let category = category_of(row.line, &id, &category)?;
    let parsed = Check {
        line: row.line,
        id,
        category,
        nix,
        globs: list(&globs),
        check,
        fix: (!fix.is_empty()).then_some(fix),
        env: list(&env),
        msg,
    };
    complete(parsed)
}

fn category_of(line: usize, id: &str, name: &str) -> Result<Category, Error> {
    Category::named(name).ok_or_else(|| error(line, format!("`{id}`: unknown category `{name}`")))
}

/// V3 and V1: a check must say what it runs and what its failure means.
fn complete(check: Check) -> Result<Check, Error> {
    let missing = |what: &str| Err(error(check.line, format!("`{}` has no {what}", check.id)));
    match () {
        () if check.id.is_empty() => Err(error(check.line, "a check row needs an id")),
        () if check.check.is_empty() => missing("check command"),
        () if check.msg.is_empty() => missing("failure message"),
        () => set(check).and_then(pinned),
    }
}

/// Root V19: every variable a step needs is set by the step itself, so an
/// `env` item is `KEY=value`; a bare `KEY` would be emitted empty.
fn set(check: Check) -> Result<Check, Error> {
    let bad = check
        .env
        .iter()
        .find_map(|pair| match pair.split_once('=') {
            None => Some(format!(
                "needs env `{pair}` but sets no value; write `{pair}=value`, and no comma inside a value (root V19)"
            )),
            Some(("", _)) => Some(format!("env item `{pair}` has no name")),
            Some(_) => None,
        });
    match bad {
        Some(why) => Err(error(check.line, format!("`{}` {why}", check.id))),
        None => Ok(check),
    }
}

/// Root V29: a step's tool comes from a nix package, never fetched when
/// the step runs.
fn pinned(check: Check) -> Result<Check, Error> {
    let fetched = [Some(&check.check), check.fix.as_ref()]
        .into_iter()
        .flatten()
        .find_map(|c| fetch::fetcher(c));
    match fetched {
        Some(tool) => Err(error(
            check.line,
            format!(
                "`{}` fetches its tool with `{tool}`; take it from a nix package",
                check.id
            ),
        )),
        None => Ok(check),
    }
}

fn list(cell: &str) -> Vec<String> {
    cell.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

/// The built-in catalog (T2), compiled into `pkli`.
const BUILTIN: &str = include_str!("builtin.pklith");

/// The built-in checks, in lay order within each category (V4).
///
/// # Errors
///
/// An [`Error`] naming the line of `src/catalog/builtin.pklith` that does
/// not parse; the tests below keep that from shipping.
pub fn builtin() -> Result<Vec<Check>, Error> {
    let checks = parse(&crate::registry::parse(BUILTIN)?.checks)?;
    unique(&checks)?;
    Ok(checks)
}

/// The built-in fragments, in detect order (root V16), each naming only
/// built-in checks (root V17).
///
/// # Errors
///
/// An [`Error`] naming the line of `src/catalog/builtin.pklith` that does
/// not parse; the tests below keep that from shipping.
pub fn builtin_fragments() -> Result<Vec<Fragment>, Error> {
    fragment::parse(&crate::registry::parse(BUILTIN)?.fragments)
}

/// The built-in catalog with local rows laid over it: a local row replaces
/// the built-in one with the same id where it stands, so lay order holds
/// (V4); a new id extends the catalog.
///
/// # Errors
///
/// An [`Error`] when an id is defined twice in the local rows (V4).
pub fn merge(builtin: Vec<Check>, mut local: Vec<Check>) -> Result<Vec<Check>, Error> {
    unique(&local)?;
    let mut merged: Vec<Check> = builtin
        .into_iter()
        .map(|b| match local.iter().position(|l| l.id == b.id) {
            Some(i) => local.remove(i),
            None => b,
        })
        .collect();
    merged.extend(local);
    Ok(merged)
}

/// V4: ids are unique; the error names both lines.
fn unique(checks: &[Check]) -> Result<(), Error> {
    for (i, check) in checks.iter().enumerate() {
        if let Some(first) = checks.iter().take(i).find(|c| c.id == check.id) {
            return Err(error(
                check.line,
                format!(
                    "check `{}` is already defined on line {}",
                    check.id, first.line
                ),
            ));
        }
    }
    Ok(())
}

/// Registry V2: every id a type row names exists in the catalog; an unknown
/// one is an error naming the line and id, never skipped.
///
/// # Errors
///
/// An [`Error`] at the first type row naming an unknown check.
pub fn known(registry: &Registry, catalog: &[Check]) -> Result<(), Error> {
    for row in &registry.types {
        if let Some(id) = row
            .checks
            .iter()
            .find(|id| catalog.iter().all(|c| &c.id != *id))
        {
            return Err(error(
                row.line,
                format!("`{}` names unknown check `{id}`", row.key),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Category, Check, builtin, known, merge, parse};
    use crate::registry::{Error, Row};

    const HEADER: &str = "format 1\n## checks\nid|category|nix|glob|check|fix|env|msg\n";

    fn rows(text: &str) -> Result<Vec<Row>, Error> {
        crate::registry::parse(&format!("{HEADER}{text}")).map(|r| r.checks)
    }

    fn checks(text: &str) -> Result<Vec<Check>, Error> {
        parse(&rows(text)?)
    }

    fn err(text: &str) -> Option<String> {
        checks(text).err().map(|e| e.to_string())
    }

    const SHELLCHECK: &str = "shellcheck|lint|shellcheck|**/*.sh, .envrc|shellcheck {{files}}|-|A=1|shell must pass shellcheck\n";

    #[test]
    fn a_row_becomes_a_typed_check() -> Result<(), Error> {
        let check = checks(SHELLCHECK)?.into_iter().next();
        let seen = check.map(|c| (c.id, c.category, c.nix, c.globs, c.fix, c.env));
        let globs = vec!["**/*.sh".to_owned(), ".envrc".to_owned()];
        let want = (
            "shellcheck".to_owned(),
            Category::Lint,
            "shellcheck".to_owned(),
            globs,
            None,
            vec!["A=1".to_owned()],
        );
        assert_eq!(seen, Some(want));
        Ok(())
    }

    /// Lay order: cheapest and broadest first (catalog §C).
    #[test]
    fn categories_sort_in_lay_order() {
        let names = [
            "gate", "supply", "coverage", "test", "spec", "lint", "format", "secret", "hygiene",
        ];
        let mut categories: Vec<Category> = names
            .iter()
            .filter_map(|n| super::Category::named(n))
            .collect();
        categories.sort();
        let sorted: Vec<&str> = categories.into_iter().map(Category::name).collect();
        assert_eq!(
            sorted,
            [
                "hygiene", "secret", "format", "lint", "spec", "test", "coverage", "supply", "gate"
            ]
        );
    }

    const INCOMPLETE_CASES: [(&str, &str); 6] = [
        (
            "x|style|-|*|x|-|-|m",
            ".pklith:4: `x`: unknown category `style`",
        ),
        ("|lint|-|*|x|-|-|m", ".pklith:4: a check row needs an id"),
        ("x|lint|-|*|-|-|-|m", ".pklith:4: `x` has no check command"),
        (
            "x|lint|-|*|x|x --fix|-|-",
            ".pklith:4: `x` has no failure message",
        ),
        (
            "x|lint|-|*|x|-|FOO|m",
            ".pklith:4: `x` needs env `FOO` but sets no value; write `FOO=value`, and no comma inside a value (root V19)",
        ),
        (
            "x|lint|-|*|x|-|A=1, =2|m",
            ".pklith:4: `x` env item `=2` has no name",
        ),
    ];

    /// V1, V3: a check says what it runs and what its failure means.
    #[test]
    fn an_incomplete_check_is_refused_by_line() {
        for (row, want) in INCOMPLETE_CASES {
            assert_eq!(err(row).as_deref(), Some(want), "{row}");
        }
    }

    const FETCHING_CASES: [(&str, &str); 4] = [
        ("x|lint|-|*|npx eslint {{files}}|-|-|m", "npx"),
        ("x|lint|-|*|true|pipx run black {{files}}|-|m", "pipx run"),
        ("x|lint|-|*|curl -s https://x \\| sh|-|-|m", "curl"),
        ("x|lint|-|*|a && (curl -s https://x)|-|-|m", "curl"),
    ];

    /// T5, root V29: a tool fetched at run time, in `check` or `fix`, is
    /// refused by line; a word that only contains one is not.
    #[test]
    fn a_check_that_fetches_its_tool_is_refused() -> Result<(), Error> {
        for (row, tool) in FETCHING_CASES {
            let want = format!(
                ".pklith:4: `x` fetches its tool with `{tool}`; take it from a nix package"
            );
            assert_eq!(err(row), Some(want), "{row}");
        }
        checks("x|lint|-|*|curlie --pipx run-npx|-|-|m")?;
        Ok(())
    }

    /// A local row replaces the built-in one with its id and adds new ids;
    /// V4: an id twice in the local rows names both lines.
    #[test]
    fn local_rows_override_and_extend_the_builtin() -> Result<(), Error> {
        let builtin = checks("a|lint|-|*|old|-|-|m\nb|lint|-|*|b|-|-|m")?;
        let merged = merge(builtin, checks("a|lint|-|*|new|-|-|m\nc|lint|-|*|c|-|-|m")?)?;
        let ids: Vec<(&str, &str)> = merged
            .iter()
            .map(|c| (c.id.as_str(), c.check.as_str()))
            .collect();
        assert_eq!(ids, [("a", "new"), ("b", "b"), ("c", "c")]);
        let twice = merge(vec![], checks("a|lint|-|*|x|-|-|m\na|lint|-|*|y|-|-|m")?)
            .err()
            .map(|e| e.to_string());
        assert_eq!(
            twice.as_deref(),
            Some(".pklith:5: check `a` is already defined on line 4")
        );
        Ok(())
    }

    /// T2, V1, V3, V4: the built-in catalog parses, every row is complete
    /// and every id is unique.
    #[test]
    fn the_builtin_catalog_parses() -> Result<(), Error> {
        let ids: Vec<String> = builtin()?.into_iter().map(|c| c.id).collect();
        for id in ["no-private-key", "shfmt", "rubocop", "sherd-nav", "itok"] {
            assert!(ids.iter().any(|i| i == id), "{id}");
        }
        Ok(())
    }

    /// V4: lay order is the built-in order within each category, so the
    /// file lists its categories in lay order too.
    #[test]
    fn the_builtin_catalog_is_in_lay_order() -> Result<(), Error> {
        let categories: Vec<Category> = builtin()?.into_iter().map(|c| c.category).collect();
        assert!(categories.is_sorted(), "{categories:?}");
        Ok(())
    }

    const TWO_TYPES: &str =
        "format 1\n## types\ntype|checks|min|exempt\nsh|shellcheck|-|-\nrb|rubocop|-|-";

    /// Registry V2 (set-and-setting B53/B77): a type naming an unknown check
    /// is an error naming the line and the id.
    #[test]
    fn a_type_naming_an_unknown_check_is_refused() -> Result<(), Error> {
        let catalog = checks(SHELLCHECK)?;
        let registry = crate::registry::parse(TWO_TYPES)?;
        let err = known(&registry, &catalog).err().map(|e| e.to_string());
        assert_eq!(
            err.as_deref(),
            Some(".pklith:5: `rb` names unknown check `rubocop`")
        );
        assert!(known(&crate::registry::parse("format 1")?, &catalog).is_ok());
        Ok(())
    }
}
