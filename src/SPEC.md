# SPEC

## §G GOAL

Product code hub. Pipeline: scan → detect; registry + catalog → rule, hook, legacy → cover, map → report, gen → lay; seed, confirm, migrate, protect beside. cli dispatches.

## §F FEDERATION

dir|owns|⊥owns|tokens
scan|enumerate files, basename → exactly 1 file type|registry syntax, verdicts|-

## §N NAV

rel|path|lens
up|.|-
self|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli

## §C CONSTRAINTS

- data flows 1 way (V1). lower node ⊥ imports higher.
- pure fns over values; fs/git/process I/O only in scan, registry load, hook, gen write, lay, cli.
- ONE glob engine shared by catalog, rule, hook, cover ∴ claim & runner match same way.

## §V INVARIANTS

V1: node deps ! DAG: {scan, registry, catalog} → {detect, rule, hook, legacy} → {cover, map, seed} → {report, gen, protect} → {lay, confirm, migrate} → cli. cycle ⊥

## §T TASKS

id|status|task|cites
T1|.|`src/lib.rs` + `src/main.rs` wiring ∀ node `mod.rs`|V1,`.:T1`
T2|.|shared glob engine; dep (`globset`) ? vs hand-rolled — decide & record in `.:R`|`.:C`

## §B BUGS

id|date|cause|fix
