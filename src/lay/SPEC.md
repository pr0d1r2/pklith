# SPEC

## §G GOAL

Lay missing checks into a repo, 1 commit per check, through real hooks.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/lay|1 commit per missing check, hooks ON, rollback on red
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
sib|src/protect|branch protection required contexts from CI job names via `gh api`

## §C CONSTRAINTS

- plan = checks registry requires − steps in `hk.pkl` (src/hook), ∧ ≥1 file covered (`.:V15`).
- order FIXED: `*` universal first, then category order (src/catalog), then id. ⊥ LLM, ⊥ heuristic.
- per check: gen fragment + config file + AGENTS row → `git add` exact paths → `git commit` w/ hooks ON.
- subject `ci(<check>): <claim>` (claim from catalog `msg`); body names deliberately absent checks & why.

## §V INVARIANTS

V1: hooks not installed | hk missing → exit 2 before any write (`.:V12`, `.:V1`)
V2: 1 check = 1 commit (`.:V13`); ⊥ bundle, ⊥ split
V3: red hook → reset `--mixed` to pre-run HEAD, remove files lay created; ⊥ `--hard`, operator's uncommitted work kept
V4: `--dry-run` → 0 fs writes, 0 git ops
V5: idempotent: 2nd run → 0 commits
V6: dirty index touching a planned path → exit 2, 0 writes
V7: same inputs → same commits (subjects, bodies, trees) (`.:V3`)
