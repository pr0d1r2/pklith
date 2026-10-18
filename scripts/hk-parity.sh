#!/usr/bin/env bash
# pklith V35 (B12): hk evaluates hk.pkl with its own Pkl implementation,
# pklr, unless told otherwise. A construct pklr mishandles makes steps vanish
# or lose their command while `pkl eval` still lists them, and the gate then
# passes having run almost nothing. This fails unless both evaluators see the
# same steps, each with a command.
#
# V36 (B13): a step with no command at all, in any hook, is refused by name
# rather than filtered. Amending a step id a mapping lacks is no Pkl error:
# it makes the mapping's default, a Step with no `check`, which runs nothing
# and cannot fail. A step meant to run nothing would be listed here with its
# reason; there is none.
#
#   scripts/hk-parity.sh
set -euo pipefail

evaluated="$(pkl eval --format json hk.pkl)"

empty="$(jq -r '.hooks | to_entries[] | .key as $hook | (.value.steps // {}) | to_entries[] |
  select(.value._type == "step" and .value.check == null and .value.check_diff == null and
    .value.check_list_files == null and .value.fix == null) | "  \($hook): \(.key)"' <<<"$evaluated")"
if [ -n "$empty" ]; then
  echo "hk-parity: these steps have no check, check_diff, check_list_files or fix, so they run nothing (V36):" >&2
  printf '%s\n' "$empty" >&2
  echo "  an id amended that its mapping lacks becomes an empty step; add the step by name from where it lives" >&2
  exit 1
fi

want="$(jq -r '.hooks.check.steps | to_entries[] | select(.value.check != null or .value.check_diff != null) | .key' <<<"$evaluated" | sort)"
seen="$(hk check --check --all --plan --json |
  jq -r '.steps[] | select([.reasons[].kind] | index("no_command") | not) | .name' | sort)"

if [ "$want" != "$seen" ]; then
  echo "hk-parity: hk's own Pkl evaluator does not see the steps pkl eval sees (V35):" >&2
  diff <(printf '%s\n' "$want") <(printf '%s\n' "$seen") |
    sed -n 's/^< /  missing in hk: /p; s/^> /  only in hk: /p' >&2
  exit 1
fi
