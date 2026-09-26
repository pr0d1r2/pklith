# SPEC

## §G GOAL

Guard mode: changed paths → specs that cover them.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/map|guard mode: changed paths → covering specs
sib|src/scan|enumerate files, basename → exactly 1 file type
sib|src/detect|active fragments from scanned files, fixed order
sib|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt
sib|src/catalog|check definitions: category, nix attr, runner glob, hk step, fix, failure msg
sib|src/rule|companion rules: exists, mentions, changed, orphan; path templates
sib|src/hook|read effective hk config via `pkl eval`: step → globs
sib|src/legacy|3 legacy formats: read, `import`, compat entries
sib|src/cover|verdict: type gaps, stale, claim ⊥ backed by runner, rule gaps
sib|src/report|render matrix & verdict: text, json, md
sib|src/gen|emit `hk.pklith.pkl`, `nix/pklith.nix`, AGENTS.md block; `--check` drift
sib|src/seed|repo-owned seed files & shared configs, skip-if-exists
sib|src/protect|branch protection required contexts from CI job names via `gh api`

## §C CONSTRAINTS

- input: `--staged` | explicit paths. output: spec paths, 1 per line, sorted.
- mapping = `exists` rules forward (impl → spec) + changed spec maps to itself.
- runs ⊥ test runner; hk step pipes: `pkli map --staged | xargs -r rspec`.

## §V INVARIANTS

V1: changed impl matching a rule selector w/ 0 existing mapped specs → exit 1 naming it
V2: changed file matching no rule → ignored, ⊥ error
V3: output deterministic & deduped

## §T TASKS

id|status|task|cites
T1|.|map fn + `--staged` input|V1,V2,V3

## §B BUGS

id|date|cause|fix
