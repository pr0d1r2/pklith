# SPEC

## §G GOAL

Product code hub. Pipeline: scan → detect; registry + catalog → rule, hook, legacy → cover, map → report, gen → lay; seed, confirm, migrate, protect beside. cli dispatches.

## §F FEDERATION

dir|owns|⊥owns|tokens

## §N NAV

rel|path|lens
up|.|-
self|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli

## §V INVARIANTS

V1: node deps ! DAG: {scan, registry, catalog} → {detect, rule, hook, legacy} → {cover, map, seed} → {report, gen, protect} → {lay, confirm, migrate} → cli. cycle ⊥
