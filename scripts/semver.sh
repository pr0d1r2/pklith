#!/usr/bin/env bash
# pklith root T41 (V33): the public API changes no more than the version
# says. cargo-semver-checks compares the crate with the last release tag;
# a removed or narrowed item without a major bump (minor, before 1.0) is
# refused. A shallow clone may lack the tags, so it is refused too. Before
# the first release there is no baseline, and the step says so rather than
# passing silently. At 0.0.z every change is allowed, which is semver's
# rule, not a gap here.
#
#   scripts/semver.sh
set -euo pipefail

command -v cargo-semver-checks >/dev/null 2>&1 || {
  echo "semver: cargo-semver-checks is not on PATH; re-enter the dev shell. A MISSING TOOL, not a finding." >&2
  exit 1
}
if [ "$(git rev-parse --is-shallow-repository)" = true ]; then
  echo "semver: this clone is shallow, so the release tags may be missing; fetch full history (actions/checkout fetch-depth: 0)" >&2
  exit 1
fi
if ! baseline="$(git describe --tags --abbrev=0 --match 'v[0-9]*' 2>/dev/null)"; then
  echo "semver: no release tag yet, so no baseline; the first release (T69) makes one" >&2
  exit 0
fi
cargo semver-checks check-release --baseline-rev "$baseline"
