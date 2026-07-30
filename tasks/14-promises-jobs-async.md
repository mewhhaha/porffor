# T14 — Promise jobs, async functions and async iteration

**Status:** In progress — Promise/job machinery is substantial; suspended async closure remains

**Parallel group:** Feature lane  
**Depends on:** T03, T04, T05, T06, T09  
**Blocks:** Async modules in T12, async iterators in T15, Atomics.waitAsync in T17

## Current repository state

The backend has Promise records, reaction/job queues, combinators, async
activation records and extensive focused real-suite coverage. README notes
still identify unsupported suspended-body and dynamic-source families, and
async generators, async iteration, module jobs and `waitAsync` share unfinished
boundaries with T12/T15/T17. The complete Promise/async filters have not met the
zero-failure acceptance gate for the current pin.

## Objective

Implement the ECMAScript job model, complete Promise semantics, async functions and async iteration with deterministic host integration suitable for Test262 and embedders.

## Job queue contract

- Define realm/agent-owned FIFO job queues and host enqueue/drain hooks.
- Keep promise reaction jobs, thenable jobs, async continuation jobs, module jobs and finalization cleanup jobs distinct where observably required.
- Drain jobs at specified host checkpoints; do not run them eagerly inside `then`/resolution.
- Preserve realm and incumbent/active execution context needed by each job.
- Integrate Test262 `$DONE`, timeouts and rejection reporting without treating an empty queue as success before async completion.

## Promise implementation

Implement:

- internal state/result/reaction lists and resolving functions;
- thenable assimilation, self-resolution rejection and already-resolved guards;
- `then`, `catch`, `finally`, species and derived promises;
- constructor executor ordering and abrupt completion;
- `resolve`, `reject`, `all`, `allSettled`, `any`, `race`, `withResolvers` and current pinned additions;
- iterator closing and AggregateError behavior;
- metadata/descriptors and cross-realm error ownership.

All combinators must use shared iterator operations rather than array-only shortcuts.

## Async functions

Lower async bodies to resumable state machines whose calls return promises immediately. Implement:

- `await` conversion/then behavior;
- suspension/resumption through queued jobs;
- return/throw/finally across suspension;
- lexical environment and `this`/arguments/new-target retention;
- async arrows/methods and class methods;
- async stack cleanup and GC rooting.

## Async iteration

Provide `AsyncFromSyncIterator`, async iterator acquisition/close, async `for-await-of`, and the interfaces required by async generators/iterator helpers in T15.

## Host and blocking behavior

`can_block` must affect Atomics/host behavior, not Promise ordering. Provide a deterministic test driver that can run jobs until completion or a deadline and report pending jobs/rejections on timeout.

## Acceptance criteria

- Promise state and resolution tests pass, including hostile thenables and side-effect ordering.
- Combinators pass iterator-close, species and subclassing tests.
- Async functions preserve environments and finally semantics across multiple awaits.
- Async Test262 cases pass/fail based on `$DONE` or returned async completion, with duplicate completion detected.
- Cross-realm promises and errors use correct intrinsics.
- No busy-loop polling is required for ordinary promise progression.
- The pinned Promise/async-function/async-iteration filters reach zero failures.

## Required tests

```sh
cargo test -p porffor-runtime job_ --quiet
cargo test -p porffor-aot-wasm promise_ --quiet
cargo test -p porffor-cli wasm_async --quiet
./target/debug/porf test262 run built-ins/Promise --execution-backend wasm --timeout-ms 120000 --threads 4
```

Also run async function, `await`, `for-await-of`, async iterator and top-level-await filters, plus intentionally hanging/duplicate `$DONE` harness tests.
