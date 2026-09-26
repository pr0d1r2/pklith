#!/usr/bin/env bash
# Per-type file size limits, from the legacy packages' measured budgets
# (pklith R8): a flake.lock over 64 KiB means a duplicated input graph, a
# flake.nix over 10 KiB means logic that belongs in its own file. Everything
# else: 500 KiB.
#
#   scripts/no-large-files.sh FILE...
set -euo pipefail

fail=0
for f in "$@"; do
  [ -f "$f" ] || continue
  case "$f" in
    *.lock) max=65536 ;;
    *.nix) max=10240 ;;
    *) max=512000 ;;
  esac
  n="$(wc -c <"$f" | tr -d ' ')"
  if [ "$n" -gt "$max" ]; then
    echo "no-large-files: $f is $n bytes; the limit for its type is $max." >&2
    fail=1
  fi
done
exit "$fail"
