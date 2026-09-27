# Port map: set-and-setting's setting half to pklith

pklith ports the *setting* half of set-and-setting: the part that works
out which checks a repository needs and wires them into git hooks. The
*set* half (skills, `mkSet`) stays in set-and-setting. This page records
where each piece went, and what was left behind and why, so nothing
disappears silently in the move.

## Entry points

| set-and-setting | pklith | notes |
|---|---|---|
| `setting/lib/detect-fragments.sh` | `pklith detect` (src/detect) | Triggers are globs in the catalog, matched the way hk matches. |
| `lib/check-fragment-map.nix` | `## fragments` in `src/catalog/builtin.pklith` | See the fragment table below. |
| `checksFor` and the 24 `mk*Check` helpers | built-in catalog rows | See the check table below. |
| `linter-coverage`, `coveragePerFileClass` | `pklith check` (src/cover) | Types in `.pklith`, with stale and unbacked claims found too. |
| `materializationFor`, `assemble-lefthook.sh` | `pklith gen`, `pklith lay` | hk steps in `hk.pklith.pkl` instead of `lefthook.yml`. |
| `sync-setting-init`, `app-seed.sh`, `canonFor` | `pklith seed` (src/seed) | Seed files are written once and then belong to the repository. |
| drift checks (`drift-check.sh`, `coverage-drift-check.sh`, `materialize-check.sh`) | `pklith gen --check` | One freshness check for everything gen writes. |
| `agents-md-compile.sh` | `pklith gen` AGENTS.md block (src/gen) | |
| `confirm.sh`, `app-confirm.sh` | `pklith confirm` (src/confirm) | |
| `migrate.sh`, `app-migrate.sh` | `pklith migrate` (src/migrate) | lefthook to hk, gated on the same check set before and after. |
| `branch-protection.sh` | `pklith protect` (src/protect) | The one command allowed to touch the network. |
| `mkDevShells` | `lib.devShell` in `flake.nix` | |
| `lock-graph-check.sh`, `nix-flake-lock-budget.sh` | dropped | pklith's own `flake-lock-graph` step covers its lock; a consumer's lock budget is its own policy. |
| `graduate-draft.sh`, `chain-ready.sh`, skill checks | stay in set-and-setting | They belong to the set half. |

## Fragments

| fragment | in pklith | why not |
|---|---|---|
| base | yes | |
| actions | yes | |
| nix | yes | |
| shell | yes | |
| rubocop | yes | |
| xml | yes | |
| awk | yes | |
| bats | yes | |
| markdown | yes | |
| yaml | yes | |
| toml | yes | |
| ruby | no | It selects no checks in set-and-setting, and a fragment that selects nothing is an error in pklith (root V18). |
| rspec, brakeman | no | They run a repository's own bundled gems, not a pinned tool (root V29); a repository adds them in its `.pklith`. |
| bundle-audit | no | `--update` fetches the advisory database while the hook runs (root V29). |
| reek | no | nixpkgs has no reek package. |
| ascii | no | `ascii-only` and `unicode-lint` are custom scripts; `no-bom` and the editorconfig checks cover the common cases. |
| just | no | Both checks are custom scripts with no nix package. |
| tcl | no | The check is a custom tclsh script. |
| set | no | Set-half checks; they stay in set-and-setting. |

pklith adds three fragments of its own: `rust`, `spec` (SPEC.md files) and `context` (`.context-limits`).

## Checks

| set-and-setting check | pklith catalog row |
|---|---|
| trailing-whitespace | `trailing-whitespace` |
| missing-final-newline | `final-newline` |
| git-conflict-markers | `no-merge-conflict` |
| gitleaks | `ripsecrets` (one secret scanner, the one the fleet already uses) |
| git-no-local-paths | `no-local-paths` |
| editorconfig-checker | `editorconfig-checker` |
| typos | `typos` |
| actionlint | `actionlint` (plus `zizmor`) |
| nixfmt, statix, deadnix | same names |
| shellcheck, shfmt | same names |
| rubocop | `rubocop`, run from nixpkgs rather than `bundle exec` |
| markdownlint | `markdownlint` |
| yamllint | `yamllint` |
| taplo | `taplo` |
| xmllint | `xmllint` |
| gawk-lint | `gawk-lint`, which parses without running the script |
| bats-parse | `bats-parse` |
| linter-coverage | `pklith check` itself |

These were dropped:

| check | why |
|---|---|
| execute-permissions, file-size-check | Driven by per-repository config files; a repository adds its own (pklith's own gate has `no-large-files`). |
| no-shell-functions, nix-no-embedded-shell | House style of set-and-setting, not a general rule. |
| flake-manifest | Reads set-and-setting's flake manifest format. |
| nix-flake-check | Slow and whole-flake; it belongs in CI, where a repository can add it. |
| commit-msg-lint, changelog-touched | Commit-message and changelog policy is the repository's; pklith's own gate has both. |
| markdownlint-agentic | Depends on set-and-setting's skill-file classifier. |
| bats-unit, tdd-order-bats | They run a repository's own test suite and history, not a file check. |
| rekall-check, rekall-gnu-sed, set-skill-*, set-ref-resolution, set-bundle-content, skill-registered | Set-half checks. |
