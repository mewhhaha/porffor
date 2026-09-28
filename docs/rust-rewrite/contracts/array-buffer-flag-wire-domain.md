# ArrayBuffer flag wire domain

Status: implemented; owner inventory refreshed on 2026-09-24. Current failing executions and replay evidence are tracked in the
[failure backlog](../../../tasks/README.md).

## Boundary

`ArrayBufferFlag::{Resizable, Shared, Immutable, Detached}` is the sole Rust
authority for the four stable bits stored in the private ArrayBuffer flags
word. The type intentionally has no clone, copy, debug, equality, hashing,
ordering or default capability. Its borrowed, exhaustive `word(&self)`
projection owns the existing `1`, `2`, `4` and `8` wire values. Product code
cannot spell a flag as an unrelated integer or add a flag without defining its
wire value.

The stored flags field remains `u64`. Resizable, shared, immutable and detached
are composable properties rather than mutually exclusive buffer states, so an
exclusive state enum would reject valid combinations. This closure types only
selection of an individual bit; bitwise composition and runtime decoding retain
their existing representation and order.

## Immutable-buffer reads

Reading the `Immutable` bit has exactly one owner: the IsImmutableBuffer
predicate `emit_array_buffer_is_immutable_i32`, which pushes an `i32` for an
ArrayBuffer payload. `emit_typed_array_buffer_is_immutable_i32` applies it to a
TypedArray's viewed buffer. Every write-side algorithm asks one of these two
instead of re-deriving the flag word: ValidateTypedArray with the `Write`
`TypedArrayAccessMode` (TypedArray mutators, Atomics read-modify-write and
store, TypedArrayCreateFromConstructor for species, `from` and `of`), the
integer-indexed `[[Set]]`, `[[DefineOwnProperty]]` and `[[GetOwnProperty]]`
paths, SetViewValue, ValidateUint8Array with `write`, DetachArrayBuffer,
ArrayBufferCopyAndDetach, `resize`, the grouped `slice` species check and the
`ArrayBuffer.prototype.immutable` getter. Algorithms that throw use
`emit_throw_if_array_buffer_immutable`, whose closed `ImmutableBufferWriter`
names each TypeError message.

## Ownership census

There are exactly 22 product projections:

- two in `emit_ordinary_prevent_extensions_i32`;
- two in `emit_array_buffer_slice_copy`;
- two in `emit_initialize_typed_array_from_array_buffer`;
- one in `emit_detach_array_buffer`;
- one in `emit_array_buffer_is_immutable_i32`, the sole reader of the
  `Immutable` bit;
- two in `ArrayBufferSliceKind::default_result_flags`;
- one in `emit_typed_array_stable_sort`; and
- eleven in `compile_standard_builtin`.

The remaining `Immutable` projections create immutable buffers
(`sliceToImmutable`, `transferToImmutable` and the `ToImmutable` slice kind).
The Uint8Array codec and `%TypedArray%.prototype.fill` no longer project any
flag: the codec's `Write` access calls the shared immutable throw with
`ImmutableBufferWriter::Uint8ArrayCodec`, and fill's entry witness validates
`TypedArrayAccessMode::Write`.

The heap layout test owns four additional projections for its complete valid-bit
mask. Across the backend source this is 28 `ArrayBufferFlag` mentions: one
declaration, one implementation and 26 named projections. No
`ARRAY_BUFFER_FLAG_*` raw constant remains.

The recursive `array_buffer_flag_wire_domain_structure` target pins the exact
four-row authority, capability absence, borrowed exhaustive mapping, recursive
mention counts, the per-owner projection census, the predicate as the sole
`Immutable` reader, zero raw constants, the codec's write-only immutable
check, and the pre-migration projection sequence in the three original product
files. Removing whitespace from the 25 legacy projection rows retains the
frozen fingerprint `(1773, 0xa28c775059daa571)`. Their raw and
whitespace-normalized SHA-256 hashes are respectively
`5d75104504642d0ff4e5e41dbfc02e253bae885b7b40b3e17fd92a708ed7d144`
and
`8b058a539e4e37d8ea53cb6a8054931e0810602a17cd49c83b8a1597aa3f4437`.
The current sequence is that legacy sequence without legacy rows 17, 21 and
23 (0-based): the per-site `Immutable` reads in `resize`, the grouped `slice`
species check and the transfer family, which now ask the predicate. The
predicate's own projection occupies the old immutable throw's position.

## Historical verification

At the Batch AN checkpoint, `cargo xc` is green, the new structure target
passes `4/4`, and the three exact CLI controls pass `3/3`.
The focused CLI targets are
`binary_data::run_wasm_backend_succeeds_for_supported_arraybuffer_prototype_core_fixture`,
`binary_data::run_wasm_backend_succeeds_for_supported_arraybuffer_resizable_getters_fixture`,
and
`binary_data::run_wasm_backend_succeeds_for_supported_arraybuffer_transfer_metadata_fixture`.
The pinned leaves are:

- `built-ins/ArrayBuffer/prototype/resizable/return-resizable.js`;
- `built-ins/ArrayBuffer/prototype/detached/detached-buffer.js`;
- `built-ins/ArrayBuffer/prototype/resize/this-is-immutable-arraybuffer-object.js`;
  and
- `built-ins/SharedArrayBuffer/prototype/growable/return-growable.js`.

Those four leaves pass all `8/8` Wasm-AOT variants with every failure bucket
at zero. No semantic golden was required or run for this source-equivalent
wire-authority migration.

That source-equivalent migration did not change the private record layout,
emitted instructions, flag combinations, backing-store lifecycle, detachment
or resize ordering, SharedArrayBuffer synchronization, Test262 rewrites or
published conformance status.
