//! `pkli import fragment-map.json` (root T51): set-and-setting's
//! `coveragePerFileClass`, as `nix eval --json` prints it, turned into
//! `.pklith` type rows. pklith never evaluates nix (legacy §C).

use super::json::{self, Value};
use crate::catalog::Check;

/// The `.pklith` text for a fragment map: one row per file class, its
/// checks renamed to pklith's (PORT.md). A class whose checks pklith does
/// not ship is exempt with reason `legacy: <checks>` rather than dropped
/// (legacy V3), and an unlinted class keeps its own reason.
///
/// # Errors
///
/// JSON that does not parse, or a map without `coveragePerFileClass`.
pub fn import(text: &str, catalog: &[Check]) -> Result<String, String> {
    let map = json::parse(text)?;
    let Some(Value::Obj(classes)) = map.get("coveragePerFileClass") else {
        return Err("not a fragment map: no `coveragePerFileClass` object".into());
    };
    let mut rows: Vec<String> = classes
        .iter()
        .map(|(class, checks)| row(class, checks, catalog))
        .collect();
    rows.extend(unlinted(&map));
    let header = "# Imported from set-and-setting's check-fragment-map.nix by `pkli import`.";
    Ok(format!(
        "format 1\n\n{header}\n\n## types\ntype|checks|min|exempt\n{}\n",
        rows.join("\n")
    ))
}

/// Exempt rows for `unlintedFileClasses`, each with its own reason.
fn unlinted(map: &Value) -> Vec<String> {
    let Some(Value::Obj(classes)) = map.get("unlintedFileClasses") else {
        return Vec::new();
    };
    classes
        .iter()
        .map(|(class, why)| format!("{}|-|-|{}", key(class), reason(why)))
        .collect()
}

fn row(class: &str, checks: &Value, catalog: &[Check]) -> String {
    let names = strings(checks);
    let mut kept: Vec<&str> = Vec::new();
    for id in names.iter().map(|n| crate::catalog::renamed(n)) {
        if catalog.iter().any(|c| c.id == id) && !kept.contains(&id) {
            kept.push(id);
        }
    }
    if kept.is_empty() {
        return format!("{}|-|-|legacy: {}", key(class), names.join(", "));
    }
    format!("{}|{}|-|-", key(class), kept.join(", "))
}

/// The registry key for a file class: `all` is the universal row, a path
/// shape a `path:` class, a name keeps its dot dropped (registry §C).
fn key(class: &str) -> String {
    match class {
        "all" => "*".to_owned(),
        c if c.ends_with('/') => format!("path:{c}**"),
        c if c.contains('*') => format!("path:{c}"),
        c if c.contains('/') => format!("path:{c}/**"),
        c => c.strip_prefix('.').unwrap_or(c).to_owned(),
    }
}

fn strings(value: &Value) -> Vec<String> {
    let Value::Arr(items) = value else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|i| {
            if let Value::Str(s) = i {
                Some(s.clone())
            } else {
                None
            }
        })
        .collect()
}

fn reason(value: &Value) -> String {
    match value {
        Value::Str(s) if !s.is_empty() => s.replace('|', "\\|"),
        _ => "unlinted in set-and-setting".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::import;

    const MAP: &str = r#"{"checksPerFragment":{"nix":["nixfmt"]},"coveragePerFileClass":{"all":["gitleaks","typos","commit-msg-lint"],"nix":["nixfmt","statix","flake-manifest"],".github/workflows":["actionlint"],"spec/":["rspec"],"set/**/*.md":["rekall-check"],".rubocop.yml":["rubocop"]},"unlintedFileClasses":{"lock":"generated"}}"#;

    const WANT: &str = "format 1\n\n# Imported from set-and-setting's check-fragment-map.nix by `pkli import`.\n\n## types\ntype|checks|min|exempt\n*|ripsecrets, typos|-|-\nnix|nixfmt, statix|-|-\npath:.github/workflows/**|actionlint|-|-\npath:spec/**|-|-|legacy: rspec\npath:set/**/*.md|-|-|legacy: rekall-check\nrubocop.yml|rubocop|-|-\nlock|-|-|generated\n";

    /// T51: classes become type rows, renamed; what pklith lacks is an
    /// exempt row naming it; the result is a registry pklith reads.
    #[test]
    fn a_fragment_map_becomes_type_rows() -> Result<(), Box<dyn std::error::Error>> {
        let text = import(MAP, &crate::catalog::builtin()?)?;
        assert_eq!(text, WANT);
        crate::registry::parse(&text)?;
        Ok(())
    }

    /// Not a fragment map, or not JSON: an error, never an empty registry.
    #[test]
    fn a_wrong_document_is_refused() {
        let want = "not a fragment map: no `coveragePerFileClass` object";
        assert_eq!(import("{}", &[]).err().as_deref(), Some(want));
        assert!(import("{", &[]).is_err());
    }
    /// Odd shapes still import without dropping a class: a class whose
    /// value is not a list, a non-string item, an empty reason, and no
    /// unlinted classes at all.
    #[test]
    fn odd_shapes_import_without_loss() -> Result<(), Box<dyn std::error::Error>> {
        let catalog = crate::catalog::builtin()?;
        let map = r#"{"coveragePerFileClass":{"sh":["shellcheck", 1],"x":"typos"}}"#;
        assert!(import(map, &catalog)?.ends_with("sh|shellcheck|-|-\nx|-|-|legacy: \n"));
        let empty = r#"{"coveragePerFileClass":{},"unlintedFileClasses":{"lock":""}}"#;
        assert!(import(empty, &catalog)?.ends_with("lock|-|-|unlinted in set-and-setting\n"));
        Ok(())
    }
}
