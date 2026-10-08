# Test262 aggregate snapshot use

Status: schema-8 compiler-binding source is authored; compilation and execution
remain unverified. Finish all task source before the confirmed 4096 MiB capped
serial checkpoint. No task or conformance count changes from this source work.

## Current checked policy

The private, capability-free `SnapshotUse::{CurrentState, ReadOnlyEvidence}`
remains exhaustive. Both choices require a current schema-8 compiler binding.
Current state additionally requires its identity to equal the actual running
compiler. Progress, verified/publication/backlog and resume consumers select that
policy. Only explicit comparison selects read-only evidence, so complete
aggregates from different compiler builds can be compared with each side's
actual producer retained.

Every node loader receives the parent's checked compiler identity and proves
that child evidence has the same producer. Matrix strategy, suite pins, backend,
manifest, exact case sets, counts and taxonomy stay independently mandatory.
Legacy schemas 4–7 retain checked historical decoding/pin-report metadata; they
cannot become current progress, aggregate joins, resume or publication and do
not gain a compiler identity from the currently running process.

The maintained structural and semantic controls follow these real consumers;
they are authored, not executed for this successor. See the
[compiler provenance contract](snapshot-compiler-provenance.md).

## Historical 2026-08-27 predecessor checkpoint

The following records the former schema policy and its scoped evidence. Its
metadata-only legacy progress behavior, source counts and passing checks do not
apply to the new schema-8 source above.

Status: implemented as a source-equivalent T03 invariant closure on 2026-08-27.

## Closed use policy

The private `SnapshotUse::{CurrentState, ReadOnlyEvidence}` domain is the
complete policy accepted by the shared aggregate snapshot validator.
`CurrentState` requires the current producer-bound schema before the shared
matrix, backend, manifest and pin checks. `ReadOnlyEvidence` skips only that
current-schema requirement so the metadata-only progress workflow may inspect
a supported legacy envelope; it retains every shared envelope check.

The domain derives no clone, copy, debug, equality or default capability and
has no manual implementation. The validator borrows it through an exhaustive
two-arm projection. A future use therefore cannot silently inherit the former
non-`CurrentState` fallback.

## Producer and consumer boundary

Seven product source sites select the policy. The two direct aggregate-resume
candidate checks, verified summary, publishable summary, internal current
summary and failure-detail lookup select `CurrentState`. Metadata-only
aggregate progress alone selects `ReadOnlyEvidence`. The verified-summary
forwarder and aggregate resolver carry the selected value unchanged; repeated
candidate validation borrows that same value.

The exhaustive consumer keeps the current-schema check ahead of the existing
matrix-strategy, backend, manifest, run-kind and pin checks. This changes no
snapshot byte, materialized test source, prelude provenance, result count,
filesystem observation or validation order.

## Evidence and limits

The recursive structure guard pins the exact attribute-free declaration,
thirteen production mentions, seven producers, both forwarding boundaries and
the exhaustive validation projection:

```console
cargo test -p lila-test262 --test snapshot_use_structure
cargo test -p lila-test262 --features spec-exec-oracle tests::complete_consumers_reject_legacy_aggregates_and_mixed_legacy_nodes -- --exact --test-threads=1
```

The structure target passes `4/4`, and the exact legacy/current aggregate unit
passes `1/1`. Independent review found and closed declaration-boundary and
resume-call binding escape hatches; the final re-review is clean. The shared
checkpoint passes `cargo fmt --all -- --check`, `cargo xc`, `git diff --check`,
the module-boundary check and the task-plan check.

The focused unit witnesses both sides: metadata-only progress accepts the
supported version-6 legacy envelope, while verified summaries, backlog
generation and snapshot comparison reject it as non-current. It also confirms
that version 4 remains too old even for progress and that current aggregate
evidence cannot promote a legacy node.

This invariant does not upgrade legacy evidence, make progress publishable,
refresh a snapshot, alter a Test262 result or close T03's semantic
materialization debt.
