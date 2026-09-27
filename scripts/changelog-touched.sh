#!/usr/bin/env bash
# pklith V33 (root T68): a commit that changes product code under src/
# records it under `## [Unreleased]` in CHANGELOG.md. The changelog is how a
# consumer learns that the bytes pkli emits moved; written at release time
# it is reconstructed from memory. Specs and tests are not product code.
#
#   scripts/changelog-touched.sh
set -euo pipefail

product="$(git diff --cached --name-only --diff-filter=ACMRD -- src |
  grep -vE '(^|/)(SPEC\.md|tests\.rs)$|/tests/' || true)"
[ -n "$product" ] || exit 0

unreleased() {
  awk '/^## \[Unreleased\]/ { on = 1; next } /^## / { on = 0 } on'
}
now="$(git show :CHANGELOG.md 2>/dev/null | unreleased)"
before="$(git show HEAD:CHANGELOG.md 2>/dev/null | unreleased)"
if [ "$now" = "$before" ]; then
  echo "changelog-touched: product code changed with no new entry under ## [Unreleased] in CHANGELOG.md (V33):" >&2
  printf '%s\n' "$product" | sed 's/^/  /' >&2
  echo "  fix: add a line saying what a consumer will see change, and stage CHANGELOG.md." >&2
  exit 1
fi
