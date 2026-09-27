# SPEC

## §G GOAL

Define checks: what each is, how nix provides it, which files its runner sees, how hk runs & fixes it, what its failure says.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/catalog|check definitions: category, nix attr, runner glob, hk step, fix, failure msg
sib|src/proc|run external programs one way: git hook env scrubbed, failures named
sib|src/scan|enumerate files, basename → exactly 1 file type
sib|src/detect|active fragments from scanned files, fixed order
sib|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt
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

- row: `id|category|nix|glob|check|fix|env|msg`. `env` = vars the step needs & their values, emitted w/ the step (`.:V19`); `-` if none. category ∈ `hygiene secret format lint spec test coverage supply gate`, IN THAT ORDER — the order lay lays them (cheapest & broadest first). `gate` = checks judging the gate itself (`pkli check`, `gen --check`): pass only once ∀ other step laid ∴ last
- built-in rows seeded from sibling gates (`.:R6`); `.pklith ## checks` overrides by id.
- `glob` = the RUNNER glob; gen emits it verbatim into the hk step. `glob` & `env` cells are comma lists; `env` items `KEY=value`. `nix` empty (`-`) = hk runs it natively (`hk util`).
- fragment row: `fragment|triggers|checks|seed`. triggers = file types, paths, globs (read by src/detect); checks = catalog ids; seed = seed file ids (src/seed). built-in order = fragment order.

## §V INVARIANTS

V1: ∀ row: `msg` ! non-empty — feeds AGENTS.md table (`.:V14`) & failure text
V2: `nix` attr resolves in pinned nixpkgs | named flake input; gate test evals ∀ built-in attr
V3: `check` ⊥ empty; `fix` ? `-`
V4: ids unique; built-in order stable (= lay order within category)

## §T TASKS

id|status|task|cites
T1|x|catalog row type + parser shared w/ registry cell rules|V3,V4
T2|x|built-in catalog: hk util hygiene family, ripsecrets, typos, nixfmt, shellcheck, shfmt, taplo, rustfmt, clippy, rubocop, actionlint, zizmor, mth, sherd, itok|V1,`.:R6`
T3|x|test: ∀ built-in nix attr evals|V2
T4|x|`env` column: gen emits it; test: step w/ unset required env ⊥ emitted|`.:V19`
T5|x|test: ∀ built-in `check` resolves to a binary of its `nix` attr; `npx`/`pipx run`/`curl` in a row → rejected|`.:V29`

## §B BUGS

id|date|cause|fix
B1|2026-09-27|built-in `rubocop` w/o `GEM_HOME`: RubyGems falls back to user gem dir (nix store read-only) ∴ loads native gems of another Ruby → crash|row sets `GEM_HOME=/nonexistent`; fixture harness keeps real HOME so the host env is what gets proven
