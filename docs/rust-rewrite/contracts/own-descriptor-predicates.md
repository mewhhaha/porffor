# Own-descriptor predicate contract

This note defines the bounded T10 seam shared by `Object.hasOwn`,
`Object.prototype.hasOwnProperty` and
`Object.prototype.propertyIsEnumerable` in the Wasm-AOT backend. These three
builtins ask one underlying question: obtain the receiver's own property
descriptor for a converted property key, then project either descriptor
presence or `[[Enumerable]]`.

Before this seam, only `Object.hasOwn` delegated the question to the public
`[[GetOwnProperty]]` protocol. The two prototype methods separately scanned
Array, arguments, boxed-String, Proxy and ordinary storage. That parallel
representation inventory omitted integer-indexed TypedArray elements, so a
valid element descriptor could be observable through
`Object.getOwnPropertyDescriptor` while both prototype predicates reported it
absent.

## Closed compiler domain

`OwnDescriptorPredicateBuiltin` is the complete compiler domain:

1. `ObjectHasOwn`;
2. `PrototypeHasOwnProperty`;
3. `PrototypePropertyIsEnumerable`.

One compiler helper matches that domain exhaustively, without a wildcard, for
three independent decisions:

1. **input source** — the static builtin takes receiver/key from arguments 0/1;
   both prototype builtins take receiver from `this` and key from argument 0;
2. **conversion order** — `Object.hasOwn` performs `ToObject` before
   `ToPropertyKey`; the prototype methods perform `ToPropertyKey` before
   `ToObject`;
3. **projection** — the two presence predicates test whether the descriptor is
   absent, while `propertyIsEnumerable` reads the `enumerable` data field from
   the returned descriptor object.

Adding a fourth builtin therefore requires choosing all three facts or fails to
compile. The public wrapper for each builtin is only a selection of one enum
variant; no wrapper is allowed to inspect heap representation details.

## Semantic operation

After conversion, the helper performs exactly one direct call through the
`Object.getOwnPropertyDescriptor` builtin metadata. The key is already a
String or Symbol and the receiver is already an object, so the builtin's
boundary conversions are idempotent and introduce no additional user-code
calls. This delegates integer-indexed, Array, arguments, boxed-String,
Function-special, ordinary and Proxy behavior to the existing canonical public
descriptor path rather than maintaining another representation list.

Abrupt completion from key conversion, object conversion or the descriptor
operation is returned before projection. `propertyIsEnumerable` reads only the
materialized descriptor object's own data field; it does not read the target
property value and therefore cannot invoke a target getter.

The conversion-order distinction is observable and must not be folded away:

- `Object.hasOwn(null, keyWithEffects)` throws its current-function-realm
  `TypeError` without coercing the key;
- each prototype predicate first coerces `keyWithEffects`, so an abrupt key
  conversion wins over the nullish-receiver `TypeError`.

The first two orders are witnessed directly by the pinned Test262 files
`built-ins/Object/hasOwn/toobject_before_topropertykey.js`,
`built-ins/Object/prototype/hasOwnProperty/topropertykey_before_toobject.js`.
The pinned `propertyIsEnumerable/symbol_property_toPrimitive.js` and
`S15.2.4.7_A13.js` cases separately witness key conversion and nullish
rejection; the focused fixture below combines them to make their required
ordering durable in this repository.

## Static boundary

`scripts/check-module-boundaries.sh` keeps the following facts reviewable:

- one closed enum, one shared compiler and exactly three wrapper selections;
- three exhaustive `match builtin` decisions and no wildcard or
  `unreachable!` escape;
- one `Object.getOwnPropertyDescriptor` metadata lookup and one direct call in
  the shared compiler;
- no raw `HEAP_*`, array/arguments descriptor scan or
  `emit_object_own_property_present` call in the shared compiler or wrappers;
- one durable CLI fixture and one exact test wiring it.

The focused regression covers valid, invalid and detached TypedArray indices;
BigInt elements; ordinary String and Symbol properties; boxed-String UTF-16
indices and `length`; accessor presence/enumerability without getter
invocation; Function `prototype`; inherited Proxy traps and abrupt trap
identity; coercion order; and current-function-realm `TypeError` identity.

## Deliberate boundary

This is a consumer consolidation, not a new descriptor-record implementation.
It does not make the public `Object.getOwnPropertyDescriptor` path
allocation-free, close recursive nested-Proxy target validation, implement
module-namespace descriptors, replace other context-specific
`emit_object_own_property_present` users, or close T10. It changes no
Test262/README status and makes no claim about the full Object, Reflect,
TypedArray or Proxy trees until their deferred runtime verification runs.
