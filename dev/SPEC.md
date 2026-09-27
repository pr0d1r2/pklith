# SPEC

## §G GOAL

Tooling that maintains THIS repository & ships to nobody: `pklith-dev`, a `publish = false` workspace member. generated docs & their drift checks, as tested Rust ⊥ shell.

## §N NAV

rel|path|lens
up|.|-
self|dev|`pklith-dev`: generated docs of THIS repo & their drift checks, `publish = false`
sib|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli

## §C CONSTRAINTS

- ⊥ published. a consumer of `pkli` ! ever receive these verbs; a 2nd `[[bin]]` in the main crate would land on their PATH (sherd dev §C).
- pure fn over `&str` ∀ parser & renderer; `main.rs` alone reads the repo, runs `git`/`pkl` & writes ∴ ∀ rule testable w/ ⊥ a repository.
- exit codes = pkli's: 0 clean · 1 drift · 2 usage | I/O.
- same lint limits & coverage floor as the product crate (`[workspace.lints]`, `sherd coverage --workspace`).

## §V INVARIANTS

V1: EVERY generated number comes from the file that OWNS it — `Cargo.toml`, `hk.pkl` (via `pkl`), `.coverage`, `flake.lock`, `ci.yml`, the `SPEC.md` set. a value that reads empty | zero → exit 2 naming the source, ⊥ a default
V2: a badge's alt text & URL come from ONE value (`{name}` & `{name|url}` in one template) ∴ they cannot disagree
V3: PLATFORM badges come from `ci.yml`'s matrix, ⊥ `flake.nix`'s systems. an unknown runner → error, ⊥ a guessed platform
V4: a spec COUNT is rows, ⊥ distinct ids. ∀ node numbers its own from V1 & B1
V5: numbers `docs/LLM-DISCLAIMER.md` quotes in prose are held to the same facts; drift → exit 1 before anything is written
V6: render is idempotent: splice(splice(x)) == splice(x) ∴ `--check` is equality, ⊥ a heuristic

## §T TASKS

id|status|task|cites
T1|x|`readme`: README badge block + disclaimer numbers, replacing `scripts/readme-badges.sh`|V1,V2,V3,V4,V5,V6

## §B BUGS

id|date|cause|fix
B1|2026-09-27|shell badge generator counted distinct V & B ids across nodes: 36 invariants & 12 bugs for 119 & 21 (1dd1832). copied from a sibling; unit-untested ∴ looked right until quoted in prose|V4, & the move to Rust w/ tests
