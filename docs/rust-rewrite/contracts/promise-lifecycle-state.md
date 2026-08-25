# Promise lifecycle state as a closed wire domain

## Specification boundary

An ECMAScript Promise has exactly three lifecycle states:

- `pending`;
- `fulfilled`; or
- `rejected`.

Only `pending` is a valid initial state. `FulfillPromise` and `RejectPromise`
perform the only terminal transition, and both require the Promise to still be
pending. `PerformPromiseThen` consumes the same domain exhaustively: it appends
both reactions while pending, enqueues the fulfil reaction when fulfilled, and
otherwise asserts that the state is rejected before enqueueing the reject
reaction.

The Wasm-AOT Promise record keeps the historical stable encoding:

| lifecycle state | wire word |
|---|---:|
| `Pending` | 0 |
| `Fulfilled` | 1 |
| `Rejected` | 2 |

`PromiseState` is the sole authority for those words. `PromiseSettlement` is a
separate two-variant source-level domain containing only `Fulfill` and `Reject`.
It is deliberately impossible to pass `Pending` to a terminal-settlement API.
Promise reaction `[[Type]]` remains a distinct domain even though its two wire
words happen to equal the terminal Promise-state words.

## The bug class

The state offset and three integer constants were crate-visible, and
`emit_settle_promise_record` accepted an arbitrary `u64`. Passing `Pending` or
an unknown integer compiled. Consumers then disagreed about its meaning:

- reaction attachment treated every non-pending, non-fulfilled word as
  rejected;
- settlement selected the reject-reaction list for every non-fulfilled word,
  but only the exact rejected word entered unhandled-rejection tracking; and
- unhandled-rejection reporting ignored every word other than exact rejected.

No JavaScript program can directly forge this internal word. This is a record
integrity and future-producer boundary: an emitter omission or memory-layout
defect must trap rather than silently choose one of several inconsistent
fallbacks.

## Producer invariant

The raw offset is private to the heap boundary. Promise allocation calls the
typed initializer, which writes `PromiseState::Pending`. Terminal producers
call the typed store through `emit_settle_promise_record` and must supply
`PromiseSettlement::Fulfill` or `PromiseSettlement::Reject`; no integer or
pending state is accepted.

All Promise-direction helpers use the same terminal domain, including static
resolve/reject dispatch, resolving functions, `allSettled` element records,
async disposal, async iteration and async-function rejection paths. Exhaustive
Rust matches select their distinct fulfil/reject behavior. Adding a terminal
variant therefore fails compilation until every policy is defined.

## Consumer invariant

The sole raw load helper compares a stored word with every member of
`PromiseState::ALL`. It copies a known word to the destination and emits Wasm
`unreachable` for an unknown word. No consumer reads the offset directly.

One reaction-pair router owns the complete valid-state behavior:

| state | behavior |
|---|---|
| `Pending` | append fulfil and reject reactions to their respective lists |
| `Fulfilled` | enqueue the fulfil reaction with `[[PromiseResult]]` |
| `Rejected` | enqueue the reject reaction with `[[PromiseResult]]` |

The router's emitted comparison chain is derived from `PromiseState::ALL`, and
its Rust behavior selection is exhaustive. Ordinary `then`, async-function
`await` and async-generator return-await share this router.

## Terminal transition order

Once the strict state load proves that a Promise is pending, settlement:

1. captures the selected reaction list;
2. stores `[[PromiseResult]]`;
3. clears both reaction-list fields;
4. stores the typed terminal state;
5. for rejection, performs the host unhandled-rejection tracking step; and
6. enqueues the captured reactions.

This preserves the specification's lifetime and ordering boundary. Retaining
the obsolete lists after settlement would keep reaction graphs alive after
they cease to be semantically reachable. Enqueueing rejection reactions before
host tracking would invert the specified `RejectPromise` order.

## Durable evidence

Heap tests pin the stable words, the terminal-to-state mapping, the private raw
offset boundary, the sole initializer/store/strict decoder, and the unknown-word
trap. A structural Promise-emitter test pins the one exhaustive reaction router
and rejects the retired raw state constants and integer direction parameters.

Existing engine contracts cover valid-state behavior: pending reactions run in
registration order, already fulfilled and rejected Promises schedule the right
reaction asynchronously, hostile thenables settle once, and Promise races keep
their first settlement. JavaScript fixtures do not attempt to manufacture an
invalid internal state.

## Recorded verification

The exact `promise_lifecycle` heap tests pass `2/2`. The engine regressions for
reaction ordering and hostile thenable scheduling each pass `1/1` on
2026-08-25. These are focused record and behavior checks; no complete Promise
Test262 filter was run for this checkpoint.

## Nonclaims

This invariant does not make the job queue realm- or agent-owned, add module or
finalization-cleanup jobs, or make rejection tracking realm-local. The separate
main-job checkpoint contract owns reporting every rejection in its detached
still-unhandled snapshot. This state-word invariant also does not type async-
generator lifecycle states, complete suspended async bodies, establish GC
completion, or close the Promise/async Test262 gate. It is not expected to
change a valid-program conformance count by itself.
