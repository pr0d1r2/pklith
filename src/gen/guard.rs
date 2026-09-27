//! The programs a step's command runs, so its guard can report a missing
//! one as missing (root V1) instead of letting the step fail as a finding.

/// Words that make a clause's first word something other than a program:
/// shell keywords and builtins, which are always there.
const NOT_PROGRAMS: [&str; 20] = [
    "for", "while", "until", "if", "case", "in", "done", "fi", "esac", "exit", "true", "false",
    "echo", "printf", "test", "[", "cd", "set", "export", "return",
];

/// Words after which a new simple command starts.
const OPENERS: [&str; 5] = ["do", "then", "else", "{", "!"];

/// cargo subcommands built into cargo itself; any other `cargo X` runs the
/// `cargo-X` binary, which a nix package must provide.
const CARGO_OWN: [&str; 14] = [
    "build", "check", "clean", "doc", "run", "test", "bench", "update", "install", "metadata",
    "tree", "fetch", "package", "rustc",
];

/// Every program `command` runs, first occurrence first: the first word of
/// each simple command, the program `xargs` runs, and `cargo-X` for a
/// cargo subcommand that is not built in.
#[must_use]
pub fn programs(command: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for clause in clauses(command) {
        for program in program(&clause) {
            if !found.contains(&program) {
                found.push(program);
            }
        }
    }
    found
}

/// The command split into simple commands, each as its words: a new one
/// starts at `|`, `;`, `&`, a paren, or an opener such as `do`, unless
/// the separator is quoted.
fn clauses(command: &str) -> Vec<Vec<String>> {
    broken(command).split('\0').flat_map(opened).collect()
}

/// `command` with each separator the shell acts on turned into `\0`; one
/// inside quotes is text, as in `grep -E 'a|b'`, and stays.
fn broken(command: &str) -> String {
    let mut quote: Option<char> = None;
    let mut mark = |c: char| match (quote, c) {
        (None, '\'' | '"') => {
            quote = Some(c);
            c
        }
        (Some(q), _) if c == q => {
            quote = None;
            c
        }
        (None, _) if "|;&()".contains(c) => '\0',
        _ => c,
    };
    command.chars().map(&mut mark).collect()
}

/// One separator-free stretch of a command, split again at openers.
fn opened(stretch: &str) -> Vec<Vec<String>> {
    let mut clauses = vec![Vec::new()];
    for word in stretch.split_whitespace() {
        if OPENERS.contains(&word) {
            clauses.push(Vec::new());
        } else if let Some(clause) = clauses.last_mut() {
            clause.push(word.to_owned());
        }
    }
    clauses
}

/// The programs one simple command runs.
fn program(clause: &[String]) -> Vec<String> {
    let mut words = clause.iter().skip_while(|w| assignment(w));
    let Some(first) = words.next() else {
        return Vec::new();
    };
    let rest: Vec<String> = words.cloned().collect();
    match first.as_str() {
        w if NOT_PROGRAMS.contains(&w) => Vec::new(),
        "xargs" => program(&without_flags(&rest)),
        "cargo" => cargo(&rest),
        w if plain(w) => vec![w.to_owned()],
        _ => Vec::new(),
    }
}

/// A word the shell runs as it stands: no variable, quote or glob to
/// expand first, so `command -v` can look it up before the step runs.
fn plain(word: &str) -> bool {
    word.chars()
        .all(|c| c.is_ascii_alphanumeric() || "_./+-".contains(c))
}

/// `KEY=value` before a command sets its environment.
fn assignment(word: &str) -> bool {
    word.split_once('=').is_some_and(|(key, _)| {
        !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

fn without_flags(words: &[String]) -> Vec<String> {
    words
        .iter()
        .skip_while(|w| w.starts_with('-'))
        .cloned()
        .collect()
}

fn cargo(rest: &[String]) -> Vec<String> {
    let cargo = "cargo".to_owned();
    match rest.first() {
        Some(sub) if !sub.starts_with('-') && !CARGO_OWN.contains(&sub.as_str()) => {
            vec![cargo, format!("cargo-{sub}")]
        }
        _ => vec![cargo],
    }
}

#[cfg(test)]
mod tests {
    use super::programs;

    const CASES: [(&str, &[&str]); 10] = [
        ("! grep -E 'a|/b/|c' {{files}} | grep -v \"x;y\"", &["grep"]),
        ("typos --force-exclude {{files}}", &["typos"]),
        (
            "(for f in {{files}}; do mth fmt --check \"$f\" || exit 1; done)",
            &["mth"],
        ),
        (
            "git ls-files -z | xargs -0 hk util check-symlinks",
            &["git", "hk"],
        ),
        (
            "git ls-files -z '*.md' | xargs -0 lychee --offline",
            &["git", "lychee"],
        ),
        ("cargo fmt --check", &["cargo", "cargo-fmt"]),
        ("cargo test --doc -q", &["cargo"]),
        ("RUSTDOCFLAGS=-D cargo doc --no-deps", &["cargo"]),
        (
            "scripts/no-large-files.sh {{files}}",
            &["scripts/no-large-files.sh"],
        ),
        ("\"$RUNNER\" check | xargs -0 'my tool'", &[]),
    ];

    /// Every program a command runs is found, past loops, pipes, `xargs`,
    /// env assignments and cargo subcommands; a word the shell expands or
    /// unquotes first cannot be named before it runs, so it is not guarded.
    #[test]
    fn programs_are_found_where_the_shell_runs_them() {
        for (command, want) in CASES {
            assert_eq!(programs(command), want, "{command}");
        }
    }
}
