//! `pklith seed --init`: a first `.pklith` from the active fragments (seed
//! T3). The repository owns it afterwards.

use crate::catalog::{Check, Fragment};
use crate::scan::Globs;
use std::collections::BTreeMap;

/// Types that are data, not text a linter reads: seeded as exempt.
const BINARY: [&str; 20] = [
    "png", "jpg", "jpeg", "gif", "ico", "webp", "bmp", "tiff", "pdf", "woff", "woff2", "ttf",
    "otf", "eot", "mp3", "mp4", "wav", "ogg", "zip", "gz",
];

/// A check narrowed to the files its runner reaches.
struct Reach<'a> {
    check: &'a Check,
    globs: Vec<(String, Globs)>,
}

impl Reach<'_> {
    /// The first of its globs that matches `file`.
    fn glob(&self, file: &str) -> Option<&str> {
        self.globs
            .iter()
            .find(|(_, g)| g.matches(file))
            .map(|(p, _)| p.as_str())
    }
}

/// The `.pklith` text for `files`: the checks of `active` fragments that
/// run on every file go in `*`; the rest are given to the types whose
/// files they reach, as a `path:` class where they reach only some files
/// of a type; binary types are exempt. A type no check reaches gets no
/// row, so `pklith check` names it until someone decides (root V5), unless
/// every file of it is one pklith writes itself (`own`: the seed files and
/// what gen writes, such as the vendored hk schema): that type is exempt,
/// so a repository is green the moment it is seeded. So are the `KNOWN`
/// files no check reads, such as `.gitignore` and `flake.lock` (seed V5).
#[must_use]
pub fn registry(files: &[String], active: &[&Fragment], catalog: &[Check], own: &[&str]) -> String {
    let (universal, mut rows) = rows(files, active, catalog);
    let (_, specific) = split(active, catalog);
    let text: Vec<&String> = files.iter().filter(|f| !binary(f)).collect();
    let open = exempt(&text, &specific, own, &mut rows);
    rows.sort();
    render(&universal, &rows, &open)
}

/// Files no built-in check reads that a repository commonly tracks, and
/// why each is exempt (seed V5): a type key, or `path:` and one path.
/// `flake.lock` is named, not `lock`, so Cargo.lock stays a gap.
const KNOWN: [(&str, &str); 2] = [
    ("gitignore", "git configuration; hygiene steps only"),
    (
        "path:flake.lock",
        "written by `nix flake lock`; hygiene steps only",
    ),
];

/// Exempt rows for the types no check reaches that need no decision:
/// those only pklith's own files have, then the `KNOWN` ones. Returns the
/// types left open, for the header to name.
fn exempt(
    text: &[&String],
    specific: &[Reach<'_>],
    own: &[&str],
    rows: &mut Vec<String>,
) -> Vec<String> {
    let (owned, open): (Vec<String>, Vec<String>) = unreached(text, specific)
        .into_iter()
        .partition(|key| only_own(text, key, own));
    rows.extend(
        owned
            .iter()
            .map(|k| format!("{k}|-|-|written by pklith (seed or gen)")),
    );
    let rest = known(text, specific, &open, rows);
    unreached(&rest, specific)
        .into_iter()
        .filter(|key| open.contains(key))
        .collect()
}

/// The `KNOWN` rows that apply, pushed to `rows`; returns `text` less
/// the files they exempt.
fn known<'a>(
    text: &[&'a String],
    specific: &[Reach<'_>],
    open: &[String],
    rows: &mut Vec<String>,
) -> Vec<&'a String> {
    let mut rest = text.to_vec();
    for (class, reason) in KNOWN {
        let hits = hits(&rest, class);
        if free(&hits, specific, open) {
            rows.push(format!("{class}|-|-|{reason}"));
            rest.retain(|f| !hits.contains(f));
        }
    }
    rest
}

/// The files a `KNOWN` class names: every file of a type key, or the one
/// path after `path:`.
fn hits<'a>(files: &[&'a String], class: &str) -> Vec<&'a String> {
    let path = class.strip_prefix("path:");
    files
        .iter()
        .filter(|f| path.map_or(crate::scan::key(f) == class, |p| f.as_str() == p))
        .copied()
        .collect()
}

