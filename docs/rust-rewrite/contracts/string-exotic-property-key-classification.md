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

## Evidence boundary

The structural regression pins the closed two-variant classifier, exhaustive
conversion to `PropertyKeyIr`, and the absence of the former
`"string index must be number"` rejection. It passes on this checkpoint. Both
exact former failures report `2/2` execution variants and the adjacent
`built-ins/String/15.5.5.5.2` family reports `28/28` under Wasm-AOT at the
harness-declared `aa55200d1310384c5cf69ea95b2a2ecba457007b` pin. This is
focused runtime evidence; it does not claim the complete `built-ins/String`
tree or a current aggregate publication.
