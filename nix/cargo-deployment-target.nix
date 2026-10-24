# The catalog's `cargo-deployment-target` (src/catalog T13, paradigm T18
# and V28): the script in ./cargo-deployment-target.sh, with taplo to
# parse `.cargo/config.toml` and jq to read what it parsed. `floor` is the
# macOS floor (./catalog.nix).
{ pkgs, floor }:
pkgs.writeShellApplication {
  name = "cargo-deployment-target";
  runtimeInputs = [
    pkgs.taplo
    pkgs.jq
  ];
  runtimeEnv.CARGO_DEPLOYMENT_TARGET = floor;
  text = builtins.readFile ./cargo-deployment-target.sh;
}
