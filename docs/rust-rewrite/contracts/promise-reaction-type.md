# Promise reaction `[[Type]]` as a closed wire domain

## The record contract

An ECMAScript Promise reaction record has one `[[Type]]`: `Fulfill` or
`Reject`. This is not the Promise's lifecycle state. A Promise can be pending,
fulfilled or rejected, but a reaction is constructed for exactly one of the
two terminal paths and can never be pending.

The Wasm-AOT heap record stores `[[Type]]` as one word. The historical encoding
reused the numeric Promise-state constants:

| reaction type | wire word |
|---|---:|
| `Fulfill` | 1 |
| `Reject` | 2 |

Those words remain stable. The Rust authority is instead the separate closed
`PromiseReactionType` domain. Sharing numbers is an encoding fact, not a type
relationship.

## The bug class

`emit_initialize_promise_reaction` previously accepted any `u64`. Its three
producer pairs happened to pass only the fulfilled and rejected Promise-state
constants, but `Pending` or an arbitrary word also compiled. The six reaction
callbacks then interpreted an illegal word differently: some treated it as a
fulfilment, some as a rejection, and two trapped.

This was not a measured valid-program failure. It was a representational hole:
one internal record could carry a state the specification does not define, and
the observable outcome depended on which callback shape consumed it.

## Producer invariant

Every reaction is initialized with `PromiseReactionType::Fulfill` or
`PromiseReactionType::Reject`. The initializer accepts no integer substitute
and writes the selected variant's stable wire word. The three construction
families are:

1. ordinary and internal intrinsic-await reactions;
2. async-generator return-await reactions;
3. `PerformPromiseThen` reactions.

Each family constructs a pair, one of each type. Adding another producer cannot
omit this decision because it must supply the closed Rust type.

## Sole decoder

The Promise-reaction job runner reads the wire word once, before dispatching
the callback shape. It compares the word against the ordered
`PromiseReactionType::ALL` domain and normalizes the result to one runtime
`is_rejected` flag. An unknown word reaches `unreachable`; no callback receives
it.

The normalization policy is an exhaustive Rust match:

| reaction type | normalized flag |
|---|---:|
| `Fulfill` | 0 |
| `Reject` | 1 |

Adding a variant therefore fails compilation until its normalization is
defined. All six callbacks consume the same validated flag rather than reading
and reinterpreting the heap word independently.

## Callback meanings

The normalized flag preserves the distinct semantics of each callback shape:

| callback | Fulfill | Reject |
|---|---|---|
| default reaction, empty handler | call the capability's resolve function | call the capability's reject function |
| async function | resume from `await` normally | resume from `await` by throwing |
| async-generator await | resume with fulfil | resume with rejection |
| async-generator await-return | complete the request normally | complete the request by throwing |
| async-generator yield | complete the yielded value | resume the body with the rejection |
| async-generator yield-return | resume with return | resume by throwing |

A callable default handler still determines the derived Promise from its own
return or throw. The reaction type is consulted only for the specification's
empty-handler propagation path; normalization must not change that precedence.

## Durable evidence

The heap wire-domain test fixes the words and normalization policy. The runner
has exactly one reaction-type load, while the initializer has exactly one
reaction-type store. Existing engine contracts exercise both branches of the
default and async-function paths plus the async-generator await, return-await,
yield and suspended-yield-return paths.

## Nonclaims

This invariant does not type the Promise lifecycle state, async resume kinds or
async-generator state machine. It does not add module or finalization-cleanup
jobs, make the queue realm-owned, change unhandled-rejection reporting, or
establish full Promise/async Test262 conformance.
