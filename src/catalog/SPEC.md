# SPEC

## §G GOAL

Define checks: what each is, how nix provides it, which files its runner sees, how hk runs & fixes it, what its failure says.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/catalog|check definitions: category, nix attr, runner glob, hk step, fix, failure msg
sib|src/scan|enumerate files, basename → exactly 1 file type
sib|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt
