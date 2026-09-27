#!/usr/bin/env bash
# pklith V28 (root T65): the vendored hk schema is the one the pinned hk
# ships. hk evaluates hk.pkl against pkl/Config.pkl; a copy from another hk
# version type-checks a config hk itself would read differently. The pinned
# hk's own source is the reference, and pkl/VENDORED.md must name that
# version and the copy's sha256.
#
#   scripts/schema-pin.sh
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
cd "$root"

fail() {
  echo "schema-pin: $*" >&2
  echo "  fix: copy Config.pkl from the pinned hk's release and update pkl/VENDORED.md in the commit that moves the pin (V28)." >&2
  exit 1
}

system="$(nix eval --raw --impure --expr builtins.currentSystem)"
hk="$root#legacyPackages.$system.catalog.hk"
src="$(nix build "$hk.src" --no-link --print-out-paths)"
version="$(nix eval --raw "$hk.version")"

cmp -s "$src/pkl/Config.pkl" pkl/Config.pkl ||
  fail "pkl/Config.pkl differs from the schema hk $version ships"
grep -q "hk $version release" pkl/VENDORED.md ||
  fail "pkl/VENDORED.md does not name hk $version"
sum="$(sha256sum pkl/Config.pkl | cut -d' ' -f1)"
grep -q "\`Config.pkl\` sha256: $sum" pkl/VENDORED.md ||
  fail "pkl/VENDORED.md does not carry Config.pkl's sha256 $sum"
