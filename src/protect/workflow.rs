//! The status contexts one GitHub workflow reports (protect T1): a job's
//! `name`, or its id, and for a matrix job one context per combination,
//! `job (v1, v2)` in the order the keys are declared, since nothing
//! reports the bare `job` then. Read line by line, for the shape workflows
//! have; anything this cannot name exactly is refused, never guessed: a
//! required context nobody reports stays pending forever.

/// One job as read: its context name, the reusable workflow it calls, and
/// its matrix as `(key, values)` in declaration order.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Job {
    /// `name:`, or the job id.
    pub name: String,
    /// `uses:` of a job calling a reusable workflow.
    pub uses: Option<String>,
    /// `strategy.matrix` keys and values.
    pub matrix: Vec<(String, Vec<String>)>,
}

/// Whether the workflow only defines a reusable workflow (`workflow_call`),
/// which reports nothing under its own name.
#[must_use]
pub fn reusable_only(text: &str) -> bool {
    let on: Vec<&str> = section(text, "on").collect();
    let inline = text
        .lines()
        .find_map(|l| l.strip_prefix("on:"))
        .map(str::trim);
    let triggers = inline
        .filter(|i| !i.is_empty())
        .map_or(on.join(" "), str::to_owned);
    triggers.contains("workflow_call")
        && !["push", "pull_request", "merge_group"]
            .iter()
            .any(|t| triggers.contains(t))
}

/// The jobs under `jobs:`, in order.
///
/// # Errors
///
/// A matrix using `include` or `exclude`, which change the set of
/// combinations in ways a line reader cannot see.
pub fn jobs(text: &str) -> Result<Vec<Job>, String> {
    let mut reader = Reader::default();
    for line in section(text, "jobs").filter(|l| !l.trim_start().starts_with('#')) {
        reader.line(line.len() - line.trim_start().len(), line.trim())?;
    }
    Ok(reader.jobs)
}

/// Where the reader is: the jobs so far, and whether the current line
/// sits in the current job's `strategy` and its `matrix`.
#[derive(Default)]
struct Reader {
    jobs: Vec<Job>,
    strategy: bool,
    matrix: bool,
}

impl Reader {
    fn line(&mut self, depth: usize, body: &str) -> Result<(), String> {
        let (key, rest) = body
            .split_once(':')
            .map_or((body, None), |(k, v)| (k, Some(v.trim())));
        match (depth, rest) {
            (2, Some("")) => self.job(key),
            (4, Some(v)) => self.setting(key, v),
            (6, Some(_)) => self.matrix = self.strategy && key == "matrix",
            (8, Some(v)) if self.matrix => return self.axis(key, v),
            (10, None) if self.matrix => self.item(body),
            _ => {}
        }
        Ok(())
    }

    fn job(&mut self, id: &str) {
        self.jobs.push(Job {
            name: id.to_owned(),
            ..Job::default()
        });
        (self.strategy, self.matrix) = (false, false);
    }

    fn setting(&mut self, key: &str, v: &str) {
        (self.strategy, self.matrix) = (key == "strategy", false);
        let Some(job) = self.jobs.last_mut() else {
            return;
        };
        match key {
            "name" => job.name = value(v),
            "uses" => job.uses = Some(value(v)),
            _ => {}
        }
    }

    /// A matrix key: `key: [a, b]`, or `key:` with `- a` items below.
    fn axis(&mut self, key: &str, v: &str) -> Result<(), String> {
        if key == "include" || key == "exclude" {
            return Err(format!(
                "a matrix `{key}` changes its combinations in ways pkli cannot read; name the contexts by hand"
            ));
        }
        let values = v.strip_prefix('[').and_then(|l| l.strip_suffix(']'));
        let values = values
            .map(|l| l.split(',').map(value).collect())
            .unwrap_or_default();
        if let Some(job) = self.jobs.last_mut() {
            job.matrix.push((key.to_owned(), values));
        }
        Ok(())
    }

    fn item(&mut self, body: &str) {
        let axis = self.jobs.last_mut().and_then(|j| j.matrix.last_mut());
        if let (Some((_, values)), Some(v)) = (axis, body.strip_prefix("- ")) {
            values.push(value(v));
        }
    }
}

/// A YAML scalar: trimmed, one pair of quotes dropped.
fn value(raw: &str) -> String {
    let v = raw.trim();
    let unquoted = ['"', '\'']
        .iter()
        .find_map(|q| v.strip_prefix(*q)?.strip_suffix(*q));
    unquoted.unwrap_or(v).to_owned()
}

/// The lines of a top-level `section:` block.
fn section<'a>(text: &'a str, name: &'a str) -> impl Iterator<Item = &'a str> {
    let head = format!("{name}:");
    text.lines()
        .skip_while(move |l| l.trim_end() != head)
        .skip(1)
        .take_while(|l| l.is_empty() || l.starts_with(' ') || l.starts_with('#'))
}
