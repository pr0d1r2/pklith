//! The verdict as JSON (report T2), hand-written (report §C): every key
//! always present, lists empty rather than absent (V2), a top-level
//! `version` a consumer checks before reading (V4).

use super::Verdict;
use crate::cover::{Covered, Gap, Stale, Unbacked, Unreflected};
use crate::rule::Failure;
use std::fmt::Write as _;

/// The JSON shape's version; a breaking field change bumps it (V4).
pub const VERSION: u32 = 1;

/// The verdict as one JSON object, keys in a fixed order (V1).
#[must_use]
pub fn render(v: &Verdict<'_>) -> String {
    let c = v.coverage;
    let ok = c.ok() && v.unbacked.is_empty();
    let fields = [
        format!("\"version\":{VERSION}"),
        format!("\"ok\":{ok}"),
        format!("\"universal\":{}", strings(v.universal)),
        format!("\"covered\":{}", list(&c.covered, covered)),
        format!("\"gaps\":{}", list(&c.gaps, gap)),
        format!("\"stale\":{}", list(&c.stale, stale)),
        format!("\"unbacked\":{}", list(v.unbacked, unbacked)),
        format!("\"rules\":{}", list(&c.failed, failed)),
        format!("\"fragments\":{}", list(&c.unreflected, unreflected)),
    ];
    format!("{{{}}}\n", fields.join(","))
}

fn list<T>(items: &[T], one: fn(&T) -> String) -> String {
    format!("[{}]", items.iter().map(one).collect::<Vec<_>>().join(","))
}

fn covered(c: &Covered) -> String {
    let exempt = c
        .exempt
        .as_deref()
        .map_or_else(|| "null".to_owned(), string);
    format!(
        "{{\"type\":{},\"files\":{},\"checks\":{},\"exempt\":{exempt}}}",
        string(&c.key),
        c.files,
        strings(&c.checks)
    )
}

fn gap(g: &Gap) -> String {
    format!(
        "{{\"type\":{},\"files\":{}}}",
        string(&g.key),
        strings(&g.files)
    )
}

fn stale(s: &Stale) -> String {
    format!("{{\"type\":{},\"line\":{}}}", string(&s.key), s.line)
}

fn unbacked(u: &Unbacked) -> String {
    let (check, problem) = (string(&u.check), string(&u.problem));
    format!(
        "{{\"check\":{check},\"problem\":{problem},\"files\":{}}}",
        strings(&u.files)
    )
}

fn failed(f: &Failure) -> String {
    let (rule, source, missing) = (string(&f.rule), string(&f.source), string(&f.missing));
    format!("{{\"rule\":{rule},\"source\":{source},\"missing\":{missing}}}")
}

fn unreflected(u: &Unreflected) -> String {
    format!(
        "{{\"fragment\":{},\"rows\":{}}}",
        string(&u.fragment),
        strings(&u.rows)
    )
}

fn strings(items: &[String]) -> String {
    format!(
        "[{}]",
        items
            .iter()
            .map(|s| string(s))
            .collect::<Vec<_>>()
            .join(",")
    )
}

/// A JSON string: quotes, backslashes and control characters escaped.
fn string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if u32::from(c) < 0x20 => drop(write!(out, "\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out + "\""
}

#[cfg(test)]
mod tests {
    use super::{render, string};
    use crate::cover::judge;
    use crate::registry::{Error, parse};

    /// Strings escape what JSON requires.
    #[test]
    fn strings_are_escaped() {
        assert_eq!(
            string("a\"b\\c\nd\te\u{1}"),
            "\"a\\\"b\\\\c\\nd\\te\\u0001\""
        );
    }

    const REGISTRY: &str = "format 1\n## types\ntype|checks|min|exempt\n*|typos|-|-\nrs|clippy|-|-\npng|-|-|binary\nrb|rubocop|-|-\n";

    /// V2, V4: every key is present, empty lists as `[]`, versioned.
    #[test]
    fn every_key_is_present_and_versioned() -> Result<(), Error> {
        let files = ["a.rs".to_owned(), "b.png".to_owned(), "x.py".to_owned()];
        let coverage = judge(&files, &parse(REGISTRY)?);
        let verdict = super::Verdict {
            coverage: &coverage,
            unbacked: &[],
            universal: &["typos".to_owned()],
        };
        let want = r#"{"version":1,"ok":false,"universal":["typos"],"covered":[{"type":"rs","files":1,"checks":["clippy"],"exempt":null},{"type":"png","files":1,"checks":[],"exempt":"binary"}],"gaps":[{"type":"py","files":["x.py"]}],"stale":[{"type":"rb","line":7}],"unbacked":[],"rules":[],"fragments":[]}"#;
        assert_eq!(render(&verdict), format!("{want}\n"));
        Ok(())
    }
    /// A coverage holding one failed rule and one unreflected fragment.
    fn findings() -> crate::cover::Coverage {
        let mut coverage = crate::cover::Coverage::default();
        let (rule, source, missing) = ("r".into(), "a".into(), "b".into());
        coverage.failed.push(crate::rule::Failure {
            rule,
            source,
            missing,
        });
        let (fragment, rows) = ("f".into(), vec!["x|y|-|-".into()]);
        coverage
            .unreflected
            .push(crate::cover::Unreflected { fragment, rows });
        coverage
    }

    const FINDINGS: &str = r#""unbacked":[{"check":"c","problem":"p","files":["a"]}],"rules":[{"rule":"r","source":"a","missing":"b"}],"fragments":[{"fragment":"f","rows":["x|y|-|-"]}]"#;

    /// Unbacked claims, failed rules and unreflected fragments render with
    /// every field.
    #[test]
    fn findings_render_with_every_field() {
        let coverage = findings();
        let (check, problem, files) = ("c".into(), "p".into(), vec!["a".into()]);
        let unbacked = [crate::cover::Unbacked {
            check,
            problem,
            files,
        }];
        let v = super::Verdict {
            coverage: &coverage,
            unbacked: &unbacked,
            universal: &[],
        };
        assert!(render(&v).contains(FINDINGS), "{}", render(&v));
    }
}
