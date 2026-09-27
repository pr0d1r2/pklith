#!/usr/bin/env bash
# pklith V27 (root T64): speed is the port's reason, so it is measured, not
# assumed. Three generated fixture repositories (small, fleet-sized, 10k
# files) are timed under pkli; each timing is the best of three runs.
#
#   scripts/bench.sh                   time pkli, fail over a budget
#   scripts/bench.sh --compare DIR     also time the legacy tools, for §R
#
# The budgets below are about five times the measurement recorded as §R R20,
# headroom for a slower machine; a regression past one fails the step.
# --compare DIR takes the directory holding the sibling legacy checkouts
# (set-and-setting, nix-lefthook-*), which the gate does not need.
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
unset GIT_DIR GIT_INDEX_FILE GIT_WORK_TREE GIT_PREFIX
compare="${2:-}"
[ "${1:-}" != --compare ] || [ -n "$compare" ] || { echo "bench: --compare needs a directory" >&2; exit 2; }

# Budgets in milliseconds: fixture:command.
declare -A budget=(
  [small:detect]=100 [small:check]=2500 [small:compat]=50
  [fleet:detect]=100 [fleet:check]=2500 [fleet:compat]=50
  [large:detect]=200 [large:check]=3000 [large:compat]=150
)

export CARGO_TARGET_DIR="$root/target"
cargo build -q --release --bin pkli
pkli="$root/target/release/pkli"
work="$root/target/bench"
mkdir -p "$work/bin"
ln -sf "$pkli" "$work/bin/lefthook-linter-coverage-full"

# fixture NAME COUNT: COUNT files over a spread of types, committed.
fixture() {
  local dir="$work/$1" n="$2" i
  [ -f "$dir/.done-$n" ] && return
  rm -rf "$dir" && mkdir -p "$dir/src" "$dir/lib" "$dir/docs" "$dir/nix"
  git -C "$dir" init -q
  for ((i = 0; i < n; i++)); do
    case $((i % 6)) in
      0) printf 'echo %d\n' "$i" >"$dir/src/s$i.sh" ;;
      1) printf '{ }\n' >"$dir/nix/n$i.nix" ;;
      2) printf '# d%d\n' "$i" >"$dir/docs/d$i.md" ;;
      3) printf 'a = %d\n' "$i" >"$dir/lib/t$i.toml" ;;
      4) printf 'x: %d\n' "$i" >"$dir/lib/y$i.yml" ;;
      5) printf 'fn f%d() {}\n' "$i" >"$dir/src/r$i.rs" ;;
    esac
  done
  cat >"$dir/docs/linters.md" <<'DOC'
| Ext |
|---|
| `.sh` `.nix` `.md` `.toml` `.yml` `.rs` |
DOC
  (cd "$dir" && "$pkli" seed --init >/dev/null && "$pkli" gen >/dev/null)
  git -C "$dir" add -A
  touch "$dir/.done-$n"
}

# best NAME DIR CMD...: best-of-three wall time in ms; exit 2 is a failure.
best() {
  local dir="$1" min='' t0 t1 ms rc
  shift
  for _ in 1 2 3; do
    t0="$(date +%s%N)"
    rc=0
    (cd "$dir" && "$@" >/dev/null 2>&1) || rc=$?
    t1="$(date +%s%N)"
    [ "$rc" -le 1 ] || { echo "bench: '$*' in $dir exited $rc" >&2; exit 1; }
    ms=$(((t1 - t0) / 1000000))
    [ -z "$min" ] || [ "$ms" -lt "$min" ] && min="$ms"
  done
  echo "$min"
}

over=0
row() {
  local name="$1" what="$2" ms="$3" legacy="${4:-}"
  local cap="${budget[$name:$what]}"
  printf '%-6s %-7s %6d ms  (budget %d)%s\n' "$name" "$what" "$ms" "$cap" "${legacy:+  legacy $legacy ms}"
  [ "$ms" -le "$cap" ] || { echo "bench: $name $what took ${ms} ms, over its ${cap} ms budget (V27)" >&2; over=1; }
}

for spec in small:50 fleet:1000 large:10000; do
  name="${spec%%:*}"
  fixture "$name" "${spec#*:}"
  dir="$work/$name"
  export LEFTHOOK_LINTER_COVERAGE_DOC=docs/linters.md LEFTHOOK_LINTER_COVERAGE_ROOT="$dir"
  legacy_detect='' legacy_compat=''
  if [ -n "$compare" ]; then
    legacy_detect="$(best "$dir" bash "$compare/set-and-setting/setting/lib/detect-fragments.sh")"
    legacy_compat="$(best "$dir" "$compare/nix-lefthook-linter-coverage-full/result/bin/lefthook-linter-coverage-full")"
  fi
  row "$name" detect "$(best "$dir" "$pkli" detect)" "$legacy_detect"
  row "$name" check "$(best "$dir" "$pkli" check)"
  row "$name" compat "$(best "$dir" "$work/bin/lefthook-linter-coverage-full")" "$legacy_compat"
done
exit "$over"
