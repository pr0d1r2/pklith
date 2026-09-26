//! Render a coverage verdict (`src/report/SPEC.md`).

use crate::cover::{Coverage, Gap, Stale};

/// The verdict as text for a person or a hook log: one line per finding,
/// each saying what to do. Empty when the verdict passes, so a passing check
/// is silent (V3). Findings are in coverage order, which is registry and
/// scan order, so the text is stable (V1).
#[must_use]
pub fn text(coverage: &Coverage) -> String {
    let gaps = coverage.gaps.iter().map(gap_line);
    let stale = coverage.stale.iter().map(stale_line);
    gaps.chain(stale).map(|line| line + "\n").collect()
}

fn gap_line(gap: &Gap) -> String {
    let (key, n, files) = (&gap.key, gap.files.len(), gap.files.join(", "));
    let noun = if n == 1 { "file" } else { "files" };
    format!(
        "gap: `{key}` has no row in .pklith ({n} {noun}: {files}); add its checks, or an exemption reason"
    )
}

fn stale_line(stale: &Stale) -> String {
    let (key, line) = (&stale.key, stale.line);
    format!("stale: `{key}` (.pklith:{line}) matches no file; remove the row, or add the files")
}

#[cfg(test)]
mod tests {
    use super::text;
    use crate::cover::{Coverage, judge};
    use crate::registry::{Error, parse};

    #[test]
    fn a_passing_verdict_is_silent() {
        assert_eq!(text(&Coverage::default()), "");
    }

    const REGISTRY: &str = "format 1\n## types\ntype|checks|min|exempt\nrb|rubocop|-|-\n";

    const WANT: &str = "gap: `gz` has no row in .pklith (2 files: a.gz, b.gz); add its checks, or an exemption reason\n\
                        gap: `py` has no row in .pklith (1 file: x.py); add its checks, or an exemption reason\n\
                        stale: `rb` (.pklith:4) matches no file; remove the row, or add the files\n";

    /// Each finding names what is wrong and what to do about it.
    #[test]
    fn each_finding_says_what_to_do() -> Result<(), Error> {
        let files = ["a.gz", "b.gz", "x.py"].map(str::to_owned);
        assert_eq!(text(&judge(&files, &parse(REGISTRY)?)), WANT);
        Ok(())
    }
}
