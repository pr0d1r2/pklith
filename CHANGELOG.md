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

- `pkli check` and `pkli seed --init` no longer take time in the square of
  the file count when suggesting rows: on 10,000 files, 39.5 s became 0.6 s.
- `pkli seed --init` no longer writes a binary type's row twice when its
  files are interleaved with another binary type's, which left a
  `.pklith` pkli could not read.
- `pkli seed --init` types the files seed and gen write too, so `seed
  --init`, `gen`, `check` is green on a fresh repository; types only pkli's
  own files have are exempt as "written by pkli (seed or gen)".
