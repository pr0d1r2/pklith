#!/usr/bin/env bash
# Re-lay reproduction (pklith root T45, V3, V13): `pklith lay` run on a copy of
# HEAD whose gate holds no steps must commit its way, through the real
# hooks, back to the very generated files HEAD tracks, byte for byte. It
# proves that every check this repository runs can be laid one commit at a
# time and that lay's output is what gen writes.
#
#   scripts/relay.sh
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
# A hook in a linked worktree gets GIT_DIR and GIT_INDEX_FILE pointing at
# the real repository; left set, every git call below would commit the copy
# into the developer's branch. Resolved root first, then dropped.
unset GIT_DIR GIT_INDEX_FILE GIT_WORK_TREE GIT_PREFIX
# The copy lives at a fixed path under the ignored target/, with a target
# of its own: cargo keys its cache by package path, so a fresh temp dir
# would rebuild pklith from nothing on every push, and a separate target
# never races this tree's build. The copy itself is rebuilt each run.
work="$root/target/relay"
repo="$work/repo"
rm -rf "$repo"
trap 'rm -rf "$repo"' EXIT
mkdir -p "$repo"
export CARGO_TARGET_DIR="$work/target"

git -C "$root" archive HEAD | tar -x -C "$repo"

# A gate with nothing laid: hk.pkl runs whatever the generated module
# holds, and the module holds no steps.
cat >"$repo/hk.pkl" <<'EOF'
amends "pkl/Config.pkl"

import "hk.pklith.pkl" as generated

hooks {
  ["pre-commit"] {
    fix = true
    stash = "git"
    steps = generated.steps
  }
  ["check"] {
    steps = (generated.steps) { ...generated.push }
  }
}
EOF
cat >"$repo/hk.pklith.pkl" <<'EOF'
import "pkl/Config.pkl"

steps: Mapping<String, Config.Step> = new {}

push: Mapping<String, Config.Step> = new {}
EOF
rm -f "$repo/nix/pklith.nix"

git -C "$repo" init -q
git -C "$repo" config user.name relay
git -C "$repo" config user.email relay@localhost
git -C "$repo" add -A
git -C "$repo" commit -q -m 'relay: the tree with an empty gate'
git -C "$repo" config core.hooksPath .githooks

cargo build -q --manifest-path "$repo/Cargo.toml" --bin pklith
(cd "$repo" && "$CARGO_TARGET_DIR/debug/pklith" lay >"$work/laid")

for generated in hk.pklith.pkl nix/pklith.nix; do
  if ! git -C "$root" show "HEAD:$generated" | cmp -s - "$repo/$generated"; then
    echo "relay: pklith lay did not reproduce $generated:" >&2
    git -C "$root" show "HEAD:$generated" | diff - "$repo/$generated" >&2 || true
    exit 1
  fi
done
echo "relay: $(wc -l <"$work/laid" | tr -d ' ') checks laid, one commit each; hk.pklith.pkl and nix/pklith.nix reproduced byte for byte"
