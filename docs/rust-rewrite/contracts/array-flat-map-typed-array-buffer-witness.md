# Generic Array `flatMap` and TypedArray observation

## Ordinary length access, live integer-indexed properties

Array.prototype.flatMap is a generic Array method. It first obtains one
LengthOfArrayLike through the receiver's observable length property. A
TypedArray's own length property or an inherited override therefore participates
in Get and ToLength. The private element count is not a valid substitute.
Length access and coercion happen before mapper validation and species effects.

When the normal TypedArray length accessor is selected by property lookup, that
accessor owns backing-buffer validation and its detached/out-of-bounds policy.
An override can report a different length, resize or detach the buffer, or throw.
FlatMap captures the resulting ToLength value once and does not grow its loop
bound if a callback grows the buffer.

Each visited source index delegates to shared HasProperty and, when present, Get.
Those integer-indexed operation owners observe the current buffer state. Shrink,
out-of-bounds views or detachment can make subsequent indices absent without
changing the captured loop bound. The mapper receives the value read at that
iteration, not a previously captured element.

## Ownership change

The August 24 implementation constructed a TypedArrayViewLocals directly inside
flatMap for a private length projection and a live presence projection. It
explicitly left ordinary length shadowing and mapper/length ordering unresolved.
The new algorithm removes that specialization rather than adding a second length
policy. There are no private TypedArray loads, raw length conversions or direct
buffer witness projections in the flatMap owner. Generic property owners remain
responsible for their existing witness capabilities.

The structural target retains its filename but now enforces delegation and
Get/HasProperty/Call ordering. It rejects private-slot reconstruction and raw
length helpers. Its six-case fixture coupling remains intact: odd-byte tracking,
growth, shrink, fixed out-of-bounds, fixed regrowth and detached views. The
existing CLI fixture is not weakened or rewritten.

## Execution boundaries

The new engine target adds own and inherited length accessors, fractional length
coercion, resize during length access, resize during mapping, and detached views
with explicit length overrides. Those tests complement the retained CLI matrix;
source-structure guards alone do not prove buffer semantics.

See [the conformance follow-up](../aot-flat-map.md) for reproducible verification.
Other generic Array methods and TypedArray-specific methods are not changed by
this ownership migration. Historical fixture pass counts are not reused as
current-head evidence.
