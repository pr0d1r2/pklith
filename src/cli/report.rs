//! `pkli report [--format text|json|md]`: the coverage matrix and
//! findings (src/report), always printed.

use super::{Options, Outcome, USAGE, data, exit, options};
use std::path::Path;

type Render = fn(&crate::report::Verdict<'_>) -> String;

/// The report on stdout (V3), exit 0 whatever it finds; 2 on a usage or
/// read error. Takes `--root` and `--registry` as check does.
pub(super) fn run(args: &[String], cwd: &Path) -> Outcome {
    let mut rest = args.to_vec();
    let format = super::check::take_value(&mut rest, "--format");
    let render = format.ok().and_then(|f| renderer(f.as_deref()));
    let (Some(render), Some(opts)) = (render, options(&rest)) else {
        return exit(2, USAGE);
    };
    match rendered(render, opts, cwd) {
        Ok(text) => data(text),
        Err(message) => exit(2, format!("pkli report: {message}\n")),
    }
}

fn rendered(render: Render, opts: Options, cwd: &Path) -> Result<String, String> {
    let (coverage, unbacked, universal) = super::check::judge(opts, None, cwd)?;
    let v = crate::report::Verdict {
        coverage: &coverage,
        unbacked: &unbacked,
        universal: &universal,
    };
    Ok(render(&v))
}

/// The renderer a `--format` names; text when none is given.
fn renderer(format: Option<&str>) -> Option<Render> {
    match format {
        None | Some("text") => Some(crate::report::matrix),
        Some("json") => Some(crate::report::json::render),
        Some("md") => Some(crate::report::md::render),
        Some(_) => None,
    }
}
