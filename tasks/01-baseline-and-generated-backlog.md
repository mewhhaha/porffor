# T01 — Reproducible baseline and generated failure backlog

## Mandatory compiler provenance — 2026-10-04 dry source

Current execution snapshots use schema 8 with mandatory checked build-source
fingerprint/scheme, source revision and executing-image SHA-256. Version and
compiler binding share one owner. Missing, null or malformed identity rejects
current evidence; legacy versions 4–7 remain explicitly unbound history and
cannot resume, join a current aggregate, publish or acquire an invented proof.

Writers and resume require the actual running compiler. Every aggregate child
must carry the same producer as its parent. Verified/progress/backlog summaries
retain that identity; read-only schema-8 comparisons can compare different
compiler builds while preserving each side's producer. Suite pins, exact case
identity, matrix coverage, outcome classifications and complete evidence remain
independent mandatory obligations.

The CLI exposes `compiler-identity` without creating a Realm. Publication JSON,
text and future generated README retain the verified producer. The publication
session's schema-3 manifest keeps observed checkout/source/suite/configuration
separate from the native embedded build fingerprint, binds the reported image
to observed executable bytes, and requires native checkpoint identity before
rewriting its progress high-water mark. Old observed sidecars do not authorize
adoption of unbound snapshots.

Meaningful Engine, harness, CLI and driver controls are authored. Compilation,
controls, emitted Wasm and pinned acceptance remain unverified. All remaining
task source precedes the next checkpoint under a confirmed 4096 MiB aggregate
kernel cap, zero swap and serial execution. No task closes, historical artifact
changes or status count refresh occurs. See the
[compiler provenance contract](../docs/rust-rewrite/contracts/snapshot-compiler-provenance.md).


**Status:** In progress — historical baseline and backlog retained; fresh compiler publication remains

**Parallel group:** Bootstrap  
**Depends on:** None  
**Blocks:** Reliable prioritization and T26 closure

## Current repository state

The retained version-7 Wasm-AOT baseline
`current-pin-wasm-aot-20260907-c5115bf03-2x1-12g` records `87,641/102,043`
passing executions and `14,402` failures under suite content tree
`aa55200d1310384c5cf69ea95b2a2ecba457007b`. Its generated
[failure backlog](../test262/backlog/aa55200d1310384c5cf69ea95b2a2ecba457007b/wasm-aot.json)
is checked in, and its
[retained status artifact](../test262/snapshots/baseline-12g/published-status-wasm-aot.json)
records a `2026-09-14` refresh. The outcomes are `9,212` Bug, `957` Crash and
`4,233` NotImplemented. These describe the historical compiler baseline;
subsequent repairs and focused replays have not produced a fresh complete
compiler aggregate or refreshed the canonical README publication pair.

Deterministic backlog generation, ownership mapping, snapshot comparison and
pin-mismatch tests exist in `lila-test262`, with CLI entry points for
`generate-backlog` and `compare-snapshots`. The checked-in ownership map routes
unknown cases to `T26-unclassified`. Ownership-map input is parsed into the
closed repository `TaskId` domain and a closed debt-category domain;
`T26-unclassified` is a separate typed closure bucket rather than an invented
task identifier. One `BacklogOwnership` enum owns the pair, so a concrete task
cannot be paired with the unclassified category and the unclassified bucket
cannot masquerade as classified debt.

Complete aggregate loading is also an integrity boundary rather than a
filename check. It freshly discovers the pinned suite, proves that the cached
matrix assigns every case to exactly one uniquely named node, validates every
aggregate entry against that matrix, reconciles all totals, kinds, outcomes and
origins, then joins every entry to a complete node snapshot whose completed and
failed case sets agree exactly. Matrix-node snapshot identity is derived in one
place from the unique node ID; sibling chunks may share a discovery filter
without sharing or losing resumable evidence. Generated text backlogs include
the same task, feature-tag, failure-hash and slow-subtree groupings as the JSON
artifact.

Physical-file discovery expands through the private, non-derived
`TestExecutionPlan::{One, SloppyAndStrict}` domain. Its flag parser owns the six
valid frontmatter combinations, and its consuming exhaustive projection emits
the exact one-mode plan or the ordered sloppy-then-strict pair. The execution
identity structure guard pins the private capability-free declaration, complete
ownership census, flag-to-plan table and ordered mode projection. This is a
source-equivalent discovery invariant and does not refresh the historical
backlog for the current compiler. The strengthened structure target passes `4/4`, and the
exact flag-plan unit witness passes `1/1`. Independent review confirmed the
capability closure, flag binding/tuple order, six rows and ordered expansion.
The coordinated workspace checkpoint passes `cargo fmt --all -- --check`,
`cargo xc`, `git diff --check`, the module boundary check and the task-plan
check; the compile retains the repository's existing warnings.

