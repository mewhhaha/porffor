# Array callback receiver-kind projection

Status: implemented and focused-verified for the Wasm-AOT Array and TypedArray
`reduce`, `reduceRight` and `forEach` compiler family, 2026-08-26.

## Closed domain

`ArrayCallbackReceiverKind` has exactly two inhabitants:

- `ArrayLike`, for the generic Array prototype entries; and
- `TypedArray`, for the strict `%TypedArray%.prototype` entries.

The capability-free `ArrayCallbackReceiverKind` implements no clone, copy,
debug, default, comparison, ordering or hashing capability. Receiver policy may
not be duplicated into independently transposable authorities or reduced to
equality, inequality, a Boolean or an `is_*` method. Each entry compiler owns
one kind and borrows that same authority through every direct and cross-product
projection. Adding a third entry family therefore requires reviewing every
receiver-sensitive decision before the compiler builds.

This compile-time domain is distinct from the generic methods' runtime
TypedArray recognition. An `ArrayLike` entry may be borrowed by a TypedArray;
its non-throwing length snapshot and live integer-indexed property observations
remain part of generic Array semantics.

## Capability-free reduction direction

The private, capability-free `ArrayReduceDirection` domain has exactly the
`LeftToRight` and `RightToLeft` variants. It is produced once for each reducer
entry and owned by the shared compiler. It implements no clone, copy, debug,
default, comparison, ordering or hashing capability. The compiler borrows the
same authority through method and diagnostic projection, initial cursor
selection, both loop-entry checks and all three advances. Diagnostics,
initialization and traversal therefore cannot be selected from independently
copied directions.

## Producer and consumer census

Six fixed family entries have exactly six receiver-kind producers:

- the fixed Array `reduce`, `reduceRight` and `forEach` entries select
  `ArrayLike`; and
- the fixed TypedArray `reduce`, `reduceRight` and `forEach` entries select
  `TypedArray`.

The dispatcher cannot import or construct `ArrayReduceDirection` or
`ArrayCallbackReceiverKind`; it calls six fixed semantic entries. The two fixed
forEach entries and four reducer entries alone select receiver kind before
entering their private shared compilers.

Exactly two consumers own a receiver kind: the shared reducer and shared
`forEach` compilers. The reducer borrows its owner through two direction helper
calls and six local matches. The `forEach` compiler borrows its owner through
five local matches. No helper accepts the kind by value.

The shared compilers contain exactly thirteen direct exhaustive receiver
projections. The reducer owns eight:

1. method name, paired exhaustively with reduction direction;
2. callback diagnostic, paired exhaustively with reduction direction;
3. generic length observation or strict validated-method entry;
4. first-pass property-key construction;
5. first-pass generic `HasProperty` or strict always-present policy;
6. empty-without-initial-value diagnostic;
7. main-pass property-key construction; and
8. main-pass generic `HasProperty` or strict always-present policy.

The shared `forEach` compiler owns five:

1. method name;
2. generic length observation or strict validated-method entry;
3. callback diagnostic;
4. property-key construction; and
5. generic `HasProperty` or strict always-present policy.

Before this closure, the two entry compilers each collapsed the kind into a
`typed_array_only` Boolean. Two equality decisions therefore controlled eleven
later branches, and a future receiver kind would silently inherit generic Array
defaults. The closed type now has zero equality, inequality, Boolean or `is_*`
receiver projections.

## Preserved buffer observations

This projection cleanup changes no TypedArray witness authority:

- the reducer strict entry retains one `ValidatedMethodEntry` witness;
- its generic property helper retains one `IntegerIndexedProperty` witness and
  is emitted at both property-presence sites;
- the `forEach` strict entry retains one `ValidatedMethodEntry` witness; and
- its generic property path retains one `IntegerIndexedProperty` witness.

The generic `forEach` `ArrayLikeLengthSnapshot` observation and runtime
`emit_is_typed_array_i32` check are intentionally preserved. They recognize a
TypedArray borrowed through the generic Array method; they do not project the
compile-time receiver kind.

