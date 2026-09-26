#!/usr/bin/env bash
# The commit message rule (pklith §C, T4): subject `type(scope): claim`, at
# most 72 characters, and a body that says why.
#
#   scripts/commit-msg.sh [MESSAGE_FILE]    (default: git's COMMIT_EDITMSG)
set -euo pipefail

f="${1:-$(git rev-parse --git-path COMMIT_EDITMSG)}"
# A missing file means nothing was checked, which is a failure, not a pass.
[ -f "$f" ] || { echo "commit-msg: no message file at $f, so nothing was checked." >&2; exit 1; }

# git strips comment lines before writing the commit; judge what it keeps.
msg="$(grep -v '^#' "$f" || true)"
subject="$(printf '%s\n' "$msg" | sed -n 1p)"

# git or the operator writes these; they are not held to the style.
case "$subject" in "Merge "* | "Revert "* | "fixup! "* | "squash! "*) exit 0 ;; esac

types='feat|fix|docs|test|chore|refactor|perf|build|ci'
if ! printf '%s\n' "$subject" | grep -Eq "^($types)(\([a-z0-9/._-]+\))?!?: [^ ]"; then
  echo "commit-msg: subject must be 'type(scope): claim', type one of ${types//|/ }. Got: $subject" >&2
  exit 1
fi

if [ "${#subject}" -gt 72 ]; then
  echo "commit-msg: subject is ${#subject} characters, the limit is 72. Say less, or move detail to the body." >&2
  exit 1
fi

body="$(printf '%s\n' "$msg" | sed -n '2,$p' | grep -v '^Co-Authored-By:' | grep -c '[^[:space:]]' || true)"
if [ "$body" -eq 0 ]; then
  echo "commit-msg: the body is empty. Say WHY: the diff already shows what changed." >&2
  exit 1
fi
