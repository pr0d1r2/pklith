//! Just enough JSON to read what `nix eval --json` prints (legacy §C):
//! objects, arrays and strings; numbers, booleans and null are read and
//! kept as [`Value::Other`].

/// A JSON value, as far as the importer cares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A string, unescaped.
    Str(String),
    /// An array.
    Arr(Vec<Value>),
    /// An object, keys in document order.
    Obj(Vec<(String, Value)>),
    /// A number, `true`, `false` or `null`.
    Other,
}

impl Value {
    /// The member `key` of an object.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Self::Obj(members) => members.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
}

/// Parse `text` as one JSON value.
///
/// # Errors
///
/// Where the text stops being JSON, by byte offset.
pub fn parse(text: &str) -> Result<Value, String> {
    let mut p = Parser {
        chars: text.char_indices().peekable(),
    };
    let value = p.value()?;
    p.blank();
    match p.chars.next() {
        None => Ok(value),
        Some((at, c)) => Err(format!("unexpected `{c}` at byte {at} after the value")),
    }
}

struct Parser<'a> {
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
}

impl Parser<'_> {
    fn blank(&mut self) {
        while self.chars.next_if(|(_, c)| c.is_whitespace()).is_some() {}
    }

    fn value(&mut self) -> Result<Value, String> {
        self.blank();
        match self.chars.peek().copied() {
            Some((_, '{')) => self.object(),
            Some((_, '[')) => self.array(),
            Some((_, '"')) => self.string().map(Value::Str),
            Some((_, c)) if c == '-' || c.is_ascii_alphanumeric() => Ok(self.scalar()),
            Some((at, c)) => Err(format!("unexpected `{c}` at byte {at}")),
            None => Err("the text ends before a value".into()),
        }
    }

    fn scalar(&mut self) -> Value {
        while self
            .chars
            .next_if(|(_, c)| *c == '-' || *c == '+' || *c == '.' || c.is_ascii_alphanumeric())
            .is_some()
        {}
        Value::Other
    }

    fn object(&mut self) -> Result<Value, String> {
        self.chars.next();
        let mut members = Vec::new();
        while self.more('}')? {
            let key = self.string()?;
            self.expect(':')?;
            members.push((key, self.value()?));
        }
        Ok(Value::Obj(members))
    }

    fn array(&mut self) -> Result<Value, String> {
        self.chars.next();
        let mut items = Vec::new();
        while self.more(']')? {
            items.push(self.value()?);
        }
        Ok(Value::Arr(items))
    }

    /// Whether another member or item follows before `close`, consuming a
    /// separating comma or the closing bracket.
    fn more(&mut self, close: char) -> Result<bool, String> {
        self.blank();
        if self.chars.next_if(|(_, c)| *c == close).is_some() {
            return Ok(false);
        }
        self.chars.next_if(|(_, c)| *c == ',');
        self.blank();
        match self.chars.peek() {
            Some(_) => Ok(true),
            None => Err("the text ends inside a bracket".into()),
        }
    }

    fn expect(&mut self, want: char) -> Result<(), String> {
        self.blank();
        match self.chars.next() {
            Some((_, c)) if c == want => Ok(()),
            Some((at, c)) => Err(format!("expected `{want}` at byte {at}, found `{c}`")),
            None => Err(format!("the text ends where `{want}` belongs")),
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            match self.chars.next().map(|(_, c)| c) {
                Some('"') => return Ok(out),
                Some('\\') => out.push(self.escaped()?),
                Some(c) => out.push(c),
                None => return Err("the text ends inside a string".into()),
            }
        }
    }

    fn escaped(&mut self) -> Result<char, String> {
        match self.chars.next().map(|(_, c)| c) {
            Some('n') => Ok('\n'),
            Some('t') => Ok('\t'),
            Some('u') => self.unicode(),
            Some(c @ ('"' | '\\' | '/')) => Ok(c),
            other => Err(format!("unknown escape `\\{}`", other.unwrap_or(' '))),
        }
    }

    fn unicode(&mut self) -> Result<char, String> {
        let hex: String = (0..4)
            .filter_map(|_| self.chars.next().map(|(_, c)| c))
            .collect();
        u32::from_str_radix(&hex, 16)
            .ok()
            .and_then(char::from_u32)
            .ok_or(format!("bad escape `\\u{hex}`"))
    }
}

#[cfg(test)]
mod tests {
    use super::{Value, parse};

    /// Objects, arrays, strings with escapes; other scalars are kept as
    /// `Other`.
    #[test]
    fn nix_eval_output_parses() -> Result<(), String> {
        let text =
            r#" {"a": ["x", "y\"\n\t\u00e9"], "b": {"c": true, "d": -1.5e3, "e": null}, "f": []} "#;
        let v = parse(text)?;
        let strings = vec![Value::Str("x".into()), Value::Str("y\"\n\té".into())];
        assert_eq!(v.get("a"), Some(&Value::Arr(strings)));
        let b = v.get("b");
        let scalars: Vec<_> = ["c", "d", "e"].iter().filter_map(|k| b?.get(k)).collect();
        assert_eq!(scalars, [&Value::Other; 3]);
        assert_eq!(v.get("f"), Some(&Value::Arr(Vec::new())));
        assert_eq!(Value::Other.get("a"), None);
        Ok(())
    }

    const BROKEN: [(&str, &str); 9] = [
        ("{\"a\"", "the text ends where `:` belongs"),
        ("{\"a\" 1}", "expected `:`"),
        ("[\"a\"", "the text ends inside a bracket"),
        ("\"a", "the text ends inside a string"),
        ("{} x", "unexpected `x` at byte 3 after the value"),
        ("\"\\q\"", "unknown escape `\\q`"),
        ("\"\\uzzzz\"", "bad escape"),
        (";", "unexpected `;` at byte 0"),
        ("", "the text ends before a value"),
    ];

    /// Where it stops being JSON is said, never guessed around.
    #[test]
    fn broken_json_is_named() {
        for (text, want) in BROKEN {
            let err = parse(text).err().unwrap_or_default();
            assert!(err.contains(want), "{text}: {err}");
        }
    }
}
