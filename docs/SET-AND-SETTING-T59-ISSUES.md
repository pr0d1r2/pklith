# Set-and-setting issues: D1, D2, and D3

## D1: register reason-only exemptions

### Problem

The exemption-ledger awk parser registers only entries containing a
`ticket:` field. An exemption that has a valid reason but no ticket is
therefore ignored, even though the contract permits a reason without a
ticket. This makes a legitimate exemption look missing and can either fail
the gate or encourage an unnecessary ticket.

### Requested change

Parse and register every exemption with a non-empty reason. Treat a ticket
as optional metadata, while retaining the existing validation for malformed
or empty entries.

### Acceptance criteria

- A reason-only exemption is registered and accepted.
- A ticket-bearing exemption remains registered and accepted.
- An empty or malformed exemption is still rejected with an actionable
  diagnostic.

## D2: do not let `all` make coverage vacuous

### Problem

The `all` file class is treated as if its checks cover every file, without
checking whether the selected checks actually run on those files. A map can
therefore claim coverage for an entire repository while its checks select
nothing, producing a green but vacuous result.

### Requested change

Resolve `all` against the concrete tracked file set and require each file to
reach at least one selected check. Preserve the normal type and path-class
coverage rules, including explicit exemptions.

### Acceptance criteria

- An `all` class with no effective checks is rejected.
- An `all` class is accepted only when every tracked file reaches a check or
  has an explicit exemption.
- A concrete file-class claim cannot be bypassed by adding `all`.

## D3: complete the fragment wrapper map

### Problem

`wrappersForFragment` omits the `just`, `xml`, `tcl`, and `awk` fragments.
Those fragments can be detected but have no corresponding wrapper, so their
checks are silently absent from the materialized setting.

### Requested change

Add wrappers for all four fragments and include them in the same completeness
and fragment-selection checks as the existing wrappers.

### Acceptance criteria

- Each of `just`, `xml`, `tcl`, and `awk` selects its declared wrapper checks.
- A detected fragment never has an empty wrapper selection.
- The wrapper map and generated setting remain deterministic and complete.