## Durable regression

`crates/lila-aot-wasm/tests/array_callback_receiver_kind_structure.rs` pins:

- the exact capability-free two-variant declaration;
- the two owning compiler parameters, two borrowed helper parameters and all
  thirteen borrowed exhaustive receiver projections, with no clone, wildcard
  or Boolean escape hatch;
- the exact six fixed producers and their receiver selections;
- the existing four reducer direction selections; and
- the validated-entry and live integer-indexed witness census.

These are bounded source-structure mutation guards. They supplement rather
than replace behavioral execution.

`crates/lila-aot-wasm/tests/array_reduce_direction_structure.rs` separately
pins the exact two-case direction domain, three borrowed surface projections,
one owned compiler parameter, the borrowed direct match and helper boundaries,
all four fixed producers and the preserved nine-decision census. It rejects
clone, copy, equality, Boolean and catch-all escape paths.

## Focused evidence

The existing `wasm_array_reduce_core.js` fixture distinguishes forward and
reverse traversal for both ordinary Arrays and strict TypedArrays. The existing
`wasm_typedarray_bigint_reduce_default_accumulator.js` fixture covers the
strict BigInt TypedArray default-accumulator path. The existing
`wasm_array_foreach_resizable_typedarray.js` fixture covers generic `forEach`
borrowing across fixed and length-tracking view resize transitions.

The bounded structure target passes `4/4`. The Array/TypedArray reduction,
generic borrowed-TypedArray `forEach` and strict BigInt TypedArray reduction CLI
witnesses each pass `1/1`, for all three focused behavior checks green. The
focused builds emitted pre-existing warnings; no warning was introduced or
repaired by this closure.

This source-only closure reserves the same locals and emits the same selected
instruction sequence as the removed Boolean branches. The shared semantic
golden passes `2/2` in 717.58 seconds with 674 dumps; it adds no fixture for this
closure, and all 671 retained dumps are equal after accounting normalization.
There is no Test262 inventory or published-count change.

For Batch AA, the capability-free receiver declaration is
`c073b0a9449fae68b12f82e43fc0bf7dc52a0a0bc98b1a6eb2bf6d5b0bce3ea1`.
The borrowed direction/receiver projection body is
`20daa9d9e1b1e235a96c6253c5f7c6ad23c13ce269b92bab79a4cd497c00c3ff`;
normalizing only its two receiver-kind parameter borrows reproduces
`ed439784343d2db70ab528aef33047b628d077db2de431935d9a372180446de4`.
The receiver-borrowed reducer is
`ab4ecea3dddb22dcfb0e812be2d05ddc657369fad9dcf7d31bdfd480329ceb90`;
normalizing only its two helper-call borrows and six direct-match borrows
reproduces
`3acf772d37f91e4c1d9ca47302e70a49dfa0bace06f8eddddddc1d9ec61331d8`.
The receiver-borrowed `forEach` compiler is
`ea047de76bef8b4c5fbc8eb440c42329e7693feecf848cef753011cf2a541c26`;
normalizing only its five direct-match borrows reproduces
`52d8982bbef8b3a99ce51a870919b604394773948aa1944d3f21e939a7aa15fb`.
The six producers retain exactly three `ArrayLike` and three `TypedArray`
selections across four fixed reducer entries and two fixed forEach entries.

At the Batch AA checkpoint, `cargo xc` is green, the strengthened structure
target passes `4/4`, and the focused Array reduce, BigInt TypedArray reduce and
resizable-TypedArray `forEach` CLI witnesses pass `3/3`. The four pinned
Array/TypedArray reduce and reduceRight leaves pass all `8/8` Wasm-AOT variants
with every failure bucket at zero. The pinned `forEach` leaves and semantic
goldens were not rerun; no earlier result is claimed as verification of this
capability closure.

