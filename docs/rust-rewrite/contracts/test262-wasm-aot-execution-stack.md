# Test262 Wasm-AOT execution stack

Status: implemented as a source-equivalent T03 harness invariant on 2026-08-27.

## Closed execution authority

The private
`WasmAotExecutionStack::{DedicatedWorker, PersistentTest262Worker}` domain is
the complete authority for choosing whether one Test262 case may use the
engine's current-thread Wasm-AOT entry points. It derives no clone, copy,
debug, equality or default capability and has no manual implementation.

`DedicatedWorker` remains available only under `cfg(test)` and is produced by
the focused `run_one_case` test entry point. Product `execute_cases` workers
produce `PersistentTest262Worker`; those workers own the large stack required
by the current-thread engine calls.

## Exhaustive routing

The shared case runner borrows the authority at both decisions and projects it
through explicit exhaustive matches. The existing routing order remains:

1. persistent-worker Wasm-AOT agent scripts use the current-thread agent call;
2. other Wasm-AOT agent scripts use the generic agent call;
3. other persistent-worker Wasm-AOT modules and scripts use their respective
   current-thread calls;
4. the remaining module and script cases use the ordinary engine calls.

Every engine call retains its existing source, compile options, timeout,
blocking policy and agent-prelude arguments. The change does not alter
materialized source, prelude selection, negative-test classification, timeout
policy, Test262 result accounting or emitted Wasm.

## Evidence and limits

The recursive structure guard pins the private, derive-free enum with its
exact `cfg(test)` variant attribute, all eight production mentions, both exact
producers, the product catch-boundary forwarding call, the typed consumer and
both exhaustive projections together with the complete ordered engine-call
region:

```console
cargo test -p lila-test262 --test wasm_aot_execution_stack_structure
cargo test -p lila-test262 tests::execute_cases_runs_wasm_aot_cases_on_persistent_workers -- --exact --test-threads=1
cargo test -p lila-test262 tests::wasm_aot_enforces_async_done_output_after_jobs_drain -- --exact --test-threads=1
```

The persistent-worker witness executes two ordinary Wasm-AOT cases through
`execute_cases`. The async-output witness reaches the test-only dedicated
entry point. The structure target passes `4/4`, and both exact behavioral
witnesses pass `1/1`. Exact agent-route ownership is structural evidence;
these focused witnesses do not claim full agent coverage, Test262 conformance,
a refreshed snapshot or closure of T03's remaining harness-materialization
debt. Independent dry re-review is clean after the recursive route censuses
were made Rust-lexical. The following shared workspace checkpoint passes
`cargo fmt --all -- --check`, `cargo xc`, the recursive module-boundary check,
the task-plan check and `git diff --check`.