Failure kind, outcome and origin also remain closed types at the snapshot and
backlog boundaries. Unknown classification labels or count-map keys reject the
artifact rather than being coerced to a catch-all or dropped. Read-only version
4 evidence has one explicit migration exception because that schema predates
outcomes: missing failure outcomes and outcome counts are derived from its
recorded evidence. Versions 5 and 6 require a recognized outcome on every
failure and an outcome-count map on every snapshot and aggregate entry.

Aggregate-entry matrix-node kinds cross the snapshot boundary through the same
closed `MatrixNodeKind` domain used by verification. The snapshot codec accepts
only the established `filter-leaf` and `chunk-leaf` spellings; an unknown label
is rejected before it can be compared with the current matrix. This leaves the
version-6 snapshot bytes unchanged and deliberately does not change the run
matrix cache's existing `FilterLeaf` and `ChunkLeaf` serde spellings.

Generated backlog backend identity is also closed. The
`BacklogArtifact.execution_backend` field stores `ExecutionBackend` rather than
a raw string, and its field codec exhaustively preserves the established
`spec-exec` and `wasm-aot` spellings. Unknown labels such as `future-backend`
are rejected with the offending value attached; file names and summaries only
project the typed authority with `as_str()`. The focused structure target
passes `4/4`, the exact deterministic-backlog witness passes `1/1`,
`cargo check -p lila-test262 --quiet` passes with existing warnings, and the
scoped rustfmt and diff checks are clean.

The retained version-7 baseline has no native compiler binding. The dry schema-8
migration above introduces mandatory checked compiler provenance for newly
produced evidence; it cannot attest or upgrade that historical baseline. The
historical backlog/status bytes remain retained evidence, while current backlog,
resume and publication require fresh compiler-bound snapshots.

The generated README status block has a separate repository provenance gate.
Only a co-change to the publisher's exact canonical output pair,
`test262/snapshots/published-status-wasm-aot.json` and
`test262/snapshots/published-status-wasm-aot.txt`, authorizes that block to
change. Node checkpoints and aggregate snapshots are inputs to verification,
not proof that the publisher produced the README text; focused, fake-suite and
`spec-exec` artifacts are likewise never publication authority.

This task is not complete because the current compiler still needs a complete
verified Wasm-AOT aggregate, regenerated failure backlog and canonical
publication. The retained baseline and nested status artifact do not authorize
hand-editing the generated README block or inferring new full-suite totals from
focused repairs.

The vendored suite is committed inside the Lila repository. Its pin is the
suite directory's Git content tree, rather than the enclosing checkout HEAD.
The native pin helper already derives that tree identity and accepts an older
enclosing-commit pin only after proving identical suite content. Unrelated
compiler commits therefore do not change the suite pin; they still require
separate compiler provenance and new execution evidence.

The low-RAM publication wrapper now prints the checkout commit and executable
SHA-256 into the publication transcript and checks both between CLI invocations.
It permits one initial checkpoint attempt, then rejects unreadable, stalled,
regressing or inconsistent progress instead of retrying indefinitely. Exact
positive completion is required before delegating to the existing Rust publisher.
The nonempty driver contract inventory runs in read-only CI; these fake-CLI
orchestration tests are not compiler or Test262 conformance evidence. See
[the driver contract](../docs/rust-rewrite/reproducible-publication-driver.md).
The schema-3 supervisor now also consumes the mandatory native compiler binding.
This does not bind older checkpoints to a compiler. Current-pin publication and
execution acceptance of the dry schema migration remain open.

## Exact comparison inputs — 2026-09-06

`compare-snapshots` now requires both requested snapshot names to match their
validated aggregate identities. A missing baseline or candidate must not resolve
to the other run (or a third run) and fabricate a zero-change comparison. The
shared loader still validates current schema, pins, the complete matrix and node
evidence; status/backlog discovery retains its existing unique-name fallback.
Explicitly comparing a snapshot with itself remains supported.

The retained `snapshot_comparison_identity_tests` library test family exercises missing
inputs, explicit self-comparison, an actual added pass and regression, discovery
fallback and incomplete/corrupt named candidates. Its compile-negative fixture
matrices use the Wasm-AOT front end without enabling the oracle; these are harness
contracts, not pinned real-suite conformance counts. See
[the comparison contract](../docs/rust-rewrite/test262-snapshot-comparison.md).

Next batch: build and record one unchanged compiler, finish the complete
current-pin Wasm-AOT matrix through the guarded publication driver, verify and
publish its canonical artifacts, and regenerate the failure backlog for that
compiler. Compare only
explicitly named compatible snapshots; do not invent a missing historical
baseline. The dry mandatory provenance migration still requires its combined
compiler and execution acceptance. T01 and T26 remain open.