The capability-hardened direction domain is
`4695f344d98c2314c721a453012a8ea9ca70d74f6e343ca53978f1cd9a1d485b`.
Normalizing its three borrowed method receivers reproduces
`aefdd7e8fc8726dceacd6ebb75ac503edc6b02d61b9fe408f5ccc6f3228bcee7`.
The raw borrowed loop-entry and advance consumers are
`d9d07c6e862ab07e5270fb3e6170ddeeeeae95234a6332cda7cc5611c901876c`
and `9140aead2a75ac4709d17207bfc7fbcc067293b2b38843967ff775e887d0f812`;
normalizing their parameter borrows reproduces
`fd34580c71f80b5399b8c6a1758d54c040f7e7ddeaa11329e5f761d792ee7078`
and `841ce487ae3fec4ecb8af4adb3d5c403900ffc342cca2bf2544ce2c656c16e3f`.
The fully borrowed reducer body is
`3acf772d37f91e4c1d9ca47302e70a49dfa0bace06f8eddddddc1d9ec61331d8`;
erasing only its direction borrow markers reproduces
`ca2e89b9653e32b049f844629a4c0a3c3df7252b229cb15e38c16bfe10ddb475`.
The standard producer range remains byte-identical at
`8658f3af7364d6923dcfe5fbd53146fe9f66e3817fa35d7af778f12a31ec101d`.

Batch AQ makes the raw `ArrayReduceDirection` and shared reducer private to
`builtins/array.rs`. Four fixed sibling-visible entries are the only reducer
boundary, so standard dispatch cannot import, construct or pass the raw
direction. The frozen 48-line direction domain and 450-line reducer body have
SHA-256 `6ba1c87199ecca660996e4d6a6a31820e521a37f59b65dcb17dd0b046e613b3d`
and `ab4ecea3dddb22dcfb0e812be2d05ddc657369fad9dcf7d31bdfd480329ceb90`.
This source-equivalent tightening claims no new Array or TypedArray behavior.
At the 2026-08-28 Batch AQ checkpoint, `cargo xc` is green, the strengthened
direction and neighboring receiver-kind structure targets each pass `4/4`, and
the exact Array/TypedArray forward/reverse reduce CLI control passes `1/1`.
No Batch AQ Test262 or semantic-golden result is claimed.

Batch BE makes the capability-free `ArrayCallbackReceiverKind` and the raw
shared `forEach` compiler private to `builtins/array.rs`. Two fixed forEach
entries are the only catalog boundary, completing the same fixed-entry policy
already used by the four reducer routes. The exact former four-line domain
retains reconstructed SHA-256
`c073b0a9449fae68b12f82e43fc0bf7dc52a0a0bc98b1a6eb2bf6d5b0bce3ea1`.
Restoring only former visibility on the 442-line raw `forEach` compiler
reproduces SHA-256
`ea047de76bef8b4c5fbc8eb440c42329e7693feecf848cef753011cf2a541c26`.
At the Batch BE checkpoint, `cargo xc` is green, this structure target and the
neighboring direction target pass `4/4` each, and the exact resizable-TypedArray
generic `forEach` CLI control passes `1/1`. Formatting, module-boundary,
task-plan and shortcut gates are green. This source-equivalent tightening has
no new Array behavior and does not close T16.
At the 2026-08-28 Batch Z checkpoint, `cargo xc` is green, the strengthened
structure target passes `3/3`, and the exact Array core, Array semantics and
BigInt TypedArray CLI controls pass `3/3`. The pinned forward/reverse Array and
TypedArray leaves pass all `8/8` Wasm-AOT executions with every failure bucket
at zero. Semantic golden verification remains deferred.

## Nonclaims

This boundary does not change callback evaluation order, `thisArg`, argument
construction, Proxy-aware invocation, reduction direction, initial-accumulator
selection, Array-like length observation, integer-indexed exotic semantics,
TypedArray witness implementation, Realm selection or conformance
materialization. It does not close T16 or T17.
