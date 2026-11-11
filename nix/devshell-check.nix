# lib.devShell on this repository (flake.nix `checks`, T61): its tools are
# on PATH, and its hook rule holds on a repository with and without
# `hk.pkl` (V22). scripts/package-nix.sh builds it.
{ pkgs, shell }:
pkgs.runCommand "pklith-devshell" { nativeBuildInputs = shell.nativeBuildInputs; } ''
  export HOME="$TMPDIR"
  for tool in hk pkl git pklith shellcheck sherd; do
    command -v "$tool" >/dev/null || { echo "devshell: $tool is not on PATH" >&2; exit 1; }
  done
  cat >hook.sh <<'HOOK'
  ${shell.shellHook}
  HOOK
  git init -q bare && mkdir bare/.githooks
  (cd bare && . ../hook.sh)
  [ -z "$(git -C bare config --local core.hooksPath)" ] || { echo "devshell: hooks on without hk.pkl (V22)" >&2; exit 1; }
  touch bare/hk.pkl
  (cd bare && . ../hook.sh)
  [ "$(git -C bare config --local core.hooksPath)" = .githooks ] || { echo "devshell: hooks off with hk.pkl" >&2; exit 1; }
  touch $out
''
