//! Read the effective hk configuration (`src/hook/SPEC.md`): which steps
//! exist and which files each sees, as `pkl` evaluates `hk.pkl`.

use std::path::Path;

/// One step of the `check` hook, which holds every step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// Step id: the check id for a generated step.
    pub id: String,
    /// The globs the runner matches; empty means the whole tree.
    pub globs: Vec<String>,
    /// Globs the runner skips.
    pub exclude: Vec<String>,
    /// Whether the step has a check command at all.
    pub checks: bool,
}

/// Why the steps could not be read.
#[derive(Debug)]
pub enum Error {
    /// `pkl` is missing, or `hk.pkl` did not evaluate (V1).
    Pkl(crate::proc::Error),
    /// `hk.pkl` evaluated to no steps (V2).
    NoSteps,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pkl(e) => write!(f, "{e}"),
            Self::NoSteps => f.write_str("hk.pkl evaluates to no steps in its `check` hook"),
        }
    }
}

impl std::error::Error for Error {}

/// One line per step: id, globs, excludes (list items joined by U+001F) and
/// whether it checks, tab-separated. Asking `pkl` for text keeps pklith
/// free of a JSON parser (hook §C: ⊥ own Pkl parser).
const STEPS: &str = r#"hooks["check"].steps.toMap().entries.map((e) -> let (s = e.value) List(e.key, (if (s.glob is String) List(s.glob) else s.glob ?? List()).join("\u{1F}"), (if (s.exclude is String) List(s.exclude) else s.exclude ?? List()).join("\u{1F}"), if (s.check != null || s.check_diff != null) "1" else "0").join("\t")).join("\n")"#;

/// The `check` hook's steps as `pkl eval` sees `root/hk.pkl`.
///
/// # Errors
///
/// [`Error::Pkl`] when `pkl` is missing or `hk.pkl` does not evaluate (V1),
/// [`Error::NoSteps`] when it evaluates to none (V2): never an empty list
/// read as "nothing to check".
pub fn steps(root: &Path) -> Result<Vec<Step>, Error> {
    let mut cmd = crate::proc::command("pkl", root);
    cmd.args(["eval", "-x", STEPS, "hk.pkl"]);
    let text =
        String::from_utf8_lossy(&crate::proc::output(&mut cmd).map_err(Error::Pkl)?).into_owned();
    let steps: Vec<Step> = text.lines().filter(|l| !l.is_empty()).map(step).collect();
    if steps.is_empty() {
        return Err(Error::NoSteps);
    }
    Ok(steps)
}

fn step(line: &str) -> Step {
    let mut fields = line.split('\t');
    let mut next = || fields.next().unwrap_or_default();
    let (id, globs, exclude, checks) =
        (next().to_owned(), list(next()), list(next()), next() == "1");
    Step {
        id,
        globs,
        exclude,
        checks,
    }
}

fn list(field: &str) -> Vec<String> {
    field
        .split('\u{1F}')
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Error, steps};
    use std::path::PathBuf;

    const SCHEMA: &str = include_str!("../../pkl/Config.pkl");

    const HK: &str = r#"amends "pkl/Config.pkl"
hooks {
  ["check"] {
    steps {
      ["a"] { glob = "*.rs"; check = "a" }
      ["b"] { glob = List("x", "y"); exclude = List("z"); check_diff = "b" }
      ["c"] { fix = "c" }
    }
  }
}
"#;

    fn dir(name: &str, hk: Option<&str>) -> std::io::Result<PathBuf> {
        let dir = std::env::temp_dir().join(format!("pklith-hook-{}-{name}", std::process::id()));
        std::fs::create_dir_all(dir.join("pkl"))?;
        std::fs::write(dir.join("pkl/Config.pkl"), SCHEMA)?;
        if let Some(text) = hk {
            std::fs::write(dir.join("hk.pkl"), text)?;
        }
        Ok(dir)
    }

    fn show(step: &super::Step) -> String {
        format!(
            "{} {:?} {:?} {}",
            step.id, step.globs, step.exclude, step.checks
        )
    }

    /// A string glob, a list with excludes, and a step with only a fix
    /// (which checks nothing), as `pkl` evaluates them.
    #[test]
    fn steps_come_back_as_pkl_evaluates_them() -> Result<(), Box<dyn std::error::Error>> {
        let dir = dir("steps", Some(HK))?;
        let got: Vec<String> = steps(&dir)?.iter().map(show).collect();
        assert_eq!(
            got,
            [
                r#"a ["*.rs"] [] true"#,
                r#"b ["x", "y"] ["z"] true"#,
                "c [] [] false"
            ]
        );
        Ok(std::fs::remove_dir_all(dir)?)
    }

    const EMPTY: &str = "amends \"pkl/Config.pkl\"\nhooks { [\"check\"] { steps {} } }\n";

    /// V1: an hk.pkl that does not evaluate is an error; V2: one with no
    /// steps is an error too, never "nothing to check".
    #[test]
    fn no_config_or_no_steps_is_an_error() -> Result<(), Box<dyn std::error::Error>> {
        let missing = dir("missing", None)?;
        let err = steps(&missing);
        assert!(matches!(&err, Err(Error::Pkl(_))));
        assert!(
            err.err()
                .is_some_and(|e| e.to_string().starts_with("`pkl eval -x "))
        );
        let empty = dir("empty", Some(EMPTY))?;
        assert_eq!(
            steps(&empty).err().map(|e| e.to_string()).as_deref(),
            Some("hk.pkl evaluates to no steps in its `check` hook")
        );
        std::fs::remove_dir_all(missing)?;
        Ok(std::fs::remove_dir_all(empty)?)
    }
}
