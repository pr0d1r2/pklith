# SPEC

## §G GOAL

Verdict: which file types lack checks, which declarations are stale, which claims no runner backs, which rules fail.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/cover|verdict: type gaps, stale, claim ⊥ backed by runner, rule gaps
sib|src/scan|enumerate files, basename → exactly 1 file type
sib|src/detect|active fragments from scanned files, fixed order
sib|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt
sib|src/catalog|check definitions: category, nix attr, runner glob, hk step, fix, failure msg
sib|src/rule|companion rules: exists, mentions, changed, orphan; path templates
sib|src/hook|read effective hk config via `pkl eval`: step → globs
sib|src/legacy|3 legacy formats: read, `import`, compat entries

## §V INVARIANTS

V1: gap = scanned type w/ ⊥ registry row | < `min` non-`*` checks → fail
V2: stale = registry type matching 0 files | check whose runner glob matches 0 files → fail. ? `--allow-stale`
V3: CLAIM ⊥ BACKED: registry says type T → check C, but ∃ file of T ⊥ matched by C's hk step globs | C ∉ `hk.pkl` → fail, naming file, check, glob
V4: exempt = covered, reported separately w/ reason
V5: empty repo → ok, ⊥ error
V6: totality: ∀ file ∈ exactly 1 type; 0 → gap, ⊥ silently dropped (`.:R14`)
