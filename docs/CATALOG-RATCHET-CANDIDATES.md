# Catalog ratchet candidates

Issue #115 is recorded as an opt-in pilot, not as three active built-in rows.
The candidates are:

- an ownership/import check derived from each node's §F `⊥owns` column: a change in
  node A must not touch sibling-owned paths, and code in A must not import from a
  sibling unless §N links them;
- a non-increasing `.duplication` share, piloting `pmd cpd`;
- a weekly non-decreasing `cargo-mutants` score ratchet outside hk.

Promote a candidate only after one member proves a planted violation, pins the
engine, defines a reproducible baseline/report, and adds the appropriate
scheduled check. Mutation must never run per worker: the previous per-worker
shape exhausted the fleet's ten-core budget. `pmd` and `cargo-mutants` resolve
from the current flake; `jscpd` and the ownership engine still require an
explicit package/engine decision.
