# A gate in an existing repository

A worked example of materializing pklith's gate in a repository that
already has code, history and an upstream it must not rewrite. The example
is [age-plugin-se PR #5](https://github.com/pr0d1r2/age-plugin-se/pull/5),
a fork of an upstream age plugin written in Swift. The PR is still open; what
follows describes its head `8b4151a` as of 2026-10-18 and may change before
it merges.

After `pklith seed --init`, the PR adds 36 commit steps, 3 push steps and 2
commit-msg steps, one check per commit, each body recording the planted
violation it refused. It is the first consumer of the swift fragment (#90),
of test checks at push (#91) and of the commandless-step guard (#93).
[INTEGRATION.md](INTEGRATION.md) is the general path; this page is about
what an existing repository adds to it.

## Laying by hand, in lay's order

The PR did not run `pklith lay`. Each commit adds one check's claim to
`.pklith`, runs `pklith gen` to rewrite `hk.pklith.pkl` and
`nix/pklith.nix`, and commits through the live hooks, in the order lay
would have used (hygiene, secret, format, lint, spec, test, coverage,
gate). Doing it by hand let each commit carry what lay does not write
today: the planted violation and the red line it produced (root V32), the
ids of the spec rows it serves, and the repository's own trailers.

That is a supported path: the result is the same gate, `pklith gen --check`
and `pklith confirm` pass on it, and every commit went through the hooks
(root V12). What `lay` would need to produce such commits itself (a proof in
the body, cited ids, a trailer) is not specified yet.

## Upstream findings: a baseline that only shrinks

Code that came from upstream is not reformatted to pass the gate. Its
findings go into baselines that may only get smaller:

- `.pklith` exempts upstream-owned paths (`.github/**`, `Documentation/**`,
  `Scripts/**`, `Makefile`, `README.md`) with a reason, and `hk.pkl`
  excludes them from the steps.
- A single upstream file with one known finding (a help-text literal with
  trailing blanks) is excluded only from the two steps it fails; every
  other step still reads it.
- Per-tool baselines hold the rest: `.typos.toml`, `.secretsignore` listing
  the upstream test-vector strings by exact string rather than by file, and
  `.markdownlintignore`.

[CATALOG.md](CATALOG.md) does not yet describe this fork or legacy recipe.
The swiftlint baseline is still open in
[#87](https://github.com/pr0d1r2/pklith/issues/87): the seeded
`.swiftlint.yml` finds 120 violations in upstream code, so the PR defers
the swiftlint row.

## Host-toolchain rows

The toolchain check, `swift build --build-tests -Xswiftc
-warnings-as-errors` and `swift test` run the host Xcode's Swift. pklith's
catalog has no way to name a tool nix cannot supply, so the PR writes them
as local `.pklith` rows with no nix package. Naming that gap in the catalog
is open in [#89](https://github.com/pr0d1r2/pklith/issues/89).

## Proofs that must not touch shared state

Most proofs plant a file and run `hk check --step <id>`. Two cannot:

- **no-core-worktree** refuses `core.worktree`, which lives in the git
  config every worktree shares. It was planted in a scratch clone, not in
  the repository being gated.
- **confirm** was proven by reducing `hk.pkl` to its `amends` line. hk
  cannot run a step from a config with no hooks, so `pklith confirm` was
  run directly; it exited 1 with "does not evaluate to steps".

## One test command for nix and the hook

A single script, `nix/test.sh`, runs the tests. The flake's
`checks.<system>.tests` and the pre-push `test` row both call it, so
`nix build .#checks.<system>.tests` and the hook cannot drift apart.

## A clash it found

`.context-limits` holds sherd's chain ceilings, read by `sherd-budget`. The
catalog's `itok` row reads the same file as per-file ceilings and refuses
the root `SPEC.md` against a ceiling meant for its chain. A repository can
have one reading or the other, not both, so the PR defers `itok`. pklith's
own gate met the same clash (root B11); the catalog fix is recorded as
`src/catalog` T8, still open.
