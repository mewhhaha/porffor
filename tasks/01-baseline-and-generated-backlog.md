# T01 — Reproducible baseline and generated failure backlog

**Status:** In progress — tooling landed; current-pin publication remains

**Parallel group:** Bootstrap  
**Depends on:** None  
**Blocks:** Reliable prioritization and T26 closure

## Current repository state

Deterministic backlog generation, ownership mapping, snapshot comparison and
pin-mismatch tests exist in `porffor-test262`, with CLI entry points for
`generate-backlog` and `compare-snapshots`. The checked-in ownership map routes
unknown cases to `T26-unclassified`. This task is not complete because the
README still reports that the current pinned Wasm-AOT aggregate has not been
fully republished, and there is no checked-in current-pin generated Wasm-AOT
backlog artifact.

## Objective

Produce a complete, reproducible view of the current pinned Test262 state for the `wasm-aot` product backend, then generate a machine-readable backlog that assigns every non-passing case to a semantic family and task ID. A `spec-exec` oracle snapshot may be produced alongside it for differential triage, but it is diagnostic data only — it is never the baseline that tasks burn down and never product status.

The current README explicitly says the last complete real-suite publication is stale for the current pin. This task replaces inference and hand-maintained lists with verified artifacts.

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
4. Human-readable summaries grouped by task ID, feature tag, failure hash, and slowest subtree.
5. A comparison command that reports added passes, regressions, changed failure hashes, and pin mismatches between two snapshots.

## Implementation steps

- Build `porf` once and record the exact binary/source commit used.
- Verify the suite pin before running. Refuse to merge or compare snapshots with different pins or matrix strategy versions.
- Run `report-all --resume` through `scripts/publish-real-status-low-ram.sh` with one matrix node per process until complete.
- Extend `porffor-test262` rather than writing a separate ad-hoc parser for snapshot files.
- Normalize unstable data such as absolute paths and wall-clock timestamps before comparison.
- Classify by the earliest trustworthy boundary. A backend message containing a runtime symptom must not overwrite a known parser or lowering origin.
- Add a checked-in ownership mapping from stable feature/subtree prefixes to task IDs; unknown cases go to `T26-unclassified`, never to an ignored bucket.

## Integrity requirements

- Totals across outcomes, failure kinds, origins, entries, and completed paths must reconcile exactly.
- A matrix is publishable only when all planned nodes are present and every case in the manifest appears once.
- Resuming must not duplicate or drop cases.
- `passed == total` is the only green aggregate. Unsupported cases remain in the denominator.
- Fake-suite data may be included as a separate section but must never be merged into real-suite totals.

## Acceptance criteria

- The `wasm-aot` backend has a complete verified snapshot for the current pin, or the PR documents a concrete infrastructure blocker with a reproducible failing node while still landing the deterministic backlog tooling. Any `spec-exec` oracle snapshot is stored and labeled separately from product data.
- Running the generator twice over the same snapshot produces byte-identical backlog output.
- Every failure is assigned to exactly one task ID or the explicit unclassified closure bucket.
- The comparison command catches an intentionally injected regression and pin mismatch.
- README status is updated only through the normal publisher after the complete matrix is verified.

## Required tests

```sh
cargo test -p porffor-test262 --quiet
cargo test -p porffor-cli test262_ --quiet
./target/debug/porf test262 progress-status --execution-backend wasm-aot
./target/debug/porf test262 triage-status --execution-backend wasm-aot
./scripts/publish-real-status-low-ram.sh wasm-aot codex-published-real
# Optional oracle triage snapshot; never published as product conformance:
./scripts/publish-real-status-low-ram.sh spec-exec codex-published-real
```

Use low thread counts for publication, but add unit tests that prove higher worker counts preserve deterministic case accounting.
