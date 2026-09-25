# Atomics TypedArray buffer witness

Status: focused-verified on 2026-08-24.

## Specification boundary

The edition-pinned ECMA-262
[`ValidateIntegerTypedArray`](https://tc39.es/ecma262/2026/multipage/structured-data.html#sec-validateintegertypedarray)
operation first validates a TypedArray and returns its TypedArray-with-buffer
witness record. A detached or out-of-bounds view therefore throws a TypeError
before the index is coerced. The subsequent
[`ValidateAtomicAccess`](https://tc39.es/ecma262/2026/multipage/structured-data.html#sec-validateatomicaccess)
operation reads the element length from that record, performs `ToIndex`, and
throws a RangeError when the normalized index is not below that captured
length.

Wasm-AOT has four owners for that entry observation:

- `FunctionBuilder::emit_atomics_notify`;
- `FunctionBuilder::emit_atomics_wait_async`;
- `FunctionBuilder::emit_atomics_wait`; and
- `FunctionBuilder::emit_atomics_integer_operation`, shared by `add`, `and`,
  `compareExchange`, `exchange`, `load`, `or`, `store`, `sub` and `xor`.

Before this migration every owner loaded the TypedArray's buffer, byte offset,
stored byte length and bytes per element independently, called
`emit_typed_array_current_byte_length`, multiplied the normalized index by
bytes per element, and compared those byte quantities. That duplicated the
buffer-witness law and admitted an index whose element began before, but ended
after, an odd current byte length.

It also treated an already out-of-bounds fixed view as a zero-length valid
view. The old path therefore coerced the index and then threw a RangeError.
This migration intentionally corrects that observable behavior: initial fixed
out-of-bounds and detached states throw TypeError before `ToIndex`, as required
by `ValidateIntegerTypedArray`.

## Closed witness projection

Each owner now loads one immutable `TypedArrayViewLocals` value with
`emit_load_typed_array_private_state` and consumes exactly one witness through:

```rust
TypedArrayWitnessUse::ValidatedMethodEntry {
    length_local: element_length_local,
}
```

No Atomics-specific compatibility variant exists. `ArrayLikeLengthSnapshot`
is deliberately not admissible here because Atomics validation throws for an
initial detached or out-of-bounds view. `IntegerIndexedProperty` is also the
wrong policy because it projects invalid states to an absent property rather
than an abrupt completion.

The witness owns the current backing-length observation, fixed versus
length-tracking out-of-bounds calculation, fixed-view stored extent and
whole-element flooring. The Atomics owner compares `index_local` directly with
the witness-produced `element_length_local`; it may not reconstruct current
byte length or divide/multiply byte quantities for that bound.

## Preserved order and pointer timing

The four owners retain their existing receiver, element-kind and shared-buffer
requirements. Their emitted entry sequence is:

1. load the arguments and reject a non-TypedArray receiver;
2. load one immutable view and retain the existing buffer-brand check;
3. retain the operation's integer-element-kind and, for `wait`/`waitAsync`,
   SharedArrayBuffer requirement;
4. snapshot the backing data pointer and reject detachment with the existing
   Atomics TypeError path;
5. create one validated buffer witness and capture its element length;
6. coerce the index with `ToNumber` and `ToIndex`;
7. compare the normalized index with the captured element length and throw the
   operation-specific RangeError when it is outside that bound; and
8. only then coerce `count`, `value`, replacement value or timeout and perform
   the existing atomic operation.

The explicit data-pointer load is not a parallel length observation; it only
backs the entry detachment check. No owner forms a memory address from that
pre-coercion snapshot any more (see the next section).

## Post-coercion RevalidateAtomicAccess

Index, value, expected and replacement coercions run user code that can detach
a non-shared buffer or shrink a resizable one after `ValidateAtomicAccess`.
The current specification therefore runs
[`RevalidateAtomicAccess`](https://tc39.es/ecma262/multipage/structured-data.html#sec-revalidateatomicaccess)
in `Atomics.load`, `store`, `compareExchange` and `AtomicReadModifyWrite` after
the last coercion and before the buffer access.

`crates/lila-aot-wasm/src/builtins/atomics/access.rs` makes that step the only
way to reach memory. The owner wraps its range-checked index in the
`#[must_use]` `ValidatedAtomicsIndex`; every element accessor (`load`, `store`,
`compareExchange` and the read-modify-write family) borrows an
`AtomicsElementAddress`, whose field is private to `access.rs`. Two emitters
mint one:

- `emit_revalidate_atomic_access`, called once by the integer-operation owner
  after all value coercions. It takes a fresh `ValidatedMethodEntry` read
  witness (ValidateTypedArrayBounds: detached or out of bounds is a TypeError),
  bounds the index against the new element length with the operation's
  RangeError, and only then reads the backing pointer again; and
- `emit_shared_atomics_element_address`, used by `wait` and `waitAsync`
  (DoWait) and by `notify`, each of which has already required a
  SharedArrayBuffer. Such a buffer can be neither detached nor shrunk, and the
  specification performs no revalidation there. `notify` returns `+0` for any
  other buffer after its count coercion, as the specification requires, before
  it forms an address.

The RangeError bound is `accessIndex ≥ TypedArrayLength(taRecord)`, that is
`byteIndexInBuffer + elementSize > [[CachedBufferByteLength]]`. The
specification states `byteIndexInBuffer ≥ [[CachedBufferByteLength]]`; the two
differ only for a length-tracking view shrunk to a partial trailing element,
where the following GetValueFromBuffer/SetValueInBuffer would violate the
specification's own sufficient-bytes assertion. The element-granular bound is
the one consistent with that assertion.

`atomics_revalidate_access_structure` pins the private mint, the typed
accessor signatures, the coercion-then-revalidation order and the non-shared
`notify` result. The CLI fixture
`crates/lila-cli/tests/fixtures/wasm_atomics_revalidate_after_coercion.js`
(`binary_data::run_wasm_backend_revalidates_atomics_access_after_argument_coercion`)
covers detachment during index and value coercion for every integer and BigInt
element kind, the compareExchange coercion order, tracking-view shrink and
partial-element RangeErrors, a fixed view pushed out of bounds, growth during
coercion and non-shared `notify`. Test262
`staging/sm/Atomics/detached-buffers.js` exercises the detachment half.

A valid length-tracking view whose current element length is zero is distinct
from an out-of-bounds view. Its witness succeeds, its side-effecting index is
coerced, and the access is checked against the originally captured zero. Growth
during index coercion therefore cannot make that access valid. Likewise, a
trailing partial element never becomes addressable: the witness floors bytes
to complete elements before publishing the bound.

## Durable structural guard

`crates/lila-aot-wasm/tests/atomics_typed_array_witness_structure.rs` bounds
only the four owner bodies above. For each it requires one private-state load,
one immutable view, one `ValidatedMethodEntry` witness, one retained data
pointer snapshot and one separate element-kind load. It rejects the legacy
current-byte-length helpers, direct backing-length loads and direct private
view-slot offsets.

The guard also fixes witness-before-index ordering, the single index coercion,
the direct `index >= elementLength` bound, absence of a local writer that could
overwrite the witness result, and the focused CLI test/fixture connection.

## Focused runtime witnesses

`crates/lila-cli/tests/fixtures/wasm_atomics_typed_array_buffer_witness.js`,
owned by
`binary_data::run_wasm_backend_validates_atomics_access_through_typed_array_witness`,
covers all four owners. It proves that detached receivers and ordinary-buffer
fixed views already out of bounds throw TypeError without invoking a poisoned
index. It separately proves that valid zero-length views snapshot their length
before an index grows the backing buffer, and that ordinary and shared
length-tracking views floor an odd byte length before checking index 1. Poisoned
later arguments prove the RangeError precedes count, value and timeout
coercion.

At Test262 pin `e9d582d6b8b13afc5ba9a676664741592b5c7f69`, the smallest exact
adjacent cohort is:

- `built-ins/Atomics/notify/null-bufferdata-throws.js` for detached entry and a
  poisoned index;
- `built-ins/Atomics/notify/retrieve-length-before-index-coercion.js`;
- `built-ins/Atomics/wait/retrieve-length-before-index-coercion.js`; and
- `built-ins/Atomics/waitAsync/retrieve-length-before-index-coercion.js` for
  zero-length growable shared views whose index coercion grows the buffer.

The pin has no exact fixed-out-of-bounds-before-index Atomics leaf. The focused
CLI fixture owns that corrected behavior until the pinned suite adds one. No
Test262 result is claimed before the cohort is executed against this patch.

## Verification evidence

The coordinated batch ran:

1. `cargo fmt --all -- --check`;
2. `cargo xc`;
3. `cargo test -p lila-aot-wasm --test atomics_typed_array_witness_structure -- --test-threads=1`;
4. `cargo test -p lila-cli --test cli binary_data::run_wasm_backend_validates_atomics_access_through_typed_array_witness -- --exact --test-threads=1`; and
5. each exact Test262 path above through `--execution-backend wasm-aot` with
   `--jobs 1 --threads 1`.

Formatting, diff hygiene and `cargo xc` are green. The structure target passes
`3/3`, and the focused CLI fixture passes `1/1`. Each of the four exact pinned
Test262 files passes `2/2`, for an aggregate `8/8` Wasm-AOT executions with
every non-success bucket at zero.

## Explicit nonclaims

The original migration did not add post-coercion atomic revalidation; that is
now owned by the section above, which also moved address formation after the
last coercion. Neither change alters sequentially consistent loads/stores,
waiter queues, agent host transport, `waitAsync` settlement, accepted element
kinds, operation-specific return values or the relative coercion order of index
and later arguments.

It does not migrate other raw TypedArray validators, retire a Test262 harness
rewrite, refresh an aggregate baseline, change published conformance counts or
complete Atomics, TypedArray or T17.
