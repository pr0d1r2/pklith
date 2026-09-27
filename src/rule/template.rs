//! Path and pattern templates (rule §C): `{var|filter|…}` over a source
//! file, rendered the same way every time (V1).

/// A value taken from the source path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Var {
    /// The whole path.
    Path,
    /// Its directory, empty at the root.
    Dir,
    /// Its basename without the last extension.
    Stem,
    /// Its last extension, without the dot.
    Ext,
}

/// A transformation applied left to right.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Filter {
    /// Drop a leading prefix, when present.
    Strip(String),
    /// Drop a trailing suffix, when present.
    Chop(String),
    /// Append a suffix.
    Suffix(String),
    /// Replace the last extension.
    Ext(String),
    /// `CamelCase` and dashes to `snake_case`.
    Snake,
    /// As `snake`, with dashes.
    Dash,
    /// English plural: fixed suffix rules, then the `## plural` table.
    Plural,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Part {
    Text(String),
    Value(Var, Vec<Filter>),
}

/// A parsed template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template(Vec<Part>);

/// Irregular plurals from `## plural`: `(singular, plural)`.
pub type Plurals = [(String, String)];

impl Template {
    /// Parse `text`.
    ///
    /// # Errors
    ///
    /// An unclosed `{`, or an unknown variable or filter, named (V1).
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut parts = Vec::new();
        let mut rest = text;
        while let Some(open) = rest.find('{') {
            parts.push(Part::Text(rest.get(..open).unwrap_or_default().to_owned()));
            let after = rest.get(open + 1..).unwrap_or_default();
            let close = after
                .find('}')
                .ok_or_else(|| format!("unclosed `{{` in `{text}`"))?;
            parts.push(value(after.get(..close).unwrap_or_default())?);
            rest = after.get(close + 1..).unwrap_or_default();
        }
        parts.push(Part::Text(rest.to_owned()));
        Ok(Self(parts))
    }

    /// The template over `source`, with `//` collapsed so an empty `{dir}`
    /// leaves a clean path.
    #[must_use]
    pub fn render(&self, source: &str, plurals: &Plurals) -> String {
        let text: String = self.0.iter().map(|p| part(p, source, plurals)).collect();
        text.split('/')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("/")
    }
}

fn value(inner: &str) -> Result<Part, String> {
    let mut pieces = inner.split('|');
    let var = match pieces.next().unwrap_or_default() {
        "path" => Var::Path,
        "dir" => Var::Dir,
        "stem" => Var::Stem,
        "ext" => Var::Ext,
        other => return Err(format!("unknown template variable `{other}`")),
    };
    let filters = pieces.map(filter).collect::<Result<Vec<_>, _>>()?;
    Ok(Part::Value(var, filters))
}

fn filter(text: &str) -> Result<Filter, String> {
    let (name, arg) = text.split_once(':').unwrap_or((text, ""));
    let arg = arg.to_owned();
    match name {
        "strip" => Ok(Filter::Strip(arg)),
        "chop" => Ok(Filter::Chop(arg)),
        "suffix" => Ok(Filter::Suffix(arg)),
        "ext" => Ok(Filter::Ext(arg)),
        "snake" => Ok(Filter::Snake),
        "dash" => Ok(Filter::Dash),
        "plural" => Ok(Filter::Plural),
        _ => Err(format!("unknown template filter `{text}`")),
    }
}

fn part(part: &Part, source: &str, plurals: &Plurals) -> String {
    match part {
        Part::Text(text) => text.clone(),
        Part::Value(var, filters) => filters
            .iter()
            .fold(var_of(*var, source), |s, f| apply(f, &s, plurals)),
    }
}

fn var_of(var: Var, source: &str) -> String {
    let (dir, base) = source.rsplit_once('/').unwrap_or(("", source));
    let (stem, ext) = base
        .rsplit_once('.')
        .filter(|(s, _)| !s.is_empty())
        .unwrap_or((base, ""));
    match var {
        Var::Path => source,
        Var::Dir => dir,
        Var::Stem => stem,
        Var::Ext => ext,
    }
    .to_owned()
}

