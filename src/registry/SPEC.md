# SPEC

## §G GOAL

Parse, validate & format `.pklith`: file types → check SETS, companion rules, plural table, exemptions.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt
sib|src/scan|enumerate files, basename → exactly 1 file type
sib|src/detect|active fragments from scanned files, fixed order
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

- syntax = sectioned pipe tables, FORMAT cell rules (`\|` escape, `-` empty), `#` comments. ⊥ TOML (adds dep).
- `## types` → `type|checks|min|exempt`. `checks` = comma list of catalog ids (MANY per type). `min` default 1. `exempt` = reason | `-`.
- type `*` = UNIVERSAL checks (typos, whitespace, …), spread to ∀ type. ⊥ a literal type.
- `## checks` → local catalog rows (override | extend src/catalog). `## rules` → src/rule rows. `## plural` → `singular|plural`.

## §V INVARIANTS

V1: type row ! have ≥`min` checks beyond `*` XOR exempt reason. both | neither → error w/ line no
V2: unknown check id → error naming line & id, ⊥ skip
V3: duplicate type | rule id → error naming both lines
V4: `fmt` idempotent; parse(fmt(x)) = parse(x)
V5: `*` ⊥ satisfies `min` (universal hygiene ≠ real coverage)
V6: absent optional section = empty, ⊥ error (`.:V11`)

## §T TASKS

id|status|task|cites
T1|.|parser + errors w/ line numbers|V1,V2,V3,V6
T2|.|`*` spread + `min` accounting|V5,B1
T3|.|formatter + roundtrip proptest|V4

## §B BUGS

id|date|cause|fix
B1|2026-08-11|set-and-setting B55: `all` class read as a literal type ∴ unassigned-file test failed|V5
B2|2026-08-11|set-and-setting B53/B77: check listed in map w/o file class ∴ completeness rejected whole map, 4 checks at once|V2
