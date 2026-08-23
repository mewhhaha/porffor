# Async-generator complete-step result kind

Status: implemented, independently reviewed and focused-verified for the
T04/T15 Wasm-AOT invariant lane on 2026-08-23.

## Specification boundary

The edition-pinned ECMA-262
[`AsyncGeneratorCompleteStep`](https://tc39.es/ecma262/2026/multipage/control-abstraction-objects.html#sec-asyncgeneratorcompletestep)
operation resolves the active async-generator request with an iterator-result
object whose `done` field is supplied by its semantic caller. The two states
used by Lila are closed:

- [`AsyncGeneratorYield`](https://tc39.es/ecma262/2026/multipage/control-abstraction-objects.html#sec-asyncgeneratoryield)
  completes a live yield request with `done = false`; and
- terminal body completion, completed-generator queue draining and awaited
  return completion complete the request with `done = true`.

The Boolean is not an arbitrary property of a call site. It is a projection of
whether this request publishes a yielded value or a completed generator. A
transposition compiles today but contradicts the generator lifecycle: a live
yield can appear terminal, or a terminal result can invite another iteration.

## Closed state

Replace the raw Boolean accepted by `emit_complete_async_generator_step` with:

```rust
pub(crate) enum AsyncGeneratorCompleteStepKind {
    Yielded,
    Completed,
}
```

Only the complete-step helper may project this state to the iterator-result
Boolean, immediately before calling the existing generic iterator-result
materializer:

```rust
let done = match kind {
    AsyncGeneratorCompleteStepKind::Yielded => false,
    AsyncGeneratorCompleteStepKind::Completed => true,
};
```

The match must remain exhaustive. The enum has no `Default`, catch-all arm,
predicate-to-bool helper, raw-bool constructor or second projection. The
broader `emit_iterator_result_object_from_locals(..., done: bool, ...)` helper
serves other iterator algorithms and remains outside this lane.

Because `promise.rs` is a private builtin module while owners live in
`functions.rs` and `standard.rs`, `builtins/mod.rs` re-exports the closed state
at crate visibility. No caller should name the private module path or recreate
the Boolean locally.

## Exact ownership

The current product path has exactly eleven complete-step calls:

- `emit_start_async_generator_body`: three `Completed` calls for awaited body
  return, failed awaited-return reaction and terminal body completion;
- `emit_drain_async_generator_queue`: three `Completed` calls for a queued
  normal completion, queued throw and rejected awaited return;
- `emit_run_async_generator_await_return_job`: one `Completed` call;
- `compile_standard_builtin`: three `Completed` calls for requests handled at
  an already-completed or suspended-start generator; and
- `emit_complete_async_generator_yield`: the sole `Yielded` call.

That is `10 Completed + 1 Yielded`. The owner identity is stronger than the
global count: a swapped pair preserves `10/1` while inverting two observable
results. A durable guard must therefore bind the sole `Yielded` state to
`emit_complete_async_generator_yield` and require only `Completed` in every
other owner body.

## Preserved complete-step order

The state migration changes only the final `done` projection. The helper must
continue to:

1. require one active request;
2. read its promise capability;
3. remove the queue head;
4. clear the active-request slot;
5. reject a Throw completion, or create the iterator-result object and resolve
   the request for a Normal completion;
6. reject any other completion kind as unreachable;
7. normalize the emitter completion state; and
8. release its temporary locals in reverse reservation order.

The iterator-result object is not materialized for a Throw completion, so some
terminal paths do not observe the `Completed` projection today. They still
carry the lifecycle state explicitly; a later settlement refactor must not
inherit an arbitrary Boolean from those callers.

## Durable structural witness

`crates/lila-aot-wasm/tests/async_generator_complete_step_kind_structure.rs`
should require:

- exactly the two enum variants and one exhaustive Boolean projection;
- no `done: bool` parameter, Boolean literal call argument, catch-all,
  `Default` implementation or alternative projection in the bounded owners;
- exactly eleven calls split among the five named owner bodies;
- exactly one `Yielded` call in `emit_complete_async_generator_yield` and ten
  `Completed` calls in the four terminal owner bodies;
- the complete-step helper's active-request, queue-removal, active-clear,
  Throw-settlement, iterator-result, resolve and completion-normalization
  order; and
- the crate-visible re-export used by `functions.rs` and `standard.rs`.

The guard should extract only the enum, helper and named owner bodies. It must
not snapshot complete Promise, function or standard-builtin emitters.

## Focused runtime witnesses

The initially selected broad CLI candidate was
`async_generator::wasm_backend_resumes_async_generator_loops_for_zero_one_and_many_iterations`,
using `crates/lila-cli/tests/fixtures/wasm_async_generator_resumable_loop.js`.
It is not a valid green gate for this invariant: both unchanged `HEAD` and the
typed-state implementation produce the same failing output. Their yielded and
terminal `done` values are correct, but resumable classic loops lose later
iterations and post-yield lexical state. That existing activation/resumption
defect remains T15 work and is not caused by this Boolean-to-enum migration.

The exact current-pin Test262 witness is:

- `language/expressions/async-generator/expression-yield-as-operand.js`.

It must be invoked by its complete suite-relative path with the Wasm-AOT
backend, `--jobs 1`, `--threads 1` and the repository timeout. Verification
must inspect the discovery total and every failure bucket.

## Verification evidence

The implementation and swap-resistant owner guard were independently reviewed.
Under the shared eight-core, 22 GB cap, `cargo fmt --all -- --check`, `cargo
xc` and `git diff --check` are green, and the structural witness passes `4/4`.
The exact Test262 leaf above discovers and passes `2/2` Wasm-AOT variants with
every failure and non-success bucket at zero under `--jobs 1 --threads 1`.

The broad CLI candidate fails `0/1` on both the pre-change `HEAD` worktree and
the current implementation with byte-identical observable output. This
baseline comparison is negative-scope evidence only; it is neither hidden nor
counted as a passing verification result for this lane.

## Explicit nonclaims

This lane does not redesign async-generator queues, completion words,
activation layout, Promise settlement, resumption, `yield*`, awaited return or
the generic iterator-result materializer. It does not claim broader
async-generator conformance, a new Test262 pass, a published-count change, or
completion of T04 or T15. It makes one already-observable lifecycle choice
unrepresentable as an accidental Boolean at its eleven product call sites.
