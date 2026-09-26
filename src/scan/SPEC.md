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

## §T TASKS

id|status|task|cites
T1|.|`git ls-files -z` source + `--root` walk source|V3,V4
T2|.|basename → candidate types fn + table tests vs legacy sed outputs|V1,V2,`.:R1`,`.:R2`
T3|.|edge cases: multi-dot `foo.spec.ts`, dotfiles `.gitignore`/`.editorconfig`, empty repo, name w/ newline|V1,V2,`.:R8`

## §B BUGS

id|date|cause|fix
