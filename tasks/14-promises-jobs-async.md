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

The AOT pending-job record now has one closed Rust `PromiseJobKind` domain for
the two job shapes the product path actually enqueues: Promise reactions and
thenable resolution. Both producers encode that type, and the main-export drain
derives its comparison chain from the domain before selecting a handler through
an exhaustive match. An unknown word traps instead of silently running as a
thenable job. A private payload-bearing `PromiseJobToEnqueue` now requires each
producer to supply its argument and realm policy before the sole FIFO append;
new job shapes cannot grow a second queue-order implementation.
The job and reaction-callback enum, ordered `ALL` set and stable wire word now
come from the same macro row, with a const dense-range proof; there is no second
hand-written variant list that can omit a new row.

Promise reaction callback words are also one closed six-variant Rust domain.
Reaction construction writes that typed word once, rather than initializing a
default and repairing internal async continuations afterward, and the runner's
ordered comparison chain selects behavior through an exhaustive match. Default
reaction jobs derive `GetFunctionRealm(handler)` at enqueue time or carry the
specification's null realm for an empty handler; internal async continuations
carry their captured realm. Thenable jobs derive the `then` callback realm.
Both callback lookups select the enqueue-time current realm for a revoked Proxy,
and the drain maps a null job realm to its saved host-checkpoint realm instead
of installing zero or leaking the preceding job's realm.

The reaction record's `[[Type]]` is now a separate closed
`PromiseReactionType::{Fulfill, Reject}` domain rather than a raw Promise-state
word. All three producer pairs must select the type before construction. The
reaction-job runner decodes the stable wire words 1/2 once into a normalized
rejection flag, traps an unknown word, and threads that flag through all six
callback shapes. No callback independently treats an invalid word as its own
fallback. This is a record-integrity boundary; valid reaction behavior and the
wire encoding are unchanged.

Ordinary async-function activations now store the completion supplied by
`Await` through one closed `AsyncFunctionResumeCompletion::{Normal, Throw}`
domain. The raw offset and stable words 0/1 are private to the heap boundary;
activation initialization and the reaction continuation must use the typed
store. Ordinary `await` and both `for-await-of` resume sites use the sole strict
decoder, which normalizes to one `is_throw` flag and traps an unknown word
instead of treating it as fulfilment. The shared `for-await-of` emitter now has
a closed async-function/async-generator layout choice, so the generator's
separate five-way resume-kind behavior stays explicit rather than being folded
into an integer tuple. This is also a record-integrity boundary: the existing
valid 0/1 behavior is unchanged, while illegal internal words fail closed.

Promise records now store `[[PromiseState]]` through one closed three-variant
`PromiseState::{Pending, Fulfilled, Rejected}` wire domain. The raw offset is
private to typed initialization, terminal-store and strict-load helpers, and an
unknown word traps instead of falling through as rejection. The separate
`PromiseSettlement::{Fulfill, Reject}` domain is accepted by every terminal
producer and Promise-direction helper, so `Pending` or an arbitrary integer can
no longer be supplied where a terminal choice is required. Promise reaction
`[[Type]]` remains distinct despite sharing the two terminal wire words.

One exhaustive reaction-pair router now owns the pending/fulfilled/rejected
behavior shared by ordinary `then`, async `await` and async-generator
return-await. Terminal settlement captures the selected reaction list, stores
the result, clears both obsolete lists, stores the typed state, performs
rejection tracking when required, and only then enqueues the captured reactions.
This closes the Promise lifecycle record and transition-order boundary; it is
not a claim of broader queue ownership, suspended-body support, GC completion or
full Promise conformance.

Main Script completion now has one closed exit policy. While source statements
are emitted, every otherwise-terminal abrupt completion targets a code-sink
tracked host-checkpoint block instead of returning from the Wasm export. The
checkpoint drains jobs and then publishes the original Script completion;
internal functions retain their direct four-word completion return. The drain
also preserves the thrown error-name/message globals alongside the completion
tuple, so an error raised by a queued job cannot overwrite the identity or
message of an already-pending top-level throw. A durable engine regression
requires the queued job's print side effect and the primary throw identity;
central compile and that focused runtime contract remain queued behind the live
current-pin matrix.

This closes the current record/ordering/realm-source boundary; it does not yet
provide the broader realm/agent-owned host queue contract. Async continuations
still ride on reaction records, while module and finalization-cleanup jobs
remain outside this two-kind queue. Full execution-context switching and
realm-correct allocation across the complete builtin surface also remain T06
work, so this is not a claim of complete cross-realm Promise conformance.

The central feature-enabled CLI compile covers the consolidated job machinery.
The typed callback-word/realm policy's durable layout contract is green, as are
the engine contracts proving that reaction jobs run after synchronous code in
registration order and that thenable-resolution jobs are asynchronous and
settle once. The ordinary async resume-completion contract has been added for
central verification, as has the typed Promise-lifecycle contract. Their
focused compile/runtime checks remain queued behind the live current-pin
matrix. Those checks are not a substitute for the full Promise/async Test262
filters.

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

### Unhandled rejections now surface (fixed 2026-08-02)

A promise that rejected with no handler used to produce no diagnostic and exit
status 0. Combined with top-level await wrapping module bodies in an async
function, that meant a `flags: [module]` Test262 case whose assertion FAILED was
scored as a PASS - the measurement reporting green on red.

Fixed: rejected-with-no-handler promises are tracked on a list, and after the
job-drain loop the main export's completion kind is set to Throw carrying the
rejection value. Verified in both directions, which is the part that matters -
a fix that reported *handled* rejections would have turned passes into failures:

| case | reported | exit |
|---|---|---|
| `(async () => { throw ... })()` | yes | 1 |
| `await 0; throw new Test262Error(...)` | yes | 1 |
| immediate `.catch` | no | 0 |
| `.catch` attached in a *later* job | no | 0 |
| `try`/`catch` around `await` | no | 0 |
| `Promise.all` rejected then caught | no | 0 |

Implementation note: the promise record grew from 64 to 72 bytes for the list
link, and the global registry gained two slots.

Two holes remain in the same story:

- **Only the oldest unhandled rejection is reported**, because the main export
  carries a single completion value. Hosts normally report every one. Adequate
  for pass/fail scoring, imprecise as a diagnostic. `emit_report_unhandled_rejection`
  already walks the whole list and could print the rest via the host `print`
  import before setting the throw completion.
- The rejection list is process-global rather than per-realm, so cross-realm
  (`$262.createRealm`) promises share one tracker. Untested territory rather
  than a known break - cross-realm is the one feature still failing the probe.

Also fixed 2026-08-02: `await` inside a loop body was miscompiled - state living
across a suspension point inside a loop was not restored, so
`for (let i = 0; i < 3; i++) { t += await Promise.resolve(i); }` summed to 0
instead of 3. Now correct, including the `const v = await ...` and `for-of`
variants.

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
cargo test -p lila-runtime job_ --quiet
cargo test -p lila-aot-wasm promise_ --quiet
cargo test -p lila-cli wasm_async --quiet
./target/debug/lila test262 run built-ins/Promise --execution-backend wasm-aot --timeout-ms 120000 --threads 4
```

Also run async function, `await`, `for-await-of`, async iterator and top-level-await filters, plus intentionally hanging/duplicate `$DONE` harness tests.
