use super::{Doc, Val, get, parse};

const TEXT: &str = "# config\nallowlist = '.mine' # why\nversion = 1\nmeta.note = \"x\"\n\n[[rules]]\nglob = \"*.sh\"\ndirs = [\n  \"a\", # first\n  'b', 3, true,\n]\nnormalize = true\n\n[meta]\nwhen = 1979-05-27T07:32:00Z\n\n[[ rules ]]\nstrip = \"x\\\"\\\\\\n\\t\\u00e9\\U0001F600\\r\\b\\f\"\nexclude = []\nempty = ''\nflag = false\n";

fn text(s: &str) -> Val {
    Val::Str(s.to_owned())
}

/// The two `[[rules]]` entries `TEXT` holds.
fn rules() -> Vec<(String, super::Table)> {
    let first = vec![
        ("glob".into(), text("*.sh")),
        (
            "dirs".into(),
            Val::List(vec!["a".into(), "b".into(), "3".into(), "true".into()]),
        ),
        ("normalize".into(), Val::Bool(true)),
    ];
    let second = vec![
        ("strip".into(), text("x\"\\\n\té\u{1F600}\r\u{8}\u{c}")),
        ("exclude".into(), Val::List(Vec::new())),
        ("empty".into(), text("")),
        ("flag".into(), Val::Bool(false)),
    ];
    vec![("rules".into(), first), ("rules".into(), second)]
}

/// What `TEXT` holds.
fn want() -> Doc {
    let meta = vec![("when".into(), Val::Other("1979-05-27T07:32:00Z".into()))];
    Doc {
        top: vec![
            ("allowlist".into(), text(".mine")),
            ("version".into(), Val::Other("1".into())),
            ("meta.note".into(), text("x")),
        ],
        tables: rules(),
        plain: vec![("meta".into(), meta)],
    }
}

/// Top-level pairs with dotted keys and numbers, array tables around a
/// plain table (whose pairs stay its own), both quotes with every escape,
/// multi-line arrays with comments, a trailing comma and a number.
#[test]
fn the_subset_parses() -> Result<(), String> {
    let doc = parse(TEXT)?;
    assert_eq!(doc, want());
    assert_eq!(get(&doc.top, "allowlist"), Some(&text(".mine")));
    assert_eq!(get(&doc.top, "none"), None);
    assert_eq!(parse("")?, Doc::default());
    Ok(())
}

const REFUSED: [(&str, &str); 13] = [
    (
        "[[rules]",
        "line 1: a table header is `[name]` or `[[name]]`",
    ),
    ("[rules", "line 1: a table header"),
    ("[[]]", "line 1: a table header"),
    ("\n= 1", "line 2: expected `key = value`"),
    ("a 1", "line 1: expected `key = value`"),
    ("a = word", "line 1: a value is a string, a number"),
    (
        "a = {b = 1}",
        "line 1: inline tables are outside the subset",
    ),
    (
        "a = \"\"\"x\"\"\"",
        "line 1: multi-line strings are outside the subset",
    ),
    (
        "a = [[\"x\"]]",
        "line 1: nested arrays are outside the subset",
    ),
    ("a = [\"x\" \"y\"]", "line 1: array items are separated"),
    ("a = \"x", "line 1: a string is not closed on its line"),
    ("a = \"\\q\"", "line 1: unknown escape `\\q`"),
    ("a = \"\\uzz\"", "line 1: bad escape"),
];

/// What leaves the subset is named by line, never guessed around.
#[test]
fn outside_the_subset_is_refused() {
    for (text, want) in REFUSED {
        let err = parse(text).err().unwrap_or_default();
        assert!(err.starts_with(want), "{text}: {err}");
    }
    let tail = parse("a = true b").err().unwrap_or_default();
    assert_eq!(tail, "line 1: unexpected `b` after the statement");
}
