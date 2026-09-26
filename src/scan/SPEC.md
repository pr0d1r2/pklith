# SPEC

## §G GOAL

Enumerate repo files & classify each basename to exactly 1 file type.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/scan|enumerate files, basename → exactly 1 file type
sib|src/proc|run external programs one way: git hook env scrubbed, failures named
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
sib|src/seed|repo-owned seed files & shared configs, skip-if-exists
sib|src/confirm|acceptance of a materialized repo: completeness, fidelity, coherence, executability, idempotence
sib|src/migrate|lefthook-materialized repo → hk, gated on check-set equivalence
sib|src/protect|branch protection required contexts from CI job names via `gh api`
sib|src/cli|arg dispatch, usage, exit codes

## §C CONSTRAINTS

- default source = `git ls-files -z` (NUL-safe). `--root DIR` → walk, skip `.git/`.
- type kinds: `bare` (basename w/o dot: `justfile`, `Makefile`) · `dotfile` (`.envrc`) · `name` (whole basename w/ dots: `Cargo.lock`) · `compound` (`tar.gz`) · `ext` (last suffix `gz`). kind recorded, ⊥ lost.
- keys drop ONE leading `.` (`.envrc` → `envrc`, `.typos.toml` → `typos.toml`), as the legacy tools did (`.:R3`). compound suffixes exclude the whole name ∴ keys never repeat.

## §V INVARIANTS

V1: ∀ file → candidate types ordered: most specific `path:` class (longest literal prefix) > exact basename > longest compound suffix > last ext. first DECLARED wins (`.:V5`)
V2: dotfile `.envrc` ≡ legacy key `envrc` (`.:R1` parity, `.:V4`)
V3: submodule gitlinks & symlinks-to-dir ⊥ files (`.:R4`)
V4: non-UTF-8 path → error naming it, ⊥ lossy skip
V5: output sorted, per type: file list + count

## §T TASKS

id|status|task|cites
T1|x|`git ls-files -z` source + `--root` walk source|V3,V4
T2|x|basename → candidate types fn + table tests vs legacy sed outputs|V1,V2,`.:R1`,`.:R2`
T3|.|edge cases: multi-dot `foo.spec.ts`, dotfiles `.gitignore`/`.editorconfig`, empty repo, name w/ newline|V1,V2,`.:R8`
T4|.|golden: output order stable across git & walk sources|V5

## §B BUGS

id|date|cause|fix
