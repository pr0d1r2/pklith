//! The seed files pkli can write, by the id a fragment's `seed` cell names
//! them with. Embedded in the binary (seed §C): seeding reads no network
//! and no clock.

/// What a seed id contributes.
pub enum Template {
    /// A whole file; `{name}` becomes the repository's directory name.
    File(&'static str, &'static str),
    /// Lines of `.gitignore`, which collects every active fragment's part.
    Ignore(&'static str),
}

/// Every seed id, in no particular order; fragment order decides output.
pub const TEMPLATES: [(&str, Template); 8] = [
    (
        "editorconfig",
        Template::File(".editorconfig", EDITORCONFIG),
    ),
    (
        "gitattributes",
        Template::File(".gitattributes", "* text=auto eol=lf\n"),
    ),
    ("readme", Template::File("README.md", "# {name}\n")),
    ("envrc", Template::File(".envrc", "use flake\n")),
    (
        "rubocop",
        Template::File(".rubocop.yml", "AllCops:\n  NewCops: enable\n"),
    ),
    (
        "ignore-nix",
        Template::Ignore(".direnv/\nresult\nresult-*\n"),
    ),
    ("ignore-rust", Template::Ignore("/target\n")),
    (
        "ignore-ruby",
        Template::Ignore("/vendor/bundle\n/.bundle\n"),
    ),
];

/// What editorconfig-checker (base fragment) holds files to: LF, UTF-8, a
/// final newline, no trailing blanks, spaces except where a tool needs a
/// tab. No indent width: languages differ, and their formatters own it.
const EDITORCONFIG: &str = "root = true

[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true
indent_style = space

[{Makefile,*.mk,go.mod,*.go}]
indent_style = tab
";

/// The template a seed id names.
#[must_use]
pub fn named(id: &str) -> Option<&'static Template> {
    TEMPLATES
        .iter()
        .find(|(name, _)| *name == id)
        .map(|(_, t)| t)
}
