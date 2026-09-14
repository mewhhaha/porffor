# TypedArray `fill` conversion and buffer witnesses

Implementation prepared on 2026-09-14. Integration compilation, native
regressions and pinned Test262 replay remain required; no conformance count is
updated by this document.

`%TypedArray%.prototype.fill` has its own `TypedArrayPrototypeFill` builtin
identity, published on the main and created realms' common TypedArray
prototype. Its one-argument function metadata, non-constructability, lowering
shape, object result type, mutation and synchronous-user-code flags, installer
roots and Wasm dispatch use the same catalog identity. Ordinary
`Array.prototype.fill` continues to observe public length and perform indexed
property writes when borrowed by a TypedArray.

The [ECMA-262 algorithm](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.fill)
requires entry validation and an internal length snapshot, followed by value,
start and optional end conversion. Value conversion uses the receiver's
Number or BigInt content type exactly once. Range bounds use the initial
length. A second bounds validation follows every successful conversion, even
for an empty range. A valid tracking view that shrinks limits the final range;
growth does not expand the original range. The method returns its receiver.

`builtins/typed_array_fill.rs` compiles that sequence through the existing
`TypedArrayViewLocals` and `TypedArrayWitnessUse::ValidatedMethodEntry`
authority. Both observations borrow one private view description. Entry also
rejects the existing immutable-buffer flag before value conversion, as required
by the [immutable ArrayBuffer proposal](https://tc39.es/proposal-immutable-arraybuffer/#sec-%typedarray%.prototype.fill). The backing
pointer used by the write loop is loaded after the second observation, so
argument coercion can replace storage without leaving a stale address. The
loop uses the existing numeric/BigInt element encoding. It copies the retained
converted payload into a temporary for each store because integer encoding
consumes that temporary. No callback or value conversion occurs in the loop.

Native coverage is in `crates/lila-engine/tests/aot_typed_array_fill.rs`:
builtin and realm identity, receiver branding, descriptors, single conversion
including zero length, all currently exposed element kinds, BigInt storage and
conversion rejection, public length overrides, abrupt conversion identity,
detachment, immutable-buffer entry, fixed and tracking resize ordering, signed zero, fractional bounds,
infinities and byte offsets. Fixtures using detach/realm host hooks explicitly
select the Test262 host policy.

The pinned source cohort at `aa55200d1310384c5cf69ea95b2a2ecba457007b`
contains 51 files and 102 strict/sloppy executions under
`built-ins/TypedArray/prototype/fill/`. The completed-baseline evidence records
seven terminal failures from fetched main with source and transcript hashes.
Replay all 102 executions, including previously passing cases, after the
focused native and builtin registry/publication/witness-owner checks. These
are cohort sizes, not passing counts.

The implementation inherits the shared backing-store and element-kind
capabilities. Float16Array participates through the shared typed-array constructor catalog and direct f64-to-binary16 conversion.
Immutable-buffer enforcement in other TypedArray writes and SharedArrayBuffer
concurrency require separate verification; this patch does not establish
either capability or change the shared witness's memory-ordering policy.
