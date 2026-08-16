# Promise `finally` completion preservation as a closed domain

## Specification boundary

`Promise.prototype.finally` creates two internal closures when `onFinally` is
callable. `ThenFinally` observes a fulfilled source and, after successful
cleanup, must restore a normal completion with the source value. `CatchFinally`
observes a rejected source and, after successful cleanup, must restore a throw
completion with the source reason.

Both closures call `onFinally`, apply `PromiseResolve(C, result)`, and chain one
zero-argument closure to that cleanup promise. The only difference is the
completion which that final closure restores:

| source path | cleanup continuation | restored completion |
|---|---|---|
| `ThenFinally` | `ValueThunk` | normal with the original value |
| `CatchFinally` | `Thrower` | throw with the original reason |

An abrupt cleanup still replaces either original completion. The preservation
direction is consulted only after cleanup resolves normally.

## The bug class

The backend previously encoded this one two-way choice as two independent
booleans. `emit_promise_finally_continuation(rejected: bool)` selected
`ValueThunk` or `Thrower`, while
`emit_promise_finally_value_thunk(throws: bool)` independently selected a normal
or throw completion. The standard-builtin dispatcher supplied four naked
`false`/`true` literals.

The existing literals were correct, so this was not a measured valid-program
failure. It was a representational hole: inverting any one literal compiled
cleanly and silently made one half of `finally` restore the wrong completion.

## Closed producer invariant

`PromiseFinallyCompletion::{Fulfill, Reject}` is the sole source-level domain
for this choice. It is deliberately separate from Promise record settlement,
Promise reaction type, and Promise lifecycle state: those domains describe
different records and phases even though some have the same cardinality.

The shared continuation emitter accepts the closed type and uses an exhaustive
match to choose `ValueThunk` or `Thrower`. The shared value-restoration emitter
accepts the same type and uses an exhaustive match to choose `Normal` or
`Throw`.

The standard-builtin dispatcher has no choice argument. It calls four named
wrappers:

- `emit_promise_then_finally` selects `Fulfill`;
- `emit_promise_catch_finally` selects `Reject`;
- `emit_promise_value_thunk` selects `Fulfill`; and
- `emit_promise_thrower` selects `Reject`.

This keeps the mapping next to semantic names and prevents a caller from
supplying an unlabelled boolean.

## Durable evidence

The structural test pins the two-variant private domain, both exhaustive
matches, the named wrapper mapping, and the absence of the retired boolean
parameters and naked boolean calls.

Existing engine coverage exercises fulfilled-value preservation, rejected-
reason preservation, awaited cleanup, and abrupt cleanup replacement. The
pinned `Promise.prototype.finally` tests independently specify the fulfilled
and rejected preservation laws.

## Nonclaims

This invariant does not add new valid JavaScript behavior, expand suspended
async-body lowering, change job scheduling or realm ownership, change
unhandled-rejection reporting, establish GC completeness, or close any Promise
or async Test262 filter. It is not expected to change conformance counts.
