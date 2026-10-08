# Plain async for-await body continuations

The 2026-10-04 source batch extends the completed plain-async iterator plan to
`for await` bodies containing Await. Compilation, emitted Wasm, runtime controls,
source guards and full T14 acceptance remain unverified. Complete the remaining
task source before the serial verification checkpoint under a confirmed 4096 MiB
aggregate kernel memory cap, zero swap and inherited CPU affinity.

## Consumed plan and state ownership

The actual ForOf dispatcher delegates to the private `lowering/for_of/async_function`
owner. The completed `AsyncFunctionForOfIteratorPlanIr` lives in the consumed
`ir/async_for_of_iterator` leaf and retains the existing head, Iterator Record,
checked body and iteration environment authorities. Its closed execution domain
selects synchronous Iterator stepping or awaited Next/Close. Only the completed
plan can construct the borrowed execution views; an awaited plan cannot enter the
synchronous resumable backend adapter.

For Next entry e, the value/body entry is e.checked_add(1). The producer seeds the
ordinary async body lowerer with that state before lowering any body Await. If the
checked body ends at b, the close resume is b.checked_add(1), and the loop exit is
the close resume's checked successor. The constructor checks the actual body's
entry and exit and the producer's final state. It owns both retained Boolean
protocol bindings; storage census and planning consume them as protocol storage,
not user declarations. No raw state tuple can replace the checked body's states.

The same exhaustive body validation preserves Await adjacency, conditional and
try/catch/finally clause spans, and current-loop unlabelled branch ownership. The
awaited execution rule also rejects materialized body or catch environments at
any depth. Captured head cells remain permitted and receive a fresh environment
per entered iteration. Uncaptured body declarations keep their ordinary storage.
The captured body/catch boundary is a real retained-environment lifecycle gap;
full support requires restoring the retained head separately from the deepest
suspended child before widening admission.

## Existing iterator algorithm

The completed awaited view reaches the existing Next/body/Close emitter. Iterator
acquisition and next-method Get happen once, and body resumption cannot request
another Next or initialize the head again. True async results observe done before
value and omit the terminal value. Sync fallback consumes the existing
AsyncFromSyncIterator continuation, which Gets and Awaits value even when done is
true. The pending completion, original rejection value and Realm transport retain
their existing owners.

Current-loop Continue finishes local awaited finalizers and advances without
return Get or Call. Break, Return and escaping Throw select their final completion
before awaited Close. Close preserves selected Throw and may replace selected
Break or Return. Next failures and the existing sync-value rejection route retain
their no-double-close behavior. No new activation layout, iterator backend,
completion ABI, opcode or Await helper is introduced.

## Source domain and pending controls

The new body-Await route admits plain async functions with an eager iterable and
eager Var, Let or Const identifier binding head. Direct Await, supported async If,
local try/catch/finally, Return/Throw and current-loop unlabelled Break/Continue
consume the existing checked body owner. Suspended iterable/head operands,
destructuring and assignment heads, resource heads, nested resumable loops,
labelled or foreign branches, additional captured body/catch scopes and
async-generator composition remain explicit capability gaps. Existing eager
for-await and async-generator Yield routes retain their own admission.

Meaningful constructor and actual lowering controls exercise state adjacency,
overflow, protocol bindings, following-state joins, captured heads and refusal of
unowned environments and branches. Three finite strict/sloppy WasmAot cohorts in
the maintained `aot_for_await_rejection_close` target cover interleaved true-async
and sync-fallback execution, cached callable-Proxy next, two body awaits, captured
head cells, local completion through awaited finalizers, exact close precedence,
abrupt cutoffs and borrowed native-error Realms. They require exact Normal 262,
a sole cohort print, one compilation worker and a finite timeout. The previous
rejection/close cohorts remain separate controls.

After all source authoring finishes, run the shared async-body constructor and
`async_for_of_continuations` lowering targets, the maintained
`plain_async_sync_for_of_iterator_record_structure` guard and the existing
`aot_for_await_rejection_close` Engine target. Retain synchronous for-of,
async-generator, captured-head, Promise/rejection and close-precedence coverage
before the broad checkpoint. Authored controls and historical type receipts do
not establish current executable acceptance or close a task.
