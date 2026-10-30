# Changelog

All notable changes to pklith are recorded here, in the
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format. The
project follows [Semantic Versioning](https://semver.org/): a change to
the bytes pklith emits for the same inputs is at least a minor version
(root V33), and its entry names the files consumers will see change.

## [Unreleased]

### Changed

- `pklith gen` writes `test`-category checks into the `push` mapping of
  `hk.pklith.pkl`, beside `coverage`, instead of `steps`: a seeded gate
  runs the test suite at pre-push and `hk check`, not on every commit, so
  a failing test can be committed before its fix (#54). The seeded
  `hk.pkl` comment says so. A hand-written `hk.pkl` that amends a test
  step by its id in the commit steps must amend it from `push` instead.

### Added

- Swift catalog support: `swift-format` and `swiftlint` now run for
  `Package.swift` as well as Swift sources; the activated `swift` fragment
  checks the manifest too.
- Built-in check `cargo-deployment-target` (lint, catalog T13, paradigm
  T18 and V28), emitted in the commit `steps` of `hk.pklith.pkl`: with a
  `Cargo.toml` at the repository root, `.cargo/config.toml` must exist
  and its `[env]` must set `MACOSX_DEPLOYMENT_TARGET` to the macOS floor,
  14.0, with `force = true`, as an inline table or a
  `[env.MACOSX_DEPLOYMENT_TARGET]` table. taplo parses the file. A
  missing file, a missing key, another value or a plain string fails,
  naming the file and the line it must hold: nix's darwin stdenv exports
  its own `MACOSX_DEPLOYMENT_TARGET` in the dev shell and in
  `buildRustPackage`, which an `[env]` value without `force` does not
  override. Without a root `Cargo.toml` it passes with a notice. Its
  package is `legacyPackages.<system>.catalog.cargo-deployment-target`.
  It joins the `rust` fragment, so `pklith seed --init` in a Rust
  repository claims it for `Cargo.toml` (and for `.cargo/config.toml`),
  and `nix/pklith.nix` lists `catalog.cargo-deployment-target`.
- Seed file `.cargo/config.toml` (seed `cargo-config`, seed T7) for the
  `rust` fragment, holding that `[env]` pin. `pklith seed` writes it
  only where it is missing; an existing one is never touched, and when
  it lacks the seeded line seed says so on stderr and still exits 0.
  This file is the one place pklith keeps the floor: the nix packages of
  both cargo checks read it.
- Built-in check `cargo-universal` (test, catalog T11, paradigm V28),
  emitted in the `push` mapping of `hk.pklith.pkl`: on macOS every bin
  target of the cargo workspace is built in release, with
  `MACOSX_DEPLOYMENT_TARGET` at the macOS floor, for
  `aarch64-apple-darwin` and `x86_64-apple-darwin`, joined by `lipo` into
  `target/universal-apple-darwin/release/`, and must hold exactly the
  arm64 and x86_64 slices, each `minos` 14.0. Off macOS, or with no bin
  target, it passes with a notice. Its package,
  `legacyPackages.<system>.catalog.cargo-universal`, carries the other
  Mac's std from the same nixpkgs revision. It joins the `rust`
  fragment, so `pklith seed --init` in a Rust repository now claims it
  for `rs` and `Cargo.toml`, and `nix/pklith.nix` lists
  `catalog.cargo-universal`.
- x86_64-darwin (Intel macOS) in the flake's systems: `lib.devShell`,
  `legacyPackages.x86_64-darwin.catalog`, `packages.x86_64-darwin` and
  `checks.x86_64-darwin` now exist, so a consumer's `nix develop` starts
  there instead of failing with the supported list. microlith and itok
  publish no x86_64-darwin package, so the catalog builds them from their
  locked source on that system. It has no CI job and no cached binaries,
  and nixpkgs 26.05 is the last nixpkgs release that carries it.
- Built-in checks `bats-changed` and `bats-unit` (test, nix `bats`, #97),
  emitted in the `push` mapping of `hk.pklith.pkl`. `bats-changed` runs on
  changed `**/*.sh` and `**/*.bats`: the spec `pklith map` finds for each
  through the `exists` rules, and each changed `.bats` file, printing only
  the TAP lines that are not `ok`; a changed script whose spec is missing
  fails. `bats-unit` runs `bats --recursive tests` over the whole tree.
  Both join the `bats` fragment, so `pklith seed --init` in a repository
  with `.bats` files now claims them and `nix/pklith.nix` keeps `bats`.
  `pklith migrate` maps lefthook commands of both names instead of
  refusing them.
- `## rules` kind `tdd-order` (#65): every commit that changes a selected
  file changes its target in that commit or an earlier one. It runs only
  over commits: `pklith check --pushed` (new: the commits on no
  remote-tracking ref), `--range A..B`, or `--staged` (the commit being
  made, after the unpushed ones). Commits before those are never judged,
  so there is no baseline file; a deletion demands nothing and a rename
  follows the new name. A failure names the file, its target and the
  commit. #88's SwiftPM `swift-test` select and target work unchanged.
- `pklith::scan::history`: the commits of a range, of `HEAD --not
  --remotes` (what a push is about to send), or those plus the index,
  oldest first with merges skipped, each with the paths it changed and
  deleted; a rename deletes its old path and changes its new one.
- Built-in check `swift-format` (format): `swift-format lint --strict` on
  `**/*.swift` and `Package.swift`, fixed by `swift-format format --in-place`.
- Built-in fragment `swift`, switched on by `Package.swift` or any `.swift`
  file; `pklith seed` adds `/.build/`, `/.swiftpm/` and `*.profraw` to
  `.gitignore`.
- Built-in check `swiftlint` (lint, in the `swift` fragment):
  `swiftlint lint --strict --quiet` on `**/*.swift` and `Package.swift`.
  `pklith seed` writes a
  `.swiftlint.yml` that turns off `opening_brace`, `trailing_comma`,
  `line_length` and `identifier_name` and opts in to `force_unwrapping`
  and `implicitly_unwrapped_optional`.

### Fixed

- `pklith seed` writes `.envrc` as `# shellcheck shell=bash` then
  `use flake`, instead of a bare `use flake` that the catalog's
  `shellcheck` step refused with SC2148, so a fresh seed is no longer red
  at `hk check --all` (#105). Consumers see the new first line in a newly
  seeded `.envrc`; an existing one is never rewritten, and one that still
  lacks a directive needs it added by hand.
- On darwin the catalog's `swiftlint` is a wrapper (`nix/swiftlint.nix`,
  applied by `nix/overlay.nix`, so `legacyPackages.<system>.catalog` and
  the `nix/pklith.nix` a consumer's `lib.devShell` imports both get it)
  that points `TOOLCHAIN_DIR` at the host's Xcode toolchain or Command
  Line Tools before running upstream swiftlint (#108). Upstream aborted
  with `Loading sourcekitdInProc.framework ... failed` inside a nix
  shell, whose `DEVELOPER_DIR` names the nix apple-sdk, even where the
  host has the framework. A host with neither now gets a `MISSING TOOL`
  line naming `sourcekitdInProc.framework` and `xcode-select --install`,
  and the step stays red. A `TOOLCHAIN_DIR` already set is kept. Linux
  is unchanged.
- `pklith seed --init` writes `gitignore|-|-|git configuration; hygiene
  steps only` and ``path:flake.lock|-|-|written by `nix flake lock`;
  hygiene steps only`` into the first `.pklith` when those files are
  tracked and no active check reads them, so a freshly seeded flake
  repository passes `pklith check` without hand edits (#104). Other lock
  files, such as `Cargo.lock`, and a nested `flake.lock` stay named gaps.
  A check that reaches `flake.lock` takes its row instead of the
  exemption. Consumers see the two rows in a newly seeded `.pklith`; an
  existing one is never rewritten.
- Built-in `bats-unit` no longer passes having run no test (#102). On a
  case-insensitive filesystem, the macOS default, `bats --recursive tests`
  also opens a SwiftPM `Tests/` directory, finds no `.bats` file, prints
  `1..0` and exits 0; a `tests/` with no `.bats` file did the same
  anywhere. The step now runs bats with the TAP formatter and fails on a
  `1..0` plan, saying no bats tests were found under `tests/`, and on a
  failure prints only the TAP lines that are not `ok`. `bats-changed`
  likewise fails when the specs it runs hold no test. Consumers see both
  steps' `check` change in `hk.pklith.pkl`, and `bats-unit`'s message in
  the AGENTS.md table.
- pklith's own gate: `scripts/hk-parity.sh` refuses a step with no
  `check`, `check_diff`, `check_list_files` or `fix` in any hook of the
  evaluated `hk.pkl`, naming the hook and step, instead of filtering it
  (#92). An id amended into a mapping that lacks it is such a step (#91).

## [0.1.0] - 2026-09-27

### Changed

- The command is `pklith`, the same name as the crate, not `pkli`: every
  verb, message and generated file header says `pklith`. `pkli` was one
  keystroke from Pkl's own `pkl` and would have been a second crate name.

### Added

- `pklith check`, `report`, `map`, `detect`, `gen`, `lay`, `seed`,
  `confirm`, `migrate`, `protect` and `import`.
- The built-in catalog of checks, and `.pklith` rows that replace or
  extend it.
- Compat entries: `pklith` installed as `lefthook-linter-coverage`,
  `lefthook-linter-coverage-full` or `lefthook-unit-coverage` behaves as
  that tool (`packages.compat`).
- `lib.devShell` for consumers, and `packages.default`.

### Fixed

- `pklith lay` lays, in one run, the checks claimed for files it generates
  itself (such as `nix/pklith.nix`), so a second run lays nothing and
  `pklith check` passes after the first.
- The built-in `clippy` check lints every workspace member
  (`cargo clippy --workspace`), not only the root package; a lint in a
  member crate used to pass. Consumers see the step's command change in
  `hk.pklith.pkl`.
- `pklith check` and `pklith seed --init` no longer take time in the square of
  the file count when suggesting rows: on 10,000 files, 39.5 s became 0.6 s.
- `pklith seed --init` no longer writes a binary type's row twice when its
  files are interleaved with another binary type's, which left a
  `.pklith` pklith could not read.
- `pklith seed --init` types the files seed and gen write too, so `seed
  --init`, `gen`, `check` is green on a fresh repository; types only pklith's
  own files have are exempt as "written by pklith (seed or gen)".
  A file the repository already had is left to it, never exempted as
  pklith's own.
- The compat entries list files as the legacy tools' `git ls-files` did:
  in the caller's git environment (`GIT_DIR`, `GIT_WORK_TREE`,
  `GIT_INDEX_FILE`), symlinks and submodules included; a git that fails
  or is missing is exit 2 instead of an empty list that passed.
- With a `*_ROOT` variable the compat entries walk as `find . -type f`
  did: regular files only, and only the top-level `.git/` skipped.
- A linter document is read as the legacy awk read it: a row whose first
  cell has no closing `|` lists nothing, and backtick pairs match leftmost.
- `lefthook-unit-coverage` reads configs with numbers, dates, `[table]`s,
  dotted keys and every string escape, as the legacy tool did; only
  multi-line strings and inline tables are refused, by line.
- A `[[rules]]` entry without `test_dir` is a config error (exit 2),
  where tests were looked for at the filesystem root.
- A `*_ROOT` naming no directory fails first with exit 1, as the legacy
  `cd "$ROOT" || exit 1` did.
