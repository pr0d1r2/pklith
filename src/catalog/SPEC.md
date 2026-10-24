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
sib|src/rule|companion rules: exists, mentions, changed, orphan, tdd-order; path templates
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

- row: `id|category|nix|glob|check|fix|env|msg`. `env` = vars the step needs & their values, emitted w/ the step (`.:V19`); `-` if none. category ∈ `hygiene secret format lint spec test coverage supply gate`, IN THAT ORDER — the order lay lays them (cheapest & broadest first). `gate` = checks judging the gate itself (`pklith check`, `gen --check`): pass only once ∀ other step laid ∴ last
- built-in rows seeded from sibling gates (`.:R6`); `.pklith ## checks` overrides by id.
- `glob` = the RUNNER glob; gen emits it verbatim into the hk step. `glob` & `env` cells are comma lists; `env` items `KEY=value`, value ⊥ comma. `nix` empty (`-`) = hk runs it natively (`hk util`).
- darwin swiftlint needs host `sourcekitdInProc`, nix ships none ∴ attr wraps it, `TOOLCHAIN_DIR` ← host `xcode-select -p`; linux unwrapped.
- `test` rows run a repo's own suite ∴ `src/gen:V8` → push. `bats-changed` (`**/*.sh` `**/*.bats`): companion `.bats` of ∀ changed file via `pklith map` (`exists` rule templates) + ∀ changed `.bats` itself; TAP lines ⊥ `ok` printed ∴ failures only; companion missing → red (`src/map:V1`). `bats-unit` (whole tree): `bats --recursive tests`, failures only. both: 0 tests run (plan `1..0`) → red (B3)
- `cargo-universal` (test; paradigm#25's universal rule): ∀ workspace bin, release builds for `aarch64-apple-darwin` & `x86_64-apple-darwin`, `lipo`-joined → exactly arm64 + x86_64, each slice `minos` 14.0 (`otool -l`; `vtool` ⊥ runs under nix). env `MACOSX_DEPLOYMENT_TARGET=14.0`. non-Darwin | no bin → notice, exit 0. nixpkgs rustc: host std only ∴ other Mac's std from same-rev rustc of that system, `--sysroot`. push vs release-only = paradigm's open question
- fragment row: `fragment|triggers|checks|seed`. triggers = file types, paths, globs (read by src/detect); checks = catalog ids; seed = seed file ids (src/seed). built-in order = fragment order.

## §V INVARIANTS

V1: ∀ row: `msg` ! non-empty — feeds AGENTS.md table (`.:V14`) & failure text
V2: `nix` attr resolves in pinned nixpkgs | named flake input; gate test evals ∀ built-in attr
V3: `check` ⊥ empty; `fix` ? `-`
V4: ids unique; built-in order stable (= lay order within category)
V5: darwin `swiftlint` ⊥ host `sourcekitdInProc` → exit ≠ 0, MISSING TOOL named (`.:V1`), ⊥ abort as finding

## §T TASKS

id|status|task|cites
T1|x|catalog row type + parser shared w/ registry cell rules|V3,V4
T2|x|built-in catalog: hk util hygiene family, ripsecrets, typos, nixfmt, shellcheck, shfmt, taplo, rustfmt, clippy, rubocop, actionlint, zizmor, mth, sherd, itok|V1,`.:R6`
T3|x|test: ∀ built-in nix attr evals|V2
T4|x|`env` column: gen emits it; test: step w/ unset required env ⊥ emitted|`.:V19`
T5|x|test: ∀ built-in `check` resolves to a binary of its `nix` attr; `npx`/`pipx run`/`curl` in a row → rejected|`.:V29`
T6|x|fragment rows: parse, non-empty (`.:V18`), built-in fragments ported from set-and-setting + completeness test|`.:V16`,`.:V17`,`.:V18`
T7|x|swift: rows `swift-format` (format, nix `swift-format`) & `swiftlint` (lint, nix `swiftlint`), fragment `swift` (triggers `**/Package.swift`, `**/*.swift`; seed `swiftlint`, `ignore-swift`); 1 fixture per row|V1,V2,V4,`.:V17`,`.:V30`
T8|.|`.context-limits` has 2 readings: fragment `spec` row `sherd-budget` = chain ceilings, fragment `context` row `itok` = per-file ceilings; a repo w/ both fragments active cannot pass both. pick 1 meaning per file (e.g. itok reads its own path, `.:T71`), then test: repo w/ `SPEC.md` + `.context-limits` → ⊥ 2 rows reading 1 file 2 ways. open: which (issue #94)|B2,`.:B11`,`.:V9`
T9|x|test rows `bats-changed` & `bats-unit` (nix `bats`, issue #97); `bats` fragment gains both (`.:V17`; PORT: `bats-unit` mapped, ⊥ dropped); 1 fixture per row + companion-only case through the built binary|V1,V2,V4,`.:V17`,`.:V30`,`src/gen:V8`
T10|x|0 tests → red in both bats rows (#102); built-binary test per B3 cause + failures only|V1,`.:V30`
T11|x|`cargo-universal` row (nix/cargo-universal.nix), joins `rust` fragment; fixture: toy bin crate, planted `[env]` floor 26.0 (macOS only); ∀ host: non-Darwin & lib-only skip, canned `lipo`/`otool` parse|V1,V2,V4,`.:V17`,`.:V30`,`src/gen:V8`
T12|x|darwin `swiftlint` wrapper (#108)|V5

## §B BUGS

id|date|cause|fix
B1|2026-09-27|built-in `rubocop` w/o `GEM_HOME`: RubyGems falls back to user gem dir (nix store read-only) ∴ loads native gems of another Ruby → crash|row sets `GEM_HOME=/nonexistent`; fixture harness keeps real HOME so the host env is what gets proven
B2|2026-10-18|age-plugin-se PR #5 (open, head `8b4151a`): seeded gate activates `spec` & `context` on 1 `.context-limits` holding sherd chain ceilings; `itok` reads them per FILE & refuses root `SPEC.md` ∴ itok row deferred. root B11 = same clash in pklith's own gate, fragment-level ⊥ fixed|T8 open
B3|2026-10-23|SwiftPM `Tests/` on a case-insensitive FS (macOS): `tests` opens it, `1..0`, exit 0 ∴ `bats-unit` green-by-nothing (age-plugin-se #21); `tests/` w/o `.bats` likewise|T10
B4|2026-10-23|#108: darwin swiftlint aborted: ⊥ host `sourcekitdInProc` found ∴ gate cut short|V5
