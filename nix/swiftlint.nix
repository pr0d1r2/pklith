# The built-in catalog's `swiftlint` (src/catalog V5, #108).
#
# swiftlint's SourceKitten loads sourcekitdInProc.framework from a HOST
# toolchain at run time. nixpkgs ships none, and a nix shell's DEVELOPER_DIR
# names the nix apple-sdk, which has none either, so on darwin upstream
# aborts before it lints. The wrapper points TOOLCHAIN_DIR at the host's
# Xcode toolchain or Command Line Tools, found by xcode-select outside the
# nix DEVELOPER_DIR, and with neither fails as a missing tool, naming it
# (root V1). A TOOLCHAIN_DIR already set is kept. linux runs the upstream
# static binary, which needs no host toolchain.
pkgs:
if pkgs.stdenv.hostPlatform.isDarwin then
  pkgs.writeShellScriptBin "swiftlint" ''
    if [ -z "''${TOOLCHAIN_DIR:-}" ]; then
      developer="$(unset DEVELOPER_DIR; /usr/bin/xcode-select -p 2>/dev/null)" || developer=
      for dir in "$developer/Toolchains/XcodeDefault.xctoolchain" "$developer"; do
        if [ -n "$developer" ] && [ -d "$dir/usr/lib/sourcekitdInProc.framework" ]; then
          export TOOLCHAIN_DIR="$dir"
          break
        fi
      done
    fi
    if [ ! -d "''${TOOLCHAIN_DIR:-/nonexistent}/usr/lib/sourcekitdInProc.framework" ]; then
      echo "swiftlint: no host toolchain holds sourcekitdInProc.framework''${TOOLCHAIN_DIR:+ (TOOLCHAIN_DIR=$TOOLCHAIN_DIR)}; install the Command Line Tools (xcode-select --install) or Xcode. A MISSING TOOL, not a finding." >&2
      exit 1
    fi
    exec ${pkgs.swiftlint}/bin/swiftlint "$@"
  ''
else
  pkgs.swiftlint
