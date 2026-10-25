#!/usr/bin/env bash
# pklith V27 (root T64): speed is the port's reason, so it is measured, not
# assumed. Three generated fixture repositories (small, fleet-sized, 10k
# files) are timed under pklith; each timing is the best of three runs.
#
#   scripts/bench.sh                   time pklith, fail over a budget
#   scripts/bench.sh --compare DIR     also time the legacy tools, for §R
#
# Budgets are CPU milliseconds, about three times the measurement recorded
# as §R R20: headroom for a slower machine; a regression past one fails.
# They are scaled by the machine first (#114): a fixed pkl workload that
# runs no pklith code is timed the same way, and every budget is multiplied
# by max(1, calibration / reference). A slower or busier machine gets the
# same ratio of headroom; on a quiet machine like the reference one the
# budgets are exactly as written, so a pklith regression still fails.
# --compare DIR takes the directory holding the sibling legacy checkouts
# (set-and-setting, nix-lefthook-*), which the gate does not need.
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
unset GIT_DIR GIT_INDEX_FILE GIT_WORK_TREE GIT_PREFIX
compare="${2:-}"
[ "${1:-}" != --compare ] || [ -n "$compare" ] || { echo "bench: --compare needs a directory" >&2; exit 2; }

# Budgets in milliseconds: fixture:command.
declare -A budget=(
  [small:detect]=150 [small:check]=7000 [small:compat]=50
  [fleet:detect]=150 [fleet:check]=8000 [fleet:compat]=50
  [large:detect]=200 [large:check]=13000 [large:compat]=150
)

# The calibration's CPU ms, best of three, on the machine the budgets were
# checked on: aarch64-darwin, load average 1.4, 2026-10-25 (§R R21).
calibration_reference=1150

export CARGO_TARGET_DIR="$root/target"
cargo build -q --release --bin pklith
pklith="$root/target/release/pklith"
stamp="$(sha256sum "$pklith" | cut -c1-16)"
tick=$'\x60'
work="$root/target/bench"
mkdir -p "$work/bin"
ln -sf "$pklith" "$work/bin/lefthook-linter-coverage-full"

# fixture NAME COUNT: COUNT files over a spread of types, committed.
fixture() {
  local dir="$work/$1" n="$2" i
  # Rebuilt whenever pklith changes: what it seeds and generates is part of
  # the fixture, and a stale one would time yesterday's work.
  [ -f "$work/.done-$1-$n-$stamp" ] && return
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
  git -C "$dir" add -A
  (cd "$dir" && "$pklith" seed --init >/dev/null && "$pklith" gen >/dev/null)
  git -C "$dir" add -A
  # A doc listing every key the tree has, so the compat run passes and a
  # failing one means something broke.
  {
    printf '| Ext |\n|---|\n'
    git -C "$dir" ls-files | sed 's|.*/||; s|.*\.||' | sort -u |
      while read -r key; do printf '| %s%s%s |\n' "$tick" "$key" "$tick"; done
  } >"$dir/docs/linters.md"
  git -C "$dir" add -A
  touch "$work/.done-$1-$n-$stamp"
}

# best DIR CMD...: best-of-three CPU time in ms, user plus system, the
# command's children (pkl, git) included. CPU time, not wall time: a busy
# machine stretches the wall clock several times over and would fail the
# step for someone else's load. Every run must pass: a run that fails fast
# (a missing doc, an unset variable) measures nothing.
best() {
  local dir="$1" min='' ms rc cpu
  shift
  for _ in 1 2 3; do
    rc=0
    cpu="$( { TIMEFORMAT='%3U %3S'; time (cd "$dir" && "$@" >/dev/null 2>&1); } 2>&1)" || rc=$?
    [ "$rc" -eq 0 ] || { echo "bench: '$*' in $dir exited $rc" >&2; exit 1; }
    ms="$(awk '{ printf "%d", ($1 + $2) * 1000 }' <<<"$cpu")"
    [ -z "$min" ] || [ "$ms" -lt "$min" ] && min="$ms"
  done
  echo "$min"
}

# A fixed workload in pkl, the tool behind most of `pklith check`'s time,
# so contention slows it the way it slows check. It runs no pklith code: a
# pklith regression cannot raise its own budgets.
cat >"$work/calibration.pkl" <<'PKL'
local rows = IntSeq(1, 40000).map((i) -> "row \(i) \(i * 7 % 13)").toList()
size = rows.fold(0, (n, r) -> n + r.length)
PKL
calibration="$(best "$work" pkl eval calibration.pkl)" || exit 1
# Full precision for the budgets; two decimals only in the printed line.
scale="$(awk -v c="$calibration" -v r="$calibration_reference" 'BEGIN { s = c / r; if (s < 1) s = 1; printf "%.17g", s }')"
printf 'calibration %d ms (reference %d ms): budgets x %.2f\n' "$calibration" "$calibration_reference" "$scale"

over=0
row() {
  local name="$1" what="$2" ms="$3" legacy="${4:-}"
  local cap
  cap="$(awk -v b="${budget[$name:$what]}" -v s="$scale" 'BEGIN { printf "%d", b * s }')"
  printf '%-6s %-7s %6d ms  (budget %d)%s\n' "$name" "$what" "$ms" "$cap" "${legacy:+  legacy $legacy ms}"
  [ "$ms" -le "$cap" ] || { echo "bench: $name $what took ${ms} ms, over its ${cap} ms budget (V27)" >&2; over=1; }
}

for spec in small:50 fleet:1000 large:10000; do
  name="${spec%%:*}"
  fixture "$name" "${spec#*:}"
  dir="$work/$name"
  export LEFTHOOK_LINTER_COVERAGE_DOC=docs/linters.md LEFTHOOK_LINTER_COVERAGE_ROOT="$dir"
  legacy_detect='' legacy_compat=''
  # best runs in a subshell, so its failure is caught here, not lost.
  if [ -n "$compare" ]; then
    legacy_detect="$(best "$dir" bash "$compare/set-and-setting/setting/lib/detect-fragments.sh")" || exit 1
    legacy_compat="$(best "$dir" "$compare/nix-lefthook-linter-coverage-full/result/bin/lefthook-linter-coverage-full")" || exit 1
  fi
  detect="$(best "$dir" "$pklith" detect)" || exit 1
  check="$(best "$dir" "$pklith" check)" || exit 1
  compat="$(best "$dir" "$work/bin/lefthook-linter-coverage-full")" || exit 1
  row "$name" detect "$detect" "$legacy_detect"
  row "$name" check "$check"
  row "$name" compat "$compat" "$legacy_compat"
done
exit "$over"
