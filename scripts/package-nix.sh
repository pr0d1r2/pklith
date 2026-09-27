#!/usr/bin/env bash
# pklith root T62 (V10): the package consumers get is tested as built, not
# only the source tree. It builds lib.devShell's check (T61) and the flake's
# pkli and compat packages, runs the built binaries on a fixture repository,
# and refuses a build that still carries an unsubstituted `@name@` or
# `*_PATH` placeholder (the legacy -full B2 failure: a placeholder left in,
# every type "uncovered").
#
#   scripts/package-nix.sh
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
unset GIT_DIR GIT_INDEX_FILE GIT_WORK_TREE GIT_PREFIX

fail() {
  echo "package-nix: $*" >&2
  exit 1
}

system="$(nix eval --raw --impure --expr builtins.currentSystem)"
# lib.devShell's own check (T61) builds alongside; it prints no path.
nix build "$root#checks.$system.devshell" --no-link
mapfile -t outs < <(nix build "$root#default" "$root#compat" --no-link --print-out-paths)
[ "${#outs[@]}" -eq 2 ] || fail "nix build printed ${#outs[@]} outputs, expected 2"
pkli="${outs[0]}/bin/pkli"
compat="${outs[1]}/bin"

# Placeholders: anywhere in a text file, or as a whole string in a binary.
placeholder='@[A-Za-z_][A-Za-z0-9_]*@|[A-Z][A-Z0-9_]*_PATH'
if grep -rIlE "$placeholder" "${outs[@]}" >&2; then
  fail "a built text file keeps a placeholder (above)"
fi
for bin in "${outs[0]}"/bin/*; do
  if strings -n 6 "$bin" | grep -xE "$placeholder" >&2; then
    fail "$bin keeps a placeholder string (above)"
  fi
done

fixture="$(mktemp -d)"
trap 'rm -rf "$fixture"' EXIT
git -C "$fixture" init -q
printf 'echo hi\n' >"$fixture/a.sh"
mkdir "$fixture/docs"
cat >"$fixture/docs/linter-coverage.md" <<'DOC'
| Extension | Linter |
|---|---|
| `.sh` | shellcheck |
| `.md` | - | docs |
DOC
git -C "$fixture" add -A

expect() {
  local what="$1" want="$2" got="$3"
  [ "$got" = "$want" ] || fail "$what: expected $(printf '%q' "$want"), got $(printf '%q' "$got")"
}

version="$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml")"
expect "pkli --version" "pkli $version" "$("$pkli" --version | cut -d' ' -f1-2)"
expect "pkli detect" "$(printf 'base\nshell\nmarkdown')" "$(cd "$fixture" && "$pkli" detect)"
expect "pkli import" "$(printf 'format 1\n\n## types\ntype|checks|min|exempt\nsh|-|-|legacy: shellcheck\nmd|-|-|docs')" \
  "$(cd "$fixture" && "$pkli" import docs/linter-coverage.md)"
(cd "$fixture" && "$compat/lefthook-linter-coverage") || fail "compat entry: a listed repository failed"
printf '{}\n' >"$fixture/x.json"
git -C "$fixture" add x.json
if (cd "$fixture" && "$compat/lefthook-linter-coverage" 2>/dev/null); then
  fail "compat entry: an unlisted .json passed"
fi
