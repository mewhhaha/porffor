# ArrayBuffer slice source re-observation

Status: normative for the Wasm AOT ArrayBuffer slice copy seam; the
`ArrayBufferSliceBound` invariant is implemented, independently reviewed, and
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
