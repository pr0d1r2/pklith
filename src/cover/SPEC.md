# SPEC

## §G GOAL

Verdict: which file types lack checks, which declarations are stale, which claims no runner backs, which rules fail.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/cover|verdict: type gaps, stale, claim ⊥ backed by runner, rule gaps
sib|src/proc|run external programs one way: git hook env scrubbed, failures named
sib|src/scan|enumerate files, basename → exactly 1 file type
sib|src/detect|active fragments from scanned files, fixed order
sib|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt
sib|src/catalog|check definitions: category, nix attr, runner glob, hk step, fix, failure msg
sib|src/rule|companion rules: exists, mentions, changed, orphan; path templates
sib|src/hook|read effective hk config via `pkl eval`: step → globs
sib|src/legacy|3 legacy formats: read, `import`, compat entries
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

- pure fn `(scan, registry, catalog, hook steps, rule results) → Coverage`. ⊥ I/O.

## §V INVARIANTS

V1: gap = scanned type w/ ⊥ registry row | < `min` non-`*` checks → fail
V2: stale = registry type matching 0 files | check whose runner glob matches 0 files → fail. ? `--allow-stale`
V3: CLAIM ⊥ BACKED: registry says type T → check C, but ∃ file of T ⊥ matched by C's hk step globs | C ∉ `hk.pkl` → fail, naming file, check, glob
V4: exempt = covered, reported separately w/ reason
V5: empty repo → ok, ⊥ error
V6: totality: ∀ file ∈ exactly 1 type; 0 → gap, ⊥ silently dropped (`.:R14`)
V7: failed rule → fail, naming rule id, source file & missing target
V8: staged mode judges only changed & deleted paths (+ rows they touch); full mode judges the tree (`.:V26`)
V9: newly active fragment ⊥ reflected in `.pklith` → finding w/ the rows to add; `.pklith` ⊥ auto-written after `seed --init`

## §T TASKS

id|status|task|cites
T1|~|Coverage type + gap/stale|V1,V2,V4,V5,V6
T2|.|claim-vs-runner join via src/hook|V3
T3|.|fold rule results|V7
T4|.|staged mode: changed + deleted paths only; test: deleting the last `.sh` flags its row stale|V8,`.:V26`
T5|.|test: first `.rb` file in a repo → finding names rubocop rows to add|V9

## §B BUGS

id|date|cause|fix
