# SPEC

## §G GOAL

Define checks: what each is, how nix provides it, which files its runner sees, how hk runs & fixes it, what its failure says.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/catalog|check definitions: category, nix attr, runner glob, hk step, fix, failure msg
sib|src/scan|enumerate files, basename → exactly 1 file type
sib|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt

## §V INVARIANTS

V1: ∀ row: `msg` ! non-empty — feeds AGENTS.md table (`.:V14`) & failure text
V2: `nix` attr resolves in pinned nixpkgs | named flake input; gate test evals ∀ built-in attr
V3: `check` ⊥ empty; `fix` ? `-`
V4: ids unique; built-in order stable (= lay order within category)
