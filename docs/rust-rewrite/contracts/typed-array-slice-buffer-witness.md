# TypedArray `slice` buffer witness

Status: focused-verified on 2026-08-24 with durable guards and runtime controls
for the T17 Wasm-AOT `%TypedArray%.prototype.slice` invariant lane.

## Specification boundary

This contract is pinned to the ECMA-262 2026 edition. The normative source is
[`%TypedArray%.prototype.slice`](https://tc39.es/ecma262/2026/multipage/indexed-collections.html#sec-%typedarray%.prototype.slice),
including its uses of `ValidateTypedArray`, `TypedArraySpeciesCreate` and the
conditional post-species buffer-witness bounds check.

The algorithm has two distinct source-buffer observations:

1. `ValidateTypedArray(obj, seq-cst)` creates an entry witness and
   `TypedArrayLength` captures `sourceArrayLength` before either argument is
   coerced.
2. Only when the initially calculated count is greater than zero, the algorithm
   calls `MakeTypedArrayWithBufferWitnessRecord(obj, seq-cst)` after argument
   coercion and species construction, rejects when `IsTypedArrayOutOfBounds`
   is true, caps its end index by the current `TypedArrayLength`, and recomputes
   the number of elements that can still be copied.

The start value is converted and clamped against the entry length first. An
absent or `undefined` end defaults to that same entry length; otherwise end is
converted and clamped against it second. Growth during either coercion cannot
extend the originally selected range. Shrinkage or detachment is observed by
the second witness only when the original count is positive.

`TypedArraySpeciesCreate(obj, « count »)` occurs between the observations. The
original count is the constructor argument and the minimum accepted result
length; a custom species may return a longer TypedArray. Species work therefore
fixes the actual target before the late source observation, even if that
observation reduces the copied count or makes it zero. Every target element
outside the copied prefix retains the state produced by species construction;
it is zero for a freshly allocated intrinsic target but need not be zero for an
arbitrary custom species result. A fixed source that becomes out of bounds, or
any source that is detached, is rejected by the conditional second validation.

When the source and result have the same element type, the algorithm copies
ascending bytes so the bit-level encoding is preserved. A species constructor
may return a view that overlaps the source, and this remains an ascending copy,
not `memmove`. When the element types differ but their Number/BigInt content
types agree, the algorithm instead performs live source indexed reads and
target indexed writes so normal element conversion occurs.

## Inventoried pre-migration owner and census

The sole compiler owner is
`FunctionBuilder::compile_typed_array_prototype_slice_builtin` in
`crates/lila-aot-wasm/src/builtins/array.rs`. The sole dispatcher owner is the
`StandardBuiltinId::TypedArrayPrototypeSlice` arm in
`crates/lila-aot-wasm/src/builtins/standard.rs`.

At the theory baseline, the bounded compiler body contains exactly:

- two direct calls to `emit_validate_typed_array_current_byte_length`, one at
  entry and one inside the positive-count branch after species construction;
- two local `Instruction::I64DivU` element-length derivations, one after each
  raw validation;
- one direct load each of the source viewed buffer, byte offset, stored byte
  length, bytes per element and element kind;
- one call to `emit_validate_typed_array_from_constructed_target` after
  species construction;
- one target element-kind load and, in the same-type copy path, one target
  viewed-buffer load and one target byte-offset load; and
- two backing-data loads in the same-type copy path, for the source and target,
  after the late source validation.

The whole AOT source tree currently has eight consumer calls to
`emit_validate_typed_array_current_byte_length`, excluding its definition.
This lane removes only the two direct source calls owned by `slice`.
`emit_validate_typed_array_from_constructed_target` has five callers across
`TypedArray.of`, `TypedArray.from`, `slice`, `map` and `filter`; the helper
itself owns another raw validation. That constructed-target lifecycle is
deliberately separate from this source-witness migration and remains unchanged.

The distinction between source and target slot reads is part of the census.
The source's four view slots are the migration target. The target viewed buffer
and byte offset used to form a same-type copy destination are not source
reconstruction and must not be deleted or accidentally folded into the source
view.

## One immutable source view, two fresh witnesses

No new witness policy is required. After the receiver-brand guard, the
compiler must load source private state exactly once with
`emit_load_typed_array_private_state` and construct exactly one immutable
`TypedArrayViewLocals`:

```rust
let source_view = TypedArrayViewLocals::new(
    receiver_payload_local,
    source_buffer_payload_local,
    source_byte_offset_local,
    source_stored_byte_length_local,
    source_bytes_per_element_local,
);
```

The stored byte-length local must be named and treated as the view's immutable
fixed extent. It must never be reused as a current length. A fixed view that
temporarily becomes out of bounds and later regrows must retain that extent.

Both live observations consume the same immutable view through the existing
closed projection:

```rust
TypedArrayWitnessUse::ValidatedMethodEntry { length_local }
```

The compiler must contain exactly two static `emit_typed_array_witness` calls:

1. the unconditional entry call publishes `source_length_local`; and
2. the call inside the original-positive-count branch publishes
   `current_source_length_local` after species construction and target
   validation.

Each call makes a fresh backing-store byte-length and data observation,
rejects detachment and fixed or tracking out-of-bounds state through the
executing builtin's Realm, floors a tracking view's available bytes to whole
elements and publishes the element length derived from that same observation.
Neither call may mutate a local held by `source_view`.

The separate source element-kind load remains outside the view. Element kind
is immutable object metadata used for intrinsic default-constructor selection
and source/target content and element-type decisions; it is not a live
backing-store observation.

## Preserved observable order

The implementation must preserve the following order:

1. reject a receiver without the TypedArray internal brand;
2. load the four source view slots once and construct the immutable source
   view;
3. load the separate source element kind;
4. perform the first validating witness and capture
   `source_length_local` before any argument coercion;
5. convert `start` with the existing `ToNumber` then
   `ToIntegerOrInfinity` decomposition and clamp it against the captured
   source length;
6. initialize end to the captured source length, and only when end is present
   and not `undefined`, convert it and clamp it against that same captured
   length;
7. calculate `count_local = max(end_index_local - start_index_local, 0)`;
8. resolve the intrinsic default constructor from the source element kind;
9. observe `receiver.constructor` and `constructor[@@species]` in that order,
   preserving abrupt completion and default-constructor fallback;
10. construct the result with the original `count_local`;
11. validate the constructed target and reject a Number/BigInt content-type
    mismatch;
12. if the original `count_local` is zero, skip the second source witness and
    every copy-data/address operation, then publish the constructed target;
13. otherwise perform the second validating source witness into
    `current_source_length_local`;
14. cap `end_index_local` to that current length and compute a distinct
    `copied_element_count_local = max(end_index_local - start_index_local, 0)`;
15. choose the same-element-type byte path or different-element-type indexed
    path; the byte path derives a separate `copied_byte_count_local`, both paths
    copy only the late count in their own unit, and then publish the constructed
    target.

The first witness may not move after start coercion. The second witness may not
move before species construction, constructed-target validation or content-type
validation, and it must remain conditional. Source/target data-pointer loads,
address construction and copy loops may not move before the second witness.

Internal metadata reads and result construction are not source copy setup. The
entry witness necessarily reads backing data to validate detachment, but that
does not authorize retaining a data pointer for the later copy.

## Zero count and the count domains

`count_local`, `copied_element_count_local` and `copied_byte_count_local` have
different roles and must remain separate carriers.

- `count_local` is derived only from indices normalized against the entry
  length. It determines the species-constructor argument and the minimum
  accepted result length; a custom species may return a longer TypedArray. No
  operation after target construction may write it.
- `copied_element_count_local` is created only inside the positive-count
  branch after the second witness. It is bounded by the source length observed
  there, remains in element units and is the sole input to the byte-count
  derivation. The different-element-type path may retain its equivalent
  `start_index_local..end_index_local` traversal; it must not consume the byte
  count.
- `copied_byte_count_local` is derived from `copied_element_count_local` and
  the immutable source bytes-per-element only in the same-type path. It controls
  only the ascending byte loop and cannot size the target or the indexed path.

If `count_local` is zero, species construction and constructed-target
validation still occur, but the source is not revalidated afterward. A species
constructor may detach the source in this branch without causing a second
source-buffer TypeError. This conditional absence of observation is normative,
not an optimization.

If the original count is positive but the late current length is at or before
the captured start index, the already-created target retains its species-
provided length, which is at least the original count, while the late copied
count becomes zero. That case remains inside the original positive-count
branch, but no byte or element may be copied.

## Separate target validation and copy paths

This lane must preserve the existing constructed-target boundary:

```rust
self.emit_validate_typed_array_from_constructed_target(
    target_payload_local,
    target_tag_local,
    count_payload_local,
    function,
)?;
```

That helper continues to own the result brand, bounds and minimum-capacity
checks used by `TypedArraySpeciesCreate`. Its internal raw validator is not
silently claimed as migrated. The source's second witness occurs after this
helper and after the content-type comparison, exactly as the specification
places the late source validation after species creation.

The two copy paths also remain distinct:

- same element type: load fresh source and target data pointers after the late
  witness, form addresses from their respective byte offsets, and copy the
  recomputed range as ascending bytes; and
- different element type with matching content type: walk the capped source
  index range, perform a live indexed read, derive the zero-based target index
  and perform the existing typed element write and conversion.

The source `TypedArrayViewLocals` does not own the target's private state. The
target buffer, byte offset and element kind may continue to be read in their
existing result-validation/copy roles.

## Durable bounded source guard

The
`crates/lila-aot-wasm/tests/typed_array_slice_witness_structure.rs` must bound
only the body from `compile_typed_array_prototype_slice_builtin` to
`compile_typed_array_prototype_map_builtin`, plus the single dispatcher arm and
the narrow entry-realm public prototype installation. It must not snapshot the
complete array, standard-builtin emitter or bootstrap path.

Within that bounded body, the guard must require:

- one completed receiver-brand rejection before any source private-state use;
- exactly one `emit_load_typed_array_private_state`, one
  `TypedArrayViewLocals::new`, two `emit_typed_array_witness` calls and two
  `TypedArrayWitnessUse::ValidatedMethodEntry` projections;
- the first witness writing only `source_length_local` and the second writing
  only `current_source_length_local`, with no later writer to either local;
- no writer to source buffer, byte offset, stored byte length or bytes per
  element after the one private-state load;
- no direct source-slot reconstruction, no
  `emit_validate_typed_array_current_byte_length`, no
  `emit_typed_array_current_byte_length` and no `Instruction::I64DivU`;
- zero direct uses of `HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET` and
  `HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET` in the body, while allowing and
  pinning exactly one target use each of
  `HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET` and
  `HEAP_TYPED_ARRAY_BYTE_OFFSET` and exactly two element-kind loads, one source
  and one target;
- start coercion/clamping before optional end coercion/clamping, with both
  clamped against `source_length_local`, followed by the original count;
- constructor then `@@species` observation, target construction, exactly one
  constructed-target validation and content-type comparison before the
  positive-count branch's second witness;
- the second witness, current-length end cap, recomputed copied count, both
  data loads, address construction and both copy paths inside the structured
  positive-count branch, with result publication after that branch;
- no write to `count_local` after it has supplied the constructor argument and
  minimum target length, no write to `copied_element_count_local` after its
  late element-count calculation, one exact element-to-byte multiplication
  into `copied_byte_count_local`, and no use of `current_source_length_local`
  to size or validate the target;
- the different-element-type path initializing its source index from
  `start_index_local`, stopping at the capped `end_index_local`, deriving the
  zero-based target index as source index minus start, and never consuming
  `copied_byte_count_local`;
- ascending byte-copy direction, exact source and target address wiring, and
  the existing different-type live read/write order; and
- exactly one dispatcher mapping from
  `StandardBuiltinId::TypedArrayPrototypeSlice` to this compiler, plus exactly
  one entry-realm bootstrap installation of that builtin as
  `TypedArray.prototype.slice`.

The guard should use normalized exact sentinels for the two witness calls,
argument guards, target validation and copy wiring. For branch containment it
should count structured Wasm `If`/`Else`/`End` boundaries rather than relying
on a nearby textual substring. Local reservation/release ownership must remain
balanced, with the new byte-count local released in reverse reservation order.
The same target must pin the focused CLI owner and the fixture markers for
coercion/species order, overlap, late shrinkage, positive-count detachment,
zero-count detachment and odd-byte flooring so those controls cannot disappear
while the fixture still returns `true`.

## Focused runtime controls

The durable fixture remains
`crates/lila-cli/tests/fixtures/wasm_typedarray_prototype_slice.js`, owned by
`typed_array::run_wasm_backend_slices_typedarrays_with_species_and_resizable_buffer_semantics`
in `crates/lila-cli/tests/cli/typed_array.rs`. No parallel fixture family is
needed.

The existing fixture already covers descriptor shape, start/end/species order,
result identity, a different-element-type target, overlapping same-type views,
floating-point bit preservation, tracking shrink during start coercion, fixed
out-of-bounds state after coercion, fixed shrink during species construction,
BigInt conversion, target content mismatch, undersized targets, entry
detachment and invalid receivers.

The implementation lane adds these controls to that fixture:

1. a positive original count whose species constructor detaches the source,
   proving target construction occurs and the conditional second witness then
   throws;
2. a zero original count whose species constructor detaches the source,
   proving the same observable species work occurs but the second witness and
   all copying are skipped; and
3. a Uint16 length-tracking source whose resizable backing store is reduced to
   an odd byte length during species construction, proving the late witness
   floors to whole elements while a freshly allocated target retains its
   original length and zero-filled suffix.

Counters make species invocation and argument-coercion order explicit.
Assertions that install an own throwing or shadowing `length` property must
inspect integer-indexed values directly rather than call a helper that reads
that public property.

## Exact pinned Test262 checkpoint

At Test262 pin `e9d582d6b8b13afc5ba9a676664741592b5c7f69`, the focused cohort is
exactly these seven complete suite-relative leaves:

- `built-ins/TypedArray/prototype/slice/return-abrupt-from-this-out-of-bounds.js`;
- `built-ins/TypedArray/prototype/slice/coerced-start-end-grow.js`;
- `built-ins/TypedArray/prototype/slice/coerced-start-end-shrink.js`;
- `built-ins/TypedArray/prototype/slice/speciesctor-resize.js`;
- `built-ins/TypedArray/prototype/slice/resize-count-bytes-to-zero.js`;
- `built-ins/TypedArray/prototype/slice/detached-buffer-custom-ctor-same-targettype.js`; and
- `built-ins/TypedArray/prototype/slice/detached-buffer-zero-count-custom-ctor-same-targettype.js`.

None has a strictness-limiting flag at this pin. Each exact leaf must therefore
discover two sloppy/strict executions, for an expected `14/14` Wasm-AOT
checkpoint. Verification must inspect the discovery total and every parser,
early-error, lowering, runtime, Wasm-backend, host-harness, unsupported,
not-implemented, crash and bug bucket. A zero exit status without the expected
discovery count is not evidence.

Each leaf must run independently by its full path with `--jobs 1`,
`--threads 1`, the Wasm execution backend and the repository timeout. The seven
exact leaves are a focused witness cohort, not a substitute for the pinned
full-tree matrix.

## Verification ladder after implementation

All compilation and test commands must run serially under the shared maximum
of eight logical CPUs and 22 GB RAM; no expensive command may overlap another
repository build or test. The implementation must be complete and source-
reviewed before this ladder begins.

1. Perform non-compiling review first: inspect the bounded diff, confirm the
   two witness/order proofs, run `git diff --check`, and ensure only the
   intended contract, source, guard, fixture, CLI owner and T17 status paths
   changed.
2. Run capped `cargo fmt --all -- --check`, then one capped workspace compile
   checkpoint with `cargo xc`.
3. Run only the exact
   `typed_array_slice_witness_structure` integration target with one test
   thread and require its complete expected test count.
4. Run only
   `typed_array::run_wasm_backend_slices_typedarrays_with_species_and_resizable_buffer_semantics`
   with `--exact --test-threads=1`, and require `backend_used: WasmAot` plus the
   fixture's final `boolean(true)` result.
5. Run the seven pinned Test262 leaves above one at a time with `--jobs 1` and
   `--threads 1`; require exactly `14/14` aggregate executions and zero in
   every non-success bucket.
6. Reuse the existing build artifacts for the affected AOT crate's broad test
   checkpoint, serially, then run the shared batch's broader workspace gate if
   this lane is part of a multi-lane batch. Report any unrelated pre-existing
   red tests separately rather than calling the broad gate green.
7. Finish with capped formatting, `git diff --check`, source-census review and
   explicit inspection for unexpected generated snapshots or fixtures.

No verification result may be recorded in this contract until it has actually
run on the implementation being described.

The 2026-08-24 checkpoint passed the complete structure target `6/6`, the
exact CLI runtime fixture `1/1`, and all seven exact Test262 leaves above for
`14/14` sloppy/strict Wasm-AOT variants. Every parser, early-error, lowering,
runtime, Wasm-backend, host-harness, unsupported, not-implemented, crash and
bug bucket was zero. The first structure run exposed a stale coordinate in the
guard: its late-witness sentinel began at the preceding zero-count guard. The
corrected assertion now proves the branch start and witness-call position
separately; the compiler body already kept revalidation and both copy paths
inside the positive-count arm.

## Explicit nonclaims

This migration does not change start/end conversion, species lookup, target
construction, constructed-target validation, content-type checks, ascending
overlap behavior, different-type conversion or result publication.

A nullish species fallback still selects its default TypedArray constructor
from entry globals rather than the executing builtin's Realm. This adjacent
species-construction debt is not evidence for or against the migrated source
buffer observations.

It does not migrate the raw validator inside
`emit_validate_typed_array_from_constructed_target`, nor any of that helper's
five callers as a family. It does not migrate `%TypedArray%.prototype.with`,
`set`, a TypedArray constructor, integer-indexed exotic operations or another
remaining raw current-length consumer. `subarray` and Atomics have separate
buffer-witness contracts.

It does not alter SharedArrayBuffer synchronization, resizable-buffer storage,
Test262 materializers or harness adaptations. It does not retire a rewrite,
refresh the full real-suite matrix, change README/published conformance counts
or establish a complete TypedArray-tree Test262 pass.

The bootstrap installation guard is entry-realm only. Created realms install
their own `TypedArray.prototype.slice` through the host realm builder; that
separate installation path is outside this bounded source guard.

The existing fixture does not prove created-Realm TypeError prototype identity
for either source witness. The shared witness structurally owns the executing
function-Realm route, but runtime Realm identity remains a nonclaim unless a
direct control is added and run. This invariant lane does not by itself
complete `slice`, TypedArray or T17.
