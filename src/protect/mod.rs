//! Branch protection's required status contexts, from CI job names
//! (`src/protect/SPEC.md`). pklith's one network verb applies them through
//! `gh api`; deriving and diffing them, here, touches nothing.

pub mod workflow;

use workflow::Job;

/// The contexts `workflows` (`(path, text)`) report, sorted and unique:
/// every job of every workflow that runs on its own, and for a job calling
/// a reusable workflow in the repository, `caller / callee` for each of
/// the callee's contexts (protect T1).
///
/// # Errors
///
/// A job calling a workflow outside the repository, or a matrix pklith
/// cannot name exactly: a wrong required context would stay pending.
pub fn contexts(workflows: &[(String, String)]) -> Result<Vec<String>, String> {
    let mut found = Vec::new();
    for (path, text) in workflows
        .iter()
        .filter(|(_, t)| !workflow::reusable_only(t))
    {
        for job in workflow::jobs(text).map_err(|e| format!("{path}: {e}"))? {
            found.extend(job_contexts(&job, workflows).map_err(|e| format!("{path}: {e}"))?);
        }
    }
    found.sort();
    found.dedup();
    Ok(found)
}

fn job_contexts(job: &Job, workflows: &[(String, String)]) -> Result<Vec<String>, String> {
    let names = matrix_names(job);
    let Some(uses) = &job.uses else {
        return Ok(names);
    };
    let inner = callee(&job.name, uses, workflows)?;
    Ok(names
        .iter()
        .flat_map(|n| inner.iter().map(move |i| format!("{n} / {i}")))
        .collect())
}

/// The contexts of the reusable workflow `uses` names, from the
/// repository.
fn callee(job: &str, uses: &str, workflows: &[(String, String)]) -> Result<Vec<String>, String> {
    let unreadable =
        || format!("job `{job}` calls {uses}, which pklith cannot read; name its contexts by hand");
    let path = uses.strip_prefix("./").ok_or_else(unreadable)?;
    let untracked = || format!("job `{job}` calls {path}, which is not tracked");
    let (file, text) = workflows
        .iter()
        .find(|(p, _)| p == path)
        .ok_or_else(untracked)?;
    contexts(&[(file.clone(), text.replace("workflow_call", "push"))])
}

/// `job`, or with a matrix `job (v1, v2)` per combination, keys in the
/// order declared.
fn matrix_names(job: &Job) -> Vec<String> {
    if job.matrix.is_empty() {
        return vec![job.name.clone()];
    }
    combinations(&job.matrix)
        .iter()
        .map(|c| format!("{} ({})", job.name, c.join(", ")))
        .collect()
}

/// Every combination of the matrix values, keys in order.
fn combinations(matrix: &[(String, Vec<String>)]) -> Vec<Vec<String>> {
    let extend = |combos: Vec<Vec<String>>, (_, values): &(String, Vec<String>)| {
        let each = |c: &Vec<String>| {
            values
                .iter()
                .map(|v| [c.clone(), vec![v.clone()]].concat())
                .collect::<Vec<_>>()
        };
        combos.iter().flat_map(each).collect()
    };
    matrix.iter().fold(vec![Vec::new()], extend)
}

/// The request body setting `contexts` as the required checks.
#[must_use]
pub fn payload(contexts: &[String]) -> String {
    let quoted: Vec<String> = contexts
        .iter()
        .map(|c| format!("\"{}\"", c.replace('\\', "\\\\").replace('"', "\\\"")))
        .collect();
    format!("{{\"strict\":true,\"contexts\":[{}]}}\n", quoted.join(","))
}

/// What applying `derived` over `current` adds and removes (V2): a removed
/// context is a rename or a lost job, shown before anything is applied.
#[must_use]
pub fn diff(current: &[String], derived: &[String]) -> (Vec<String>, Vec<String>) {
    let added = derived
        .iter()
        .filter(|c| !current.contains(c))
        .cloned()
        .collect();
    let removed = current
        .iter()
        .filter(|c| !derived.contains(c))
        .cloned()
        .collect();
    (added, removed)
}

#[cfg(test)]
mod tests;
