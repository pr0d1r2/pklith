# SPEC

## §G GOAL

Write repo-owned seed files & shared configs once; never overwrite what the repo grew.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/seed|repo-owned seed files & shared configs, skip-if-exists
sib|src/proc|run external programs one way: git hook env scrubbed, failures named
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
sib|src/confirm|acceptance of a materialized repo: completeness, fidelity, coherence, executability, idempotence
sib|src/migrate|lefthook-materialized repo → hk, gated on check-set equivalence
sib|src/protect|branch protection required contexts from CI job names via `gh api`
sib|src/cli|arg dispatch, usage, exit codes

## §C CONSTRAINTS

- seed set per fragment from catalog: `.editorconfig`, `.gitattributes`, `.gitignore` fragments, size limits, exemptions ledger, `.typos.toml`, LICENSE, README stub.
- templates embedded in binary; ⊥ network.
- `--init` also writes `.pklith` from detected fragments: their checks per type, & binary types (images, fonts, archives) as exempt w/ reason `binary asset`. repo edits it after.

## §V INVARIANTS

V1: `--init` skip-if-exists; existing file ⊥ touched (`.:V25`)
V2: `.gitignore` fragments concatenated in fragment order
V3: seeded exemptions ledger = empty list, valid (`.:V11`, `.:V23`)

## §T TASKS

id|status|task|cites
T1|.|seed writer + skip-if-exists + empty ledger valid|V1,V3,`.:T55`
T2|.|`.gitignore` composition|V2
T3|.|`.pklith` from detected fragments + binary-asset exemptions|V1,`.:V11`

## §B BUGS

id|date|cause|fix
