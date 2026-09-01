# TypedArray accessor buffer witness

Status: normative for the Wasm AOT `%TypedArray%.prototype` accessor seam.

## Semantic boundary

The `byteLength`, `byteOffset`, and `length` accessors first require a genuine
TypedArray receiver. They then make one live observation of the receiver and
its backing buffer. All three results must be projected from that same
observation:

- a detached or out-of-bounds view reports zero for every accessor;
- an in-bounds `byteOffset` reports the view's stored offset;
- an in-bounds fixed-length view retains its stored byte extent across a
  temporary out-of-bounds resize and uses it again after regrowth; and
- an in-bounds length-tracking view exposes only whole elements. Its `length`
  is the available bytes divided by the element size, rounded down, and its
  `byteLength` is that element length multiplied by the element size.

The final rule matters when a resizable backing buffer ends in a partial
element. For example, a length-tracking `Uint16Array` with five bytes available
has length two and byte length four, while its in-bounds byte offset remains
unchanged.

## Closed protocol

`TypedArrayAccessorKind::{ByteLength, ByteOffset, Length}` is the complete
accessor result domain. Each `StandardBuiltinId` accessor delegates explicitly
with one of those variants; no raw boolean or builtin identifier reaches the
observation emitter.

The result selector is capability-free: it cannot be cloned, copied, compared,
ordered, hashed or formatted through a derived implementation. Four producers
move one selection into the witness protocol: the three standard accessor
entries and the generic TypedArray `length` read. The witness consumes that
selection in its sole exhaustive three-arm projection. An additional selector
consumer, equality shortcut or duplicated decision is therefore a Rust source
change guarded by the bounded census rather than an incidental operation the
type already permits.

The accessor compiler loads the immutable private view state into
`TypedArrayViewLocals`, then requests
`TypedArrayWitnessUse::Accessor { kind, result_local }`. The existing sole
witness emitter reads the backing-store byte length once, derives detachment,
out-of-bounds state, and element length from that cached value, and exhaustively
projects the selected accessor result before releasing its temporary locals.

In particular, the accessor compiler does not read backing-store data or byte
length directly and does not inspect the length-tracking slot itself. This
leaves one source of truth for the resize law and prevents an accessor from
overwriting the stored fixed extent with a transient current length.

Receiver-brand validation and Number boxing remain outside the witness. They
are common to all three accessors and neither changes nor performs observable
user code between the witness and its result projection.

## Durable evidence

The existing `wasm_typedarray_accessors.js` fixture covers descriptor calls,
wrong receivers, detachment, fixed-view out-of-bounds/regrowth, and
length-tracking resize behavior. Its partial-element case fixes the distinction
between available backing bytes and observable whole-element byte length.

A structural Rust regression pins the three-kind domain, the three explicit
builtin delegates, the sole accessor-witness call, and the absence of direct
buffer observation in the bounded accessor compiler. It also pins the
capability-free declaration, 12 product mentions, four producers, one owned
compiler handoff and the sole exhaustive projection. The declaration body is
`4ce6e008183a7157593950bb1f3f37b10fc02e23a4838e4e811137001158bd54`, the
projection is
`2432528af60e6e41782b24ff671453987486c0692aee51e971f144217f8b25a1`, the
accessor compiler is
`63e797108010410db586f0a136c1238c0502b4e3f816241604ea4b6a3f02e648`, and
the three-standard-producer block is
`ee60ce214986d62e9811eb190636a02468f9099c60d334b79d3d3341310a1fe7`.

## Nonclaims and verification

This seam does not establish complete TypedArray, constructor/subclass,
integer-indexed exotic, iterator, SharedArrayBuffer, DataView, Atomics, agent,
or shared-race correctness. It does not retire Test262 source rewrites, change
published conformance counts, or close T17.

Batch AF changes only the Rust capabilities of the selector and the bounded
evidence; all producer and consumer instruction bodies remain byte-identical.
At the shared checkpoint, `cargo xc` is green, exact guard
`tests::typed_array_accessors_use_the_closed_buffer_witness` passes `1/1`, and
the exact `typed_array::run_wasm_backend_succeeds_for_typedarray_accessors_fixture`
CLI witness passes `1/1`. The pinned
`built-ins/TypedArray/prototype/byteLength/return-bytelength.js`,
`built-ins/TypedArray/prototype/byteOffset/return-byteoffset.js` and
`built-ins/TypedArray/prototype/length/return-length.js` leaves pass all `6/6`
Wasm-AOT executions with every failure bucket at zero. Batch AF did not rerun
the semantic golden.
