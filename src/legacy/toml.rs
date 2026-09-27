//! The TOML subset `.unit-coverage.toml` uses (legacy §C, root R9): top
//! level `key = value` pairs, `[[name]]` array tables, and values that are
//! strings, booleans or arrays of strings. Anything else is refused by
//! line, never guessed at; pklith takes no general TOML dependency.

/// A value the subset holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Val {
    /// A basic (`"…"`) or literal (`'…'`) string, unescaped.
    Str(String),
    /// `true` or `false`.
    Bool(bool),
    /// An array of strings; it may span lines and end with a comma.
    List(Vec<String>),
}

/// The pairs of one table, in document order.
pub type Table = Vec<(String, Val)>;

/// A parsed document: the top-level pairs and each `[[name]]` entry.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Doc {
    /// Pairs before the first array table.
    pub top: Table,
    /// Every `[[name]]` entry with its pairs, in document order.
    pub tables: Vec<(String, Table)>,
}

/// The value `key` holds in `table`, first wins.
#[must_use]
pub fn get<'a>(table: &'a Table, key: &str) -> Option<&'a Val> {
    table.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

/// Parse `text` as the subset.
///
/// # Errors
///
/// The line where the text leaves the subset, and why.
pub fn parse(text: &str) -> Result<Doc, String> {
    let mut p = Parser {
        chars: text.chars().peekable(),
        line: 1,
    };
    let mut doc = Doc::default();
    while p.skip_blank() {
        let at = p.line;
        p.statement(&mut doc)
            .map_err(|e| format!("line {at}: {e}"))?;
    }
    Ok(doc)
}

struct Parser<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    line: usize,
}

