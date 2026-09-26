# SPEC

## §G GOAL

Parse, validate & format `.pklith`: file types → check SETS, companion rules, plural table, exemptions.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt
sib|src/scan|enumerate files, basename → exactly 1 file type

## §V INVARIANTS

V1: type row ! have ≥`min` checks beyond `*` XOR exempt reason. both | neither → error w/ line no
V2: unknown check id → error naming line & id, ⊥ skip
V3: duplicate type | rule id → error naming both lines
V4: `fmt` idempotent; parse(fmt(x)) = parse(x)
V5: `*` ⊥ satisfies `min` (universal hygiene ≠ real coverage)
V6: absent optional section = empty, ⊥ error (`.:V11`)
