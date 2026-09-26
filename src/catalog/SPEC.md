# SPEC

## §G GOAL

Define checks: what each is, how nix provides it, which files its runner sees, how hk runs & fixes it, what its failure says.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/catalog|check definitions: category, nix attr, runner glob, hk step, fix, failure msg
sib|src/scan|enumerate files, basename → exactly 1 file type
sib|src/detect|active fragments from scanned files, fixed order
sib|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt
sib|src/rule|companion rules: exists, mentions, changed, orphan; path templates
sib|src/hook|read effective hk config via `pkl eval`: step → globs

## §C CONSTRAINTS

- row: `id|category|nix|glob|check|fix|msg`. category ∈ `hygiene lint format secret spec test coverage supply`.
- built-in rows seeded from sibling gates (`.:R6`); `.pklith ## checks` overrides by id.
- `glob` = the RUNNER glob; gen emits it verbatim into the hk step.

## §V INVARIANTS

V1: ∀ row: `msg` ! non-empty — feeds AGENTS.md table (`.:V14`) & failure text
V2: `nix` attr resolves in pinned nixpkgs | named flake input; gate test evals ∀ built-in attr
V3: `check` ⊥ empty; `fix` ? `-`
V4: ids unique; built-in order stable (= lay order within category)

## §T TASKS

id|status|task|cites
T1|.|catalog row type + parser shared w/ registry cell rules|V3,V4
T2|.|built-in catalog: hk util hygiene family, ripsecrets, typos, nixfmt, shellcheck, shfmt, taplo, rustfmt, clippy, rubocop, actionlint, zizmor, mth, sherd, itok|V1,`.:R6`
T3|.|test: ∀ built-in nix attr evals|V2

## §B BUGS

id|date|cause|fix
