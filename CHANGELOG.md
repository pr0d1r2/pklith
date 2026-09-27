# Changelog

All notable changes to pklith are recorded here, in the
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format. The
project follows [Semantic Versioning](https://semver.org/): a change to
the bytes pkli emits for the same inputs is at least a minor version
(root V33), and its entry names the files consumers will see change.

## [Unreleased]

### Added

- `pkli check`, `report`, `map`, `detect`, `gen`, `lay`, `seed`,
  `confirm`, `migrate`, `protect` and `import`.
- The built-in catalog of checks, and `.pklith` rows that replace or
  extend it.
- Compat entries: `pkli` installed as `lefthook-linter-coverage`,
  `lefthook-linter-coverage-full` or `lefthook-unit-coverage` behaves as
  that tool (`packages.compat`).
- `lib.devShell` for consumers, and `packages.default`.

### Fixed

- `pkli lay` lays, in one run, the checks claimed for files it generates
  itself (such as `nix/pklith.nix`), so a second run lays nothing and
  `pkli check` passes after the first.
- The built-in `clippy` check lints every workspace member
  (`cargo clippy --workspace`), not only the root package; a lint in a
  member crate used to pass. Consumers see the step's command change in
  `hk.pklith.pkl`.
- `pkli check` and `pkli seed --init` no longer take time in the square of
  the file count when suggesting rows: on 10,000 files, 39.5 s became 0.6 s.
- `pkli seed --init` no longer writes a binary type's row twice when its
  files are interleaved with another binary type's, which left a
  `.pklith` pkli could not read.
- `pkli seed --init` types the files seed and gen write too, so `seed
  --init`, `gen`, `check` is green on a fresh repository; types only pkli's
  own files have are exempt as "written by pkli (seed or gen)".
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
