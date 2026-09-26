# SPEC

## §G GOAL

Render Coverage: matrix file type × check w/ file counts, rule results, verdict.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/report|render matrix & verdict: text, json, md
sib|src/scan|enumerate files, basename → exactly 1 file type
sib|src/detect|active fragments from scanned files, fixed order
sib|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt
sib|src/catalog|check definitions: category, nix attr, runner glob, hk step, fix, failure msg
sib|src/rule|companion rules: exists, mentions, changed, orphan; path templates
sib|src/hook|read effective hk config via `pkl eval`: step → globs
sib|src/legacy|3 legacy formats: read, `import`, compat entries
sib|src/cover|verdict: type gaps, stale, claim ⊥ backed by runner, rule gaps
sib|src/map|guard mode: changed paths → covering specs
sib|src/seed|repo-owned seed files & shared configs, skip-if-exists

## §C CONSTRAINTS

- json hand-written | ≤1 dep; justify in `.:R`.
- text on failure: name each gap + fix hint (add check | exempt w/ reason).

## §V INVARIANTS

V1: stable row & column order ∀ formats (`.:V3`)
V2: json keys always present; empty = `[]`, ⊥ absent key
V3: `check` success = silence; `report` always prints
