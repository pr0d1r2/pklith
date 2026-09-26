# SPEC

## §G GOAL

Derive & apply branch protection required contexts from CI job names.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/protect|branch protection required contexts from CI job names via `gh api`
sib|src/scan|enumerate files, basename → exactly 1 file type
sib|src/detect|active fragments from scanned files, fixed order
sib|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt
sib|src/catalog|check definitions: category, nix attr, runner glob, hk step, fix, failure msg
sib|src/rule|companion rules: exists, mentions, changed, orphan; path templates
sib|src/hook|read effective hk config via `pkl eval`: step → globs
sib|src/legacy|3 legacy formats: read, `import`, compat entries
sib|src/cover|verdict: type gaps, stale, claim ⊥ backed by runner, rule gaps
sib|src/map|guard mode: changed paths → covering specs
sib|src/report|render matrix & verdict: text, json, md
sib|src/gen|emit `hk.pklith.pkl`, `nix/pklith.nix`, AGENTS.md block; `--check` drift
sib|src/lay|1 commit per missing check, hooks ON, rollback on red
sib|src/seed|repo-owned seed files & shared configs, skip-if-exists

## §C CONSTRAINTS

- contexts parsed from workflow files; applied via `gh api`. network ONLY here, named & opt-in.

## §V INVARIANTS

V1: `--dry-run` → prints payload, 0 API calls
V2: job rename → context set changes → shown as breaking diff before apply

## §T TASKS

id|status|task|cites
T1|.|context derivation from workflows + golden tests|V2,`.:T58`
T2|.|`gh api` apply + dry-run|V1

## §B BUGS

id|date|cause|fix
