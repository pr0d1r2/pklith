# Integration

How to put pklith's gate into a repository, keep it true as the repository
grows, and run the same gate in CI. The [registry](REGISTRY.md), the
[built-in catalog](CATALOG.md) and the [command line](CLI.md) are the
references this page leans on.

## Getting the tools

pklith is a nix flake. Its `lib.devShell` gives a repository a dev shell
with hk, pkl, git and `pklith`, plus every tool its generated
`nix/pklith.nix` names, all from the pinned catalog pklith was proven
against:

```nix
{
  inputs.pklith.url = "github:pr0d1r2/pklith";

  # hk is built by nix-hk and pushed to this cache; without it the first
  # shell builds hk from source.
  nixConfig = {
    extra-substituters = [ "https://pr0d1r2.cachix.org" ];
    extra-trusted-public-keys = [
      "pr0d1r2.cachix.org-1:NfWjbhgAj41byXhCKiaE+av3Vnphm1fTezHXEGsiQIM="
    ];
  };

  outputs = { nixpkgs, pklith, ... }: {
    devShells.aarch64-darwin.default = pklith.lib.devShell {
      pkgs = nixpkgs.legacyPackages.aarch64-darwin;
      src = ./.;
    };
  };
}
```

With an `.envrc` holding `use flake`, `direnv allow` enters it on `cd`.
Supported systems are aarch64-darwin, x86_64-linux and aarch64-linux;
asking `lib.devShell` for another fails naming these.

## Putting the gate in

On a repository with no gate yet:

1. **See what your files switch on.** `pklith detect` prints the fragments
   (shell, nix, markdown, …) your tracked files select.
2. **Seed.** `pklith seed --init` writes the hk gate (`hk.pkl`, an empty
   generated module, the vendored schema, tracked hooks in `.githooks/`),
   a few repository files (`.editorconfig`, `.gitattributes`, a README
   stub), whatever the fragments ask for, and a first `.pklith`. It never
   overwrites a file you already have. Read `.pklith`: it is yours now, and
   pklith never rewrites it.
3. **Switch the hooks on.** Re-enter the shell (`direnv reload`):
   `lib.devShell` points git at `.githooks/` once `hk.pkl` exists. Commit
   the seed; the gate has no steps yet, so hk has nothing to run.
4. **Lay the checks.** `pklith lay --dry-run` lists one commit per check;
   `pklith lay` makes them, cheapest and broadest first, each through your
   hooks. If a hook refuses one, everything lay did is rolled back and your
   own uncommitted work is left alone.
5. **Confirm.** `pklith check` passes when every tracked file has a type and
   every claimed check has an hk step that reaches it; `pklith confirm`
   additionally proves the gate is complete, generated files are fresh,
   every tool is on `PATH`, and hk evaluates it.

If you would rather not have one commit per check, `pklith gen` writes every
step at once for a single commit of your own.

## Keeping it true

A new file type is a **gap**: `pklith check` fails naming it until `.pklith`
gives it checks or an exemption reason. After editing `.pklith`, `pklith gen`
rewrites the generated steps, and `pklith gen --check` fails whenever they
are stale; pklith's own gate runs both as steps (`gen-check`,
`pklith-check` in its [`.pklith`](../.pklith)), a pattern worth copying. A row whose files are
all gone is **stale** and fails too, so the registry cannot drift from the
tree.

`pklith map --staged` lists the specs a change touches, for guard modes that
want them; `pklith report` prints the whole coverage matrix as text, JSON or
Markdown.

## CI

Run the same definition CI and laptops share, from the same dev shell:

```yaml
- uses: actions/checkout@v4
  with:
    persist-credentials: false
- uses: cachix/install-nix-action@v31
- run: nix develop --command hk check --all
```

`pklith protect --dry-run` prints the branch-protection payload the
workflow's jobs imply; `pklith protect` applies it through `gh`, refusing to
drop a required check unless you pass `--accept-removals`. It is the only
verb that uses the network.

## Coming from elsewhere

- **lefthook.** `pklith migrate` moves a lefthook-materialized repository
  onto hk, and only when every lefthook command survives as a check;
  `--drop ID,...` lets named ones go on purpose, recorded in `.pklith`.
- **The legacy coverage tools.** Swap them for `packages.compat` first,
  then import their document; the [README](../README.md) walks through it,
  and [PORT.md](PORT.md) maps set-and-setting's pieces to pklith's.

## What pklith will not do to your repository

- Overwrite a file you have: seeding skips what exists.
- Rewrite `.pklith` after seeding.
- Write outside its own files: `gen` owns `hk.pklith.pkl`,
  `nix/pklith.nix` and a block between `<!-- BEGIN pklith -->` and
  `<!-- END pklith -->` in AGENTS.md, if you add those markers.
- Commit around your hooks: every commit `lay` makes goes through them,
  and one refused commit rolls the whole run back.
- Touch the network, except `pklith protect` when you run it.

## Known gaps

- **No step timeout.** hk 1.58 has no timeout field for a step, and
  wrapping commands in `timeout` would re-quote `{{files}}`; a hung tool
  hangs the hook (`src/gen` T6).
- **Platforms.** No x86_64-darwin (Intel macOS) and no Windows.
