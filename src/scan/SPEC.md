# SPEC

## §G GOAL

Enumerate repo files & classify each basename to exactly 1 file type.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/scan|enumerate files, basename → exactly 1 file type
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
sib|src/seed|repo-owned seed files & shared configs, skip-if-exists
sib|src/protect|branch protection required contexts from CI job names via `gh api`

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
