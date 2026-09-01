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

The completion domain is non-`Clone`, non-`Copy`. Its two projections consume
`self`, so a continuation or restoration helper cannot consult the same choice
twice by value: a second projection is an E0382 move error. The two runtime
stages remain independently constructed by their four exact wrapper producers;
this ownership closure does not pretend that one Rust value crosses the Promise
job boundary.

## Private child boundary

`builtins/promise/promise_finally_completion.rs` is the sole owner of the
domain, both exhaustive projections, all four named producers and both raw
consumers. The Promise parent and standard-builtin dispatcher cannot name,
import, re-export or construct `PromiseFinallyCompletion`; the dispatcher can
only call the unchanged `pub(crate)` semantic wrappers. The two raw consumers
remain private to the child.

The pre-extraction 25-line domain/policy and 219-line method lifecycle retain
SHA-256
`c9f079447b9c82792ae69b5438e13811ef9ac5a99c62080052837eb9a6b0edf3`
and
`e42e360bf0d1739557b05ffd2c5429b3cde863d6f4bb9c75e8737f858aa4dcb6`.
Their combined 244 selected lines retain SHA-256
`464867bbceb6ffda71e52cef1b733fe254abd36d76a3eef9fb16c95d4c1501d6`.
The formatted 248-line child has SHA-256
`8aff4f6c500f2eaaf48f61b68ddfe5c5fac0f534cd95b1d2fabd5302bb1ceef3`
and reduces the concurrent Promise parent from 8,717 to 8,473 lines.

## Durable evidence

The structural test pins the two-variant private domain, both exhaustive
matches, the named wrapper mapping, and the absence of the retired boolean
parameters and naked boolean calls.

The hardened Rust-lexical guard owns all eight lexical mentions, the four exact
wrapper producers, and the two consuming projections. Each bounded consumer
contains exactly its parameter declaration and one projection, while the
continuation selection remains before function metadata and context work and
the restored completion remains after both value loads and before local
release. Normalized length/FNV fingerprints additionally pin both complete
consumer bodies, so inserted, duplicated or reordered emission cannot evade
those semantic assertions. The include-only recursive structure target passes
`4/4` after the private child extraction. After the extraction, the exact
Promise-finally settlement engine witness and created-Realm Promise
internal-callback CLI witness each pass `1/1`. Independent review confirmed the
capability, wrapper and cross-stage nonclaims and added the complete consumer
fingerprints. The shared `cargo xc`, formatting, diff, module-boundary and
task-plan checks are green. No semantic golden, broad suite or Promise Test262
run was performed for this source-equivalent move.

Existing engine coverage exercises fulfilled-value preservation, rejected-
reason preservation, awaited cleanup, and abrupt cleanup replacement. The
pinned `Promise.prototype.finally` tests independently specify the fulfilled
and rejected preservation laws.

## Nonclaims

This invariant does not add new valid JavaScript behavior, expand suspended
async-body lowering, change job scheduling or realm ownership, change
unhandled-rejection reporting, establish GC completeness, or close any Promise
or async Test262 filter. It is not expected to change conformance counts.
