//! `pkli seed --init`: a first `.pklith` from the active fragments (seed
//! T3). The repository owns it afterwards.

use crate::catalog::{Check, Fragment};
use crate::scan::Globs;

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
/// of a type; binary types are exempt. Types no check reaches get no row,
/// so `pkli check` names them until someone decides (root V5).
#[must_use]
pub fn registry(files: &[String], active: &[&Fragment], catalog: &[Check]) -> String {
    let (universal, rows) = rows(files, active, catalog);
    let (_, specific) = split(active, catalog);
    let text: Vec<&String> = files.iter().filter(|f| !binary(f)).collect();
    render(&universal, &rows, &unreached(&text, &specific))
}

/// What `registry` writes, before rendering: the `*` checks, and the type
/// rows sorted bytewise (root V3). Also what `pkli check` suggests for a
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
            .map(|(glob, ids)| row(&format!("path:{glob}"), ids)),
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

/// `path:` classes: where a check reaches only some files of a type, its
/// matching glob becomes a class, claiming what every file it matches
/// shares.
fn classes(files: &[&String], specific: &[Reach<'_>]) -> Vec<(String, Vec<String>)> {
    let mut globs: Vec<String> = Vec::new();
    for file in files {
        if let Some(glob) = narrow(file, files, specific).filter(|g| !globs.contains(g)) {
            globs.push(glob);
        }
    }
    globs
        .into_iter()
        .map(|g| {
            let shared = shared(files.iter().filter(|f| matches(&g, f)).copied(), specific);
            (g, shared)
        })
        .collect()
}

/// The glob of the first check reaching `file` that misses another file
/// of its type, or reaches it by name (`Cargo.toml`): a named file is its
/// own class, so the next file of its type is not claimed for that check.
fn narrow(file: &str, files: &[&String], specific: &[Reach<'_>]) -> Option<String> {
    let key = crate::scan::key(file);
    let peers: Vec<&&String> = files
        .iter()
        .filter(|f| crate::scan::key(f) == key)
        .collect();
    specific.iter().find_map(|r| {
        let glob = r.glob(file)?;
        let named = !glob.contains(['*', '?', '[', '{']);
        (named || peers.iter().any(|p| r.glob(p).is_none())).then(|| glob.to_owned())
    })
}

fn matches(glob: &str, file: &str) -> bool {
    Globs::new(&[glob.to_owned()]).is_ok_and(|g| g.matches(file))
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
fn type_rows(
    files: &[&String],
    specific: &[Reach<'_>],
    classes: &[(String, Vec<String>)],
) -> Vec<String> {
    let free: Vec<&String> = files
        .iter()
        .filter(|f| classes.iter().all(|(g, _)| !matches(g, f)))
        .copied()
        .collect();
    let mut keys: Vec<String> = free.iter().map(|f| crate::scan::key(f)).collect();
    keys.sort();
    keys.dedup();
    keys.iter()
        .filter_map(|k| type_row(k, &free, specific))
        .collect()
}

/// The row for type `key` over `free` files, if they share any check.
fn type_row(key: &str, free: &[&String], specific: &[Reach<'_>]) -> Option<String> {
    let of_key = free.iter().filter(|f| crate::scan::key(f) == key).copied();
    let ids = shared(of_key, specific);
    (!ids.is_empty()).then(|| row(key, &ids))
}

fn binary_rows(files: &[String]) -> Vec<String> {
    let mut keys: Vec<String> = files
        .iter()
        .filter(|f| binary(f))
        .map(|f| crate::scan::key(f))
        .collect();
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
            "#\n# No built-in check reads these types yet, so `pkli check` names them\n# until each gets checks or an exempt reason: {}.\n",
            unreached.join(", ")
        )
    };
    format!(
        "format 1\n\n# Seeded by `pkli seed --init` from the fragments this repository's\n# files switch on (`pkli detect`). It is yours now: pkli never rewrites it.\n{open}\n## types\ntype|checks|min|exempt\n*|{}|-|-\n{}\n",
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

# Seeded by `pkli seed --init` from the fragments this repository's
# files switch on (`pkli detect`). It is yours now: pkli never rewrites it.
#
# No built-in check reads these types yet, so `pkli check` names them
# until each gets checks or an exempt reason: LICENSE.

## types
type|checks|min|exempt
*|trailing-whitespace, final-newline, line-endings, no-bom, no-merge-conflict, no-case-conflict, no-broken-symlinks, no-private-key, ripsecrets, typos, editorconfig-checker, no-local-paths|-|-
md|markdownlint|-|-
nix|nixfmt, statix, deadnix|-|-
path:**/SPEC.md|markdownlint, mth-fmt, mth-check, sherd-check, sherd-nav, sherd-budget|-|-
path:.envrc|shellcheck|-|-
path:.github/workflows/*.yml|actionlint, zizmor, yamllint|-|-
path:Cargo.toml|taplo, clippy|-|-
path:clippy.toml|taplo, clippy|-|-
png|-|-|binary asset
rs|rustfmt, clippy, sherd-check|-|-
sh|shellcheck, shfmt|-|-
yml|yamllint|-|-
";

    fn seeded() -> Result<(Vec<String>, String), Error> {
        let files: Vec<String> = TREE.iter().map(|f| (*f).to_owned()).collect();
        let (fragments, catalog) = (
            crate::catalog::builtin_fragments()?,
            crate::catalog::builtin()?,
        );
        let text = registry(&files, &crate::detect::active(&files, &fragments), &catalog);
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
