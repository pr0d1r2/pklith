# The catalog's `cargo-universal` (src/catalog T11, paradigm V28): the
# script in ./cargo-universal.sh, with this nixpkgs' cargo, rustc and jq.
# lipo and otool are the darwin stdenv's cctools, already in every dev shell
# there.
#
# nixpkgs' rustc ships std for its own host only, and the check builds both
# Macs' slices on either Mac. The other Mac's std is copied from the same
# nixpkgs rev's rustc for that system: cache.nixos.org holds it, and it
# carries the same version string, so this rustc loads it through
# `--sysroot`. Only its std crates (.rlib, .rmeta) are copied, with no store
# references, but building that copies the whole rustc first, once.
{ pkgs, nixpkgs }:
let
  inherit (pkgs) lib;
  triples = {
    aarch64-darwin = "aarch64-apple-darwin";
    x86_64-darwin = "x86_64-apple-darwin";
  };
  other = lib.head (lib.remove pkgs.stdenv.hostPlatform.system (lib.attrNames triples));
  theirs = nixpkgs.legacyPackages.${other}.rustc;
  triple = triples.${other};
  std =
    assert theirs.version == pkgs.rustc.version;
    pkgs.runCommand "rust-std-${triple}-${theirs.version}" { } ''
      mkdir -p $out/lib
      for rlib in ${theirs.unwrapped}/lib/rustlib/${triple}/lib/*.rlib; do
        cp "$rlib" $out/lib/
        meta="''${rlib%.rlib}.rmeta"
        if [ -f "$meta" ]; then cp "$meta" $out/lib/; fi
      done
    '';
  sysroot = pkgs.runCommand "rust-sysroot-apple-darwin-${pkgs.rustc.version}" { } ''
    mkdir -p $out/lib/rustlib
    ln -s ${pkgs.rustc.unwrapped}/lib/rustlib/* $out/lib/rustlib/
    ln -s ${std} $out/lib/rustlib/${triple}
  '';
in
pkgs.writeShellApplication {
  name = "cargo-universal";
  runtimeInputs = [
    pkgs.cargo
    pkgs.rustc
    pkgs.jq
  ];
  runtimeEnv = lib.optionalAttrs pkgs.stdenv.hostPlatform.isDarwin {
    CARGO_UNIVERSAL_SYSROOT = sysroot;
  };
  text = builtins.readFile ./cargo-universal.sh;
}
