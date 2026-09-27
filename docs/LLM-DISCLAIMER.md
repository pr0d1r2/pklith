# Built by an LLM, deliberately and in the open

This repository (code, spec, tests and prose) was written by
[Claude Code](https://claude.com/claude-code) running Anthropic's **Claude
Opus 5.5**. Every commit carries a `Co-Authored-By: Claude Opus 5.5`
trailer: not most of them, all of them, which
`git log --format=%B | grep -c Co-Authored-By` will confirm against
`git rev-list --count HEAD`. A human owns every decision, reviews every
diff, and is accountable for what ships.

That is the disclaimer. The rest of this file is why it is a design note
rather than an apology, and what you can check for yourself.

## Why say it at all

An LLM writes plausible code, and plausible is not correct. A reader who
does not know how a repository was produced cannot calibrate how hard to
look at it.

pklith makes the point sharper. Its whole job is to turn "this file type
should be linted" from a sentence into a gate that refuses a commit. A
repository that did that for others while its own rules lived in prose
would be arguing against itself. So the standard here is: **a claim in
prose is a defect until something executes it.**

## The method is spec-driven development

[`SPEC.md`](../SPEC.md) and one `SPEC.md` per node under `src/` are the
law, not a description written afterwards. Together they carry
**125 `§V` invariants** (what must stay true, with the reasoning), `§T`
tasks (what is decided and what is not), **22 `§B` bugs** (each paired
with the invariant that now catches it), and `§R` research (the
measurements the constraints rest on). A rule and its checker land in the
same commit, because a rule with no runner gates nothing.

The spec's format is checked mechanically by
[`microlith`](https://github.com/pr0d1r2/microlith) and its federation by
[`sherd`](https://github.com/pr0d1r2/sherd), so "the spec says so" is a
statement about a file that was parsed, not one that was skimmed.

## The guardrails are git hooks that also run on CI

Entering the dev shell (`nix develop`, or `direnv allow`) points git at
the tracked hooks, which run [hk](https://github.com/jdx/hk) against one
definition of the gate in [`hk.pkl`](../hk.pkl): **39 steps on commit,
46 on push**. The push half adds coverage, a re-lay of every check into a
copy of the repository, the nix build tested as built, the packaged
tarball, the API against the last release, and the benchmark.
[`ci.yml`](../.github/workflows/ci.yml) calls that same definition on
three platforms, so a laptop and a runner cannot disagree. Line coverage
has a floor of 100% that may only rise, `unsafe` is forbidden, and the
only command that touches the network is `pkli protect`.

## The record is deliberately unflattering

Five defects, chosen because each was green before it was found. They are
the reason to trust the process somewhat and the numbers above rather
less. Each names the commit that fixed it.

**`08de36a`: `pkli check` took 39.5 seconds on 10,000 files.** Every test
passed; no test had 10,000 files. The rows it suggests came from a loop
that rescanned every file for every file. It was found only when the
benchmark the spec demanded was first run.

**`92c8953`: a freshly seeded repository failed its first `pkli check`.**
`seed --init` typed the files present before seeding and forgot the ones
it was about to write. Nearly three hundred tests were green; it showed
the first time the built binary was run on an empty repository.

**`258f4ad`: the compat entries passed having checked nothing.** Any git
error became an empty file list, so a repository with no git on `PATH`
passed where the legacy tool failed. The 32 ported legacy cases passed;
none had git failing. An independent review found it by running
each entry beside the tool it replaces.

**`02e3a33` and `8935e4c`: the benchmark could pass without measuring.**
It accepted exit 1, so a missing input "passed" in a millisecond; its
fixtures were seeded before their files were tracked, so it timed a red
run that stopped early; and it timed the wall clock, which swung two to
five times with other work on the same machine.

**`1dd1832`: the badges said 36 invariants when the specs held 119.** The
formula counted distinct ids, and every node numbers its own from V1. It
was copied from a sibling project and looked right until the number was
quoted in this file and checked.

None of these was found by reviewing a diff. Each was found by running
the thing, which is the argument for the gate, and also the argument for
reading `§B` before trusting `§V`.

## What a reader should check

1. **Does the gate run for you?** `nix develop`, then `hk check --all`. If
   a claim here is false, that is where it shows.
2. **Does `§B` look like a real bug log or a curated one?** The five above
   are unedited; judge the rest.
3. **Do the `§V` invariants have runners?** A `§V` row that nothing
   executes is exactly the defect this project exists to remove.
4. **Is `§R` checkable?** Some rows cite sibling repositories that are not
   public; those are the owner's record, not yours to verify.

## Accountability

The human named in [`LICENSE`](../LICENSE) is responsible for this code,
including the parts a model wrote and the parts nobody caught. "The LLM
wrote it" explains provenance; it never transfers responsibility.

Unflattering bug reports are the most useful kind. See
[`SECURITY.md`](SECURITY.md) for the ones that should not be public and
[`CONTRIBUTING.md`](CONTRIBUTING.md) for everything else.
