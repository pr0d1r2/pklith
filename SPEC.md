# SPEC

## §G GOAL

`pklith` (cmd `pkli`) — hk guardrails MATERIALIZER: from a repo's tracked files & a declared setting, derive which checks ∀ file needs — MULTIPLE linters & checks per file type, 1-to-1 unit test per impl, companion files (e.g. model → factory) — then GENERATE hk steps (Pkl) + nix packages, LAY them 1 check per commit, and CHECK, REPORT & ENFORCE coverage. purely mechanical: ⊥ LLM, ⊥ inference, ⊥ network.

PORT: the SETTING half of `set-and-setting` (nix + ~30 bash scripts, lefthook) → Rust on hk, for speed (R15). set half (skills, `mkSet`) stays there.

NAME: Pkl (hk's config language) + lith (stone) — fleet family `microlith`, `xenolith`. unique on 12 registries, GitHub & DNS (R13); `lydite` rejected — clashes w/ active same-space tool (R16).

REPLACES (open-sourced, sunsets them): setting half of `set-and-setting`; `nix-lefthook-linter-coverage`, `nix-lefthook-linter-coverage-full`, `nix-lefthook-unit-coverage`; existence half of `rspec-specification-coverage`. consumer swaps input, gate stays green (V4).

FIRST BUILT: this repo's own guardrails (§T T1–T43), check by check, 1 commit each, hooks live from first hk commit (V12) — before any product code.

## §F FEDERATION

dir|owns|⊥owns|tokens

## §N NAV

rel|path|lens
up|-|-
self|.|-

## §V INVARIANTS

V1: gate that cannot run has ⊥ passed. missing tool → exit ≠ 0, ⊥ silent skip
V2: ∀ hk step adopted → proven to reject a PLANTED violation first. finds-nothing ≠ can-find-nothing
V3: output sorted bytewise, ⊥ timestamps, ⊥ abs paths ∴ reproducible
V4: ∀ repo green under a legacy tool → green under `pkli check` on `import`ed `.pklith` (superset, ⊥ regression)
V5: ∀ tracked file ! resolve to exactly 1 type & type ! have ≥1 check beyond `all` checks | exempt+reason. ⊥ silent default
V6: generated files (`hk.pklith.pkl`, `nix/pklith.nix`, AGENTS.md block, `§N`) ⊥ hand-edited; stale → gate fails
V7: `pkl/` & vendored files ⊥ touched by fixers
V8: ∀ commit on `main` passes full `hk check`
V9: `.context-limits` ceilings ∀ `SPEC.md` enforced by `itok`/`sherd budget`; raise = reviewed commit w/ reason
V10: shipped nix package tested AS BUILT (`nix build` + fixture repo), ⊥ only source-tree run; built artifact carries ⊥ leftover `@…@` | `*_PATH` placeholder
V11: missing `.pklith` → exit 2, ⊥ skip, ⊥ pass. missing OPTIONAL input (exempt list, plural table) = empty, ⊥ error
V12: hk hooks installed IN the commit that brings hk. ∀ later commit made through real `pre-commit`/`commit-msg`; ⊥ `--no-verify`, ⊥ hooks-disabled commit. red hook → commit ⊥ lands
V13: 1 check = 1 commit = config + hk step + doc row. commit body names what is DELIBERATELY absent & why
V14: AGENTS.md step table & every step count in docs DERIVED from `hk.pkl`; `integration-doc` step fails on mismatch
V15: check laid only when ≥1 file it covers exists; ⊥ pre-laid check over 0 files (stale by birth)
V16: ONE fragment detector; gen, lay, confirm, migrate ∀ read its output. 2nd detector ⊥
V17: ∀ built-in fragment → ≥1 check & ∀ named check resolves to a catalog row w/ nix attr; completeness test in gate
V18: active fragment whose step selection is empty → error, ⊥ silent green
V19: ⊥ emitted step requires env pklith ⊥ sets; required env declared in catalog row & emitted w/ step
V20: child processes run w/ `GIT_DIR` `GIT_INDEX_FILE` `GIT_WORK_TREE` scrubbed unless the call needs them
V21: ⊥ system literal (`x86_64-linux` …) in emitted files; system comes from nix eval
V22: hooks installed only AFTER `hk.pkl` materialized ∴ ⊥ stub config ever gates a commit (complements V12)
V23: exemption w/ reason & ⊥ ticket is valid & registers
V24: `migrate`: check set before == check set after, else refuse w/ 0 writes
V25: seed files repo-owned (written once, skip-if-exists); materialized files regenerated & gitignored
