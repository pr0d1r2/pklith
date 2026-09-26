{
  # pklith's toolchain. The crate builds with plain cargo; nix is here to pin
  # what cargo, rustc and every gate tool are, not to take over the build.
  description = "pklith -- materialize hk git hook guardrails from what a repository's files require";

  # The nixpkgs rev is FOLLOWED, not spelled. nixpkgs-lock is the one place the
  # fleet decides it (nixos-26.05, rustc 1.95.0 = this crate's MSRV); a literal
  # rev here would be one more opinion about the compiler that nobody refreshes.
  inputs = {
    nixpkgs-lock.url = "github:pr0d1r2/nixpkgs-lock";
    nixpkgs.follows = "nixpkgs-lock/nixpkgs";
  };

  outputs =
    { nixpkgs, ... }:
    let
      # The tier-1 systems of §C. No system literal appears anywhere else
      # (V21): everything below is written against whatever `pkgs` it is given.
      systems = [
        "aarch64-darwin"
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAll = f: nixpkgs.lib.genAttrs systems (s: f nixpkgs.legacyPackages.${s});
    in
    {
      devShells = forAll (pkgs: {
        default = pkgs.mkShell {
          packages = [
            pkgs.rustc
            pkgs.cargo
            pkgs.clippy
            pkgs.rustfmt
            pkgs.git
          ];
          RUST_BACKTRACE = "1";
        };
      });
    };
}
