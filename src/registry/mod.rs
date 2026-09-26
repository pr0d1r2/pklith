//! Parse and validate `.pklith` (`src/registry/SPEC.md`): file types to
//! check sets, local catalog rows, companion rules, plural table.
//!
//! ```text
//! format 1
//!
//! ## types
//! type|checks|min|exempt
//! *|typos,trailing-whitespace|-|-
//! rs|rustfmt,clippy|-|-
//! png|-|-|binary asset
//! ```

use std::fmt;

/// The newest `.pklith` format this build reads (root V31).
pub const FORMAT: u32 = 1;

/// A section of `.pklith`, each a pipe table with a fixed header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Types,
    Checks,
    Rules,
    Plural,
}

impl Section {
    fn named(name: &str) -> Option<Self> {
        match name {
            "types" => Some(Self::Types),
            "checks" => Some(Self::Checks),
            "rules" => Some(Self::Rules),
            "plural" => Some(Self::Plural),
            _ => None,
        }
    }

    fn header(self) -> &'static str {
        match self {
            Self::Types => "type|checks|min|exempt",
            Self::Checks => "id|category|nix|glob|check|fix|env|msg",
            Self::Rules => "id|kind|select|target|except",
            Self::Plural => "singular|plural",
        }
    }
}

/// One `## types` row: a file type and what it requires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeRow {
    /// Line in `.pklith`, for errors.
    pub line: usize,
    /// Type key as scan derives it, `*` for the universal row, or `path:<glob>`.
    pub key: String,
    /// Catalog ids; many per type.
    pub checks: Vec<String>,
    /// How many checks, beyond the universal ones, the type needs.
    pub min: usize,
    /// Why the type needs no checks, if it does not.
    pub exempt: Option<String>,
}

/// A row whose cells another node interprets (`## checks`, `## rules`,
/// `## plural`), with its line for errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Line in `.pklith`.
    pub line: usize,
    /// Trimmed, unescaped cells; `-` becomes empty.
    pub cells: Vec<String>,
}

/// A parsed `.pklith`. Absent sections are empty (V6).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Registry {
    /// `## types`.
    pub types: Vec<TypeRow>,
    /// `## checks`: local catalog rows.
    pub checks: Vec<Row>,
    /// `## rules`: companion rules.
    pub rules: Vec<Row>,
    /// `## plural`: irregular plurals.
    pub plural: Vec<Row>,
}

/// What is wrong with `.pklith`, and where.
#[derive(Debug, PartialEq, Eq)]
pub struct Error {
    /// Line in `.pklith`; 0 when the file is empty.
    pub line: usize,
    /// What is wrong.
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".pklith:{}: {}", self.line, self.message)
    }
}

impl std::error::Error for Error {}

fn error(line: usize, message: impl Into<String>) -> Error {
    Error {
        line,
        message: message.into(),
    }
}

/// Parse `.pklith` text.
///
/// # Errors
///
/// An [`Error`] naming the line: a missing or newer format line (V31), an
/// unknown section, a wrong header or cell count, a malformed type row
/// (V1), or a duplicate type or rule id (V3).
pub fn parse(text: &str) -> Result<Registry, Error> {
    let mut lines = text
        .lines()
        .enumerate()
        .map(|(i, l)| (i + 1, l.trim()))
        .filter(|(_, l)| !l.is_empty() && !is_comment(l));
    version(lines.next())?;
    let mut parser = Parser::default();
    for (line, content) in lines {
        parser.line(line, content)?;
    }
    no_duplicates(&parser.registry)?;
    Ok(parser.registry)
}

/// `#` starts a comment, except `## ` which starts a section.
fn is_comment(line: &str) -> bool {
    line.starts_with('#') && !line.starts_with("## ")
}

