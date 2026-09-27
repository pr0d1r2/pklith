//! `.unit-coverage.toml` and `.coverage-allowlist` (legacy T3, root R9):
//! every implementation file a rule selects has its test file, mapped the
//! way `lefthook-unit-coverage` maps it, so the compat entry's missing
//! lines match the legacy tool's (legacy V2).

use super::toml::{self, Table, Val};

/// The allowlist `lefthook-unit-coverage` reads when the config names none.
pub const ALLOWLIST: &str = ".coverage-allowlist";

/// One `[[rules]]` entry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Rule {
    /// `*.ext`: the implementation files' extension.
    pub glob: String,
    /// Directories the implementation files live under.
    pub dirs: Vec<String>,
    /// Directories under `dirs` to leave out.
    pub exclude: Vec<String>,
    /// Where the tests live.
    pub test_dir: String,
    /// `mirror` keeps the implementation's directory under `test_dir`,
    /// `flat` drops it.
    pub pattern: String,
    /// The tests' extension; the implementation's when empty.
    pub test_ext: String,
    /// Appended to the stem, `_spec` or `_test`.
    pub test_suffix: String,
    /// A prefix dropped from the implementation's directory.
    pub strip: String,
    /// Underscores in the stem become dashes; the raw stem still passes.
    pub normalize: bool,
}

/// A parsed `.unit-coverage.toml`.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Config {
    /// The allowlist file, relative to the root.
    pub allowlist: String,
    /// The rules, up to the first one without a `glob`: the legacy tool
    /// stopped reading there.
    pub rules: Vec<Rule>,
}

/// Parse a `.unit-coverage.toml`.
///
/// # Errors
///
/// Text outside the subset `.unit-coverage.toml` uses, by line.
pub fn config(text: &str) -> Result<Config, String> {
    let doc = toml::parse(text)?;
    let rules = doc
        .tables
        .iter()
        .filter(|(name, _)| name == "rules")
        .map_while(|(_, t)| rule(t))
        .collect();
    let allowlist = text_of(&doc.top, "allowlist").unwrap_or_else(|| ALLOWLIST.to_owned());
    Ok(Config { allowlist, rules })
}

fn rule(t: &Table) -> Option<Rule> {
    let text = |key| text_of(t, key).unwrap_or_default();
    Some(Rule {
        glob: text_of(t, "glob")?,
        dirs: list_of(t, "dirs"),
        exclude: list_of(t, "exclude"),
        test_dir: text("test_dir"),
        pattern: text_of(t, "pattern").unwrap_or_else(|| "mirror".to_owned()),
        test_ext: text("test_ext"),
        test_suffix: text("test_suffix"),
        strip: text("strip"),
        normalize: text("normalize") == "true",
    })
}

/// A scalar as `taplo get -o value` printed it.
fn text_of(t: &Table, key: &str) -> Option<String> {
    match toml::get(t, key)? {
        Val::Str(s) => Some(s.clone()),
        Val::Bool(b) => Some(b.to_string()),
        Val::List(_) => None,
    }
}

fn list_of(t: &Table, key: &str) -> Vec<String> {
    match toml::get(t, key) {
        Some(Val::List(items)) => items.clone(),
        _ => Vec::new(),
    }
}

/// The allowlisted paths: every line but blank ones and `#` comments,
/// kept verbatim.
#[must_use]
pub fn allowed(text: &str) -> Vec<&str> {
    text.split('\n')
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

impl Rule {
    fn ext(&self) -> &str {
        self.glob.strip_prefix("*.").unwrap_or(&self.glob)
    }

    /// Whether `rule` selects `file`: its extension, under a directory,
    /// not under an excluded one.
    #[must_use]
    pub fn selects(&self, file: &str) -> bool {
        let under = |dirs: &[String]| {
            dirs.iter()
                .any(|d| !d.is_empty() && file.starts_with(&format!("{d}/")))
        };
        file.ends_with(&format!(".{}", self.ext())) && under(&self.dirs) && !under(&self.exclude)
    }

    /// The test file `file` needs, and the raw-stem alternative a
    /// normalized rule also accepts.
    ///
    /// # Errors
    ///
    /// A pattern other than `mirror` or `flat`.
    pub fn specs(&self, file: &str) -> Result<(String, Option<String>), String> {
        let (dir, base) = file.rsplit_once('/').unwrap_or((".", file));
        let raw = base
            .strip_suffix(&format!(".{}", self.ext()))
            .unwrap_or(base);
        let norm = if self.normalize {
            raw.replace('_', "-")
        } else {
            raw.to_owned()
        };
        let spec = self.spec(dir, &norm)?;
        let alt = (norm != raw).then(|| self.spec(dir, raw)).transpose()?;
        Ok((spec, alt))
    }

    fn spec(&self, dir: &str, stem: &str) -> Result<String, String> {
        let ext = if self.test_ext.is_empty() {
            self.ext()
        } else {
            &self.test_ext
        };
        let (base, suffix) = (&self.test_dir, &self.test_suffix);
        match self.pattern.as_str() {
            "mirror" => Ok(format!(
                "{base}/{}/{stem}{suffix}.{ext}",
                self.relative(dir)
            )),
            "flat" => Ok(format!("{base}/{stem}{suffix}.{ext}")),
            other => Err(format!("unknown pattern '{other}'")),
        }
    }

    /// `dir` with `strip` dropped, as the legacy parameter expansion did.
    fn relative<'a>(&self, dir: &'a str) -> &'a str {
        if self.strip.is_empty() {
            return dir;
        }
        let rest = dir.strip_prefix(self.strip.as_str()).unwrap_or(dir);
        rest.strip_prefix('/').unwrap_or(rest)
    }

    /// `file -> spec` for every file the rule selects that is neither
    /// allowlisted nor has its test, per `exists`.
    ///
    /// # Errors
    ///
    /// A pattern other than `mirror` or `flat`, once a file needs a test.
    pub fn missing(
        &self,
        files: &[String],
        allow: &[&str],
        exists: impl Fn(&str) -> bool,
    ) -> Result<Vec<String>, String> {
        let mut missing = Vec::new();
        let due = files
            .iter()
            .filter(|f| self.selects(f) && !allow.contains(&f.as_str()));
        for file in due {
            let (spec, alt) = self.specs(file)?;
            if !exists(&spec) && !alt.as_deref().is_some_and(&exists) {
                missing.push(format!("{file} -> {spec}"));
            }
        }
        Ok(missing)
    }
}

#[cfg(test)]
mod tests;
