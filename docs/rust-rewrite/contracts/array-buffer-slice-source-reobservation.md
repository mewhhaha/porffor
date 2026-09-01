# ArrayBuffer slice source re-observation

Status: normative for the Wasm AOT ArrayBuffer slice copy seam; the
`ArrayBufferSliceBound` invariant is implemented, independently reviewed, and
focused-verified. The copy-policy ownership invariant is implemented and
focused-verified.

## Semantic boundary

`ArrayBuffer.prototype.slice`, `SharedArrayBuffer.prototype.slice`, and
`ArrayBuffer.prototype.sliceToImmutable` normalize `start` and `end` against
the byte length observed at method entry. Index conversion, species lookup,
and species construction are observable. They may detach or resize an
ordinary source before any bytes are copied, but the two ordinary operations
do not have the same late-length policy.

For `ArrayBuffer.prototype.slice`, the normalized `first` and requested
`newLen` remain derived from the initial byte length, but the source's copy
state does not. After species work and target validation, its bounded copy
must:

1. re-read the ordinary source's flags and throw `TypeError` if it is detached;
2. re-read its current byte length;
3. compute `available = max(currentByteLength - first, 0)` and
   `copyLen = min(newLen, available)` without unsigned underflow;
4. read the current source data pointer only when `copyLen` is nonzero; and
5. copy exactly `copyLen` bytes into the beginning of the already validated
   target.

The bytes in `target[copyLen..newLen]` are not part of that bounded copy. They remain
unchanged: normally zero for a fresh default buffer, but possibly nonzero when
a species constructor supplies a prefilled target.

`sliceToImmutable` instead has an exact-final contract. After both index
coercions it must recheck detachment, reload `currentLen`, and throw
`RangeError` when `currentLen < final`. That check precedes default immutable
target allocation. Only a source that still covers the whole resolved range
may allocate a `newLen` target and copy exactly `newLen` bytes from `first`;
clamping and leaving a zero or prefilled suffix is not valid for this operation.

These are respectively the post-species bounded-copy rule in ECMA-262
`ArrayBuffer.prototype.slice` (25.1.6.7) and the exact-final rule in the
Immutable ArrayBuffers proposal's `ArrayBuffer.prototype.sliceToImmutable`
(25.1.6.8).

## Closed protocol

### Bound role

The normalized slice bound is one closed algorithm role, not an independently
selected argument index and default policy. The backend representation is:

```rust
pub(super) enum ArrayBufferSliceBound {
    Start,
    End,
}
```

Its complete projection is:

- `Start` reads argument 0 and, when that argument is absent or `undefined`,
  writes 0; and
- `End` reads argument 1 and, when that argument is absent or `undefined`,
  writes the byte length observed at method entry.

The bound-consuming emitter accepts only `ArrayBufferSliceBound`. It derives
the argument index internally and emits the missing-or-undefined default from
exhaustive matches with no catch-all arm. It does not accept a caller-supplied
integer index, a `default_to_length` Boolean, or a Boolean projection from the
enum. Consequently the invalid pairings "argument 0 defaults to length" and
"argument 1 defaults to zero", as well as arbitrary argument positions, are
unrepresentable at the caller boundary.

The bound role is also capability-free. One owned `Start` or `End` selection
is borrowed by both exhaustive projections: `argument_index(&self)` owns the
argument position, and `match &bound` owns the missing-or-undefined default.
Clone, copy, debug, default, comparison, ordering and hashing capabilities are
absent. The two decisions therefore cannot be forked into independently
selected copies without a Rust compile error.

There are exactly two source calls, both in the one grouped standard-builtin
body shared by `ArrayBuffer.prototype.slice`,
`SharedArrayBuffer.prototype.slice`, and
`ArrayBuffer.prototype.sliceToImmutable`: `Start` writes `start_local`, then
`End` writes `end_local`. The durable ordering is:

1. read the source byte length at method entry;
2. normalize `Start`;
3. normalize `End`;
4. calculate the requested length; and
5. perform species or target work before the policy-specific late source
   re-observation and copy.

The structural guard pins the exact two-variant domain, both exhaustive
projections, the absence of the raw Boolean/integer pair, the two-call global
inventory, each role-to-destination mapping, the three grouped builtin owners,
and the ordering above. This prevents a variant swap or a new bypass from
passing merely because the enum still exists.

Focused runtime witnesses are the existing exact CLI species-capture fixture,
whose no-argument `slice()` must request an eight-byte result, and the pinned
Test262 leaves
`built-ins/ArrayBuffer/prototype/slice/start-default-if-undefined.js` and
`built-ins/ArrayBuffer/prototype/slice/end-default-if-absent.js`. The two
Test262 leaves independently distinguish the Start default from the End
default; the CLI fixture exercises both roles together through the production
species path.

