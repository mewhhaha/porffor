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
buffer observation in the bounded accessor compiler.

## Nonclaims and deferred gates

This seam does not establish complete TypedArray, constructor/subclass,
integer-indexed exotic, iterator, SharedArrayBuffer, DataView, Atomics, agent,
or shared-race correctness. It does not retire Test262 source rewrites, change
published conformance counts, or close T17.

Static freeze gates are `rustfmt --check` for touched Rust files, `node --check`
for the fixture, focused source searches, `git diff --check`, and manual local
lifetime review. Cargo, fixture execution, focused pinned Test262 accessor
trees, and the broad batch ladder remain deferred until the frozen patch is
independently reviewed and the shared low-RAM baseline releases Cargo.
