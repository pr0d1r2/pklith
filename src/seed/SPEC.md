# SPEC

## §G GOAL

Write repo-owned seed files & shared configs once; never overwrite what the repo grew.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/seed|repo-owned seed files & shared configs, skip-if-exists
sib|src/scan|enumerate files, basename → exactly 1 file type
sib|src/detect|active fragments from scanned files, fixed order
sib|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt
sib|src/catalog|check definitions: category, nix attr, runner glob, hk step, fix, failure msg
sib|src/rule|companion rules: exists, mentions, changed, orphan; path templates
sib|src/hook|read effective hk config via `pkl eval`: step → globs
sib|src/legacy|3 legacy formats: read, `import`, compat entries
sib|src/cover|verdict: type gaps, stale, claim ⊥ backed by runner, rule gaps
sib|src/map|guard mode: changed paths → covering specs

## §C CONSTRAINTS

- seed set per fragment from catalog: `.editorconfig`, `.gitattributes`, `.gitignore` fragments, size limits, exemptions ledger, `.typos.toml`, LICENSE, README stub.
- templates embedded in binary; ⊥ network.

## §V INVARIANTS

V1: `--init` skip-if-exists; existing file ⊥ touched (`.:V25`)
V2: `.gitignore` fragments concatenated in fragment order
V3: seeded exemptions ledger = empty list, valid (`.:V11`, `.:V23`)
