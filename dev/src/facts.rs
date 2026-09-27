//! The facts the README badges and the LLM disclaimer quote, each read from
//! the file that owns it (dev V1). Pure functions over text: the caller
//! reads the repository and hands the text in, so every rule is testable
//! without one.

/// A `key = "value"` line's value, from a TOML file read as text.
#[must_use]
pub fn quoted(toml: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = \"");
    toml.lines()
        .find_map(|l| l.strip_prefix(&prefix)?.strip_suffix('"'))
        .map(str::to_owned)
}

/// How many entries the `[dependencies]` table of a Cargo.toml holds.
#[must_use]
pub fn dependencies(toml: &str) -> usize {
    toml.lines()
        .skip_while(|l| l.trim() != "[dependencies]")
        .skip(1)
        .take_while(|l| !l.starts_with('['))
        .filter(|l| l.split('=').next().is_some_and(is_key))
        .count()
}

fn is_key(s: &str) -> bool {
    let key = s.trim();
    !key.is_empty()
        && !key.starts_with('#')
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

/// The coverage floor `.coverage` records, trailing zeros dropped:
/// `lines 100.00` is `100`, `lines 92.30` is `92.3`.
#[must_use]
pub fn floor(coverage: &str) -> Option<String> {
    let value = coverage.lines().find_map(|l| l.strip_prefix("lines "))?;
    let value = value.trim();
    let trimmed = if value.contains('.') {
        value.trim_end_matches('0').trim_end_matches('.')
    } else {
        value
    };
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

/// Rows of a spec table across `specs`: lines starting with `prefix`,
/// digits, then `sep`. Every node numbers its own rows from 1, so rows are
/// counted, never distinct ids.
#[must_use]
pub fn rows(specs: &[String], prefix: char, sep: char) -> usize {
    let row = |l: &&str| {
        l.strip_prefix(prefix).is_some_and(|rest| {
            let digits = rest.trim_start_matches(|c: char| c.is_ascii_digit());
            digits.len() < rest.len() && digits.starts_with(sep)
        })
    };
    specs.iter().map(|s| s.lines().filter(row).count()).sum()
}

/// The nixpkgs the flake builds with, from `flake.lock`.
#[derive(Debug, PartialEq, Eq)]
pub struct Nixpkgs {
    /// The release series, `26.05` from `nixos-26.05`.
    pub series: String,
    /// The locked commit, 7 characters.
    pub rev: String,
    /// The commit date, `YYYY-MM-DD`.
    pub day: String,
}

/// The `nixpkgs` node of a flake lock. Its header is `"nixpkgs": {`; an
/// input that follows it is `"nixpkgs": [`, which a line match on the
/// name alone would enter by mistake (sherd dev B1).
#[must_use]
pub fn nixpkgs(lock: &str) -> Option<Nixpkgs> {
    let node: Vec<&str> = lock
        .lines()
        .skip_while(|l| l.trim() != "\"nixpkgs\": {")
        .take_while(|l| !l.trim_start().starts_with('}') || l.starts_with("      "))
        .collect();
    let field = |name: &str| {
        let key = format!("\"{name}\":");
        node.iter()
            .find_map(|l| l.trim().strip_prefix(&key))
            .map(|v| v.trim().trim_end_matches(',').trim_matches('"').to_owned())
    };
    let series = field("ref")?.strip_prefix("nixos-")?.to_owned();
    let rev = field("rev")?.get(..7)?.to_owned();
    let day = day(field("lastModified")?.parse().ok()?)?;
    Some(Nixpkgs { series, rev, day })
}

/// `YYYY-MM-DD` for a Unix time, in UTC.
#[must_use]
pub fn day(secs: i64) -> Option<String> {
    // Howard Hinnant's days-to-civil, for the proleptic Gregorian calendar.
    let z = secs.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    (secs >= 0).then(|| format!("{y:04}-{m:02}-{d:02}"))
}

/// A platform badge: architecture and operating system.
pub type Platform = (&'static str, &'static str);

/// The platforms CI's matrix builds on (dev V3), in badge order. Not the
/// flake's systems: a system no runner builds is not a claim CI backs.
///
/// # Errors
///
/// A runner this table does not know, named: guessing its platform would
/// badge something nothing checked.
pub fn platforms(ci: &str) -> Result<Vec<Platform>, String> {
    let list = ci
        .lines()
        .find_map(|l| l.trim().strip_prefix("os: ["))
        .and_then(|l| l.strip_suffix(']'))
        .ok_or("ci.yml has no `os: [...]` matrix")?;
    let mut found = Vec::new();
    for os in list.split(',').map(str::trim) {
        found.extend(runner(os).ok_or(format!("ci.yml runner `{os}` has no known platform"))?);
    }
    Ok(ORDER.into_iter().filter(|p| found.contains(p)).collect())
}

/// Badge order, whatever order the matrix lists runners in.
const ORDER: [Platform; 5] = [
    ("intel", "linux"),
    ("amd", "linux"),
    ("arm", "linux"),
    ("intel", "macos"),
    ("arm", "macos"),
];

fn runner(os: &str) -> Option<Vec<Platform>> {
    match os {
        o if o.starts_with("ubuntu") && o.ends_with("-arm") => Some(vec![("arm", "linux")]),
        o if o.starts_with("ubuntu") => Some(vec![("intel", "linux"), ("amd", "linux")]),
        "macos-13" => Some(vec![("intel", "macos")]),
        o if o.starts_with("macos") => Some(vec![("arm", "macos")]),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
