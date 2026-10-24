# pklith's one `pkgs` (flake.nix `forAll`): nix-hk's overlay, so `pkgs.hk`
# is nix-hk's hk, then the darwin swiftlint wrapper (nix/swiftlint.nix,
# src/catalog V5), so the catalog namespace and pklith's dev shell both
# run the swiftlint that finds the host toolchain.
nix-hk: final: prev:
nix-hk.overlays.default final prev // { swiftlint = import ./swiftlint.nix prev; }
