# String exotic property-key classification

## Normative basis

For `base[key]` where `base` is a String primitive, evaluation first produces
the receiver and the computed key. The key is converted once through
`ToPropertyKey`. String exotic `[[GetOwnProperty]]` then distinguishes a
canonical, non-negative integer index from every other property key.

- An in-bounds canonical index names one non-writable, enumerable,
  non-configurable own property containing exactly one UTF-16 code unit.
- An out-of-bounds canonical index is absent and continues through ordinary
  prototype lookup.
- Every non-index String key and every Symbol key is an ordinary property key;
  it is not an unsupported String-index form. It continues through ordinary
  lookup against `%String.prototype%` and normally produces `undefined` when
  no property exists.
- Key conversion and any abrupt completion it produces occur exactly once.
  Lowering must not coerce a key merely to decide which IR variant to emit.

In particular, `String("hello world")["foo"]` is an ordinary property lookup,
not a malformed indexed access.

## Compiler invariant

Computed String keys cross lowering through one private closed classification:

1. `CanonicalIndex` is emitted only when lowering proves a canonical
   non-negative integer key without observable conversion.
2. `OrdinaryPropertyKey` carries every other key through `PropertyKeyIr`.
   Dynamic values use `StringExpr`, whose backend boundary owns the single
   `ToPropertyKey` and runtime String-exotic classification.

There is no rejection arm. Adding a new computed-key shape must either prove a
canonical index or preserve it as an ordinary property key. The backend must
not receive a bare numeric payload under `OrdinaryPropertyKey`; it receives the
source expression and performs the normal property-key operation.

Static proof is an optimization only. Failure to prove that a key is an index
must select `OrdinaryPropertyKey`, never `Unsupported`. The runtime remains the
authority for dynamic canonical-index recognition and prototype fallback.

## Constructed String length

[StringCreate](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-stringcreate)
installs an own `length` data property with the number of UTF-16 code units and
with writable, enumerable and configurable all false. This property belongs to
the ordinary header; the String-exotic descriptor path supplies virtual indexed
characters. Inheriting `%String.prototype%.length` is insufficient for a wrapper.

String construction and `ToObject` share
`emit_initialize_string_object_length`, which accepts a rooted `StringValue`
and fresh ordinary header. Construction invokes it after String conversion and
the observable `newTarget.prototype` lookup, and before publishing the wrapper.
The initializer appends the own data property without invoking prototype hooks.
The prototype, wrapped primitive, conversion errors and error Realm continue
through their existing owners.

The native String constructor controls retain the UTF-16 surrogate and property
descriptor assertions, and cover empty/undefined values, ordinary boxing,
custom prototypes, foreign constructors, mutation refusal and own-key order.
The native Array callback control borrows map/filter/every/some onto constructed
Strings in strict and sloppy code, asserting the captured UTF-16 length, exact
callback receiver and indices, inherited out-of-bounds exclusion and quantifier
short circuit. These controls require native verification; their presence does
not establish a passing runtime or pinned Test262 result.

## Evidence boundary

The structural regression pins the closed two-variant classifier, exhaustive
conversion to `PropertyKeyIr`, and the absence of the former
`"string index must be number"` rejection. It passes on this checkpoint. Both
exact former failures report `2/2` execution variants and the adjacent
`built-ins/String/15.5.5.5.2` family reports `28/28` under Wasm-AOT at the
harness-declared `aa55200d1310384c5cf69ea95b2a2ecba457007b` pin. This is
focused runtime evidence; it does not claim the complete `built-ins/String`
tree or a current aggregate publication.
