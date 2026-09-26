# SPEC

## §G GOAL

Run external programs (git, hk, pkl) one way: git hook env scrubbed, failures named.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/proc|run external programs one way: git hook env scrubbed, failures named
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
sib|src/cli|arg dispatch, usage, exit codes

## §C CONSTRAINTS

- sole place `std::process::Command` is built (`.:V20`); ⊥ other node spawns directly.
- removes `GIT_DIR` `GIT_INDEX_FILE` `GIT_WORK_TREE` unless the caller opts in.

## §V INVARIANTS

V1: ∀ spawned command lacks the 3 git hook vars by default
V2: spawn failure | non-zero exit → error naming program, args & stderr, ⊥ empty output as success
V3: stdout returned as bytes; decoding is the caller's decision

## §T TASKS

id|status|task|cites
T1|.|`command` builder w/ scrub + `output` runner + error type|V1,V2,V3,`.:T63`

## §B BUGS

id|date|cause|fix
