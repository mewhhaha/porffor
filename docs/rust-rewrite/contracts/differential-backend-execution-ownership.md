# Differential backend-execution ownership

## Current worker authority — 2026-10-07 joined source

Replay now requires a selected `DifferentialWorkerRunner`. Each fresh child
owns JavaScript admission, Realm construction and the sole `execute_case`
backend producer. The supervisor admits a private non-cloneable
`CompletedWorkerAttempt` only after a bound header, complete ordered journal,
process-group retirement and successful staging cleanup. It owns worker
failure publication and retains validated incomplete print prefixes and
available compiler provenance. No parent raw backend executor remains.

WorkerFailure precedes the existing output, disposition and completion
comparisons; matching failures stay red and receive no mismatch signature.
The Debug-only backend envelope remains inside the worker. Its closed result
now distinguishes scalar completion, rooted graph completion and Engine
failure. The single consuming projection has nine exhaustive protocol/result
routes, including explicit refusal of a scalar result for v7 and a graph result
for v1–v6. The scalar and graph producers converge on the same actual backend
identity and captured-hook transcript checks before the one envelope is minted.

The shared transport retires the child and finishes staging cleanup before it
returns retained evidence. The supervisor decodes that evidence, gives process
and cleanup failures precedence, and only then admits a completed observation.
Graph terminals additionally require the exact requested limits. The affected
ownership guards are updated for these physical owners; their new source is
unrun pending the joined checkpoint. See the [worker lifecycle](differential-worker-lifecycle.md).

The guard pins the complete `CompletedWorkerAttempt` declaration and consuming
accessor, the Wasm-then-oracle replay move, all nine closed projection routes,
and the connected terminal count/domain/requested-limit condition before
`JournalTail::Completed` is published. It accepts the formatter's trailing
commas while retaining the original ownership and feature boundaries.

## Historical envelope checkpoint

The following census and passes describe the earlier in-process ownership
checkpoint, not the current worker source or its acceptance.

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
full normalized fingerprints for each relevant body. V4's Script/Module goal
selection and exhaustive comparison/projection rows update those existing
guards without granting another authority or projection capability.

## Focused evidence

Run:

```sh
cargo test -p lila-test262 --test backend_execution_ownership_structure -- --test-threads=1
cargo test -p lila-test262 differential::tests::v1_disposition_mismatch_keeps_its_pinned_machine_signature -- --exact --test-threads=1
cargo test -p lila-test262 differential::tests::either_backend_output_makes_a_no_output_case_red -- --exact --test-threads=1
cargo test -p lila-test262 differential::tests::v3_matches_primitive_completion_and_exact_ordered_print_transcript -- --exact --test-threads=1
cargo test -p lila-test262 differential::tests::v3_backend_failures_are_always_red -- --exact --test-threads=1
```

At the earlier ownership checkpoint, the structure target passed `6/6`, the
neighboring output-policy target passed `4/4`, and all four exact semantic
witnesses passed `1/1`. These results belong to that source. Current v4 guard
updates are authored; compilation and execution remain pending. The feature-gated
two-backend foundation replay remains part of the broader T25 checkpoint rather
than this capability-only focused gate.

The earlier capability-only migration changed no corpus or report wire bytes, case
fingerprints, mismatch signatures, output rules, verdicts, backend execution or
comparison order. It does not add an observation dimension, module replay,
oracle, reducer or semantic-equivalence claim. The additive v4 graph replay
adapter consumes the same ownership chain and observation dimensions.
