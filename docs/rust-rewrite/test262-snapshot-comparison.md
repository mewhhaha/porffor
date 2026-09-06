# T01: exact snapshot identity in comparisons

## Contract

A comparison answers what changed between two specific runs. Both names must
identify complete current aggregate snapshots for the selected backend and
matrix. A missing baseline, candidate, or both is an error even when the shared
snapshot discovery code can find exactly one compatible run under another name.
The diagnostic identifies the requested and resolved names rather than reporting
an empty diff for two aliases of the same evidence.

The comparison-specific loader first uses the existing complete-evidence loader,
then checks its resolved identity before permitting comparison. It does not
introduce another snapshot parser or bypass schema, producer, pin, manifest,
matrix-completeness or node-evidence validation. An exact but invalid artifact
continues to fail validation; it is not replaced with another snapshot.

Explicit self-comparison remains valid. Discovery-oriented status and backlog
commands retain their existing unambiguous fallback and expose the actual
resolved name. This change is deliberately limited to the comparison consumer.

## Usage

Use names of two real compatible runs already on disk:

```sh
./target/release/lila test262 compare-snapshots baseline \
  --snapshot-name candidate \
  --execution-backend wasm-aot \
  --suite-root test262/vendor/test262 \
  --snapshot-dir test262/snapshots
```

The previous implementation could resolve a nonexistent `baseline` to the sole
`candidate` aggregate, load that run twice, and print zero added passes,
regressions and changed hashes while displaying two different requested names.
The same substitution was possible for an absent candidate or for two missing
names. Do not infer improvement or stability from a comparison whose requested
input was never produced.

This repair does not establish compiler provenance for old checkpoints. Preserve
source-commit and executable-digest logs as described by the
[publication driver](reproducible-publication-driver.md). Mandatory provenance
still needs its separately designed Rust snapshot-schema migration.

## Regression validation

```sh
cargo test --locked -p lila-test262 --test snapshot_comparison_identity
cargo test --locked -p lila-test262 \
  --test snapshot_use_structure \
  --test aggregate_evidence_requirement_structure \
  --test execution_identity_structure
cargo fmt --all -- --check
./scripts/check-task-plan.sh
./scripts/check-module-boundaries.sh
```

The eight-test integration target covers missing base/candidate/both names,
explicit self-comparison, actual pass/regression detection, preserved discovery
fallback, and incomplete/corrupt named candidates. Fixtures are isolated temporary
matrices of parse-negative JavaScript; they use the Wasm-AOT front end without
turning on the spec-exec oracle. Passing and failing totals are asserted before
testing the comparison result. These are harness contracts, not real Test262
conformance counts.

The read-only `Snapshot comparison contracts` workflow uses the retained
complete-inventory executor to require every compiled test to execute exactly
once without failures, ignores or timeouts. It records the source identity,
input hashes, inventory and per-test results.

Next: complete the fixed-compiler current-pin Wasm-AOT matrix, publish verified
canonical artifacts, and generate the failure backlog. T01 and T26 remain open;
no suite pin, source, exclusion, snapshot schema or published count is changed.
