#!/usr/bin/env bash
# pklith §C: ONE nixpkgs rev. A second nixpkgs node means an input stopped
# following nixpkgs-lock. The node ceiling catches graph growth before it
# compounds (a legacy lock once carried 59 nodes, R8); raising it is a
# reviewed commit with a reason.
#
#   scripts/flake-lock-graph.sh FLAKE_LOCK...
set -euo pipefail

ceiling=12
for f in "$@"; do
  nodes="$(jq '.nodes | length' "$f")"
  if [ "$nodes" -gt "$ceiling" ]; then
    echo "flake-lock-graph: $f has $nodes nodes; the ceiling is $ceiling." >&2
    exit 1
  fi
  np="$(jq '[.nodes[] | select(.original.owner == "NixOS" and .original.repo == "nixpkgs")] | length' "$f")"
  if [ "$np" -gt 1 ]; then
    echo "flake-lock-graph: $f locks $np nixpkgs; exactly one is allowed. Make the new input follow nixpkgs-lock." >&2
    exit 1
  fi
done
