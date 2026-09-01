# TypedArray length-mode wire domain

Status: implemented and focused-verified at the Batch AO checkpoint.

## Boundary

`TypedArrayLengthMode::{Fixed, Tracking}` is the sole Rust authority for the
word stored in a TypedArray's private length-tracking slot. The two states are
exclusive rather than composable. `Fixed` owns wire value `0`; `Tracking` owns
wire value `1`. Runtime Wasm locals and the private slot remain `u64`, while an
exhaustive borrowed `word(&self)` projection owns every constant emitted into
that runtime representation.

The type intentionally has no clone, copy, debug, equality, hashing, ordering
or default capability. A new mode cannot silently inherit a wire value, and the
six current decisions cannot spell an unrelated integer. The grouped
constructor still publishes one runtime local to the private slot, so emitted
instruction and storage order are unchanged.

## Ownership census

There are exactly three writers:

- `emit_initialize_typed_array_from_array_buffer` first selects `Fixed`, then
  selects `Tracking` only for a resizable or growable buffer with no explicit
  length; and
- the grouped eleven-constructor standard-builtin arm initializes every other
  construction path as `Fixed` before its existing source classification.

There are exactly three readers. `emit_typed_array_witness`,
`emit_ordinary_prevent_extensions_i32`, and the
`TypedArray.prototype.subarray` arm compare the runtime word with the named
`Fixed` word before preserving their existing fixed/tracking branch behavior.
This yields six product projections with the exact `Fixed 5 / Tracking 1`
split and `objects 1 / binary_data 3 / standard 2` distribution.

The private-slot offset retains seven source mentions: declaration and layout,
one source-inventory guard string, the three reader loads, and the grouped
constructor's sole publication. This migration does not type the separate
DataView length-tracking slot and does not add runtime validation for corrupted
heap words.

## Durable evidence

The four-test recursive
`typed_array_length_mode_wire_domain_structure` target pins the exact two-row
authority, absence of incidental capabilities, eight total type mentions, six
named projections, all three readers and writers, the one slot publication and
the unchanged seven-offset census. It also pins the frozen projection sequence
`Fixed, Fixed, Fixed, Tracking, Fixed, Fixed`. Mapping those names back to the
pre-migration constants retains raw fingerprint
`(358, 0xc988361080d6b5cc)` and whitespace-normalized fingerprint
`(288, 0x7e691158ebdeda94)`. Their raw and normalized SHA-256 hashes are
`9b36ddbd8cb543cd8e4780c84c458695e5fd846fe9593cd5da206274d13ebfba`
and
`04193b36264ff26e0780c45446bc517e85aef7b306fcf1d2fe0ede71994d6d4f`.

Focused behavioral controls are the exact CLI fixtures
`typed_array::run_wasm_backend_succeeds_for_typedarray_accessors_fixture` and
`typed_array::run_wasm_backend_subarray_uses_non_throwing_typed_array_buffer_witness`.
The pinned Test262 leaves are:

- `built-ins/TypedArray/prototype/length/resizable-array-buffer-fixed.js`;
- `built-ins/TypedArray/prototype/length/resizable-array-buffer-auto.js`; and
- `built-ins/TypedArray/resizable-buffer-length-tracking-2.js`.

At the Batch AO checkpoint, `cargo xc` is green, the structure target passes
`4/4`, and the two exact CLI controls pass `2/2`. The first two pinned leaves
pass `4/4` Wasm-AOT variants with every failure bucket at zero. The final
leaf's two variants stop at the existing declared `resizable-arraybuffer`
unsupported feature gate, so this migration makes no unsupported-retirement
claim. No semantic golden was required or run. This source-equivalent
migration does not change constructor classification, resizable/growable-buffer
behavior, element-length flooring, subarray species arguments,
prevent-extensions semantics, Test262 rewrites or published status.
