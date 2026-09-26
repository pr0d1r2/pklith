# SPEC

## §G GOAL

Derive the ordered list of active fragments from the scanned files.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/detect|active fragments from scanned files, fixed order
sib|src/scan|enumerate files, basename → exactly 1 file type
sib|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt
sib|src/catalog|check definitions: category, nix attr, runner glob, hk step, fix, failure msg

## §C CONSTRAINTS

- fragment triggers declared in catalog (file types, paths, globs), ⊥ prose-only triggers.
- order = catalog fragment order, ⊥ discovery order.

## §V INVARIANTS

V1: sole detector (`.:V16`); ⊥ other node re-derives fragments
V2: same files → same fragment list, same order
V3: `--root` & git sources give identical result on same tree

## §T TASKS

id|status|task|cites
T1|.|detector + golden tests ported from `detect-fragments.sh` bats|V1,V2,`.:T52`

## §B BUGS

id|date|cause|fix
B1|2026-07-15|set-and-setting `B32`: `grep -q` under `pipefail` in detector ∴ spurious failure|V2
