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

checked=0
for file in "$@"; do
  while IFS=$'\t' read -r id attr command; do
    [ "$attr" = - ] && continue
    out=''
    for path in $(nix build --no-link --print-out-paths ".#legacyPackages.$system.catalog.$attr" 2>/dev/null); do
      [ -d "$path/bin" ] && out="$path" && break
    done
    if [ -z "$out" ]; then
      echo "catalog-nix: $file: \`$id\` names nix attribute \`$attr\`, which does not build to a package with binaries; run \`nix build .#legacyPackages.$system.catalog.$attr\` to see why" >&2
      exit 1
    fi
    if ! calls "$out" "$command"; then
      echo "catalog-nix: $file: \`$id\` runs \`$command\`, but \`$attr\` provides none of its binaries ($(find "$out/bin" -mindepth 1 -maxdepth 1 -printf '%f ' 2>/dev/null || basename -a "$out"/bin/*))" >&2
      exit 1
    fi
    checked=$((checked + 1))
  done < <(rows "$file")
done
# Nothing checked must not pass (V1).
[ "$checked" -gt 0 ] || { echo 'catalog-nix: no row with a nix attribute was found, so nothing was checked.' >&2; exit 1; }
echo "catalog-nix: $checked rows resolve to their nix binaries"
