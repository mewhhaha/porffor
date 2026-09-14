# TypedArray byte copies and backing-buffer Realm

Constructing a TypedArray from a TypedArray with the same element kind copies
the source bytes. `TypedArray.prototype.set` also preserves bytes when both
element kinds match. Passing these values through Number would quiet signaling
NaNs, and the Float16 encoder also replaces NaN payload bits. Different element
kinds continue to use their numeric conversions.

The shared ascending-byte emitter has explicit source and target Wasm memory
indices. Backing stores use the selected buffer memory; temporary snapshots use
private runtime memory zero. `set` compares backing-store base addresses to
recognize shared blocks even when distinct SharedArrayBuffer wrappers alias
them, snapshots aliased source bytes before writing, and copies disjoint blocks
in ascending order. The existing live view witnesses and capacity checks run
before addresses are loaded. Capacity errors precede Number/BigInt content
errors, including empty sources with an out-of-range offset.

Constructor-owned backing buffers select the active builtin function's
`HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET`, already populated by entry
and foreign Realm construction. This is independent of the newTarget used for
the outer TypedArray prototype. Supplied ArrayBuffer objects retain identity.

`lila-engine --test aot_typed_array_byte_copy` covers every binary16 NaN
encoding, Float32/Float64 signaling NaNs, overlapping views in both directions,
shared and resizable buffers, detachment during offset coercion, exception
precedence, numeric cross-kind controls, and foreign constructor/newTarget
combinations. Source review and formatting alone do not establish Wasm behavior;
the native target remains mandatory before declaring verification complete.

Constructor-generated call and iterator TypeErrors use the executing builtin's
stored Realm, including when its public binding or internal prototype changes.
Calling without `new` rejects before observing arguments; exceptions thrown by
iterator hooks retain their original identity. The
`aot_typed_array_constructor_error_realm` target covers all twelve constructors,
ordinary and callable iterable sources, and the ordering and identity controls.
