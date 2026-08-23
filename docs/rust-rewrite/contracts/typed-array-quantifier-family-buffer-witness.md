# TypedArray quantifier-family buffer witness

Status: implemented, independently reviewed and focused-verified for the
Wasm-AOT `%TypedArray%.prototype.every` and `some` method-entry seam,
2026-08-23.

## Specification boundary

The living ECMA-262 clauses for
[`every`](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.every)
and
[`some`](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.some),
and the corresponding 2026 clauses for
[`every`](https://tc39.es/ecma262/2026/multipage/indexed-collections.html#sec-%typedarray%.prototype.every)
and
[`some`](https://tc39.es/ecma262/2026/multipage/indexed-collections.html#sec-%typedarray%.prototype.some),
share the following ordered method boundary:

1. let `O` be the `this` value;
2. create `taRecord` with `ValidateTypedArray(O, seq-cst)`;
3. derive `len` with `TypedArrayLength(taRecord)`;
4. throw a TypeError if `callback` is not callable; and
5. iterate from `k = 0` while `k < len`.

Receiver validation and the element-length snapshot therefore precede callback
validation. A detached backing buffer or an out-of-bounds fixed or tracking view
must fail at method entry before callback admissibility can determine the
outcome. An in-bounds fixed view contributes its stored element length. An
in-bounds length-tracking view contributes the whole-element length derived from
the backing-store byte length observed by `taRecord`. A trailing partial element
is not visible.

The captured `len` is the loop bound for the complete call. Callback-driven
growth cannot add visited indices, and shrinkage or detachment does not shorten
the loop. The snapshot does not cache values: each iteration performs the live
integer-indexed `Get` for that `k`. A mutation before a later read is observable,
while an index made invalid by shrinkage or detachment produces the current
integer-indexed result for that later read.

For each visited index, `every` and `some` both call `callback` with `thisArg`
and the ordered argument list `(value, index, O)`, then apply `ToBoolean` to the
callback result. `every` returns `false` at the first false result and otherwise
returns `true`; `some` returns `true` at the first true result and otherwise
returns `false`. An empty view therefore returns `true` from `every` and `false`
from `some` without calling the callback.

## Closed method-entry projection

`TypedArrayQuantifierKind` remains the sole two-way compiler domain:

- `Every` owns `%TypedArray%.prototype.every`; and
- `Some` owns `%TypedArray%.prototype.some`.

The `StandardBuiltinId` dispatcher must map each method to its matching kind
explicitly. Per-kind occurrence counts are insufficient because they allow the
two public method surfaces to exchange kinds while preserving the totals. The
migration adds no boolean policy and no parallel method dispatcher.

The shared TypedArray quantifier compiler must complete its receiver-brand guard
before reading private view state. It then loads exactly one immutable private
view through `emit_load_typed_array_private_state`, constructs exactly one
`TypedArrayViewLocals` value and consumes exactly one live witness with:

```rust
TypedArrayWitnessUse::ValidatedMethodEntry {
    length_local: len_local,
}
```

That single witness owns the complete `taRecord` and `TypedArrayLength`
observation for both quantifier kinds:

1. it observes the backing data pointer and backing byte length once;
2. it distinguishes detachment and fixed or tracking out-of-bounds state
   without mutating the view's stored fixed extent;
3. it routes method-entry buffer failures through the executing builtin's
   current-function-Realm TypeError path;
4. it floors a tracking view's available bytes to whole elements; and
5. it publishes `len_local` from that same cached observation.

The compiler consumes that element length directly. It may not call
`emit_validate_typed_array_current_byte_length`, call
`emit_typed_array_current_byte_length`, reconstruct any of the four private view
slots independently, observe the backing store through a parallel helper,
derive `len_local` by dividing a byte-length local or overwrite the
witness-produced `len_local` later. `ValidatedMethodEntry` already expresses the
required policy; this lane does not add a new `TypedArrayWitnessUse` variant.

## Preserved quantifier algorithm

Only the method-entry observation changes. The shared quantifier algorithm
retains the following sequence and identities:

1. validate the callback after consuming the method-entry witness;
2. preserve the exact optional `thisArg` value, defaulting it to `undefined`;
3. compare the ascending index against the one captured `len_local`;
4. perform one live read through
   `emit_typed_array_or_object_index_read_from_locals`;
5. propagate any indexed-read abrupt completion before preparing or calling the
   callback;
6. construct the numeric index and the exact `(value, index, receiver)` argument
   vector;
7. invoke the callback through the Proxy-aware call boundary with the preserved
   `thisArg`;
8. propagate callback abrupt completion before applying truthiness;
9. apply `ToBoolean`, then exhaustively project the selected quantifier's
   short-circuit result; and
10. increment the index only after a non-short-circuiting callback result.

The polarity is part of the closed contract:

| Kind | Short-circuit condition | Short-circuit result | Terminal result |
| --- | --- | --- | --- |
| `Every` | callback result is false | `false` | `true` |
| `Some` | callback result is true | `true` | `false` |

No second buffer witness or legacy validator belongs inside the loop. Such a
re-observation would incorrectly allow callbacks to change the captured loop
bound. The existing live indexed-read helper remains responsible for the value
observed at each iteration; this contract does not replace or broaden that
helper.

## Durable structural regression

The bounded regression is
`crates/lila-aot-wasm/tests/typed_array_quantifier_family_witness_structure.rs`.
It must isolate `compile_typed_array_prototype_quantifier_builtin` through the
start of `compile_array_prototype_every_builtin`, so unrelated Array consumers
and other TypedArray migrations cannot satisfy its counts.

The regression must pin all of the following:

- `TypedArrayQuantifierKind` has exactly `Every` and `Some`;
- exact normalized dispatcher sentinels bind
  `TypedArrayPrototypeEvery` to `Every` and `TypedArrayPrototypeSome` to `Some`;
- the completed receiver-brand guard precedes exactly one private-state load,
  one `TypedArrayViewLocals` construction, one live witness and one
  `ValidatedMethodEntry` projection;
- witness consumption precedes the sole callback callable check;
- the body contains no raw validator, current-byte-length helper, direct load of
  the viewed-buffer, byte-offset, stored-byte-length, bytes-per-element or
  length-tracking slots, direct backing-store observation, parallel byte-length
  division, entry-global TypeError construction or direct assignment to
  `len_local`;
- the live read and its abrupt propagation precede index construction and the
  exact callback argument vector;
- the Proxy-aware callback consumer receives the preserved `thisArg` and
  `(element, numeric index, original receiver)` locals in that order;
- callback abrupt propagation precedes truthiness and successful projection;
  and
- exact exhaustive match sentinels fix all three polarity decisions: `Every`
  inverts truthiness before the short-circuit branch, short-circuit values are
  `Every = false` and `Some = true`, and terminal values are `Every = true` and
  `Some = false`.

The two receiver-brand error literals may be used as separately counted anchors
for the two exhaustive arms, but their prose is not an ECMAScript-observable
message contract. Normalizing whitespace is appropriate for the ordered wiring
sentinels; broad whole-file text snapshots are not. These checks are
source-structure mutation guards, not runtime evidence.

## Focused evidence

The existing exact CLI fixture is
`crates/lila-cli/tests/fixtures/wasm_typedarray_every_some.js`, registered by
`run_wasm_backend_succeeds_for_typedarray_every_some_fixture`. It already covers
both quantifiers, short-circuit and terminal results, empty views, numeric and
BigInt element kinds, exact callback arguments and receiver identity,
`thisArg`, callable Proxies, public private-slot spoofs, generic Array method
separation, mutation visible to later reads, snapshot lengths across growth and
shrinkage, later `undefined` values after detachment or out-of-bounds shrinkage,
invalid receivers, invalid callbacks and abrupt callbacks.

The exact current-pin Test262 checkpoint is two source files and their four
ordinary sloppy/strict variants:

- `built-ins/TypedArray/prototype/every/return-abrupt-from-this-out-of-bounds.js`;
  and
- `built-ins/TypedArray/prototype/some/detached-buffer.js`.

The first fixes out-of-bounds method-entry failure on the `Every` projection.
The second fixes detached-buffer method-entry failure on the `Some` projection
and uses an abrupt callback to ensure the entry failure wins. Together with the
CLI fixture and the closed dispatcher/algorithm guard, they form the focused
checkpoint for this seam.

Under the shared eight-core cap, `cargo fmt --all -- --check` and `cargo xc` are
green. The structural guard passes `3/3`, and the exact
`wasm_typedarray_every_some.js` CLI fixture passes `1/1`. The exact current-pin
`every/return-abrupt-from-this-out-of-bounds.js` and
`some/detached-buffer.js` Test262 leaves each pass `2/2`, for `4/4` Wasm-AOT
executions with every failure bucket at zero under `--jobs 1 --threads 1`.

## Nonclaims

This seam does not change generic `Array.prototype.every` or `some`, the shared
indexed-read helper, integer-indexed exotic semantics, Proxy call semantics,
callback truthiness, result allocation, SharedArrayBuffer synchronization or
the other remaining raw TypedArray validators. In particular, it does not
migrate `toLocaleString`, `copyWithin`, `with`, `set`, `slice`, `map`, `filter`
or constructor/species target validation.

The shared witness structurally routes entry failures through the executing
builtin's Realm, but the existing quantifier CLI fixture does not assert
created-Realm error-prototype identity. Runtime Realm identity remains an
explicit nonclaim unless a later phase adds and verifies a cross-Realm fixture.

This lane removes no Test262 materializer or harness adaptation, changes no
published conformance count, proves no full TypedArray subtree or baseline, and
does not make the witness a universal integer-indexed exotic protocol. T17
remains in progress.