The implementation and its strengthened structural guard were independently
reviewed. The capped `cargo fmt --all -- --check` and `cargo xc` gates are green,
`array_buffer_slice_bound_structure` passes `3/3`, and the exact
`run_wasm_backend_succeeds_for_supported_arraybuffer_slice_species_capture_fixture`
CLI witness passes `1/1`. Each Test262 leaf above was run as an exact path with
`--jobs 1 --threads 1`; each passes `2/2` sloppy/strict Wasm-AOT executions,
with every reported failure bucket at zero.

Batch AH preserves those instruction bodies while strengthening the same
guard with the exact eight-mention, two-producer and two-borrowed-projection
census. The borrowed implementation is
`97ce3ae5aa8c7de1615d675b4836107a3e77cd7e74915eb68f2348bf3d9cf69b`,
the borrowed helper is
`f5c68fdc3acc539e902205d7991db025c8bfc5015863f6a87bbe91e9d6534766`,
and the unchanged grouped producer span remains
`9473147f5242fa296038457091c82408b1bfb7bbea1b5a90bd7a08ecafde7599`.
The guard additionally normalizes away only the two borrow markers and pins
the exact pre-AH bodies. At the shared Batch AH checkpoint, `cargo xc` exits
zero, `array_buffer_slice_bound_structure` passes `3/3`, and the exact
`binary_data::run_wasm_backend_succeeds_for_supported_arraybuffer_slice_species_capture_fixture`
CLI witness passes `1/1`. The pinned
`built-ins/ArrayBuffer/prototype/slice/start-default-if-undefined.js` and
`built-ins/ArrayBuffer/prototype/slice/end-default-if-absent.js` leaves pass all
`4/4` sloppy/strict Wasm-AOT executions. Batch AH did not run a semantic
golden. Final formatter, diff, module-boundary, task-plan and 240-entry
shortcut-inventory gates are green.

This is an invariant-only migration. It does not replace the feature-local
numeric normalization with the authoritative shared `ToIntegerOrInfinity`
operation, change abrupt-completion routing, alter any slice copy policy, prove
SharedArrayBuffer or immutable-slice behavior independently, refresh a broad
ArrayBuffer/Test262 cohort, or establish a conformance gain.

### Source and copy policy

The builtin-facing domain is `ArrayBufferSliceKind`:

- `Ordinary` uses species, requires ordinary ArrayBuffer source and result
  brands, and rejects an immutable species result;
- `Shared` uses species and requires SharedArrayBuffer source and result
  brands; and
- `ToImmutable` skips species, requires an ordinary detachable source, and
  creates an immutable ordinary result.

It projects exhaustively to
`ArrayBufferSliceCopyPolicy::{DetachableBounded, SharedBounded,
DetachableExactFinal}`. The sole byte-copy writer,
`emit_array_buffer_slice_copy`, consumes that policy together with
`ArrayBufferSliceCopyLocals`. Its exhaustive match owns the late source
observation, policy-specific bound or exact-final check, target-allocation
ordering for the exact-final arm, and private loop. A new operation therefore
cannot reuse a copy policy until Rust forces an explicit choice, callers cannot
pass a cached source pointer to the writer, and the exact-final variant does
not accept an already allocated target.

The payload-bearing copy policy is non-`Clone`, non-`Copy` and has exactly 31
lexical mentions in production Rust. The grouped standard-builtin owner makes
five borrowed pre-handoff decisions, covering source validation, target
validation and species-result rules, before its sole owned handoff to the copy
writer. The writer makes two borrowed writer decisions for late detachment and
target selection, then ends the policy lifecycle in the final consuming
source-selection decision. Reusing the policy after either owned boundary is
therefore a Rust move error; new policy variants remain exhaustive decisions at
all eight observations.

The dedicated structural guard pins the exact attribute-free domain, all three
producer mappings, the 31-mention and nine-per-variant censuses, the sole owned
writer route, the five-borrow/one-handoff and two-borrow/one-consume lifecycles,
and complete normalized fingerprints for the grouped owner and copy writer.
This is ownership and exhaustiveness hardening only: no emitted instruction,
local, error, allocation or release ordering changes, so emitted Wasm remains
byte-identical.

Focused evidence is green: `array_buffer_slice_copy_policy_structure` passes
`5/5`, the neighboring `array_buffer_slice_bound_structure` remains `3/3`, and
the exact
`run_wasm_backend_reobserves_arraybuffer_slice_source_after_observable_work`
CLI witness passes `1/1` through Wasm AOT.

Batch AI makes `ArrayBufferSliceCopyLocals` a single move-only carrier for the
source object, normalized start, normalized final and requested length. Its
five production mentions are exactly the private declaration, constructor,
standard-builtin import, sole constructor call and owned copy-writer
parameter. The writer is the only consumer and performs thirteen field
projections with the exact `4/3/1/5` split for source object, start, final and
requested length. Clone, copy, debug, default, comparison, alias and borrowed
handoff routes are absent, so the four roles cannot be forked or retained past
the one owned writer boundary without a Rust move error.

