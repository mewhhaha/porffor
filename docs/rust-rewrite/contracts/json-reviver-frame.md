# JSON reviver frame protocol

## Scope

This contract owns the iterative Wasm-AOT implementation of
`InternalizeJSONProperty`, including the static-JSON specialization's final
reviver application. It does not own JSON tokenization, primitive decoding,
`JSON.stringify`, or the validity of the static source proof.

The reviver walk is depth-first postorder. For each property it observes the
current value, recursively internalizes that value's children, calls the
reviver, and only then applies the reviver result to the holder. A reviver can
replace an unvisited child, install accessors or proxies, mutate later
properties, or throw an arbitrary JavaScript value. The walk must observe all
of those effects through the ordinary object operations at their specified
positions.

## Frame state

The dynamic walk stores one private frame per active property. Its state is the
closed domain:

- `Enter`: read the current property value and classify it;
- `ArrayChildren`: visit the snapshotted array-index range in ascending order;
- `ObjectChildren`: visit the snapshotted enumerable own string keys in order;
- `Apply`: call the reviver and consume its result.

`Enter` performs `Get` before classification. For an Array it observes and
converts `length` once, then stores that limit on the frame. For another Object
it obtains the enumerable own string-key list once, then stores that list and
its length. Child frames are pushed in ascending cursor order onto a LIFO
stack, which makes their `Apply` steps run before the parent's `Apply` step.

Every persisted state word comes from `JsonReviverFrameState`. Runtime dispatch
is generated from its complete ordered set and reaches an exhaustive Rust
match. An invalid word traps as an internal invariant failure instead of
silently inheriting one state's behavior. Adding a state therefore requires an
explicit emitter decision before the backend builds.

## Root versus nested properties

The synthetic wrapper property used by `JSON.parse` is semantically different
from an ordinary child property. That distinction is the closed
`JsonReviverPropertyRole` domain:

- `Root`: the reviver result is the result of `JSON.parse` and does not mutate
  the wrapper;
- `Nested`: `undefined` requests deletion from the holder, while any other
  result requests creation or replacement of the holder property.

The role is explicit at both static and dynamic reviver call sites. It is never
derived from the key spelling: an ordinary nested property named the empty
string is still `Nested`. Dynamic frames persist the role through its stable
wire word, and frame creation accepts the typed role rather than a Boolean
local. Only the shared post-call emitter consumes the distinction, so the
static specialization and dynamic parser cannot drift into different root or
child mutation rules.

## Source context and abrupt completion

Parse metadata may provide the third reviver argument's `source` property only
for a primitive whose current value remains `SameValue` to the value produced
from that source slice. Mutation clears that eligibility. Arrays and Objects
receive an empty context object.

Every observable `Get`, `IsArray`, key enumeration, length conversion, reviver
call, deletion and property creation retains its existing abrupt-completion
edge. A throw stops the walk immediately and is propagated unchanged. State or
role validation is an internal boundary check; it must not turn an ordinary
JavaScript abrupt completion into a parser error or a default result.

## Non-claims

This protocol does not close T20 or the pinned JSON tree. It does not validate
JSON grammar, remove the static specialization, prove deep-input resource
bounds, change non-configurable-property behavior, or cover stringify,
replacer, cycle, BigInt or proxy semantics outside the reviver walk. Complete
current-pin Wasm-AOT evidence remains a separate verification requirement.
