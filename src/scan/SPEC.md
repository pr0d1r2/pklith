# SPEC

## §G GOAL

Enumerate repo files & classify each basename to exactly 1 file type.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/scan|enumerate files, basename → exactly 1 file type

## §C CONSTRAINTS

- default source = `git ls-files -z` (NUL-safe). `--root DIR` → walk, skip `.git/`.
- type kinds: `bare` (basename w/o dot: `justfile`, `Makefile`) · `ext` (last suffix `gz`) · `compound` (`tar.gz`) · `dotfile` (`.envrc`). kind recorded, ⊥ lost.

## §V INVARIANTS

V1: ∀ file → candidate types ordered: exact basename > longest compound suffix > last ext. first DECLARED wins (`.:V5`)
V2: dotfile `.envrc` ≡ legacy key `envrc` (`.:R1` parity, `.:V4`)
V3: submodule gitlinks & symlinks-to-dir ⊥ files (`.:R4`)
V4: non-UTF-8 path → error naming it, ⊥ lossy skip
V5: output sorted, per type: file list + count
