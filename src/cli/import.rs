//! `pklith import DOC`: a `.pklith` from a legacy document, chosen by its
//! extension: a set-and-setting fragment map (`.json`, what `nix eval
//! --json` prints), else a linter coverage document.

use super::{Outcome, data, exit, parsed};
use std::path::Path;

/// The `.pklith` text is data, so it goes to stdout (V3).
pub(super) fn run(doc: &Path) -> Outcome {
    let imported = std::fs::read_to_string(doc)
        .map_err(|e| format!("cannot read {}: {e}", doc.display()))
        .and_then(|text| convert(doc, &text));
    match imported {
        Ok(text) => data(text),
        Err(message) => exit(2, format!("pklith import: {message}\n")),
    }
}

fn convert(doc: &Path, text: &str) -> Result<String, String> {
    if doc
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("json"))
    {
        let catalog = parsed(super::BUILTIN_ONLY)?.catalog;
        return crate::legacy::fragment_map::import(text, &catalog);
    }
    crate::legacy::import(text).map_err(|e| e.to_string())
}
