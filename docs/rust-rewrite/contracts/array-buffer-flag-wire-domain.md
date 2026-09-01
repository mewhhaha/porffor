# ArrayBuffer flag wire domain

Status: implemented and verified through Batch AN.

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

## Ownership census

There are exactly 25 product projections:

- two in `emit_ordinary_prevent_extensions_i32`;
- two in `emit_array_buffer_slice_copy`;
- two in `emit_initialize_typed_array_from_array_buffer`;
- one in `emit_detach_array_buffer`;
- one in `emit_throw_if_array_buffer_immutable`;
- two in `ArrayBufferSliceKind::default_result_flags`;
- one in `emit_typed_array_stable_sort`; and
- fourteen in `compile_standard_builtin`.

The heap layout test owns four additional projections for its complete valid-bit
mask. Across production Rust this is 31 `ArrayBufferFlag` mentions: one
declaration, one implementation and 29 named projections. No
`ARRAY_BUFFER_FLAG_*` raw constant remains.

The recursive `array_buffer_flag_wire_domain_structure` target pins the exact
four-row authority, capability absence, borrowed exhaustive mapping, recursive
mention counts, four-file and eight-owner projection census, zero raw
constants, and the exact pre-migration projection sequence. Removing whitespace
from the 25 legacy projection rows retains the frozen fingerprint
`(1773, 0xa28c775059daa571)`. Their raw and whitespace-normalized SHA-256 hashes
are respectively
`5d75104504642d0ff4e5e41dbfc02e253bae885b7b40b3e17fd92a708ed7d144`
and
`8b058a539e4e37d8ea53cb6a8054931e0810602a17cd49c83b8a1597aa3f4437`.

## Verification

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

This source-equivalent migration does not change the private record layout,
emitted instructions, flag combinations, backing-store lifecycle, detachment
or resize ordering, SharedArrayBuffer synchronization, Test262 rewrites or
published conformance status.
