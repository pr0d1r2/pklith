//! Canonical `.pklith` text (registry T3): what `parse` reads back to the
//! same registry (V4). Sections in a fixed order, cells escaped, `-` for
//! empty; comments are not part of a registry, so they are not kept.

use super::{FORMAT, Registry, Row, Section, TypeRow};

/// `registry` as `.pklith` text; `format(parse(format(r)))` is
/// `format(r)` (V4).
#[must_use]
pub fn format(registry: &Registry) -> String {
    let types: Vec<Vec<String>> = registry.types.iter().map(type_cells).collect();
    let rows = |rows: &[Row]| rows.iter().map(|r| r.cells.clone()).collect::<Vec<_>>();
    let sections = [
        (Section::Types, types),
        (Section::Checks, rows(&registry.checks)),
        (Section::Rules, rows(&registry.rules)),
        (Section::Plural, rows(&registry.plural)),
        (Section::Fragments, rows(&registry.fragments)),
    ];
    let body: String = sections
        .iter()
        .filter(|(_, r)| !r.is_empty())
        .map(|(s, r)| section(*s, r))
        .collect();
    format!("format {FORMAT}\n{body}")
}

fn type_cells(row: &TypeRow) -> Vec<String> {
    let min = if row.min == 1 {
        String::new()
    } else {
        row.min.to_string()
    };
    let exempt = row.exempt.clone().unwrap_or_default();
    vec![row.key.clone(), row.checks.join(", "), min, exempt]
}

fn section(section: Section, rows: &[Vec<String>]) -> String {
    let name = section.name();
    let lines: String = rows.iter().map(|cells| line(cells) + "\n").collect();
    format!("\n## {name}\n{}\n{lines}", section.header())
}

fn line(cells: &[String]) -> String {
    let cell = |c: &String| {
        if c.is_empty() {
            "-".to_owned()
        } else {
            c.replace('|', "\\|")
        }
    };
    cells.iter().map(cell).collect::<Vec<_>>().join("|")
}

#[cfg(test)]
mod tests {
    use super::format;
    use crate::registry::{Error, parse};

    /// A tiny xorshift: property cases without a dependency (root §C).
    struct Rng(u64);

    impl Rng {
        fn pick<'a>(&mut self, from: &[&'a str]) -> &'a str {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            let len = u64::try_from(from.len()).unwrap_or(1).max(1);
            let at = usize::try_from(self.0 % len).unwrap_or(0);
            from.get(at).copied().unwrap_or_default()
        }
    }

    const TYPE_ROWS: [&str; 6] = [
        "rs|clippy, fmt|-|-",
        "png|-|-|binary asset",
        "*|typos|-|-",
        "path:docs/**|lint|-|-",
        "md|lint, fmt|2|-",
        "lock|-|-|generated \\| by tools",
    ];
    const CHECK_ROWS: [&str; 3] = [
        "lint|lint|-|*|lint {{files}} \\| tee|-|-|m",
        "fmt|format|x|**/*.rs|f|f --fix|A=1|msg",
        "typos|lint|typos|**/*|typos|-|-|miss",
    ];
    const OTHER: [&str; 3] = [
        "## rules\nid|kind|select|target|except\nr|exists|a/*.rb|b/{stem\\|snake}.rb|-\n",
        "## plural\nsingular|plural\nperson|people\n",
        "## fragments\nfragment|triggers|checks|seed\nf|always|lint|-\n",
    ];

    /// A registry text from `rng`: some types, some checks, maybe more.
    fn sample(rng: &mut Rng) -> String {
        let mut types: Vec<&str> = (0..4).map(|_| rng.pick(&TYPE_ROWS)).collect();
        types.sort_unstable();
        types.dedup();
        let checks = ["lint|lint|-|*|l|-|-|m", rng.pick(&CHECK_ROWS)];
        let (a, b) = (rng.pick(&OTHER), rng.pick(&OTHER));
        let extra = if a == b {
            a.to_owned()
        } else {
            format!("{a}{b}")
        };
        format!(
            "format 1\n## checks\nid|category|nix|glob|check|fix|env|msg\n{}\n## types\ntype|checks|min|exempt\n{}\n{extra}",
            checks.join("\n"),
            types.join("\n")
        )
    }

    /// V4: over generated registries, formatting is a fixed point and
    /// reads back to the same registry.
    #[test]
    fn format_round_trips() -> Result<(), Error> {
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
        for _ in 0..300 {
            let parsed = parse(&sample(&mut rng))?;
            let text = format(&parsed);
            assert_eq!(format(&parse(&text)?), text, "{text}");
        }
        Ok(())
    }
}
