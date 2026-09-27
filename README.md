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

## Migrating from the legacy coverage tools

pklith replaces `lefthook-linter-coverage`, `lefthook-linter-coverage-full`
and `lefthook-unit-coverage`. Move in two steps, each leaving the gate
green.

1. **Swap the package.** Replace the three legacy flake inputs with this
   flake's `packages.<system>.compat`. It installs `pkli` under each legacy
   name, and under those names it reads the same variables
   (`LEFTHOOK_LINTER_COVERAGE_DOC`, `LEFTHOOK_LINTER_COVERAGE_ROOT`,
   `LEFTHOOK_UNIT_COVERAGE_CONFIG`, `LEFTHOOK_UNIT_COVERAGE_ROOT`), prints
   the same messages and exits with the same codes. Your hook config does
   not change. There are two deliberate differences. A linter document with
   table rows but no backtick token is an error (exit 2), not a report that
   every extension is unlisted. An unreadable input is also exit 2, and the
   message names its cause.
2. **Move onto `pkli check`.** Run `pkli import docs/linter-coverage.md >
   .pklith` (a set-and-setting fragment map, from `nix eval --json`, imports
   the same way from a `.json` file). Every listed type becomes an exempt
   row, so the verdict carries over unchanged. Then replace exemptions with
   the checks that actually cover each type, run `pkli gen`, and drop the
   compat package once `pkli check` gates the repository.
   `pkli migrate` moves a repository whose hooks lefthook materialized
   onto hk. It goes only when the check sets match.

## License

MIT