fn version(first: Option<(usize, &str)>) -> Result<(), Error> {
    let (line, content) =
        first.ok_or_else(|| error(0, "empty; the first line must be `format 1`"))?;
    match content
        .strip_prefix("format ")
        .and_then(|n| n.parse::<u32>().ok())
    {
        None => Err(error(
            line,
            format!("expected `format {FORMAT}`, got `{content}`"),
        )),
        Some(n) if n > FORMAT => Err(error(line, newer(n))),
        Some(_) => Ok(()),
    }
}

fn newer(n: u32) -> String {
    format!("format {n} is newer than this pkli reads ({FORMAT}); upgrade pkli")
}

#[derive(Default)]
struct Parser {
    registry: Registry,
    section: Option<(Section, bool)>,
}

impl Parser {
    fn line(&mut self, line: usize, content: &str) -> Result<(), Error> {
        if let Some(name) = content.strip_prefix("## ") {
            let section = Section::named(name)
                .ok_or_else(|| error(line, format!("unknown section `## {name}`")))?;
            self.section = Some((section, false));
            return Ok(());
        }
        match self.section {
            None => Err(error(line, "row outside any `## section`")),
            Some((section, false)) => self.header(line, section, content),
            Some((section, true)) => self.row(line, section, content),
        }
    }

    fn header(&mut self, line: usize, section: Section, content: &str) -> Result<(), Error> {
        if content != section.header() {
            return Err(error(
                line,
                format!("expected header `{}`", section.header()),
            ));
        }
        self.section = Some((section, true));
        Ok(())
    }

    fn row(&mut self, line: usize, section: Section, content: &str) -> Result<(), Error> {
        let cells = cells(content);
        let want = section.header().split('|').count();
        if cells.len() != want {
            return Err(error(
                line,
                format!("{} cells, the header has {want}", cells.len()),
            ));
        }
        self.push(section, Row { line, cells })
    }

    fn push(&mut self, section: Section, row: Row) -> Result<(), Error> {
        match section {
            Section::Types => self.registry.types.push(type_row(row)?),
            Section::Checks => self.registry.checks.push(row),
            Section::Rules => self.registry.rules.push(row),
            Section::Plural => self.registry.plural.push(row),
        }
        Ok(())
    }
}

/// Split a row on unescaped `|`, unescape `\|`, trim, and read `-` as empty.
fn cells(content: &str) -> Vec<String> {
    content
        .replace("\\|", "\u{0}")
        .split('|')
        .map(|c| cell(&c.replace('\u{0}', "|")))
        .collect()
}

fn cell(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed == "-" {
        String::new()
    } else {
        trimmed.to_owned()
    }
}

/// V1: a type row needs at least `min` checks or an exemption reason, never
/// both. The universal row `*` is never exempt.
fn type_row(row: Row) -> Result<TypeRow, Error> {
    let [key, checks, min, exempt] = <[String; 4]>::try_from(row.cells).unwrap_or_default();
    let parsed = TypeRow {
        line: row.line,
        key,
        checks: ids(&checks),
        min: minimum(row.line, &min)?,
        exempt: (!exempt.is_empty()).then_some(exempt),
    };
    requirement(&parsed)?;
    path_class(&parsed)?;
    Ok(parsed)
}

/// A comma list of catalog ids.
fn ids(list: &str) -> Vec<String> {
    list.split(',')
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .map(str::to_owned)
        .collect()
}

/// `min` is 1 when the cell is `-`.
fn minimum(line: usize, cell: &str) -> Result<usize, Error> {
    if cell.is_empty() {
        return Ok(1);
    }
    cell.parse()
        .map_err(|_| error(line, format!("min `{cell}` is not a number")))
}

/// A `path:<glob>` type must compile, as hk would compile it (root R19).
fn path_class(row: &TypeRow) -> Result<(), Error> {
    let Some(glob) = row.key.strip_prefix("path:") else {
        return Ok(());
    };
    let compiled = crate::scan::Globs::new(&[glob.to_owned()]);
    compiled
        .map(|_| ())
        .map_err(|e| error(row.line, format!("`{}`: {e}", row.key)))
}

