# SPEC

## §G GOAL

Product code hub. Pipeline: scan → detect; registry + catalog → rule, hook, legacy → cover, map → report, gen → lay; seed, confirm, migrate, protect beside. cli dispatches.

## §F FEDERATION

dir|owns|⊥owns|tokens

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
