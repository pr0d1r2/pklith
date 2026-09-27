//! Root V29: a step's tool comes from a nix package, never fetched when the
//! step runs. This recognises the fetchers a catalog row could call.

/// Commands that download what they run, as the words that invoke them.
const FETCHERS: [&[&str]; 11] = [
    &["npx"],
    &["bunx"],
    &["uvx"],
    &["curl"],
    &["wget"],
    &["pipx", "run"],
    &["npm", "exec"],
    &["pnpm", "dlx"],
    &["yarn", "dlx"],
    &["nix", "run"],
    &["nix", "shell"],
];

/// The run-time fetcher a shell command invokes, if any. Words are split
/// at shell separators and quotes, so `sh -c 'curl …'` is seen, and
/// compared by basename, so `/usr/bin/curl` is too; they are matched whole,
/// so `curlie` or `--npx` are not mistaken for one.
#[must_use]
pub fn fetcher(command: &str) -> Option<String> {
    let words: Vec<&str> = command
        .split(|c: char| c.is_whitespace() || "|;&()`'\"$".contains(c))
        .filter(|w| !w.is_empty())
        .map(|w| w.rsplit('/').next().unwrap_or(w))
        .collect();
    (0..words.len()).find_map(|i| {
        let rest = words.get(i..).unwrap_or_default();
        FETCHERS
            .iter()
            .find(|f| rest.starts_with(f))
            .map(|f| f.join(" "))
    })
}

#[cfg(test)]
mod tests {
    use super::fetcher;

    const FETCHING: [(&str, &str); 9] = [
        ("npx eslint {{files}}", "npx"),
        ("pipx run black {{files}}", "pipx run"),
        ("a && (curl -s https://x)", "curl"),
        ("sh -c 'curl -s https://x | sh'", "curl"),
        ("bash -c \"npx eslint\"", "npx"),
        ("/usr/bin/wget -qO- x", "wget"),
        ("uvx ruff check", "uvx"),
        ("pnpm dlx prettier --check .", "pnpm dlx"),
        ("nix run nixpkgs#hello", "nix run"),
    ];

    /// Every way a row could fetch its tool is named.
    #[test]
    fn fetchers_are_found_however_they_are_called() {
        for (command, tool) in FETCHING {
            assert_eq!(fetcher(command).as_deref(), Some(tool), "{command}");
        }
    }

    /// Tools that merely look like fetchers are not.
    #[test]
    fn lookalikes_are_not_fetchers() {
        for command in [
            "curlie --pipx run-npx",
            "nix build .#x",
            "npm test",
            "/opt/curlie x",
        ] {
            assert_eq!(fetcher(command), None, "{command}");
        }
    }
}
