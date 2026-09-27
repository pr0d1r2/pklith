//! `pkli import DOC`: a `.pklith` from a legacy document.

use super::{Outcome, data, exit};
use std::path::Path;

/// The `.pklith` text is data, so it goes to stdout (V3).
pub(super) fn run(doc: &Path) -> Outcome {
    let imported = std::fs::read_to_string(doc)
        .map_err(|e| format!("cannot read {}: {e}", doc.display()))
        .and_then(|text| crate::legacy::import(&text).map_err(|e| e.to_string()));
    match imported {
        Ok(text) => data(text),
        Err(message) => exit(2, format!("pkli import: {message}\n")),
    }
}