## Objective

Produce a complete, reproducible view of the current pinned Test262 state for the `wasm-aot` product backend, then generate a machine-readable backlog that assigns every non-passing case to a semantic family and task ID. A `spec-exec` oracle snapshot may be produced alongside it for differential triage, but it is diagnostic data only — it is never the baseline that tasks burn down and never product status.

The generated README block remains historical. The retained complete real-suite
baseline and generated backlog supply earlier failure evidence; this task must
refresh both with a fixed current compiler before publishing new totals.

## Deliverables

1. A deterministic baseline command sequence for the current `ecma262` and `test262` revisions.
2. A complete `wasm-aot` aggregate snapshot produced by the resumable matrix path; optionally a separately labeled `spec-exec` oracle snapshot for triage.
3. A generated backlog artifact, for example `test262/backlog/<test262-sha>/wasm-aot.json`, with one record per non-passing case:
   - test path and metadata features;
   - flags/includes/negative phase;
   - failure kind, outcome, origin, normalized detail and detail hash;
   - duration and timeout status;
   - matrix node;
   - likely owner task ID;
   - whether the failure is parser, semantic, host, dynamic-source, performance, or infrastructure debt.
   Owner and debt-category strings are serialization-boundary spellings of
   closed Rust types; arbitrary values cannot enter the in-memory backlog.
4. Human-readable summaries grouped by task ID, feature tag, failure hash, and slowest subtree.
5. A comparison command that reports added passes, regressions, changed failure hashes, and pin mismatches between two snapshots.

## Implementation steps

- Build `lila` once and record the exact binary/source commit used.
- Verify the suite pin before running. Refuse to merge or compare snapshots with different pins or matrix strategy versions.
- Run `report-all --resume` through `scripts/publish-real-status-low-ram.sh` with one matrix node per process until complete.
- Extend `lila-test262` rather than writing a separate ad-hoc parser for snapshot files.
- Normalize unstable data such as absolute paths and wall-clock timestamps before comparison.
- Classify by the earliest trustworthy boundary. A backend message containing a runtime symptom must not overwrite a known parser or lowering origin.
- Add a checked-in ownership mapping from stable feature/subtree prefixes to task IDs; unknown cases go to `T26-unclassified`, never to an ignored bucket.
- Keep writer, resume, verification and backlog lookup on the shared matrix-node
  manifest identity function. A chunk's filter is not its identity.

## Integrity requirements

- Totals across outcomes, failure kinds, origins, entries, and completed paths must reconcile exactly.
- Snapshot and backlog classification labels and count-map keys must decode
  into the closed failure-kind, outcome and origin domains. Only version 4 may
  omit a per-failure outcome, through its explicit read-only migration.
- A matrix is publishable only when all planned nodes are present and every case in the manifest appears once.
- Aggregate publication reopens every node snapshot and reconciles its exact
  completed/failure sets and classification counts with the aggregate entry.
- Resuming must not duplicate or drop cases.
- `passed == total` is the only green aggregate. Unsupported cases remain in the denominator.
- Fake-suite data may be included as a separate section but must never be merged into real-suite totals.
- A generated README status change must carry both exact canonical Wasm-AOT
  status artifacts. Neither matrix evidence nor one half of that output pair
  is sufficient provenance.

## Acceptance criteria

- The `wasm-aot` backend has a complete verified snapshot for the current pin, or the PR documents a concrete infrastructure blocker with a reproducible failing node while still landing the deterministic backlog tooling. Any `spec-exec` oracle snapshot is stored and labeled separately from product data.
- Running the generator twice over the same snapshot produces byte-identical backlog output.
- Every failure is assigned to exactly one task ID or the explicit unclassified closure bucket.
- The comparison command catches an intentionally injected regression and pin mismatch.
- README status is updated only through the normal publisher after the complete
  matrix is verified, with both canonical Wasm-AOT status artifacts committed
  beside the generated block.

## Required tests

```sh
cargo test -p lila-test262 --quiet
cargo test -p lila-cli test262_ --quiet
./target/debug/lila test262 progress-status --execution-backend wasm-aot
./target/debug/lila test262 triage-status --execution-backend wasm-aot
./scripts/publish-real-status-low-ram.sh wasm-aot codex-published-real
# Optional oracle triage snapshot; never published as product conformance:
./target/debug/lila test262 report-all --execution-backend spec-exec \
  --snapshot-name codex-oracle-real
```

Use low thread counts for publication, but add unit tests that prove higher worker counts preserve deterministic case accounting.
