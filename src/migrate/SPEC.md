# SPEC

## §G GOAL

Move a lefthook-materialized repo onto hk without losing a check.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/migrate|lefthook-materialized repo → hk, gated on check-set equivalence
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
sib|src/protect|branch protection required contexts from CI job names via `gh api`
sib|src/cli|arg dispatch, usage, exit codes

## §C CONSTRAINTS

- input: `lefthook.yml` (+ overrides, local) → check set; output: `.pklith` + generated `hk.pkl` fragment.
- lefthook parsed only here; ⊥ elsewhere.

## §V INVARIANTS

V1: check set before == after, else refuse w/ 0 writes (`.:V24`)
V2: deterministic & idempotent: 2nd run → 0 changes
V3: unmapped lefthook command → named in refusal, ⊥ dropped

## §T TASKS

id|status|task|cites
T1|.|lefthook check-set reader|V3
T2|.|equivalence gate + writer|V1,V2,`.:T57`

## §B BUGS

id|date|cause|fix
