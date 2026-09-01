# Test262 aggregate evidence requirement

Status: implemented and structure-verified on 2026-08-27.

## Closed resolution policy

The private `AggregateEvidenceRequirement::{Envelope, Complete}` domain is the
complete policy for accepting an alternate-name aggregate snapshot. `Envelope`
requires the schema, backend, version, pin and matrix envelope to match.
`Complete` additionally loads and validates the aggregate's complete node
evidence before the candidate may participate in unique resolution.

The domain derives no clone, copy, debug, equality or default capability. The
candidate loop borrows it in one exhaustive match. Adding another evidence
policy therefore requires its acceptance behavior to be stated before the
crate builds; it cannot silently inherit the former non-`Complete` fallback.

Exactly three product consumers select the policy. Verified aggregate loading
chooses `Complete`. Read-only progress and one-node failure-detail lookup choose
`Envelope`; both subsequently consume only the envelope-owned information or
validate the selected node through their existing boundaries.

## Source equivalence and evidence

This change replaces one equality test with the same two-row exhaustive truth
table. Candidate discovery, validation calls, error selection and filesystem
observation order are unchanged.

`aggregate_evidence_requirement_structure.rs` recursively pins the seven source
mentions, exact private declaration, absent capabilities, exhaustive candidate
decision and all three producer contexts:

```console
cargo test -p lila-test262 --test aggregate_evidence_requirement_structure
cargo test -p lila-test262 --features spec-exec-oracle tests::backlog_snapshot_name_resolution_finds_unique_compatible_aggregate -- --exact --test-threads=1
```

The focused structure target passes `4/4`, and the exact alternate-name product
witness passes `1/1`. Independent review confirmed the complete seven-mention
census, exact three producer contexts, exhaustive policy bodies, preserved
candidate/error order and alternate-name-only contract scope. Coordinated
`cargo xc`, full formatter, diff, module-boundary and task-plan checks are
green.

This invariant does not claim that the current aggregate is complete or green,
refresh a snapshot, change the Test262 denominator, or satisfy the T26 release
gate. Full publication and conformance verification remain deferred.
