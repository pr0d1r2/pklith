//! The lefthook check set (migrate T1): the commands `lefthook.yml` runs,
//! less those `lefthook-local.yml` skips. Lefthook is read here and
//! nowhere else (migrate §C); only the shape a materialized lefthook.yml
//! has is read, and anything else is refused by name, never dropped (V3).

/// Every command a hook runs, as `(hook, command)`, in file order, less
/// the ones `local` marks `skip: true`.
///
/// # Errors
///
/// A hook that uses `scripts` or `jobs`, which migrate does not read: its
/// checks would be lost.
pub fn commands(main: &str, local: &str) -> Result<Vec<(String, String)>, String> {
    let skipped: Vec<Entry> = entries(local).into_iter().filter(|e| e.skip).collect();
    let skip = |e: &Entry| skipped.iter().any(|s| s.hook == e.hook && s.name == e.name);
    let mut found = Vec::new();
    for entry in entries(main).into_iter().filter(|e| !skip(e)) {
        if entry.key != "commands" {
            let (hook, key) = (entry.hook, entry.key);
            return Err(format!(
                "lefthook `{hook}` uses `{key}`, which migrate does not read; move them by hand"
            ));
        }
        found.push((entry.hook, entry.name));
    }
    Ok(found)
}

/// One named entry: a name at indent 4, under a key at indent 2, under a
/// hook at indent 0, and whether a `skip: true` follows it.
#[derive(Default)]
struct Entry {
    hook: String,
    key: String,
    name: String,
    skip: bool,
}

/// Every entry, in file order. Comments, the `---` marker and other
/// settings are passed over.
fn entries(text: &str) -> Vec<Entry> {
    let mut reader = Reader::default();
    let lines = text
        .lines()
        .filter(|l| !l.trim_start().starts_with('#') && l.trim() != "---");
    lines.for_each(|l| reader.line(l.len() - l.trim_start().len(), l.trim()));
    reader.found
}

#[derive(Default)]
struct Reader {
    hook: String,
    key: String,
    found: Vec<Entry>,
}

impl Reader {
    fn push(&mut self, name: &str) {
        let (hook, key, name) = (self.hook.clone(), self.key.clone(), name.to_owned());
        self.found.push(Entry {
            hook,
            key,
            name,
            skip: false,
        });
    }

    fn line(&mut self, depth: usize, body: &str) {
        match (depth, body.strip_suffix(':')) {
            (0, Some(name)) => name.clone_into(&mut self.hook),
            (2, Some(name)) => name.clone_into(&mut self.key),
            (4, Some(name)) => self.push(name),
            (6, None) if body == "skip: true" => {
                self.found
                    .last_mut()
                    .into_iter()
                    .for_each(|e| e.skip = true);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::commands;

    const MAIN: &str = "---\n\npre-commit:\n  parallel: true\n  commands:\n    gitleaks:\n      run: lefthook-gitleaks {staged_files}\n    markdownlint:\n      glob: \"*.md\"\n      run: x\n    markdownlint-agentic:\n      run: y\npre-push:\n  commands:\n    bats-unit:\n      run: z\n";

    const LOCAL: &str = "---\n# comment\npre-commit:\n  commands:\n    markdownlint:\n      timeout: 300s\n    markdownlint-agentic:\n      skip: true\n";

    /// T1: the commands per hook, less what lefthook-local.yml skips; a
    /// setting such as `timeout` changes nothing.
    #[test]
    fn commands_are_read_less_local_skips() {
        let found = commands(MAIN, LOCAL).unwrap_or_default();
        let pairs: Vec<String> = found.iter().map(|(h, c)| format!("{h}/{c}")).collect();
        assert_eq!(
            pairs,
            [
                "pre-commit/gitleaks",
                "pre-commit/markdownlint",
                "pre-push/bats-unit"
            ]
        );
    }

    /// V3: a hook using `scripts` is refused by name, not dropped.
    #[test]
    fn scripts_are_refused_by_name() {
        let main = "pre-commit:\n  scripts:\n    a.sh:\n      runner: bash\n";
        let want =
            "lefthook `pre-commit` uses `scripts`, which migrate does not read; move them by hand";
        assert_eq!(commands(main, "").err().as_deref(), Some(want));
    }
}
