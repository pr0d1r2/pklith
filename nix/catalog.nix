# The built-in catalog's own packages, beside nixpkgs' attributes in
# flake.nix's `legacyPackages.<system>.catalog`. Both cargo checks hold
# paradigm V28's macOS floor, read here from the one place pklith keeps
# it: src/seed/cargo-config.toml, the `.cargo/config.toml` that `pklith
# seed` writes for the rust fragment (src/catalog T13).
{ pkgs, nixpkgs }:
let
  config = builtins.fromTOML (builtins.readFile ../src/seed/cargo-config.toml);
  floor = config.env.MACOSX_DEPLOYMENT_TARGET.value;
in
{
  cargo-universal = import ./cargo-universal.nix { inherit pkgs nixpkgs floor; };
  cargo-deployment-target = import ./cargo-deployment-target.nix { inherit pkgs floor; };
}
