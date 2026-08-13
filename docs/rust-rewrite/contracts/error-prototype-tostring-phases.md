# Error prototype stringification phases

Status: normative for the Wasm-AOT `Error.prototype.toString` seam.

## Semantic boundary

`Error.prototype.toString` is generic over every ECMAScript Object. The
receiver does not need `[[ErrorData]]`, and the backend's ordinary Object,
Function, Array and Arguments representations therefore all enter the same
algorithm. A Proxy enters through the ordinary Object representation and its
`[[Get]]` traps remain observable.

The algorithm has two ordered observable phases:

1. reject a non-Object receiver with a `TypeError` from the builtin's defining
   realm;
2. perform `Get(receiver, "name")`;
3. replace `undefined` with `"Error"`, otherwise apply `ToString`;
4. only after that conversion completes, perform `Get(receiver, "message")`;
5. replace `undefined` with `""`, otherwise apply `ToString`; and
6. return `message` when the prepared name is empty, return the prepared name
   when the prepared message is empty, or concatenate `name + ": " + message`.

The ordering is load-bearing. A name getter or conversion can throw, in which
case the message property must remain unobserved. A successful name conversion
can mutate the message property, and the later `Get` must observe that mutation.

## Closed receiver admission

The emitter uses the shared `emit_is_heap_object_like_tag_i32` predicate once.
That predicate is the backend authority for the four object representations:
Object, Function, Array and Arguments. No second hand-written subset may decide
whether the property reads run. Once admitted, both reads use the generic,
proxy-aware `emit_object_read` operation with the original receiver as the
`Receiver` argument.

This matters for more than Array and Arguments named properties. A second
representation list would also be free to bypass Proxy `get` traps or to drift
when the backend gains another Object representation.

## Compiler-enforced phase state

The private, `must_use` `PreparedErrorNameLocal` is returned only after the
name `Get`, defaulting and `ToString` emission have completed. The message and
result emitter requires and consumes that type; it cannot be called with the
raw receiver, name value or an unprepared Wasm local in place of the prepared
name.

The type exists only while Rust emits the Wasm body. Its payload remains an
ordinary local handle in the generated module. The name local is reserved
before the phase's transient locals and is released only after the message and
result phase consumes it, preserving the emitter's temporary-local lifetime.

## Conversion-error realm

The name and message phases invoke two abstract operations that can create a
fresh `TypeError` themselves: `ToPrimitive` rejects a non-callable
`@@toPrimitive` or a conversion that never produces a primitive, and
`ToString` rejects a Symbol. Those are errors created by
`Error.prototype.toString`, so their prototype comes from that builtin's
defining realm, not necessarily the entry realm.

The shared operation layer carries the private, closed
`ConversionErrorRealm::{MainRealm, CurrentFunctionRealm}` policy. Its existing
public ToPrimitive and primitive-ToString wrappers fix the policy to
`MainRealm`, preserving their current contract. The separately named
current-function-realm ToPrimitive wrapper fixes `CurrentFunctionRealm` and
returns an opaque, `must_use` primitive token whose private fields include that
same policy. Only the matching current-function-realm primitive-ToString
wrapper can consume the token, so the two conversion halves cannot drift to
different realms and the T24 emitter never receives a raw policy choice.

When ToPrimitive is outlined, helper parameter 2 carries the policy's closed
two-word ABI and parameter 6 carries the current Realm environment. The helper
body exhaustively decodes both words; an unknown word traps rather than
silently selecting a Realm. Existing callers emit the main-Realm word, while
the dedicated current-Realm wrapper emits the current-function word and passes
the builtin's Realm environment. Both ToPrimitive's internally created errors
and primitive ToString's Symbol error use the stored policy. There is no
boolean/default policy argument.

## Durable witness

`wasm_error_tostring_order_and_receivers.js` records getter and conversion
observations. Its name conversion mutates the value captured by the later
message getter, so both the order and the returned string are visible. A
separate throwing name conversion proves that the message getter is not run.
Array, Arguments, Function and Proxy receivers prove that every admitted
representation reaches the generic property path; the Proxy additionally
records the ordered `name,message` traps. A foreign-realm Error method checks
Symbol-valued name and message fields, a non-primitive conversion result and a
non-callable `@@toPrimitive`; each must produce the foreign realm's TypeError
and not an entry-realm TypeError.

The Rust structural gate pins the builtin body to one shared object-like guard,
the typed name preparation followed by the consuming message phase, and the
absence of a local `ValueKind::{Object,Function,Array,Arguments}` admission
list. It also pins the conversion boundary to the explicitly current-realm
ToPrimitive and primitive-ToString wrappers.

## Nonclaims and deferred gates

This seam does not change Error construction, Error cause installation,
`Error.isError`, native-error metadata, `Object.prototype.toString`, Proxy or
ordinary-object semantics, or the Test262 harness. It does not close the full
Error, NativeErrors or T24 trees and changes no published conformance count.

Static freeze gates are exact-file formatting, fixture syntax inspection,
source-structure inspection, `git diff --check` and independent review. Cargo,
fixture execution, focused pinned Error leaves and the broad T24 verification
ladder remain deferred until the active current-pin matrix releases the shared
runtime.
