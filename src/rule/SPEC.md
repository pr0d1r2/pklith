# SPEC

## §G GOAL

Companion rules: ∀ file matching a selector, something ! exist | be mentioned | change with it.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/rule|companion rules: exists, mentions, changed, orphan; path templates
sib|src/scan|enumerate files, basename → exactly 1 file type
sib|src/detect|active fragments from scanned files, fixed order
sib|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt
sib|src/catalog|check definitions: category, nix attr, runner glob, hk step, fix, failure msg

## §V INVARIANTS

V1: template render deterministic; unknown var | filter → error at parse, ⊥ empty string
V2: `mentions` pattern = LITERAL after render, ⊥ regex built from file text
V3: `changed` runs only w/ explicit diff input (`--staged` | range); ⊥ guess base
V4: unit-coverage `mirror`/`flat`/`strip`/`test_suffix`/`test_ext`/`normalize` ∀ expressible as `exists` rule (`.:R9`)
V5: ⊥ rule result depends on file content except `mentions`