impl Parser<'_> {
    fn next(&mut self) -> Option<char> {
        let c = self.chars.next();
        self.line += usize::from(c == Some('\n'));
        c
    }

    fn eat(&mut self, want: char) -> bool {
        let hit = self.chars.peek() == Some(&want);
        if hit {
            self.next();
        }
        hit
    }

    /// Skip spaces and tabs on this line.
    fn spaces(&mut self) {
        while self.chars.next_if(|c| *c == ' ' || *c == '\t').is_some() {}
    }

    /// Skip to the end of this line.
    fn comment(&mut self) {
        while self.chars.next_if(|c| *c != '\n').is_some() {}
    }

    /// Skip blank lines and comments; whether anything is left.
    fn skip_blank(&mut self) -> bool {
        loop {
            match self.chars.peek() {
                Some(c) if c.is_whitespace() => {}
                Some('#') => self.comment(),
                other => return other.is_some(),
            }
            self.next();
        }
    }

    fn statement(&mut self, doc: &mut Doc) -> Result<(), String> {
        if self.eat('[') {
            let name = self.header()?;
            doc.tables.push((name, Table::new()));
        } else {
            let pair = self.pair()?;
            match doc.tables.last_mut() {
                Some((_, table)) => table.push(pair),
                None => doc.top.push(pair),
            }
        }
        self.line_end()
    }

    /// `[[name]]`, the first `[` already read.
    fn header(&mut self) -> Result<String, String> {
        if !self.eat('[') {
            return Err("only `[[array]]` tables are in the subset".into());
        }
        let name = self.key();
        if self.eat(']') && self.eat(']') && !name.is_empty() {
            return Ok(name);
        }
        Err("a table header is `[[name]]`".into())
    }

    fn key(&mut self) -> String {
        self.spaces();
        let mut key = String::new();
        while let Some(c) = self
            .chars
            .next_if(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        {
            key.push(c);
        }
        self.spaces();
        key
    }

    fn pair(&mut self) -> Result<(String, Val), String> {
        let key = self.key();
        if key.is_empty() || !self.eat('=') {
            return Err("expected `key = value`".into());
        }
        self.spaces();
        Ok((key, self.value()?))
    }

    fn value(&mut self) -> Result<Val, String> {
        match self.chars.peek() {
            Some('"' | '\'') => self.string().map(Val::Str),
            Some('[') => self.list().map(Val::List),
            _ => self.boolean(),
        }
    }

    fn boolean(&mut self) -> Result<Val, String> {
        let mut word = String::new();
        while let Some(c) = self.chars.next_if(char::is_ascii_alphanumeric) {
            word.push(c);
        }
        match word.as_str() {
            "true" => Ok(Val::Bool(true)),
            "false" => Ok(Val::Bool(false)),
            _ => Err("a value is a string, a boolean or an array of strings".into()),
        }
    }

    fn list(&mut self) -> Result<Vec<String>, String> {
        self.next();
        let mut items = Vec::new();
        loop {
            self.skip_blank();
            if self.eat(']') {
                return Ok(items);
            }
            items.push(self.string()?);
            self.skip_blank();
            if !self.eat(',') && self.chars.peek() != Some(&']') {
                return Err("array items are separated by `,`".into());
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        let Some(quote) = self.next().filter(|q| *q == '"' || *q == '\'') else {
            return Err("an array holds strings only".into());
        };
        let mut out = String::new();
        loop {
            match self.next() {
                Some(c) if c == quote => return Ok(out),
                Some('\\') if quote == '"' => out.push(self.escaped()?),
                Some('\n') | None => return Err("a string is not closed on its line".into()),
                Some(c) => out.push(c),
            }
        }
    }

    fn escaped(&mut self) -> Result<char, String> {
        match self.next() {
            Some('n') => Ok('\n'),
            Some('t') => Ok('\t'),
            Some(c @ ('"' | '\\')) => Ok(c),
            other => Err(format!("unknown escape `\\{}`", other.unwrap_or(' '))),
        }
    }

    /// Only a comment may follow a statement on its line.
    fn line_end(&mut self) -> Result<(), String> {
        self.spaces();
        match self.chars.peek() {
            None | Some('\n' | '\r' | '#') => Ok(()),
            Some(c) => Err(format!("unexpected `{c}` after the statement")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Doc, Val, get, parse};

    const TEXT: &str = "# config\nallowlist = '.mine' # why\n\n[[rules]]\nglob = \"*.sh\"\ndirs = [\n  \"a\", # first\n  'b',\n]\nnormalize = true\n\n[[ rules ]]\nstrip = \"x\\\"\\\\\\n\\t\"\nexclude = []\nflag = false\n";

    /// Top-level pairs, array tables, strings of both quotes with escapes,
    /// multi-line arrays with comments and a trailing comma, booleans.
    #[test]
    fn the_subset_parses() -> Result<(), String> {
        let doc = parse(TEXT)?;
        assert_eq!(doc, want());
        assert_eq!(get(&doc.top, "allowlist"), Some(&text(".mine")));
        assert_eq!(get(&doc.top, "none"), None);
        assert_eq!(parse("")?, Doc::default());
        Ok(())
    }

    fn text(s: &str) -> Val {
        Val::Str(s.to_owned())
    }

    /// What `TEXT` holds.
    fn want() -> Doc {
        let first = vec![
            ("glob".into(), text("*.sh")),
            ("dirs".into(), Val::List(vec!["a".into(), "b".into()])),
            ("normalize".into(), Val::Bool(true)),
        ];
        let second = vec![
            ("strip".into(), text("x\"\\\n\t")),
            ("exclude".into(), Val::List(Vec::new())),
            ("flag".into(), Val::Bool(false)),
        ];
        Doc {
            top: vec![("allowlist".into(), text(".mine"))],
            tables: vec![("rules".into(), first), ("rules".into(), second)],
        }
    }

    const REFUSED: [(&str, &str); 11] = [
        ("[rules]", "line 1: only `[[array]]` tables"),
        ("[[rules]", "line 1: a table header is `[[name]]`"),
        ("[[]]", "line 1: a table header"),
        ("\n= 1", "line 2: expected `key = value`"),
        ("a 1", "line 1: expected `key = value`"),
        ("a = 1", "line 1: a value is a string, a boolean"),
        ("a = [1]", "line 1: an array holds strings only"),
        ("a = [\"x\" \"y\"]", "line 1: array items are separated"),
        ("a = \"x", "line 1: a string is not closed on its line"),
        ("a = \"\\q\"", "line 1: unknown escape `\\q`"),
        ("a = true b", "line 1: unexpected `b` after the statement"),
    ];

    /// What leaves the subset is named by line, never guessed around.
    #[test]
    fn outside_the_subset_is_refused() {
        for (text, want) in REFUSED {
            let err = parse(text).err().unwrap_or_default();
            assert!(err.starts_with(want), "{text}: {err}");
        }
    }
}
