//! Enumerate repository files and classify each basename to exactly one file
//! type (`src/scan/SPEC.md`).

/// How a candidate key was derived from a basename (scan §C).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Whole basename without a dot: `justfile`, `Makefile`.
    Bare,
    /// A leading dot and no other: `.envrc`.
    Dotfile,
    /// Whole basename containing dots: `Cargo.lock`.
    Name,
    /// Multi-part suffix: `tar.gz`.
    Compound,
    /// Last suffix: `gz`.
    Ext,
}

/// One way a file could be typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// Registry key, with one leading `.` dropped.
    pub key: String,
    /// How the key was derived.
    pub kind: Kind,
}

/// Candidate types for a path, most specific first (scan V1): the whole
/// basename, then compound suffixes longest first, then the last extension.
///
/// ```
/// let keys: Vec<_> = pklith::scan::candidates("a/foo.tar.gz")
///     .into_iter()
///     .map(|c| c.key)
///     .collect();
/// assert_eq!(keys, ["foo.tar.gz", "tar.gz", "gz"]);
/// ```
#[must_use]
pub fn candidates(path: &str) -> Vec<Candidate> {
    let base = path.rsplit('/').next().unwrap_or(path);
    let name = base.strip_prefix('.').unwrap_or(base);
    let whole = Candidate {
        key: name.to_owned(),
        kind: whole_kind(base, name),
    };
    let parts = suffixes(name).map(|s| Candidate {
        key: s.to_owned(),
        kind: suffix_kind(s),
    });
    std::iter::once(whole).chain(parts).collect()
}

fn whole_kind(base: &str, name: &str) -> Kind {
    match (name.contains('.'), base.starts_with('.')) {
        (true, _) => Kind::Name,
        (false, true) => Kind::Dotfile,
        (false, false) => Kind::Bare,
    }
}

fn suffix_kind(suffix: &str) -> Kind {
    if suffix.contains('.') {
        Kind::Compound
    } else {
        Kind::Ext
    }
}

/// Every non-empty suffix after a dot, longest first.
fn suffixes(name: &str) -> impl Iterator<Item = &str> {
    name.match_indices('.')
        .filter_map(move |(i, _)| name.get(i + 1..))
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{Kind, candidates};

    fn typed(path: &str) -> Vec<(String, Kind)> {
        candidates(path)
            .into_iter()
            .map(|c| (c.key, c.kind))
            .collect()
    }

    fn pair(key: &str, kind: Kind) -> (String, Kind) {
        (key.to_owned(), kind)
    }

    #[test]
    fn every_kind_is_recorded_most_specific_first() {
        assert_eq!(typed("justfile"), [pair("justfile", Kind::Bare)]);
        assert_eq!(typed(".envrc"), [pair("envrc", Kind::Dotfile)]);
        assert_eq!(
            typed("Cargo.lock"),
            [pair("Cargo.lock", Kind::Name), pair("lock", Kind::Ext)]
        );
        let spec = [
            pair("foo.spec.ts", Kind::Name),
            pair("spec.ts", Kind::Compound),
            pair("ts", Kind::Ext),
        ];
        assert_eq!(typed("web/foo.spec.ts"), spec);
    }

    #[test]
    fn a_dotted_dotfile_drops_one_dot_and_keeps_its_suffix() {
        assert_eq!(
            typed(".typos.toml"),
            [pair("typos.toml", Kind::Name), pair("toml", Kind::Ext)]
        );
    }

    #[test]
    fn a_trailing_dot_adds_no_empty_suffix() {
        assert_eq!(typed("notes."), [pair("notes.", Kind::Name)]);
    }

    /// V2: the least specific candidate is what the legacy tools keyed on,
    /// `sed 's/.*\.//'` over the basename (R1, R2).
    #[test]
    fn the_last_candidate_is_the_legacy_key() {
        let legacy = [
            ("x.tar.gz", "gz"),
            (".envrc", "envrc"),
            ("justfile", "justfile"),
            ("a/b/Cargo.lock", "lock"),
            (".typos.toml", "toml"),
        ];
        for (path, key) in legacy {
            assert_eq!(
                candidates(path).last().map(|c| c.key.as_str()),
                Some(key),
                "{path}"
            );
        }
    }
}
