# SPEC

## §G GOAL

Read effective hk config: which steps exist & which files each step sees.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/hook|read effective hk config via `pkl eval`: step → globs
sib|src/scan|enumerate files, basename → exactly 1 file type
sib|src/detect|active fragments from scanned files, fixed order
sib|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt
sib|src/catalog|check definitions: category, nix attr, runner glob, hk step, fix, failure msg
sib|src/rule|companion rules: exists, mentions, changed, orphan; path templates
sib|src/legacy|3 legacy formats: read, `import`, compat entries
sib|src/cover|verdict: type gaps, stale, claim ⊥ backed by runner, rule gaps
sib|src/map|guard mode: changed paths → covering specs
sib|src/seed|repo-owned seed files & shared configs, skip-if-exists

## §C CONSTRAINTS

- source = `pkl eval --format json hk.pkl` (pkl from devShell). ⊥ own pkl parser.
- output: step id → globs, excludes, hook membership.

## §V INVARIANTS

V1: `pkl` missing | eval fails → exit 2 naming cause (`.:V1`), ⊥ "0 steps"
V2: 0 steps from a non-empty `hk.pkl` → error, ⊥ empty map (same shape as `.:B1`)
V3: globs matched by src shared engine, same as catalog

## §T TASKS

id|status|task|cites
T1|.|`pkl eval` driver + json → step map|V1,V2
T2|.|fixture `hk.pkl` files: fast/all split, `depends`, excludes|V3

## §B BUGS

id|date|cause|fix
