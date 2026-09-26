#!/usr/bin/env bash
# Per-file token ceilings (pklith root T26). sherd measures every `.rs` file
# against its V50 (4,000 tokens of code, 2,000 of tests) but only ADVISES:
# `sherd check` exits 0 over a ceiling. This makes the measurement binding.
#
#   scripts/file-ceilings.sh
set -euo pipefail

out="$(sherd check 2>&1)"
over="$(printf '%s\n' "$out" | sed -n 's/.* \([0-9][0-9]*\) over ceiling.*/\1/p')"
# No count printed means nothing was checked, which must not pass.
[ -n "$over" ] || { echo 'file-ceilings: sherd printed no ceiling count, so nothing was checked.' >&2; exit 1; }
if [ "$over" != 0 ]; then
  printf '%s\n' "$out" | grep 'sherd/V50' >&2
  echo "file-ceilings: $over file(s) over their token ceiling. Split the file; a raise is a reviewed decision." >&2
  exit 1
fi
