# ArrayBuffer slice source re-observation

Status: normative for the Wasm AOT ArrayBuffer slice copy seam.

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

## Nonclaims and deferred gates

This seam does not establish complete ArrayBuffer, resizable/growable buffer,
species, SharedArrayBuffer, TypedArray, DataView, Atomics, or shared-race
correctness. It does not retire Test262 rewrites or change published
conformance counts.

Static freeze gates are `rustfmt --check` for the touched Rust files,
`node --check` for the fixture, focused source searches, `git diff --check`, and
manual local-lifetime review. Cargo, fixture execution, focused pinned Test262
slice detachment/resize/species trees, and the broad batch ladder remain
deferred until the frozen patch is independently reviewed.
