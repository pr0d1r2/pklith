# SPEC

## §G GOAL

Read 3 legacy formats & serve compat entries so legacy consumers migrate w/o gate change.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/legacy|3 legacy formats: read, `import`, compat entries
sib|src/scan|enumerate files, basename → exactly 1 file type
sib|src/detect|active fragments from scanned files, fixed order
sib|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt
sib|src/catalog|check definitions: category, nix attr, runner glob, hk step, fix, failure msg
sib|src/rule|companion rules: exists, mentions, changed, orphan; path templates
sib|src/hook|read effective hk config via `pkl eval`: step → globs
sib|src/cover|verdict: type gaps, stale, claim ⊥ backed by runner, rule gaps
sib|src/map|guard mode: changed paths → covering specs
sib|src/report|render matrix & verdict: text, json, md
sib|src/gen|emit `hk.pklith.pkl`, `nix/pklith.nix`, AGENTS.md block; `--check` drift
sib|src/lay|1 commit per missing check, hooks ON, rollback on red
sib|src/seed|repo-owned seed files & shared configs, skip-if-exists
sib|src/confirm|acceptance of a materialized repo: completeness, fidelity, coherence, executability, idempotence
sib|src/migrate|lefthook-materialized repo → hk, gated on check-set equivalence
sib|src/protect|branch protection required contexts from CI job names via `gh api`
sib|src/cli|arg dispatch, usage, exit codes

## §C CONSTRAINTS

- linter doc parse = `.:R3` exactly. col 2 read as check hint only on `import`.
- set-and-setting fragment map read as JSON produced by `nix eval --json`; ⊥ nix parsing, ⊥ nix at runtime.
- `.unit-coverage.toml` = fixed schema (`.:R9`) → hand parser for that subset ?, ⊥ general TOML dep.
- compat env: `LEFTHOOK_LINTER_COVERAGE_DOC` (base default `docs/linter-coverage.md`, full required), `LEFTHOOK_LINTER_COVERAGE_ROOT`, `LEFTHOOK_UNIT_COVERAGE_CONFIG` (default `.unit-coverage.toml`), `LEFTHOOK_UNIT_COVERAGE_ROOT`.

## §V INVARIANTS

V1: token set == legacy awk token set ∀ fixture doc (golden, legacy bats ported)
V2: compat exit codes & missing-list lines == legacy (`.:R1` `.ext`/`name`, `.:R2` `.ext`, `.:R9` unit-coverage missing-test lines & allowlist)
V3: `import`: col 2 `-` → exempt, reason = col 3 (Notes); unknown check hint → exempt `legacy: <col2>` ⊥ silent drop
V4: doc parse in-process. ⊥ runtime awk/sed/gawk
V5: read error | 0 tokens from doc w/ table rows → exit 2 "parsed nothing", ⊥ gap list

## §T TASKS

id|status|task|cites
T1|.|port legacy bats fixtures (3 repos) as Rust golden tests|V1,V2,`.:R1`,`.:R2`,`.:R3`,`.:R9`
T2|.|linter doc parser|V1,V4,V5
T3|.|`.unit-coverage.toml` subset parser + `.coverage-allowlist`|`.:R9`
T4|.|`import` both formats|V3
T5|.|compat entries ×3|V2,`.:T46`

## §B BUGS

id|date|cause|fix
B1|2026-05-09|legacy base: `awk -- "$doc"` — GNU awk reads `--` as filename ∴ parse broke (21f30f5)|V4
B2|2026-09-07|legacy `-full` #5: build placeholder `…_AWK_PROGRAM_PATH` unsubstituted when consumer builds from `flake = false` src; gawk error to stderr inside `< <(…)` ∴ loop reads nothing ∴ ∀ ext "uncovered" (20 false gaps)|V5,`.:V10`
