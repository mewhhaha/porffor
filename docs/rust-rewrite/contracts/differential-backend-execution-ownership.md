# Differential backend-execution ownership

`BackendExecution` is the private, owned result of running one differential
backend. It couples backend identity, the captured-output observation and the
closed `BackendExecutionResult` payload that is later projected into the public
report. Neither authority is a wire type or a comparison domain.

## Owned lifecycle

Both private authorities are Debug-only: diagnostic formatting remains
available to the feature-gated module-loader tests, while clone, copy, equality
and default capabilities are absent. There are seven production mentions of
`BackendExecution` and 12 production result mentions. `execute_case` is the
sole envelope producer. It constructs either a completed payload or a runner
failure directly, while `observe_engine_error` owns the remaining typed failure
route.

Replay constructs Wasm-AOT first and spec-exec second, then moves both envelopes
into `compare_executions`. That function retains its exact borrow-before-consume
order: it borrows Wasm then spec-exec output, borrows Wasm then spec-exec
disposition, and finally moves Wasm then spec-exec into the same projection.
The five-arm consuming projection destructures each complete envelope and
exhaustively maps every current protocol/result pairing. No envelope or result
can be cloned for a second projection or compared as a shortcut around the
protocol-specific typed comparison.

The Rust-lexical structure guard pins the production-only 7/12 census, both
Debug-only declarations, every result producer, the complete replay and
execution producer, the borrow/move sequence, the sole projection route and
full normalized fingerprints for each relevant body.

## Focused evidence

Run:

```sh
cargo test -p lila-test262 --test backend_execution_ownership_structure -- --test-threads=1
cargo test -p lila-test262 differential::tests::v1_disposition_mismatch_keeps_its_pinned_machine_signature -- --exact --test-threads=1
cargo test -p lila-test262 differential::tests::either_backend_output_makes_a_no_output_case_red -- --exact --test-threads=1
cargo test -p lila-test262 differential::tests::v3_matches_primitive_completion_and_exact_ordered_print_transcript -- --exact --test-threads=1
cargo test -p lila-test262 differential::tests::v3_backend_failures_are_always_red -- --exact --test-threads=1
```

The structure target passes `6/6`, the neighboring output-policy target remains
`4/4`, and all four exact semantic witnesses pass `1/1`. The feature-gated
two-backend foundation replay remains part of the broader T25 checkpoint rather
than this capability-only focused gate.

This capability-only migration changes no corpus or report wire bytes, case
fingerprints, mismatch signatures, output rules, verdicts, backend execution or
comparison order. It does not add an observation dimension, module replay,
oracle, reducer or semantic-equivalence claim.