The strengthened recursive guard pins the attribute-free four-field carrier,
the sole producer and handoff, the complete field-projection census, the exact
constructor, and the unchanged grouped-owner and writer fingerprints. Removing
the incidental derive changes no instruction, local, observation, allocation,
copy or release ordering. At the shared Batch AI checkpoint, `cargo xc` exits
zero, `array_buffer_slice_copy_policy_structure` passes `6/6`, and the exact
`binary_data::run_wasm_backend_reobserves_arraybuffer_slice_source_after_observable_work`
CLI witness passes `1/1`. The exact pinned
`built-ins/ArrayBuffer/prototype/slice/species.js`,
`built-ins/ArrayBuffer/prototype/slice/species-returns-larger-arraybuffer.js`
and `built-ins/SharedArrayBuffer/prototype/slice/species.js` leaves pass all
`6/6` sloppy/strict Wasm-AOT executions with every failure bucket at zero. No
semantic golden was needed or run for Batch AI. Final formatter, diff,
module-boundary, task-plan and 240-entry shortcut-inventory gates are green.

The attribute-excluding carrier declaration remains
`c27d446dd7c67d0222a3d8e3bff7517b8ee65aa9adbedd64a77ad7217d839355`,
its constructor remains
`8e40b3be759a28a4f2240a79c86ed83a281d6ccf249aefb0442f9dbb18454e4f`,
and the raw writer body remains
`b95a56d5e6b021795271d5f61cf0fa05acea24c7c5b0802aabccaa3372eb6f7b`.
The existing normalized grouped-owner and writer fingerprints remain
`(14341, 0xd07f66f964485b66)` and `(7153, 0x32291bb08809c608)`.

Batch AJ makes `ArrayBufferSliceKind` a single capability-free slice-kind
authority. Its five production type mentions are exactly the private
declaration and implementation plus the three grouped builtin producers for
ordinary, shared and immutable slicing. The one owned `slice_kind` selection
has six borrowed projections: copy policy, species use,
default result prototype, brand and flags, and immutable-species rejection.
Clone, copy, debug, default, comparison, hashing and ordering capabilities are
absent, so those decisions cannot be forked from incidental copies or retained
past an owned handoff without a Rust move error.

The strengthened seven-test recursive guard pins all three producers, the
five-mention type census, the one-owned/six-borrowed lifecycle, every exhaustive
mapping, and the absence of alias, clone, dereference and mutable-borrow routes.
The attribute-excluding domain remains
`1860871f6edaec4bf2afd40c0a737ae469e58faeef6b5895c5a51e6e49aad664`.
The borrowed implementation is
`f8d8a88fcfd4720095628c1f95c978f8f48e2881586621537b2fc0cbf45dc3b9`;
its whitespace-normalized form is
`484c872e14c67c4834faae4a5e1778f651eacb3a5f9bc1a4fe233b647ca0ef1e`,
and erasing only the six borrow markers reproduces the pre-AJ semantic hash
`c5d2ec645ff3c40ea4a20971528ebcb8a099ba3f09efbd48bf9c0fd150452392`
and the guard fingerprint `(1179, 0x21c812f9ad84ac3e)`. The grouped owner
remains `(14341, 0xd07f66f964485b66)`. No emitted instruction or observable
ordering changes. Shared `cargo xc` passes, the structure target passes `7/7`,
both exact ArrayBuffer CLI witnesses pass `2/2`, and the ordinary,
immutable-result and shared species leaves pass all `6/6` sloppy/strict
Wasm-AOT executions with every failure bucket at zero. No semantic golden was
needed or run.

`Shared` is intentionally distinct. SharedArrayBuffer is not detachable, so
its branch reloads current length and data without manufacturing an ordinary
detachment check. This type distinction is not a claim that shared-memory
races or every growable-buffer edge case are closed.

## Durable witness

`wasm_arraybuffer_slice_source_reobservation.js` fixes the observable seam:

- detachment in `start.valueOf` still permits later `end` and species effects,
  then throws before copying;
- detachment inside the species constructor throws before copying;
- species-triggered shrinking bounds the copy while preserving a prefilled
  target suffix, including the case where `first` is no longer available; and
- `sliceToImmutable` rechecks a source detached during index coercion; and
- `sliceToImmutable` throws `RangeError`, rather than clamping, when index
  coercion shrinks a still-attached resizable source below the resolved final
  bound.

The existing SharedArrayBuffer grow-and-slice fixture continues to exercise
the distinct shared branch.

## Remaining nonclaims and broad gates

This seam does not establish complete ArrayBuffer, resizable/growable buffer,
species, SharedArrayBuffer, TypedArray, DataView, Atomics, or shared-race
correctness. It does not retire Test262 rewrites or change published
conformance counts.

The completed focused gates are recorded above. No broader pinned Test262 slice
detachment, resize, or species tree was refreshed, and the broad batch ladder
remains deferred. The focused results do not establish a conformance gain or
replace aggregate publication evidence.
