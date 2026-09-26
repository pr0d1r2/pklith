# SPEC

## §G GOAL

Lay missing checks into a repo, 1 commit per check, through real hooks.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/lay|1 commit per missing check, hooks ON, rollback on red
sib|src/proc|run external programs one way: git hook env scrubbed, failures named
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
sib|src/seed|repo-owned seed files & shared configs, skip-if-exists
sib|src/confirm|acceptance of a materialized repo: completeness, fidelity, coherence, executability, idempotence
sib|src/migrate|lefthook-materialized repo → hk, gated on check-set equivalence
sib|src/protect|branch protection required contexts from CI job names via `gh api`
sib|src/cli|arg dispatch, usage, exit codes

## §C CONSTRAINTS

- plan = checks registry requires − steps in `hk.pkl` (src/hook), ∧ ≥1 file covered (`.:V15`).
- order FIXED: `*` universal first, then category order (src/catalog), then catalog order (as src/gen emits). ⊥ LLM, ⊥ heuristic.
- per check: gen fragment + config file + AGENTS row → `git add` exact paths → `git commit` w/ hooks ON.
- subject `ci(<check>): add the <check> check`: catalog `msg` describes a FAILURE & can exceed 72 chars, so it goes in the body. body: msg, deliberately absent checks & why, + planted-violation proof from the catalog fixture (`.:V32`).

## §V INVARIANTS

V1: hooks not installed | hk missing → exit 2 before any write (`.:V12`, `.:V1`)
V2: 1 check = 1 commit (`.:V13`); ⊥ bundle, ⊥ split
V3: red hook → reset `--mixed` to pre-run HEAD, remove files lay created; ⊥ `--hard`, operator's uncommitted work kept
V4: `--dry-run` → 0 fs writes, 0 git ops
V5: idempotent: 2nd run → 0 commits
V6: dirty index touching a planned path → exit 2, 0 writes
V7: same inputs → same commits (subjects, bodies, trees) (`.:V3`)
V8: commit carries ONLY lay's paths; operator's other staged changes stay staged & uncommitted
V9: stdout: 1 line per commit made (`<sha> <subject>`); nothing to lay → silence, exit 0

## §T TASKS

id|status|task|cites
T1|x|planner: required − present, ordered|V5,V7,`.:V15`
T2|x|per-check writer via src/gen fragment fn|V2
T3|x|git driver: add exact paths, commit through hooks|V1,V2
T4|x|rollback|V3,V6
T5|x|tests: PLANT red check → rollback; apply twice → 0 commits; dry-run → 0 writes|V3,V4,V5
T6|x|test: operator has unrelated staged change → lay commits exclude it, index still holds it|V8
T7|x|test: stdout lists exactly the commits made; 2nd run prints nothing|V9,V5

## §B BUGS

id|date|cause|fix
