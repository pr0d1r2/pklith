# Ashlar issue: course guardrails

## Problem

Ashlar's course workflow permits commits with hooks disabled and degrades
when `hk` is unavailable. Both behaviors conflict with the guardrail contract
we adopted from it:

- V12 requires hooks to be installed in the commit that brings `hk`, every
  later commit to pass the real `pre-commit` and `commit-msg` hooks, and
  rejects `--no-verify` or any hooks-disabled commit.
- V1 requires a gate that cannot run to refuse with a non-zero status; a
  missing `hk` must never become a silent pass.
- V13 requires one course/check per commit, including its configuration,
  runner, and documentation row. Bundled C3/C4 courses make it impossible to
  review or prove those units independently.

## Requested changes

1. Make course commits fail when hooks are disabled, including an explicit
   check that the repository's configured hooks actually ran.
2. Make missing `hk` a refusal with a non-zero exit status, rather than a
   degraded course run.
3. Split bundled C3/C4 courses so each check lands as one course and one
   commit, with the rule, runner, and documentation row together.
4. Record the planted-violation red verdict in each course commit so the
   guardrail proof survives outside the terminal session.

## Acceptance criteria

- A hooks-disabled commit is rejected.
- A course run without `hk` exits non-zero and performs no write.
- C3/C4 produce one commit per check and reject a planted violation at each
  step.