fn apply(filter: &Filter, s: &str, plurals: &Plurals) -> String {
    match filter {
        Filter::Strip(prefix) => s.strip_prefix(prefix.as_str()).unwrap_or(s).to_owned(),
        Filter::Chop(suffix) => s.strip_suffix(suffix.as_str()).unwrap_or(s).to_owned(),
        Filter::Suffix(suffix) => format!("{s}{suffix}"),
        Filter::Ext(ext) => with_ext(s, ext),
        Filter::Snake => snake(s),
        Filter::Dash => snake(s).replace('_', "-"),
        Filter::Plural => plural(s, plurals),
    }
}

/// `s` with its last extension replaced, its directory kept.
fn with_ext(s: &str, ext: &str) -> String {
    let dir = var_of(Var::Dir, s);
    let file = format!("{}.{ext}", var_of(Var::Stem, s));
    if dir.is_empty() {
        file
    } else {
        format!("{dir}/{file}")
    }
}

/// `UserProfile` and `user-profile` both become `user_profile`.
fn snake(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 && !out.ends_with('_') {
            out.push('_');
        }
        out.push(if c == '-' {
            '_'
        } else {
            c.to_ascii_lowercase()
        });
    }
    out
}

/// The `## plural` table first, then fixed suffix rules: a consonant and
/// `y` become `ies`; `s x z ch sh` take `es`; anything else `s`. Never a
/// dictionary guess (rule §C).
fn plural(word: &str, plurals: &Plurals) -> String {
    if let Some((_, many)) = plurals.iter().find(|(one, _)| one == word) {
        return many.clone();
    }
    let consonant_y = word
        .strip_suffix('y')
        .filter(|w| !w.ends_with(['a', 'e', 'i', 'o', 'u']));
    let sibilant = ["s", "x", "z", "ch", "sh"]
        .iter()
        .any(|e| word.ends_with(e));
    match consonant_y {
        Some(stem) if !stem.is_empty() => format!("{stem}ies"),
        _ if sibilant => format!("{word}es"),
        _ => format!("{word}s"),
    }
}

#[cfg(test)]
mod tests {
    use super::Template;

    const TABLE: [(&str, &str); 1] = [("person", "people")];

    fn render(template: &str, source: &str) -> Result<String, String> {
        let table: Vec<(String, String)> = TABLE
            .iter()
            .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
            .collect();
        Ok(Template::parse(template)?.render(source, &table))
    }

    const CASES: [(&str, &str, &str); 12] = [
        ("{path|ext:md}", "a.txt", "a.md"),
        ("{stem|snake}", "user-profile.rb", "user_profile"),
        (
            "spec/{dir|strip:app/}/{stem}_spec.rb",
            "app/models/user.rb",
            "spec/models/user_spec.rb",
        ),
        ("spec/{dir}/{stem}_spec.rb", "user.rb", "spec/user_spec.rb"),
        (
            "app/{dir|strip:spec/}/{stem|chop:_spec}.rb",
            "spec/models/user_spec.rb",
            "app/models/user.rb",
        ),
        ("{path|ext:md}", "docs/a.txt", "docs/a.md"),
        ("{stem|snake}", "UserProfile.rb", "user_profile"),
        ("{stem|dash}", "UserProfile.rb", "user-profile"),
        ("{stem|plural}", "a/city.rb", "cities"),
        ("{stem|plural}", "a/day.rb", "days"),
        ("{stem|plural}{ext|suffix:!}", "box.rb", "boxesrb!"),
        ("{stem|plural}", "person.rb", "people"),
    ];

    /// V1: variables, filters and plurals render deterministically.
    #[test]
    fn templates_render_from_the_source_path() -> Result<(), String> {
        for (template, source, want) in CASES {
            assert_eq!(render(template, source)?, want, "{template} on {source}");
        }
        Ok(())
    }

    /// V1: an unknown variable or filter, or an unclosed brace, is an error
    /// at parse, never an empty string.
    #[test]
    fn a_bad_template_is_refused_at_parse() {
        for (text, want) in [
            ("{name}", "unknown template variable `name`"),
            ("{stem|upper}", "unknown template filter `upper`"),
            ("a/{stem", "unclosed `{` in `a/{stem`"),
        ] {
            assert_eq!(Template::parse(text).err().as_deref(), Some(want));
        }
    }
}
