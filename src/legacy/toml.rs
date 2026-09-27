//! The TOML `.unit-coverage.toml` files are written in (legacy §C, root
//! R9), read by hand: pklith takes no general TOML dependency. Top-level
//! pairs, `[[name]]` array tables, `[name]` tables (read and set aside),
//! dotted keys, and values that are strings, booleans, numbers, dates or
//! arrays of those. Multi-line strings and inline tables are refused by
//! line, never guessed at: no rule key the legacy tool read takes one.

/// A value the subset holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Val {
    /// A basic (`"…"`) or literal (`'…'`) string, unescaped.
    Str(String),
    /// `true` or `false`.
    Bool(bool),
    /// A number or a date, as written.
    Other(String),
    /// An array, each item as `taplo get` printed it; it may span lines
    /// and end with a comma.
    List(Vec<String>),
}

/// The pairs of one table, in document order.
pub type Table = Vec<(String, Val)>;

/// A parsed document: the top-level pairs, each `[[name]]` entry, and each
/// `[name]` table.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Doc {
    /// Pairs before the first table.
    pub top: Table,
    /// Every `[[name]]` entry with its pairs, in document order.
    pub tables: Vec<(String, Table)>,
    /// Every `[name]` table with its pairs; no rule is read from one.
    pub plain: Vec<(String, Table)>,
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
        plain: false,
    };
    let mut doc = Doc::default();
    while p.skip_blank() {
        let at = p.line;
        p.statement(&mut doc)
            .map_err(|e| format!("line {at}: {e}"))?;
    }
    Ok(doc)
}

/// Escapes that stand for one fixed character.
const SIMPLE: [(char, char); 7] = [
    ('n', '\n'),
    ('t', '\t'),
    ('r', '\r'),
    ('b', '\u{8}'),
    ('f', '\u{c}'),
    ('"', '"'),
    ('\\', '\\'),
];

/// The array tables, or the plain ones.
fn tables(doc: &mut Doc, array: bool) -> &mut Vec<(String, Table)> {
    if array {
        &mut doc.tables
    } else {
        &mut doc.plain
    }
}

struct Parser<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    line: usize,
    /// Whether pairs go to the last `[name]` table rather than the last
    /// `[[name]]` entry.
    plain: bool,
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
            let (name, array) = self.header()?;
            self.plain = !array;
            tables(doc, array).push((name, Table::new()));
        } else {
            let pair = self.pair()?;
            match tables(doc, !self.plain).last_mut() {
                Some((_, table)) => table.push(pair),
                None => doc.top.push(pair),
            }
        }
        self.line_end()
    }

    /// `[[name]]` or `[name]`, the first `[` already read; whether it is
    /// an array table.
    fn header(&mut self) -> Result<(String, bool), String> {
        let array = self.eat('[');
        let name = self.key();
        let closed = self.eat(']') && (!array || self.eat(']'));
        if closed && !name.is_empty() {
            return Ok((name, array));
        }
        Err("a table header is `[name]` or `[[name]]`".into())
    }

    /// A bare key, dots joining its parts.
    fn key(&mut self) -> String {
        self.spaces();
        let mut key = String::new();
        let bare = |c: &char| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.');
        while let Some(c) = self.chars.next_if(bare) {
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
            Some('{') => Err("inline tables are outside the subset".into()),
            _ => self.scalar(),
        }
    }

    /// A boolean, or a number or date kept as written.
    fn scalar(&mut self) -> Result<Val, String> {
        let mut word = String::new();
        let part = |c: &char| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.' | '_' | ':');
        while let Some(c) = self.chars.next_if(part) {
            word.push(c);
        }
        let numeric = word.starts_with(|c: char| c.is_ascii_digit() || c == '+' || c == '-');
        match word.as_str() {
            "true" => Ok(Val::Bool(true)),
            "false" => Ok(Val::Bool(false)),
            "inf" | "nan" => Ok(Val::Other(word)),
            _ if numeric => Ok(Val::Other(word)),
            _ => Err("a value is a string, a number, a boolean, a date or an array".into()),
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
            items.push(self.item()?);
            self.skip_blank();
            if !self.eat(',') && self.chars.peek() != Some(&']') {
                return Err("array items are separated by `,`".into());
            }
        }
    }

    /// One array item, as text.
    fn item(&mut self) -> Result<String, String> {
        match self.value()? {
            Val::Str(s) | Val::Other(s) => Ok(s),
            Val::Bool(b) => Ok(b.to_string()),
            Val::List(_) => Err("nested arrays are outside the subset".into()),
        }
    }

    fn string(&mut self) -> Result<String, String> {
        let quote = self.next().unwrap_or('"');
        if self.eat(quote) {
            return self.empty(quote);
        }
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

    /// Two quotes read: an empty string, or the start of a multi-line one.
    fn empty(&mut self, quote: char) -> Result<String, String> {
        if self.chars.peek() == Some(&quote) {
            return Err("multi-line strings are outside the subset".into());
        }
        Ok(String::new())
    }

    fn escaped(&mut self) -> Result<char, String> {
        match self.next() {
            Some('u') => self.unicode(4),
            Some('U') => self.unicode(8),
            other => SIMPLE
                .iter()
                .find(|(e, _)| Some(*e) == other)
                .map(|(_, c)| *c)
                .ok_or(format!("unknown escape `\\{}`", other.unwrap_or(' '))),
        }
    }

    fn unicode(&mut self, digits: usize) -> Result<char, String> {
        let hex: String = (0..digits).filter_map(|_| self.next()).collect();
        u32::from_str_radix(&hex, 16)
            .ok()
            .and_then(char::from_u32)
            .ok_or(format!("bad escape `{hex}`"))
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
mod tests;
