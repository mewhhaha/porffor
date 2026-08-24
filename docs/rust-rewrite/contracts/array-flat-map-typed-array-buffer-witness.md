# Generic Array `flatMap` TypedArray buffer witness

Status: normative theory, implementation, focused fixture, structural guard and
central focused verification complete for the generic
`Array.prototype.flatMap` TypedArray observation seam, 2026-08-24.

## Specification and compiler boundary

`Array.prototype.flatMap` obtains one `LengthOfArrayLike` snapshot and then
walks the integer indices below that captured bound. Each iteration performs a
fresh `HasProperty` before `Get` and the mapper call. For a TypedArray receiver,
these are two distinct non-throwing backing-buffer policies:

- the initial length snapshot projects a detached or out-of-bounds view to
  zero, retains the stored extent of an available fixed view and floors the
  available bytes of a length-tracking view to whole elements; and
- each later integer-indexed property observation reports detached,
  out-of-bounds or currently unavailable indices as absent.

The captured length does not grow with the buffer. The per-index property
observation remains live, so mapper-triggered shrinkage or detachment can skip
later indices. Element values are not captured with either observation: after
a present result, the existing shared indexed-read owner performs the live
`Get` before the mapper is called.

The Wasm-AOT compiler has one TypedArray specialization inside
`compile_array_prototype_flat_map_builtin`. This migration changes only that
specialization's two backing-store observations. It does not establish complete
observable `LengthOfArrayLike` behavior for own or inherited `length`
properties on TypedArray receivers.

## Closed projections and exact owner census

`compile_array_prototype_flat_map_builtin` has one standard-builtin dispatcher
edge, `StandardBuiltinId::ArrayPrototypeFlatMap`. The owner loads the receiver's
private state once and constructs one `TypedArrayViewLocals` from:

- the receiver payload;
- its viewed buffer;
- byte offset;
- immutable stored byte extent; and
- bytes per element.

The same view is consumed through exactly two closed witness projections:

```rust
TypedArrayWitnessUse::ArrayLikeLengthSnapshot {
    length_local: current_len_local,
}
```

and, inside the source loop:

```rust
TypedArrayWitnessUse::IntegerIndexedProperty {
    index_local: src_index_local,
    result_local: has_property_local,
}
```

Neither projection throws for a detached or out-of-bounds view. The first owns
the loop-bound snapshot; the second owns the current integer-indexed presence
result. They replace both raw current-byte-length calls formerly in this one
compiler. Consequently `crates/lila-aot-wasm/src/builtins/array.rs` has no
remaining call to `emit_typed_array_current_byte_length`. The legacy emitter
itself remains because three separate non-throwing consumers in `objects.rs`
still use it: static TypedArray `length` property compilation, shared indexed
read and shared element write.

## Observable ordering

For the TypedArray specialization, the bounded compiler retains this order:

1. the existing nullish guard and mapper validation;
2. the existing `ToObject` boundary;
3. TypedArray classification and one private-state load;
4. the `ArrayLikeLengthSnapshot` witness;
5. zero-length target allocation or the existing selected species target;
6. comparison of the source index with the captured length;
7. a fresh `IntegerIndexedProperty` witness;
8. the existing live indexed read; and
9. the mapper call.

The source loop executes steps 6 through 9 again after each mapper call. A
mapper-triggered growth therefore cannot extend the captured loop, while a
mapper-triggered shrink is visible to the next property observation.

The compiler already validates mapper callability before the general
`LengthOfArrayLike` path. This lane preserves that ordering; it does not claim
to correct the separate ordinary-object case where a throwing `length` getter
and a non-callable mapper distinguish that order.

## Durable structural regression

`crates/lila-aot-wasm/tests/array_flat_map_typed_array_witness_structure.rs`
bounds the flatMap compiler through the following generic `map` compiler. It
requires one private-state load, one immutable view, two witness calls and
exactly one projection of each required kind. Exact normalized snippets pin the
view fields and both output locals, preventing byte-offset, stored-length,
bytes-per-element, source-index or result-local transposition.

The guard rejects both legacy current-length emitters, direct backing-buffer
loads, direct TypedArray private-slot reconstruction, local unsigned division,
throwing entry/accessor projections and any raw current-length call anywhere in
the complete Array builtin source. A separate ordering assertion retains the
`ToObject`, snapshot, target allocation, source loop, live presence check,
indexed read and mapper sequence. The standard dispatcher must keep exactly one
flatMap edge.

The fixture guard inventories six exact generic TypedArray calls, five exact
resize transitions, one transfer and six distinct failure bits. It couples the
odd-byte, growth, shrink, fixed out-of-bounds, fixed regrowth and detached setup,
transition, call and assertion snippets in strict source order through one
unique final `failures === 0` publication. These are source-structure mutation
guards, not runtime pass evidence.

## Focused runtime and Test262 evidence

`crates/lila-cli/tests/fixtures/wasm_array_flat_map_resizable_typedarray.js` is
registered by
`array::run_wasm_backend_succeeds_for_supported_array_flat_map_resizable_typedarray_fixture`.
Its failure-bit matrix covers:

- a length-tracking `Uint16Array` whose five available bytes expose only two
  whole elements;
- mapper-triggered growth that does not extend the captured source length;
- mapper-triggered shrink to three bytes, making later `Uint16` indices absent;
- a fixed offset view that contributes zero while out of bounds and restores
  its stored one-element extent after regrowth; and
- a detached view that contributes zero and never invokes the mapper.

The exact direct pinned Test262 cohort is one unrewritten vendored source leaf,
materialized with the normal harness preludes into two ordinary sloppy/strict
executions:

- `built-ins/Array/prototype/flatMap/array-like-objects-typedarrays.js`.

That leaf covers fixed `Int32Array` borrowing and the non-Array species path. It
does not exercise resizable-buffer, odd-byte, detachment or live per-index
behavior. The pinned flatMap directory has no direct resizable-buffer leaf, so
the focused CLI fixture is the only runtime case in this checkpoint for those
buffer transitions. At the 2026-08-24 central checkpoint, `cargo check` and
`cargo xc` were green, the structure target passed `3/3`, the exact CLI fixture
passed `1/1`, and the unrewritten vendored leaf passed both ordinary executions
`2/2` with every failure bucket at zero. The exact focused commands were:

```text
cargo test -p lila-aot-wasm --test array_flat_map_typed_array_witness_structure
cargo test -p lila-cli --test cli array::run_wasm_backend_succeeds_for_supported_array_flat_map_resizable_typedarray_fixture -- --exact
./target/debug/lila --jobs 1 test262 run built-ins/Array/prototype/flatMap/array-like-objects-typedarrays.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
```

## Explicit nonclaims

This lane does not change nullish rejection, mapper validation, ordinary-object
length lookup/coercion, own or inherited TypedArray `length` shadowing, Array or
Arguments paths, Proxy behavior, species selection, target construction,
flattening depth, target writes, callback Realm behavior or abrupt completion.
It does not migrate or make atomic the downstream shared indexed `Get`; that
owner still performs its own live raw backing-store observation after a present
result.

It does not migrate the three remaining raw consumers in `objects.rs`, change
integer-indexed exotic descriptor/result policy, prove SharedArrayBuffer
synchronization, cover every TypedArray constructor or BigInt element kind,
retire a Test262 materializer, change a baseline snapshot, README status or
published count, or claim complete flatMap, Array, TypedArray, T16, T17 or
Test262 conformance.
