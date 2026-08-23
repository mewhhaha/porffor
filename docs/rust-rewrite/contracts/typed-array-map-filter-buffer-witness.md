# TypedArray `map` / `filter` buffer witness

Status: implemented, independently reviewed and focused-verified for the T17
Wasm-AOT invariant lane on 2026-08-23.

## Specification boundary

The edition-pinned ECMA-262 algorithms for
[`%TypedArray%.prototype.map`](https://tc39.es/ecma262/2026/multipage/indexed-collections.html#sec-%typedarray%.prototype.map)
and
[`%TypedArray%.prototype.filter`](https://tc39.es/ecma262/2026/multipage/indexed-collections.html#sec-%typedarray%.prototype.filter)
begin with the same non-generic entry sequence:

1. require a genuine TypedArray receiver;
2. create one TypedArray-with-buffer-witness record through
   `ValidateTypedArray(O, seq-cst)`;
3. derive one `TypedArrayLength` snapshot from that record; and
4. only then reject a non-callable callback.

Both algorithms keep that captured length for their complete callback walk.
They do not cache element values: each index performs a live integer-indexed
`Get`. Growth during a callback must not extend the walk, while shrinkage or
detachment must not shorten it; an index that is no longer available produces
the current integer-indexed result.

Their allocation order is intentionally different. `map` performs
`TypedArraySpeciesCreate(O, « len »)` before its first callback. `filter`
collects selected source values during every callback and performs
`TypedArraySpeciesCreate(O, « captured »)` only after that walk. This lane
shares method-entry validation, not the two complete algorithms.

## Existing closed witness

No new policy variant is needed. Both compilers must consume the existing
closed projection:

```rust
TypedArrayWitnessUse::ValidatedMethodEntry {
    length_local,
}
```

After the receiver-brand guard, each compiler must:

1. load one immutable view with `emit_load_typed_array_private_state`;
2. construct one `TypedArrayViewLocals` value; and
3. consume exactly one `ValidatedMethodEntry` witness whose output is that
   compiler's sole callback-loop length.

The witness owns the one backing-store data/byte-length observation, detached
and out-of-bounds rejection, length-tracking whole-element floor and final
`length_local` write. The method compilers may retain the separate element-kind
load needed by species construction, but they may not reconstruct viewed
buffer, byte offset, stored byte length or bytes per element independently.
They may not call `emit_validate_typed_array_current_byte_length`, call
`emit_typed_array_current_byte_length`, divide a byte length locally, overwrite
the witness-produced length or insert a second method-entry witness.

This is an invariant migration rather than a new abstraction. A plausible
future mistake that bypasses the cached witness must fail the bounded source
guard instead of surviving until an expensive conformance run.

## Preserved algorithm order

For `map`, the source order after the witness remains:

1. callback presence and callability validation;
2. source constructor and `@@species` observation;
3. result TypedArray construction and content-type validation;
4. the ascending `0 .. length_local` loop;
5. one live source read, callback call and target write per index; and
6. result publication.

For `filter`, the source order after the witness remains:

1. callback presence and callability validation;
2. the ascending `0 .. length_local` callback walk and selected-value capture;
3. source constructor and `@@species` observation;
4. result TypedArray construction and content-type validation;
5. selected-value writes in original order; and
6. result publication.

The migration must not move callback validation before buffer validation,
move `map` species construction after callbacks, move `filter` species
construction before callbacks, cache source element values at entry, or change
callback `this`/argument wiring.

## Durable structural witness

`crates/lila-aot-wasm/tests/typed_array_map_filter_witness_structure.rs`
should bound the two compiler bodies independently and require:

- exactly one receiver-brand guard, one private-state load, one
  `TypedArrayViewLocals` construction and one `ValidatedMethodEntry` witness in
  each body;
- the witness before callback validation, species work and callback loops;
- no raw validating/current-byte-length helper, direct view-slot
  reconstruction, local byte-length division, second witness or later
  `length_local` writer in either body;
- the distinct map-species-before-loop and filter-loop-before-species orders;
- live indexed source reads and the existing callback arguments
  `(value, index, receiver)` inside each captured-length loop;
- one dispatcher owner for each of `TypedArrayPrototypeMap` and
  `TypedArrayPrototypeFilter`, mapped to the matching compiler; and
- result publication before temporary release, with each body's release order
  remaining the exact reverse of its reservation order.

The guard should extract only these two bodies and the bounded dispatcher
arms. It must not snapshot the complete large array emitter or duplicate the
runtime algorithms.

## Focused runtime controls

The durable CLI controls remain:

- `crates/lila-cli/tests/fixtures/wasm_typedarray_map.js`, owned by
  `typed_array::run_wasm_backend_succeeds_for_typedarray_map_fixture`; and
- `crates/lila-cli/tests/fixtures/wasm_typedarray_filter.js`, owned by
  `typed_array::run_wasm_backend_succeeds_for_typedarray_filter_fixture`.

They already pin callback wiring, result non-aliasing, species order and target
validation, live values after mid-callback shrink, abrupt callbacks and the
map/filter-specific result rules. This lane may add only missing direct
method-entry detached/out-of-bounds controls to those fixtures; it should not
create a parallel fixture family.

The exact current-pin Test262 checkpoint is:

- `built-ins/TypedArray/prototype/map/detached-buffer.js`;
- `built-ins/TypedArray/prototype/map/return-abrupt-from-this-out-of-bounds.js`;
- `built-ins/TypedArray/prototype/map/resizable-buffer-grow-mid-iteration.js`;
- `built-ins/TypedArray/prototype/map/resizable-buffer-shrink-mid-iteration.js`;
- `built-ins/TypedArray/prototype/filter/detached-buffer.js`;
- `built-ins/TypedArray/prototype/filter/return-abrupt-from-this-out-of-bounds.js`;
- `built-ins/TypedArray/prototype/filter/resizable-buffer-grow-mid-iteration.js`;
- `built-ins/TypedArray/prototype/filter/resizable-buffer-shrink-mid-iteration.js`.

Each physical leaf must be invoked by its complete suite-relative path with
the Wasm-AOT backend, `--jobs 1`, `--threads 1` and the repository timeout.
Verification must inspect discovery totals and every failure bucket rather than
trusting process status alone.

## Verification evidence

The implementation and bounded source guard were independently reviewed. Under
the shared eight-core, 22 GB cap, `cargo fmt --all -- --check`, `cargo xc` and
`git diff --check` are green. The structural witness passes `3/3`, and the
exact `map` and `filter` CLI fixtures each pass `1/1`. Those fixtures now also
prove that detached and out-of-bounds method-entry failures occur before a
callback can run.

Each of the eight complete Test262 leaves above discovered and passed two
Wasm-AOT variants, for exactly `16/16`. Every parser, early-error, lowering,
runtime, Wasm-backend, host-harness, unsupported, not-implemented, crash and bug
bucket was zero under `--jobs 1 --threads 1`.

## Explicit nonclaims

This lane does not change map or filter species semantics, result allocation,
content-type validation, conversion/write behavior, callback abrupt
propagation, integer-indexed `Get`, resizable-buffer mutation semantics or
created-Realm bootstrap. It does not migrate `copyWithin`, `with`, `set`,
`slice`, constructor/species-target validation or other remaining raw
TypedArray consumers.

The existing CLI fixtures do not prove created-Realm prototype identity for a
detached or out-of-bounds method-entry TypeError. The shared witness
structurally routes that error through the executing builtin's Realm, but that
identity remains a runtime nonclaim unless this lane adds a direct control.

This invariant-only migration is not a broad Test262 refresh, does not retire a
materializer or harness adaptation, changes no published count, establishes no
new pass by itself and does not complete `map`, `filter`, TypedArray or T17.
