# Security policy

## Reporting a vulnerability

Please report vulnerabilities privately, through GitHub's
[private vulnerability reporting](https://github.com/pr0d1r2/pklith/security/advisories/new),
not in a public issue. Include what you ran, what happened, and what you
expected.

You will get an acknowledgement within a week. A fix, or a reason there
will not be one, follows as soon as the issue is understood.

## Supported versions

pklith has no release yet. Fixes land on `main`; once releases exist,
the latest one is supported.

## What pklith trusts

pklith runs the commands a repository tells it to, so treat these files
as code, the same as a Makefile:

- `.pklith`: a `## checks` row's `check` and `fix` columns become hk step
  commands, which run on every commit.
- `hk.pkl` and `hk.pklith.pkl`: hk runs their steps; `pklith` evaluates
  `hk.pkl` with `pkl`.
- `pklith seed` writes git hooks into `.githooks/`, and `lib.devShell`
  points git at them once `hk.pkl` exists.

Only `pklith protect` reaches the network, through `gh api`, and only when
you run it. A report that one of these runs something it was not told
to, or that a file pklith generates can be made to run commands its inputs
did not name, is in scope.
