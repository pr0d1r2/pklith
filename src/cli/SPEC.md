# SPEC

## §G GOAL

Parse args, dispatch verbs, map outcomes to exit codes.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/cli|arg dispatch, usage, exit codes
sib|src/scan|enumerate files, basename → exactly 1 file type
sib|src/detect|active fragments from scanned files, fixed order
sib|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt
sib|src/catalog|check definitions: category, nix attr, runner glob, hk step, fix, failure msg
sib|src/rule|companion rules: exists, mentions, changed, orphan; path templates
sib|src/hook|read effective hk config via `pkl eval`: step → globs
sib|src/legacy|3 legacy formats: read, `import`, compat entries
sib|src/cover|verdict: type gaps, stale, claim ⊥ backed by runner, rule gaps
sib|src/map|guard mode: changed paths → covering specs
sib|src/report|render matrix & verdict: text, json, md
sib|src/gen|emit `hk.pklith.pkl`, `nix/pklith.nix`, AGENTS.md block; `--check` drift
sib|src/lay|1 commit per missing check, hooks ON, rollback on red
sib|src/seed|repo-owned seed files & shared configs, skip-if-exists
sib|src/confirm|acceptance of a materialized repo: completeness, fidelity, coherence, executability, idempotence
sib|src/migrate|lefthook-materialized repo → hk, gated on check-set equivalence
sib|src/protect|branch protection required contexts from CI job names via `gh api`

## §C CONSTRAINTS

- hand arg parse (fleet style); ? `lexopt` if hand parse breaks fn limits.

## §V INVARIANTS

V1: exit codes: 0 ok / 1 finding / 2 usage | I/O error. ∀ verb
V2: unknown flag → usage on stderr, exit 2
V3: stdout = data only; diagnostics → stderr
V4: ∀ path arg opened as path, ⊥ read as flag
V5: run from any subdirectory → acts on repo root (`git rev-parse --show-toplevel`); `--root` overrides; outside a repo w/o `--root` → exit 2
V6: `--version` prints crate version & built-in catalog version

## §T TASKS

id|status|task|cites
T1|.|dispatch `check`/`report`/`gen`/`lay`/`map`/`import`/`detect`/`seed`/`confirm`/`migrate`/`protect` + usage|V1,V2,V3,V4,`.:I`
T2|.|root discovery from subdirectory + `--version`|V5,V6

## §B BUGS

id|date|cause|fix
B1|2026-05-09|legacy base: doc path starting `-` could be read as awk flag (bd2f981)|V4
