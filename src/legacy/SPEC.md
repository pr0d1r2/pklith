# SPEC

## §G GOAL

Read 3 legacy formats & serve compat entries so legacy consumers migrate w/o gate change.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/legacy|3 legacy formats: read, `import`, compat entries
sib|src/scan|enumerate files, basename → exactly 1 file type
sib|src/detect|active fragments from scanned files, fixed order
sib|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt
sib|src/catalog|check definitions: category, nix attr, runner glob, hk step, fix, failure msg
sib|src/rule|companion rules: exists, mentions, changed, orphan; path templates
sib|src/hook|read effective hk config via `pkl eval`: step → globs

## §V INVARIANTS

V1: token set == legacy awk token set ∀ fixture doc (golden, legacy bats ported)
V2: compat exit codes & missing-list lines == legacy (`.:R1` `.ext`/`name`, `.:R2` `.ext`)
V3: `import`: col 2 `-` → exempt, reason = col 3 (Notes); unknown check hint → exempt `legacy: <col2>` ⊥ silent drop
V4: doc parse in-process. ⊥ runtime awk/sed/gawk
V5: read error | 0 tokens from doc w/ table rows → exit 2 "parsed nothing", ⊥ gap list
