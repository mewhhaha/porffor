# Array `concat` TypedArray buffer witness

Status: focused-verified on 2026-08-24.

## Specification boundary

After `Array.prototype.concat` decides that an input is spreadable and obtains
its array-like length, it iterates the numeric keys and performs `HasProperty`
before reading and copying each element. A TypedArray with
`Symbol.isConcatSpreadable = true` therefore uses its integer-indexed exotic
property policy; it is not a validated `%TypedArray%.prototype` method entry.

`FunctionBuilder::emit_concat_typed_array_has_index_i32` is the Wasm-AOT
predicate for that branch. It receives an already normalized non-negative loop
index and publishes two results:

- `typed_array_like_local` distinguishes a genuine TypedArray from the
  ordinary-object fallback; and
- `result_local` reports whether that integer-indexed property is present.

The same predicate is also consumed by the TypedArray receiver branch of
`Array.prototype.slice`. Both Array methods require non-throwing HasProperty
semantics: a detached or out-of-bounds TypedArray index is absent rather than a
TypeError, and an invalid integer index does not fall through to the prototype
chain.

## Previous raw observation

The predicate previously loaded a private TypedArray view, called
`emit_typed_array_current_byte_length`, multiplied the candidate index by bytes
per element and compared that byte offset with the reconstructed byte length.
That path duplicated the shared buffer-witness law. For a length-tracking view
with an odd current byte length, it treated the start of a trailing partial
element as present even though the complete element did not fit.

Four unrelated temporary locals and a bytes-per-element zero branch remained
from older property lookup scaffolding. Genuine TypedArray construction makes
zero bytes per element unrepresentable, so preserving that corrupt-state
branch would hide the invariant rather than protect an external boundary.

## Closed non-throwing projection

The predicate now initializes both outputs to false, performs the existing
TypedArray brand check, and only in the branded branch sets
`typed_array_like_local` to true. It then loads one immutable
`TypedArrayViewLocals` value and consumes exactly one witness through:

```rust
TypedArrayWitnessUse::IntegerIndexedProperty {
    index_local,
    result_local,
}
```

That closed projection owns the complete observation. It treats detached,
fixed out-of-bounds, length-tracking out-of-bounds and index-at-or-above-current
element-length states as absent; preserves a fixed view's stored extent across
shrink and regrow; and floors a tracking view's available bytes to complete
elements before comparing the index.

The predicate may not select `ValidatedMethodEntry`, which would turn Array
HasProperty into a throwing method-entry check. It may not load backing data or
length independently, call either legacy current-byte-length emitter, read
private view slots directly, or reconstruct the element bound with local byte
multiplication or division.

## Preserved concat result policy

The witness does not own concat's spreadability decision, `LengthOfArrayLike`,
loop bound, element `Get`, target property creation or final target length. A
TypedArray can have an own ordinary `length` property larger than its current
integer-indexed element length. Concat still iterates that captured ordinary
length; the witness answers each numeric HasProperty observation, and every
absent TypedArray index becomes a hole in the result.

For a non-TypedArray receiver, `typed_array_like_local` remains false and the
callers retain their existing ordinary-object HasProperty fallback. For a
TypedArray it remains true even when the buffer is detached or the view is out
of bounds, preventing those absent integer indices from being reinterpreted as
ordinary or inherited properties.

## Durable structural guard

`crates/lila-aot-wasm/tests/concat_typed_array_witness_structure.rs` bounds the
predicate at the following `compile_array_prototype_concat_builtin` definition.
It requires exactly four immutable-view locals, one private-state load, one
view construction, one `IntegerIndexedProperty` witness, one absent-result
initialization and the two false/true TypedArray classification writes. It pins
their emitted order and reverse-order temporary release.

Within that body the guard rejects direct view offsets, backing data or length
loads, both legacy current-byte-length emitters, byte multiplication/division,
throwing error paths and the deleted unrelated locals. A companion census
requires exactly one predicate definition plus the established concat and
Array-slice consumers. The focused CLI guard fixes the fixture/test connection
and its detached, fixed, tracking, regrow and partial-element controls.

## Focused runtime witness

`crates/lila-cli/tests/fixtures/wasm_array_concat_typed_array_buffer_witness.js`,
owned by
`array::run_wasm_backend_checks_concat_typedarray_indices_through_buffer_witness`,
sets an own `length` and `Symbol.isConcatSpreadable` on each TypedArray so the
concat loop cannot hide absent-property behavior behind the view accessor's
zero length. It checks:

- detached indices become holes without throwing;
- a fixed view has properties while in bounds, becomes entirely absent after
  shrink, and restores its original index extent after regrow;
- a length-tracking view exposes only complete elements after odd-byte shrink
  and growth; and
- a length-tracking view whose byte offset exceeds current backing length is
  entirely absent.

At Test262 pin `e9d582d6b8b13afc5ba9a676664741592b5c7f69`, the smallest direct
concat control is
`built-ins/Array/prototype/concat/Array.prototype.concat_small-typed-array.js`.
It makes TypedArrays spreadable, installs an own `length` of 4000 on a one-
element view and requires the remaining result indices to stay holes. The pin
has no direct concat detached/resizable-buffer leaf.

The smallest exact resizable-buffer control for the same shared predicate is
`built-ins/Array/prototype/slice/resizable-buffer.js`. It exercises fixed and
length-tracking views through shrink, out-of-bounds offset states and regrow.
It is an adjacent Array-slice consumer, not evidence that concat's broader
species/spreadability algorithm is complete. Each exact path passes `2/2`
sloppy/strict Wasm-AOT executions, for `4/4` total with every non-success
bucket at zero.

## Verification checkpoint

The coordinated batch ran:

1. `cargo fmt --all -- --check`;
2. `cargo xc`;
3. `cargo test -p lila-aot-wasm --test concat_typed_array_witness_structure -- --test-threads=1`;
4. `cargo test -p lila-cli --test cli array::run_wasm_backend_checks_concat_typedarray_indices_through_buffer_witness -- --exact --test-threads=1`; and
5. both exact Test262 paths above through `--execution-backend wasm-aot` with
   `--jobs 1 --threads 1`, inspecting every non-success bucket.

Formatting, `cargo xc` and diff hygiene are green. The bounded structure target
passes `3/3`, the focused CLI fixture passes `1/1`, and the exact Test262 cohort
passes `4/4`.

## Explicit nonclaims

This migration does not change `IsConcatSpreadable`, array-like length lookup,
species construction, ordinary Array/object/arguments HasProperty, element
reads, target writes or `Array.prototype.slice` argument coercion. It does not
migrate the remaining raw current-byte-length owners in Array builtins or
object/property emitters.

It retires no Test262 rewrite, changes no aggregate or published conformance
count and does not complete concat, Array methods, TypedArray or T17.