/// Whether a `KNOWN` exemption applies to `hits`: there are some, no
/// check reaches them, and their type is open. A check that later reads
/// them supersedes the exemption (seed V5).
fn free(hits: &[&String], specific: &[Reach<'_>], open: &[String]) -> bool {
    !hits.is_empty()
        && hits
            .iter()
            .all(|f| reaching(f, specific).is_empty() && open.contains(&crate::scan::key(f)))
}

/// Whether every file of type `key` is one pklith writes itself.
fn only_own(files: &[&String], key: &str, own: &[&str]) -> bool {
    files
        .iter()
        .filter(|f| crate::scan::key(f) == key)
        .all(|f| own.contains(&f.as_str()))
}

/// What `registry` writes, before rendering: the `*` checks, and the type
/// rows sorted bytewise (root V3). Also what `pklith check` suggests for a
/// fragment `.pklith` does not reflect (cover V9).
#[must_use]
pub fn rows(
    files: &[String],
    active: &[&Fragment],
    catalog: &[Check],
) -> (Vec<String>, Vec<String>) {
    let (universal, specific) = split(active, catalog);
    let text: Vec<&String> = files.iter().filter(|f| !binary(f)).collect();
    let classes = classes(&text, &specific);
    let mut rows: Vec<String> = type_rows(&text, &specific, &classes);
    rows.extend(
        classes
            .iter()
            .map(|c| row(&format!("path:{}", c.glob), &c.ids)),
    );
    rows.extend(binary_rows(files));
    rows.sort();
    (universal.into_iter().map(|c| c.id.clone()).collect(), rows)
}

/// Active checks in fragment order, once each: those of `always`
/// fragments or with a whole-tree runner go in `*`, the rest are specific.
fn split<'a>(active: &[&Fragment], catalog: &'a [Check]) -> (Vec<&'a Check>, Vec<Reach<'a>>) {
    let ids = selected(active);
    let found =
        |(id, always): (&str, bool)| catalog.iter().find(|c| c.id == id).map(|c| (c, always));
    let (universal, specific): (Vec<_>, Vec<_>) = ids
        .into_iter()
        .filter_map(found)
        .partition(|(c, always)| *always || whole_tree(c));
    (
        universal.into_iter().map(|(c, _)| c).collect(),
        specific.into_iter().filter_map(reach).collect(),
    )
}

/// Each check the fragments select, once, with whether an `always`
/// fragment selected it first.
fn selected<'a>(active: &[&'a Fragment]) -> Vec<(&'a str, bool)> {
    let mut ids: Vec<(&str, bool)> = Vec::new();
    for fragment in active {
        for id in &fragment.checks {
            if ids.iter().all(|(seen, _)| seen != id) {
                ids.push((id, fragment.always));
            }
        }
    }
    ids
}

/// A runner with no glob, or one matching every path, runs on the whole
/// tree: claiming it for one type would make every type look covered.
fn whole_tree(check: &Check) -> bool {
    check.globs.is_empty() || check.globs.iter().any(|g| g == "**/*" || g == "*")
}

fn reach(entry: (&Check, bool)) -> Option<Reach<'_>> {
    let check = entry.0;
    let compile = |g: &String| Some((g.clone(), Globs::new(std::slice::from_ref(g)).ok()?));
    let globs = check
        .globs
        .iter()
        .map(compile)
        .collect::<Option<Vec<_>>>()?;
    Some(Reach { check, globs })
}

fn binary(file: &str) -> bool {
    BINARY.contains(&crate::scan::key(file).as_str())
}

/// The ids of the specific checks that reach `file`, in order.
fn reaching<'a>(file: &str, specific: &'a [Reach<'_>]) -> Vec<&'a str> {
    specific
        .iter()
        .filter(|r| r.glob(file).is_some())
        .map(|r| r.check.id.as_str())
        .collect()
}

/// Files grouped by type key, keys in byte order.
fn by_key<'a>(files: &[&'a String]) -> BTreeMap<String, Vec<&'a String>> {
    let mut groups: BTreeMap<String, Vec<&String>> = BTreeMap::new();
    for file in files {
        groups.entry(crate::scan::key(file)).or_default().push(file);
    }
    groups
}

