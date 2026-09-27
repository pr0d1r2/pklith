#!/usr/bin/env bash
# Built-in catalog, proven against nix (src/catalog T3, T5): every row's
# `nix` attribute builds from this flake's `catalog` namespace, and its
# `check` command calls a binary that attribute provides (`cargo fmt` means
# `cargo-fmt`). Rows whose `nix` is `-` are hk's own and are skipped.
#
#   scripts/catalog-nix.sh CATALOG.pklith...
set -euo pipefail

system="$(nix eval --raw --impure --expr builtins.currentSystem)"

# id, nix attr and check command of each `## checks` row, tab-separated.
# `\|` is an escaped pipe inside a cell, so it is hidden before splitting.
rows() {
  awk -F'|' '
    /^## / { inside = ($0 == "## checks"); header = inside; next }
    !inside || /^#/ || !NF { next }
    header { header = 0; next }
    { print $1 "\t" $3 "\t" $5 }
  ' < <(sed 's/\\|/\x1f/g' "$1")
}

# Whether $out/bin holds a binary the command names, as a word or as a
# `cargo SUB` subcommand.
calls() {
  local out="$1" prev='' word
  for word in $(printf '%s' "$2" | tr '\037;&()`' '      '); do
    [ -x "$out/bin/$word" ] && return 0
    [ "$prev" = cargo ] && [ -x "$out/bin/cargo-$word" ] && return 0
    prev="$word"
  done
  return 1
}

ids=() attrs=() commands=() files=()
for file in "$@"; do
  while IFS=$'\t' read -r id attr command; do
    [ "$attr" = - ] && continue
    ids+=("$id") attrs+=("$attr") commands+=("$command") files+=("$file")
  done < <(rows "$file")
done
# Nothing checked must not pass (V1).
[ "${#ids[@]}" -gt 0 ] || { echo 'catalog-nix: no row with a nix attribute was found, so nothing was checked.' >&2; exit 1; }

# One evaluation and one build for every row: each nix call re-evaluates
# the flake and copies the tree into the store, which is the slow part.
catalog=".#legacyPackages.$system.catalog"
names="$(printf '"%s" ' "${attrs[@]}")"
if ! bins="$(nix eval --json "$catalog" --apply "c: builtins.listToAttrs (map (n: { name = n; value = let p = c.\${n}; in (p.bin or p.out or p).outPath; }) [ $names ])")"; then
  echo "catalog-nix: a row's nix attribute does not evaluate in $catalog; nix names it above" >&2
  exit 1
fi
installables=()
for attr in "${attrs[@]}"; do installables+=("$catalog.$attr^*"); done
if ! nix build --no-link "${installables[@]}"; then
  echo "catalog-nix: a row's nix attribute does not build; nix names it above" >&2
  exit 1
fi

for i in "${!ids[@]}"; do
  id="${ids[$i]}" attr="${attrs[$i]}" command="${commands[$i]}" file="${files[$i]}"
  out="$(jq -r --arg a "$attr" '.[$a]' <<<"$bins")"
  if [ ! -d "$out/bin" ]; then
    echo "catalog-nix: $file: \`$id\` names nix attribute \`$attr\`, which has no binaries ($out)" >&2
    exit 1
  fi
  if ! calls "$out" "$command"; then
    echo "catalog-nix: $file: \`$id\` runs \`$command\`, but \`$attr\` provides none of its binaries ($(basename -a "$out"/bin/* | tr '\n' ' '))" >&2
    exit 1
  fi
done
echo "catalog-nix: ${#ids[@]} rows resolve to their nix binaries"
