# SPEC

## §G GOAL

Product code hub. Pipeline: scan → detect; registry + catalog → rule, hook, legacy → cover, map → report, gen → lay; seed, confirm, migrate, protect beside. cli dispatches.

## §F FEDERATION

dir|owns|⊥owns|tokens
proc|run external programs one way: git hook env scrubbed, failures named|what any program is for|-
scan|enumerate files, basename → exactly 1 file type|registry syntax, verdicts|-
detect|active fragments from scanned files, fixed order|which checks a fragment brings|-
registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt|check definitions, file walk|-
catalog|check definitions: category, nix attr, runner glob, hk step, fix, failure msg|which type needs which check|-
rule|companion rules: exists, mentions, changed, orphan; path templates|linter checks, running tests|-
hook|read effective hk config via `pkl eval`: step → globs|writing hk config|-
legacy|3 legacy formats: read, `import`, compat entries|native `.pklith` syntax|-
cover|verdict: type gaps, stale, claim ⊥ backed by runner, rule gaps|walking, rendering, exit codes|-
map|guard mode: changed paths → covering specs|running test runners|-
report|render matrix & verdict: text, json, md|computing coverage|-
gen|emit `hk.pklith.pkl`, `nix/pklith.nix`, AGENTS.md block; `--check` drift|running hk, running nix, committing|-
lay|1 commit per missing check, hooks ON, rollback on red|deciding checks, rendering steps|-
seed|repo-owned seed files & shared configs, skip-if-exists|generated hk/nix files|-
confirm|acceptance of a materialized repo: completeness, fidelity, coherence, executability, idempotence|writing anything|-
migrate|lefthook-materialized repo → hk, gated on check-set equivalence|steady-state generation|-
protect|branch protection required contexts from CI job names via `gh api`|local hooks|-
cli|arg dispatch, usage, exit codes|every verb's logic|-

## §N NAV

rel|path|lens
up|.|-
self|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli

## §C CONSTRAINTS

- data flows 1 way (V1). lower node ⊥ imports higher.
- pure fns over values; fs/git/process I/O only in proc (spawning), scan, registry load, hook, gen write, seed write, lay, confirm, migrate, cli. network only in protect.
- ONE glob engine shared by catalog, rule, hook, cover ∴ claim & runner match same way.

## §V INVARIANTS

V1: node deps ! DAG: proc → {scan, registry, catalog} → {detect, rule, hook, legacy} → {cover, map, seed} → {report, gen, protect} → {lay, confirm, migrate} → cli. cycle ⊥

## §T TASKS

id|status|task|cites
T1|.|`src/lib.rs` + `src/main.rs` wiring ∀ node `mod.rs`|V1,`.:T1`
T2|.|shared glob engine: `globset` 0.4 w/ hk's options (`.:R19`)|`.:C`

## §B BUGS

id|date|cause|fix
