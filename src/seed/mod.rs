//! Repo-owned seed files (`src/seed/SPEC.md`): written once from the
//! active fragments, never rewritten (root V25).

use crate::catalog::Fragment;
use std::path::Path;

pub mod init;
pub mod templates;

use templates::Template;

/// One file to seed.
#[derive(Debug, PartialEq, Eq)]
pub struct Seed {
    /// Path from the repository root.
    pub path: &'static str,
    /// Its contents.
    pub text: String,
}

/// The files the `active` fragments seed, in fragment order: each named
/// file once, and one `.gitignore` gathering every fragment's part in
/// fragment order (V2). `name` fills a template's `{name}`.
///
/// # Errors
///
/// The first seed id no template carries, with the fragment naming it.
pub fn files(active: &[&Fragment], name: &str) -> Result<Vec<Seed>, String> {
    let (mut seeds, mut ignore) = (Vec::new(), String::new());
    for fragment in active {
        for id in &fragment.seed {
            let template = templates::named(id)
                .ok_or_else(|| format!("fragment `{}` seeds unknown `{id}`", fragment.id))?;
            add(&mut seeds, &mut ignore, template, name);
        }
    }
    if !ignore.is_empty() {
        seeds.push(Seed {
            path: ".gitignore",
            text: ignore,
        });
    }
    Ok(seeds)
}

/// One template's part: a file the first time its path is named, or
/// `.gitignore` lines.
fn add(seeds: &mut Vec<Seed>, ignore: &mut String, template: &Template, name: &str) {
    match template {
        Template::File(path, text) if seeds.iter().all(|s| s.path != *path) => {
            let text = text.replace("{name}", name);
            seeds.push(Seed { path, text });
        }
        Template::File(..) => {}
        Template::Ignore(lines) => ignore.push_str(lines),
    }
}

/// Write each seed that does not exist yet under `root`; one that does is
/// left exactly as the repository has it (V1). The paths written, in
/// order.
///
/// # Errors
///
/// The first path that could not be written, and why.
pub fn write(root: &Path, seeds: &[Seed]) -> Result<Vec<&'static str>, String> {
    let mut written = Vec::new();
    for seed in seeds {
        let path = root.join(seed.path);
        if path.symlink_metadata().is_ok() {
            continue;
        }
        std::fs::write(&path, &seed.text)
            .map_err(|e| format!("cannot write {}: {e}", seed.path))?;
        written.push(seed.path);
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::{Seed, files, write};
    use crate::catalog::Fragment;

    fn fragments(ids: &[&str]) -> Vec<Fragment> {
        let all = crate::catalog::builtin_fragments().unwrap_or_default();
        all.into_iter()
            .filter(|f| ids.contains(&f.id.as_str()))
            .collect()
    }

    fn text<'a>(seeds: &'a [Seed], path: &str) -> Option<&'a str> {
        seeds
            .iter()
            .find(|s| s.path == path)
            .map(|s| s.text.as_str())
    }

    fn seeded(ids: &[&str]) -> Result<Vec<Seed>, String> {
        let owned = fragments(ids);
        let active: Vec<&Fragment> = owned.iter().collect();
        files(&active, "demo")
    }

    /// Base seeds its files, filling the repository name; nix adds `.envrc`
    /// and a `.gitignore`.
    #[test]
    fn fragments_seed_their_files() -> Result<(), String> {
        let seeds = seeded(&["base", "nix", "rust"])?;
        let paths: Vec<&str> = seeds.iter().map(|s| s.path).collect();
        let want = [
            ".editorconfig",
            ".gitattributes",
            "README.md",
            ".envrc",
            ".gitignore",
        ];
        assert_eq!(paths, want);
        assert_eq!(text(&seeds, "README.md"), Some("# demo\n"));
        Ok(())
    }

    /// V2: one `.gitignore` gathers every fragment's part, in fragment
    /// order.
    #[test]
    fn gitignore_gathers_parts_in_fragment_order() -> Result<(), String> {
        let seeds = seeded(&["nix", "rust"])?;
        assert_eq!(
            text(&seeds, ".gitignore"),
            Some(".direnv/\nresult\nresult-*\n/target\n")
        );
        Ok(())
    }

    /// Every seed id a built-in fragment names has a template.
    #[test]
    fn every_builtin_seed_id_has_a_template() -> Result<(), String> {
        let all = crate::catalog::builtin_fragments().map_err(|e| e.to_string())?;
        let ids: Vec<&str> = all.iter().map(|f| f.id.as_str()).collect();
        seeded(&ids)?;
        Ok(())
    }

    /// V1 (root V25): an existing file is never rewritten; only missing
    /// ones are written, and they are what write returns.
    #[test]
    fn existing_files_are_left_alone() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir().join(format!("pklith-seed-{}", std::process::id()));
        std::fs::create_dir_all(&dir)?;
        std::fs::write(dir.join("README.md"), "mine\n")?;
        let written = write(&dir, &seeded(&["base"])?)?;
        assert_eq!(written, [".editorconfig", ".gitattributes"]);
        assert_eq!(std::fs::read_to_string(dir.join("README.md"))?, "mine\n");
        assert_eq!(write(&dir, &seeded(&["base"])?)?, Vec::<&str>::new());
        Ok(std::fs::remove_dir_all(dir)?)
    }
    /// A fragment naming a seed pkli lacks is an error naming both; a seed
    /// that cannot be written names its path.
    #[test]
    fn unknown_seeds_and_failed_writes_are_errors() -> Result<(), Box<dyn std::error::Error>> {
        let text = "format 1\n## fragments\nfragment|triggers|checks|seed\nx|always|typos|nope\n";
        let local = crate::catalog::fragment::parse(&crate::registry::parse(text)?.fragments)?;
        let active: Vec<&Fragment> = local.iter().collect();
        assert_eq!(
            files(&active, "d"),
            Err("fragment `x` seeds unknown `nope`".to_owned())
        );
        let missing = std::env::temp_dir().join("pklith-seed-no-such-dir/deeper");
        let err = write(&missing, &seeded(&["base"])?)
            .err()
            .unwrap_or_default();
        assert!(err.starts_with("cannot write .editorconfig: "), "{err}");
        Ok(())
    }
}
