# SPEC

## §G GOAL

Write repo-owned seed files & shared configs once; never overwrite what the repo grew.

## §N NAV

rel|path|lens
up|.|-
up|src|product code nodes — scan, detect, registry, catalog, rule, hook, legacy, cover, map, report, gen, lay, seed, confirm, migrate, protect, cli
self|src/seed|repo-owned seed files & shared configs, skip-if-exists
sib|src/proc|run external programs one way: git hook env scrubbed, failures named
sib|src/scan|enumerate files, basename → exactly 1 file type
sib|src/detect|active fragments from scanned files, fixed order
sib|src/registry|`.pklith` parse/fmt: types → check sets, rules, plural, exempt
sib|src/catalog|check definitions: category, nix attr, runner glob, hk step, fix, failure msg
sib|src/rule|companion rules: exists, mentions, changed, orphan, tdd-order; path templates
sib|src/hook|read effective hk config via `pkl eval`: step → globs
sib|src/legacy|3 legacy formats: read, `import`, compat entries
sib|src/cover|verdict: type gaps, stale, claim ⊥ backed by runner, rule gaps
sib|src/map|guard mode: changed paths → covering specs
sib|src/report|render matrix & verdict: text, json, md
sib|src/gen|emit `hk.pklith.pkl`, `nix/pklith.nix`, AGENTS.md block; `--check` drift
sib|src/lay|1 commit per missing check, hooks ON, rollback on red
sib|src/confirm|acceptance of a materialized repo: completeness, fidelity, coherence, executability, idempotence
sib|src/migrate|lefthook-materialized repo → hk, gated on check-set equivalence
sib|src/protect|branch protection required contexts from CI job names via `gh api`
sib|src/cli|arg dispatch, usage, exit codes

## §C CONSTRAINTS

- seed set per fragment from catalog `seed` cell: base `.editorconfig` `.gitattributes` README stub & an hk gate (`hk.pkl` running `hk.pklith.pkl`, that module empty, vendored `pkl/Config.pkl`, `.githooks/pre-commit` & `pre-push` executable, refusing w/o hk: `.:V1`) · nix `.envrc` (`# shellcheck shell=bash`, `use flake`) · rubocop `.rubocop.yml` · swiftlint `.swiftlint.yml` (style rules swift-format owns OFF, `force_unwrapping` & `implicitly_unwrapped_optional` ON) · `.gitignore` parts (nix, rust, ruby, swift). ⊥ LICENSE: a license is the owner's choice & its year needs a clock (`.:C`). ⊥ size limits, ⊥ `.typos.toml`: tools run fine w/o them. exemptions live in `.pklith` `exempt` cells, ⊥ separate ledger.
- templates embedded in binary; ⊥ network.
- `--init` also writes `.pklith` from detected fragments: their checks per type, & binary types (images, fonts, archives) as exempt w/ reason `binary asset`. repo edits it after.

## §V INVARIANTS

V1: `--init` skip-if-exists; existing file ⊥ touched (`.:V25`)
V2: `.gitignore` fragments concatenated in fragment order
V3: seeded `.pklith` parses & cover accepts it on the tree as seed & gen leave it (their own files included): ⊥ stale row, ⊥ unbacked claim; a type no check reads is left a gap & named, ⊥ silent exempt (`.:V5`, `.:V11`) — except a type ONLY pklith's own files have, exempt w/ reason `written by pklith (seed or gen)` ∴ a fresh repo is green once seeded (92c8953)
V4: seeded file passes the steps its fragments add, as written: `.envrc` names its shell (direnv runs bash) ∴ shellcheck ⊥ SC2148
V5: known rowless files no active check reads → seeded exempt w/ fixed reason, ⊥ gap: type `gitignore` (`git configuration; hygiene steps only`), `path:flake.lock` at root only (``written by `nix flake lock`; hygiene steps only``). ⊥ broader: other `lock` files (Cargo.lock) stay named. a check reaching the file supersedes: its row replaces the exemption, ⊥ both

## §T TASKS

id|status|task|cites
T1|x|seed writer + skip-if-exists + empty ledger valid|V1,V3,`.:T55`
T2|x|`.gitignore` composition|V2
T3|x|`.pklith` from detected fragments + binary-asset exemptions|V1,`.:V11`
T4|x|swift seeds: `swiftlint` → `.swiftlint.yml`, `ignore-swift` → `/.build/` `/.swiftpm/` `*.profraw`|V1,V2
T5|x|`.envrc` shell directive + shellcheck fixture on the seeded file (#105)|V4
T6|.|known exemptions: `gitignore`, root `flake.lock` (#104)|V5,V3

## §B BUGS

id|date|cause|fix
B1|2026-10-23|#105: seeded `.envrc` = bare `use flake` ∴ catalog shellcheck SC2148 ∴ fresh seed red at `hk check --all`|V4
B2|2026-10-23|#104: `seed --init` on a flake repo left `gitignore` & `lock` (flake.lock) rowless ∴ 1st `pklith check` red; ∀ adopter hand-added the same 2 rows|V5
