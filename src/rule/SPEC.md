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
sib|src/hook|read effective hk config via `pkl eval`: step → globs
sib|src/legacy|3 legacy formats: read, `import`, compat entries
sib|src/cover|verdict: type gaps, stale, claim ⊥ backed by runner, rule gaps
sib|src/map|guard mode: changed paths → covering specs
sib|src/report|render matrix & verdict: text, json, md
sib|src/gen|emit `hk.pklith.pkl`, `nix/pklith.nix`, AGENTS.md block; `--check` drift
sib|src/seed|repo-owned seed files & shared configs, skip-if-exists

## §C CONSTRAINTS

- row: `id|kind|select|target|except`. kinds: `exists` (path template) · `mentions` (glob + literal pattern template) · `changed` (target changes when source changes; diff input) · `orphan` (reverse: target w/o source).
- template vars `{path} {dir} {stem} {ext}`; filters `strip:<prefix>` `suffix:<s>` `ext:<e>` `snake` `dash` `plural`.
- `plural` = fixed suffix rules (`+s`; `y`→`ies`; `s x ch sh`→`+es`) + `## plural` table for irregulars. ⊥ inflector guess.
- ⊥ `spec_bigger`, ⊥ `mentioning_methods` — quality proxies (`.:R10`).

## §V INVARIANTS

V1: template render deterministic; unknown var | filter → error at parse, ⊥ empty string
V2: `mentions` pattern = LITERAL after render, ⊥ regex built from file text
V3: `changed` runs only w/ explicit diff input (`--staged` | range); ⊥ guess base
V4: unit-coverage `mirror`/`flat`/`strip`/`test_suffix`/`test_ext`/`normalize` ∀ expressible as `exists` rule (`.:R9`)
V5: ⊥ rule result depends on file content except `mentions`

## §T TASKS

id|status|task|cites
T1|.|template parser + filters + plural rules|V1
T2|.|`exists` + `orphan`|V4
T3|.|`mentions` (factory_bot `factory :{stem\|snake}` under `spec/factories/**`)|V2
T4|.|`changed` over staged diff|V3

## §B BUGS

id|date|cause|fix