fn requirement(row: &TypeRow) -> Result<(), Error> {
    let fail = |m: String| Err(error(row.line, m));
    match (&row.exempt, row.checks.len()) {
        _ if row.key.is_empty() => fail("a type row needs a type".into()),
        (Some(_), _) if row.key == "*" => fail("the universal row `*` cannot be exempt".into()),
        (Some(_), n) if n > 0 => fail(format!(
            "`{}` has checks and an exemption; pick one",
            row.key
        )),
        (None, n) if n < row.min && row.key != "*" => fail(format!(
            "`{}` has {n} checks, needs {}, or an exemption reason",
            row.key, row.min
        )),
        _ => Ok(()),
    }
}

/// V3: a type or rule id appears once; the error names both lines.
fn no_duplicates(registry: &Registry) -> Result<(), Error> {
    let types = registry.types.iter().map(|t| (t.key.as_str(), t.line));
    let rules = registry
        .rules
        .iter()
        .map(|r| (r.cells.first().map_or("", String::as_str), r.line));
    first_duplicate(types)
        .or_else(|| first_duplicate(rules))
        .map_or(Ok(()), Err)
}

fn first_duplicate<'a>(items: impl Iterator<Item = (&'a str, usize)>) -> Option<Error> {
    let mut seen: Vec<(&str, usize)> = Vec::new();
    for (key, line) in items {
        if let Some((_, first)) = seen.iter().find(|(k, _)| *k == key) {
            return Some(error(
                line,
                format!("`{key}` is already declared on line {first}"),
            ));
        }
        seen.push((key, line));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{Registry, parse};

    fn err(text: &str) -> Option<String> {
        parse(text).err().map(|e| e.to_string())
    }

    fn assert_errors(cases: &[(&str, &str)]) {
        for (rows, want) in cases {
            let text = if rows.starts_with("format") {
                (*rows).to_owned()
            } else {
                format!("{TYPES}{rows}")
            };
            assert_eq!(err(&text).as_deref(), Some(*want), "{rows}");
        }
    }

    const TYPES: &str = "format 1\n## types\ntype|checks|min|exempt\n";

    const FULL: &str = "# comment\n\nformat 1\n\n## types\ntype|checks|min|exempt\n*|typos|-|-\nrs|rustfmt, clippy|2|-\npng|-|-|binary asset\n\n## checks\nid|category|nix|glob|check|fix|env|msg\nx|lint|x|**/*.x|x a \\| b|-|-|m\n## rules\nid|kind|select|target|except\nspec|exists|app/**/*.rb|spec/{stem}_spec.rb|-\n## plural\nsingular|plural\nperson|people\n";

    /// One line per type row: `line key [checks] min exempt`.
    fn types(registry: &Registry) -> Vec<String> {
        let show = |t: &super::TypeRow| {
            format!(
                "{} {} {:?} {} {:?}",
                t.line, t.key, t.checks, t.min, t.exempt
            )
        };
        registry.types.iter().map(show).collect()
    }

    /// One entry per row: `line:cell|cell|...`.
    fn cells(rows: &[super::Row]) -> Vec<String> {
        rows.iter()
            .map(|r| format!("{}:{}", r.line, r.cells.join("|")))
            .collect()
    }

    /// Comments and blank lines skipped; `-` read as empty (min 1 by
    /// default). T5: a reason alone exempts a type.
    #[test]
    fn type_rows_parse_with_their_lines() -> Result<(), super::Error> {
        let want = [
            r#"7 * ["typos"] 1 None"#,
            r#"8 rs ["rustfmt", "clippy"] 2 None"#,
            r#"9 png [] 1 Some("binary asset")"#,
        ];
        assert_eq!(types(&parse(FULL)?), want);
        Ok(())
    }

    /// The other sections keep their cells for their own nodes; `\|` stays
    /// a literal pipe inside a cell.
    #[test]
    fn other_sections_keep_their_cells() -> Result<(), super::Error> {
        let registry = parse(FULL)?;
        assert_eq!(cells(&registry.checks), ["13:x|lint|x|**/*.x|x a | b|||m"]);
        assert_eq!(
            cells(&registry.rules),
            ["16:spec|exists|app/**/*.rb|spec/{stem}_spec.rb|"]
        );
        assert_eq!(cells(&registry.plural), ["19:person|people"]);
        Ok(())
    }

    /// V6: absent sections are empty.
    #[test]
    fn only_the_format_line_is_an_empty_registry() -> Result<(), super::Error> {
        assert_eq!(parse("format 1\n")?, Registry::default());
        Ok(())
    }

    /// V31: the format line comes first, and a newer one is refused by name.
    const FORMAT_CASES: [(&str, &str); 2] = [
        (
            "format 2",
            ".pklith:1: format 2 is newer than this pkli reads (1); upgrade pkli",
        ),
        (
            "format 1x",
            ".pklith:1: expected `format 1`, got `format 1x`",
        ),
    ];

    #[test]
    fn the_format_line_is_required_and_versioned() {
        assert_eq!(
            err("").as_deref(),
            Some(".pklith:0: empty; the first line must be `format 1`")
        );
        assert_errors(&FORMAT_CASES);
    }

    const STRUCTURE_CASES: [(&str, &str); 5] = [
        ("format 1\n## nope", ".pklith:2: unknown section `## nope`"),
        ("format 1\na|b", ".pklith:2: row outside any `## section`"),
        (
            "format 1\n## plural\na|b",
            ".pklith:3: expected header `singular|plural`",
        ),
        ("rs|x|-", ".pklith:4: 3 cells, the header has 4"),
        ("rs|x|two|-", ".pklith:4: min `two` is not a number"),
    ];

    #[test]
    fn structure_errors_name_their_line() {
        assert_errors(&STRUCTURE_CASES);
    }

    /// V1: checks or an exemption, never both, never neither. V5 (set-and-
    /// setting B55): universal checks never count toward a type's minimum.
    const REQUIREMENT_CASES: [(&str, &str); 5] = [
        ("|x|-|-", ".pklith:4: a type row needs a type"),
        (
            "rs|x|-|why",
            ".pklith:4: `rs` has checks and an exemption; pick one",
        ),
        (
            "rs|x|2|-",
            ".pklith:4: `rs` has 1 checks, needs 2, or an exemption reason",
        ),
        (
            "*|-|-|why",
            ".pklith:4: the universal row `*` cannot be exempt",
        ),
        (
            "*|typos|-|-\nrs|-|-|-",
            ".pklith:5: `rs` has 0 checks, needs 1, or an exemption reason",
        ),
    ];

    #[test]
    fn a_type_needs_checks_or_a_reason_never_both() {
        assert_errors(&REQUIREMENT_CASES);
    }

    /// V3: duplicates name both lines, for types and for rule ids.
    const DUPLICATE_CASES: [(&str, &str); 2] = [
        (
            "rs|x|-|-\nrs|y|-|-",
            ".pklith:5: `rs` is already declared on line 4",
        ),
        (
            "format 1\n## rules\nid|kind|select|target|except\nr|exists|a|b|-\nr|exists|c|d|-",
            ".pklith:5: `r` is already declared on line 4",
        ),
    ];

    #[test]
    fn duplicates_name_both_lines() {
        assert_errors(&DUPLICATE_CASES);
    }
}

#[cfg(test)]
mod path_tests {
    /// A `path:` class compiles as hk would compile it, or names its line.
    #[test]
    fn a_path_class_glob_must_compile() {
        let err = super::parse("format 1\n## types\ntype|checks|min|exempt\npath:src/[bad|-|-|x\n")
            .err()
            .map(|e| e.to_string());
        assert!(err.is_some_and(|e| e.starts_with(".pklith:4: `path:src/[bad`: ")));
        assert!(
            super::parse("format 1\n## types\ntype|checks|min|exempt\npath:src/**|-|-|x\n").is_ok()
        );
    }
}
