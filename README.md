# pklith

Materialize [hk](https://hk.jdx.dev) git hook guardrails from what a
repository's files require.

pklith looks at every tracked file, works out which linters and checks its
type needs (several per type, plus rules such as "every implementation has a
test"), and turns that into hk steps and the nix packages that run them. It
can lay those checks into a repository one commit per check, and it reports
and enforces that no file goes unchecked.

The command is `pkli`. It is not part of [Pkl](https://pkl-lang.org), the
configuration language hk uses; the name only borrows it.

Status: early, but the core loop works, and pklith's own gate is built with
it:

- `pkli check`: every tracked file type has a row in `.pklith`, no row is
  stale, and every check a row claims is an hk step that actually reaches
  those files.
- `pkli gen [--check]`: renders `.pklith` into `hk.pklith.pkl`, the hk steps
  your `hk.pkl` imports.
- `pkli lay [--dry-run]`: adds each missing check as its own commit, through
  your git hooks, and rolls everything back if one is refused.
- `pkli import DOC`: turns a legacy linter-coverage document into `.pklith`.

`pkli` ships a built-in catalog of checks (hk util hygiene, ripsecrets,
typos, nixfmt, shfmt, shellcheck, taplo, rustfmt, clippy, rubocop,
actionlint, zizmor, mth, sherd, itok); a `.pklith` row with the same id
replaces one, a new id adds one.
The design lives in [`SPEC.md`](SPEC.md) and the `SPEC.md` files under `src/`.

## License

MIT
