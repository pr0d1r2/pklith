{
  # pklith's toolchain. The crate builds with plain cargo; nix is here to pin
  # what cargo, rustc and every gate tool are, not to take over the build.
  description = "pklith -- materialize hk git hook guardrails from what a repository's files require";

  # hk is built by nix-hk and pushed to this cache. nixos-26.05 ships no hk, so
  # without the substituter every entry into this shell would build it from
  # source. A user outside `trusted-users` still gets a source build, with only
  # a warning.
  nixConfig = {
    extra-substituters = [ "https://pr0d1r2.cachix.org" ];
    extra-trusted-public-keys = [
      "pr0d1r2.cachix.org-1:NfWjbhgAj41byXhCKiaE+av3Vnphm1fTezHXEGsiQIM="
    ];
  };

  # The nixpkgs rev is FOLLOWED, not spelled. nixpkgs-lock is the one place the
  # fleet decides it (nixos-26.05, rustc 1.95.0 = this crate's MSRV); a literal
  # rev here would be one more opinion about the compiler that nobody refreshes.
  inputs = {
    nixpkgs-lock.url = "github:pr0d1r2/nixpkgs-lock";
    nixpkgs.follows = "nixpkgs-lock/nixpkgs";
    # The gate runner. It follows the same lock: a second nixpkgs edge would
    # fork the rev and every cached hk would miss while looking like success.
    nix-hk.url = "github:pr0d1r2/nix-hk";
    nix-hk.inputs.nixpkgs-lock.follows = "nixpkgs-lock";
    # Spec tooling, pinned to releases. Each follows our nixpkgs-lock and
    # nix-hk so the lock keeps one nixpkgs (flake-lock-graph).
    microlith.url = "github:pr0d1r2/microlith/v0.7.3";
    microlith.inputs.nixpkgs-lock.follows = "nixpkgs-lock";
    microlith.inputs.nix-hk.follows = "nix-hk";
    sherd.url = "github:pr0d1r2/sherd/v0.5.1";
    sherd.inputs.nixpkgs-lock.follows = "nixpkgs-lock";
    sherd.inputs.nix-hk.follows = "nix-hk";
    # itok, for the built-in catalog's `itok` row. microlith already locks
    # one; following it adds no lock node and keeps a single itok.
    itok.follows = "microlith/itok";
  };

  outputs =
    {
      nixpkgs,
      nix-hk,
      microlith,
      sherd,
      itok,
      ...
    }:
    let
      # The tier-1 systems of §C. No system literal appears anywhere else
      # (V21): everything below is written against whatever `pkgs` it is given.
      systems = [
        "aarch64-darwin"
        "x86_64-linux"
        "aarch64-linux"
      ];
      # The overlay makes `pkgs.hk` mean nix-hk's hk, so there is one `pkgs`.
      forAll =
        f: nixpkgs.lib.genAttrs systems (s: f (nixpkgs.legacyPackages.${s}.extend nix-hk.overlays.default));
    in
    {
      # The namespace the built-in catalog's `nix` column names
      # (src/catalog/builtin.pklith): nixpkgs, plus the flake inputs for
      # tools nixpkgs does not ship. scripts/catalog-nix.sh builds from it.
      legacyPackages = forAll (
        pkgs:
        let
          input = i: i.packages.${pkgs.stdenv.hostPlatform.system}.default;
        in
        {
          catalog = pkgs // {
            microlith = input microlith;
            sherd = input sherd;
            itok = input itok;
          };
        }
      );

      devShells = forAll (pkgs: {
        default = pkgs.mkShell {
          packages = [
            pkgs.rustc
            pkgs.cargo
            pkgs.clippy
            pkgs.rustfmt
            pkgs.git
            # The gate: hk runs the steps declared in hk.pkl, and evaluates
            # hk.pkl with pkl.
            pkgs.hk
            pkgs.pkl
            # Gate tools that need a real binary, one per step, each added in
            # the commit that adds its step.
            pkgs.ripsecrets
            pkgs.jq
            pkgs.typos
            pkgs.nixfmt
            pkgs.taplo
            pkgs.cargo-nextest
            pkgs.shellcheck
            pkgs.lychee
            pkgs.actionlint
            pkgs.zizmor
            pkgs.cargo-deny
            # Coverage: cargo-llvm-cov plus llvm-cov/llvm-profdata, which nixpkgs
            # rustc does not ship; wired through the env vars it looks for.
            pkgs.cargo-llvm-cov
            pkgs.llvmPackages.llvm
            # SPEC.md format and structure (mth), from the pinned microlith.
            microlith.packages.${pkgs.stdenv.hostPlatform.system}.default
            # Federation checks (check, sync, budget), from the pinned sherd.
            sherd.packages.${pkgs.stdenv.hostPlatform.system}.default
            # Built-in catalog tools pklith's own gate does not run, so its
            # test suite can prove each one against a planted violation (V30).
            pkgs.shfmt
            pkgs.rubocop
            pkgs.editorconfig-checker
            pkgs.statix
            itok.packages.${pkgs.stdenv.hostPlatform.system}.default
          ];
          RUST_BACKTRACE = "1";
          LLVM_COV = "${pkgs.llvmPackages.llvm}/bin/llvm-cov";
          LLVM_PROFDATA = "${pkgs.llvmPackages.llvm}/bin/llvm-profdata";

          # Hooks are tracked in .githooks/ and REFUSE when hk is missing
          # (V1). Pointing git at them only once hk.pkl exists means no
          # stub config ever gates a commit (V22). Repo-local config only.
          shellHook = ''
            if [ -f hk.pkl ] && [ "$(git config --local core.hooksPath)" != .githooks ]; then
              git config --local core.hooksPath .githooks
            fi
          '';
        };
      });
    };
}
