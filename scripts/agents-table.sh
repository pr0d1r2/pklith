#!/usr/bin/env bash
# Regenerate the step table in AGENTS.md from hk.pkl (pklith V14).
# --check: write nothing, exit 1 when AGENTS.md differs from what hk.pkl says.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

table() {
  pkl eval --format json hk.pkl | jq -r '
    .hooks["pre-commit"].steps as $fast | .hooks["pre-push"].steps as $all |
    "<!-- BEGIN steps: generated from hk.pkl by scripts/agents-table.sh; do not edit -->",
    "\($fast | length) steps run on every commit and \($all | length) on push and `hk check`; the commit-msg hook checks the message.",
    "",
    "| step | runs on | files | fixes |",
    "|---|---|---|---|",
    ($all | to_entries[] |
      "| `\(.key)` | \(if $fast[.key] then "commit" else "push" end) | \(if .value.glob then (.value.glob | map("`" + . + "`") | join(" ")) else "whole tree" end) | \(if .value.fix then "yes" else "-" end) |"),
    "<!-- END steps -->"'
}

render() {
  local generated
  generated="$(table)"
  awk -v gen="$generated" '
    /^<!-- BEGIN steps/ { print gen; skip = 1; next }
    /^<!-- END steps -->/ { skip = 0; next }
    !skip { print }
  ' AGENTS.md
}

if [ "${1:-}" = --check ]; then
  if ! render | diff -u AGENTS.md - >&2; then
    echo "agents-table: AGENTS.md no longer matches hk.pkl. Run scripts/agents-table.sh and stage the result." >&2
    exit 1
  fi
else
  out="$(render)"
  printf '%s\n' "$out" > AGENTS.md
fi
