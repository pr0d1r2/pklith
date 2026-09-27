#!/usr/bin/env bash
# pklith V33 (root T68): a commit that changes what pklith does records it
# under `## [Unreleased]` in CHANGELOG.md. The changelog is how a consumer
# learns that the bytes pkli emits moved; written at release time it is
# reconstructed from memory.
#
# "Changes what pklith does" is a `feat`, `fix` or `perf` commit touching
# product code under src/ (specs and test files are not product code). It
# runs at commit-msg, because only the message says which kind a change
# is: a test or refactor that edits a module's inline tests needs no entry.
#
#   scripts/changelog-touched.sh [MESSAGE_FILE]
#
# The hook passes the file git gave it; by hand it defaults to git's
# COMMIT_EDITMSG.
set -euo pipefail

f="${1:-$(git rev-parse --git-path COMMIT_EDITMSG)}"
[ -f "$f" ] || { echo "changelog-touched: no message file at $f, so nothing was checked." >&2; exit 1; }
subject="$(grep -v '^#' "$f" | sed -n 1p)"
printf '%s\n' "$subject" | grep -Eq '^(feat|fix|perf)(\(|!|:)' || exit 0

product="$(git diff --cached --name-only --diff-filter=ACMRD -- src |
  grep -vE '(^|/)(SPEC\.md|tests\.rs)$|/tests/' || true)"
[ -n "$product" ] || exit 0

# The `## [Unreleased]` section of CHANGELOG.md at REV (`:` is the index).
unreleased() {
  git show "$1CHANGELOG.md" 2>/dev/null | awk '/^## \[Unreleased\]/ { on = 1; next } /^## / { on = 0 } on'
}
now="$(unreleased :)"
before="$(unreleased HEAD:)"
[ "$now" = "$before" ] || exit 0

# An amend: HEAD is the commit being rewritten. When it brought its own
# entry and the message keeps its subject, that entry still stands.
if [ "$(git log -1 --format=%s 2>/dev/null)" = "$subject" ] &&
  [ "$before" != "$(unreleased HEAD~1:)" ]; then
  exit 0
fi
echo "changelog-touched: a ${subject%%[(:!]*} commit changed product code with no new entry under ## [Unreleased] in CHANGELOG.md (V33):" >&2
printf '%s\n' "$product" | sed 's/^/  /' >&2
echo "  fix: add a line saying what a consumer will see change, and stage CHANGELOG.md." >&2
exit 1
