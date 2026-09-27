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