/// A `path:` class: its glob, compiled once, and the checks it claims.
struct Class {
    glob: String,
    matcher: Globs,
    ids: Vec<String>,
}

/// `path:` classes: where a check reaches only some files of a type, its
/// matching glob becomes a class, claiming what every file it matches
/// shares. Whether a check misses some file of a type is decided once per
/// type, so the work grows with the files, not their square (root T64).
fn classes(files: &[&String], specific: &[Reach<'_>]) -> Vec<Class> {
    let mut globs: Vec<String> = Vec::new();
    for peers in by_key(files).values() {
        let partial = partial(peers, specific);
        for file in peers {
            if let Some(glob) = narrow(file, specific, &partial).filter(|g| !globs.contains(g)) {
                globs.push(glob);
            }
        }
    }
    globs
        .into_iter()
        .filter_map(|g| class(g, files, specific))
        .collect()
}

/// For each check, whether it misses some of `peers` (one type's files).
fn partial(peers: &[&String], specific: &[Reach<'_>]) -> Vec<bool> {
    specific
        .iter()
        .map(|r| peers.iter().any(|p| r.glob(p).is_none()))
        .collect()
}

fn class(glob: String, files: &[&String], specific: &[Reach<'_>]) -> Option<Class> {
    let matcher = Globs::new(std::slice::from_ref(&glob)).ok()?;
    let ids = shared(
        files.iter().filter(|f| matcher.matches(f)).copied(),
        specific,
    );
    Some(Class { glob, matcher, ids })
}

/// The glob of the first check reaching `file` that misses another file
/// of its type (`partial`, per check), or reaches it by name
/// (`Cargo.toml`): a named file is its own class, so the next file of its
/// type is not claimed for that check.
fn narrow(file: &str, specific: &[Reach<'_>], partial: &[bool]) -> Option<String> {
    specific.iter().zip(partial).find_map(|(r, partial)| {
        let glob = r.glob(file)?;
        let named = !glob.contains(['*', '?', '[', '{']);
        (named || *partial).then(|| glob.to_owned())
    })
}

/// The checks every one of `files` is reached by, in catalog order.
fn shared<'a>(files: impl Iterator<Item = &'a String>, specific: &[Reach<'_>]) -> Vec<String> {
    let common = files
        .map(|f| reaching(f, specific))
        .reduce(|mut ids, these| {
            ids.retain(|id| these.contains(id));
            ids
        });
    common
        .unwrap_or_default()
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// One row per type, over the files no class takes, claiming what they
/// all share; a type sharing nothing gets no row.
fn type_rows(files: &[&String], specific: &[Reach<'_>], classes: &[Class]) -> Vec<String> {
    let free: Vec<&String> = files
        .iter()
        .filter(|f| classes.iter().all(|c| !c.matcher.matches(f)))
        .copied()
        .collect();
    by_key(&free)
        .iter()
        .filter_map(|(key, files)| {
            let ids = shared(files.iter().copied(), specific);
            (!ids.is_empty()).then(|| row(key, &ids))
        })
        .collect()
}

fn binary_rows(files: &[String]) -> Vec<String> {
    let mut keys: Vec<String> = files
        .iter()
        .filter(|f| binary(f))
        .map(|f| crate::scan::key(f))
        .collect();
    keys.sort();
    keys.dedup();
    keys.into_iter()
        .map(|k| format!("{k}|-|-|binary asset"))
        .collect()
}

fn row(key: &str, ids: &[String]) -> String {
    format!("{key}|{}|-|-", ids.join(", "))
}

/// Types no check reaches, for the header comment.
fn unreached(files: &[&String], specific: &[Reach<'_>]) -> Vec<String> {
    let mut keys: Vec<String> = files
        .iter()
        .filter(|f| reaching(f, specific).is_empty())
        .map(|f| crate::scan::key(f))
        .collect();
    keys.sort();
    keys.dedup();
    keys
}

fn render(star: &[String], rows: &[String], unreached: &[String]) -> String {
    let open = if unreached.is_empty() {
        String::new()
    } else {
        format!(
            "#\n# No built-in check reads these types yet, so `pklith check` names them\n# until each gets checks or an exempt reason: {}.\n",
            unreached.join(", ")
        )
    };
    format!(
        "format 1\n\n# Seeded by `pklith seed --init` from the fragments this repository's\n# files switch on (`pklith detect`). It is yours now: pklith never rewrites it.\n{open}\n## types\ntype|checks|min|exempt\n*|{}|-|-\n{}\n",
        star.join(", "),
        rows.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::registry;
    use crate::registry::Error;

    /// A small mixed tree: rust, nix, shell, workflows, specs, a picture,
    /// and a LICENSE no built-in check reads.
    const TREE: [&str; 14] = [
        ".envrc",
        ".github/workflows/ci.yml",
        "Cargo.toml",
        "LICENSE",
        "README.md",
        "SPEC.md",
        "clippy.toml",
        "docs/x.yml",
        "flake.nix",
        "logo.png",
        "scripts/a.sh",
        "src/SPEC.md",
        "src/a.rs",
        "src/b.rs",
    ];

    const SEEDED: &str = "format 1

# Seeded by `pklith seed --init` from the fragments this repository's
# files switch on (`pklith detect`). It is yours now: pklith never rewrites it.
#
# No built-in check reads these types yet, so `pklith check` names them
# until each gets checks or an exempt reason: LICENSE.

## types
type|checks|min|exempt
*|trailing-whitespace, final-newline, line-endings, no-bom, no-merge-conflict, no-case-conflict, no-broken-symlinks, no-private-key, ripsecrets, typos, editorconfig-checker, no-local-paths|-|-
md|markdownlint|-|-
nix|nixfmt, statix, deadnix|-|-
path:**/SPEC.md|markdownlint, mth-fmt, mth-check, sherd-check, sherd-nav, sherd-budget|-|-
path:.envrc|shellcheck|-|-
path:.github/workflows/*.yml|actionlint, zizmor, yamllint|-|-
path:Cargo.toml|taplo, clippy, cargo-deployment-target, cargo-universal|-|-
path:clippy.toml|taplo, clippy|-|-
png|-|-|binary asset
rs|rustfmt, clippy, cargo-universal, sherd-check|-|-
sh|shellcheck, shfmt|-|-
yml|yamllint|-|-
";

    fn seeded() -> Result<(Vec<String>, String), Error> {
        let files: Vec<String> = TREE.iter().map(|f| (*f).to_owned()).collect();
        let (fragments, catalog) = (
            crate::catalog::builtin_fragments()?,
            crate::catalog::builtin()?,
        );
        let text = registry(
            &files,
            &crate::detect::active(&files, &fragments),
            &catalog,
            &[],
        );
        Ok((files, text))
    }

    /// Whole-tree checks go in `*`; a check reaching only some files of a
    /// type, or a file by name, becomes a `path:` class; binary types are
    /// exempt; a type no check reads is left for a person, and named.
    #[test]
    fn the_seeded_registry_follows_the_tree() -> Result<(), Error> {
        assert_eq!(seeded()?.1, SEEDED);
        Ok(())
    }

    /// Judged by cover, the seeded registry has no stale row, leaves only
    /// the named type a gap, and claims nothing its runner misses.
    #[test]
    fn cover_accepts_the_seeded_registry() -> Result<(), Error> {
        let (files, text) = seeded()?;
        let parsed = crate::registry::parse(&text)?;
        let coverage = crate::cover::judge(&files, &parsed);
        let gaps: Vec<&str> = coverage.gaps.iter().map(|g| g.key.as_str()).collect();
        assert_eq!((gaps, coverage.stale.len()), (vec!["LICENSE"], 0));
        let steps: Vec<crate::hook::Step> =
            crate::catalog::builtin()?.into_iter().map(step).collect();
        assert!(crate::cover::unbacked(&files, &parsed, &steps).is_empty());
        Ok(())
    }

    /// Binary files of one type, interleaved with another, still seed one
    /// row per type: a repeated row would make the registry unreadable.
    #[test]
    fn interleaved_binary_types_seed_one_row_each() -> Result<(), Error> {
        let files: Vec<String> = ["a.png", "b.gz", "c.png"].map(str::to_owned).into();
        let text = registry(&files, &[], &crate::catalog::builtin()?, &[]);
        assert!(
            text.ends_with("gz|-|-|binary asset\npng|-|-|binary asset\n"),
            "{text}"
        );
        crate::registry::parse(&text)?;
        Ok(())
    }

    /// The rows seed V5 writes for `.gitignore` and the root `flake.lock`.
    const KNOWN: [&str; 2] = [
        "\ngitignore|-|-|git configuration; hygiene steps only\n",
        "\npath:flake.lock|-|-|written by `nix flake lock`; hygiene steps only\n",
    ];

    /// A rust and nix tree with `.gitignore`, two lock files and a nested
    /// `flake.lock`, seeded from the built-in catalog.
    fn locked() -> Result<(Vec<String>, String), Error> {
        let files: Vec<String> = [
            ".gitignore",
            "Cargo.lock",
            "flake.lock",
            "flake.nix",
            "tests/fixtures/flake.lock",
        ]
        .map(str::to_owned)
        .into();
        let fragments = crate::catalog::builtin_fragments()?;
        let active = crate::detect::active(&files, &fragments);
        let text = registry(&files, &active, &crate::catalog::builtin()?, &[]);
        Ok((files, text))
    }

    /// Seed V5 (#104): `.gitignore` and the root `flake.lock` no check
    /// reads are seeded exempt with their reasons; another lock file
    /// (Cargo.lock) and a nested flake.lock stay gaps, and the header
    /// names only what is left.
    #[test]
    fn known_files_seed_their_exemptions() -> Result<(), Error> {
        let (files, text) = locked()?;
        assert!(KNOWN.iter().all(|row| text.contains(row)), "{text}");
        assert!(text.contains("exempt reason: lock.\n"), "{text}");
        let coverage = crate::cover::judge(&files, &crate::registry::parse(&text)?);
        let gaps: Vec<(&str, usize)> = coverage
            .gaps
            .iter()
            .map(|g| (g.key.as_str(), g.files.len()))
            .collect();
        assert_eq!((gaps, coverage.stale.len()), (vec![("lock", 2)], 0));
        Ok(())
    }

    /// A catalog of one check, `lock-budget`, reaching `flake.lock`.
    fn budget() -> crate::catalog::Check {
        crate::catalog::Check {
            line: 0,
            id: "lock-budget".to_owned(),
            category: crate::catalog::Category::Lint,
            nix: String::new(),
            globs: vec!["flake.lock".to_owned()],
            check: "true".to_owned(),
            fix: None,
            env: Vec::new(),
            msg: "m".to_owned(),
        }
    }

    /// The fragment `budget` belongs to, switched on by `flake.lock`.
    fn lock_fragment() -> Result<crate::catalog::Fragment, crate::scan::Error> {
        Ok(crate::catalog::Fragment {
            line: 0,
            id: "lock".to_owned(),
            always: false,
            triggers: crate::scan::Globs::new(&["flake.lock".to_owned()])?,
            checks: vec!["lock-budget".to_owned()],
            seed: Vec::new(),
        })
    }

    /// Seed V5: a check that reaches `flake.lock` replaces the exemption,
    /// so a later catalog check needs no change to seed.
    #[test]
    fn a_lock_check_supersedes_it() -> Result<(), Box<dyn std::error::Error>> {
        let files: Vec<String> = ["flake.lock", "flake.nix"].map(str::to_owned).into();
        let fragment = lock_fragment()?;
        let text = registry(&files, &[&fragment], &[budget()], &[]);
        assert!(
            text.contains("\npath:flake.lock|lock-budget|-|-\n"),
            "{text}"
        );
        assert!(!text.contains("nix flake lock"), "{text}");
        Ok(())
    }

    /// The hk step gen would emit for a check: its runner globs.
    fn step(check: crate::catalog::Check) -> crate::hook::Step {
        crate::hook::Step {
            id: check.id,
            globs: check.globs,
            exclude: Vec::new(),
            checks: true,
        }
    }
}
