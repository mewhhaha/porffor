# Object.defineProperty descriptor roles

The native entry rejects a primitive target before ToPropertyKey and descriptor
conversion. It consumes the sole ToPropertyDescriptor owner, whose six field
presence bits and whole Value/get/set roots produce the validated borrowed
WasmDescriptor accepted by the shared DefineOwnProperty owner. Normal false
becomes a current-execution-Realm TypeError; the original whole Throw propagates
unchanged. Normal true returns the original target.

Object.defineProperties retains these same converted partial descriptors in
private immutable GC records. Absent fields remain absent; conversion does not
complete the descriptor or expose a JavaScript carrier. Complete
PropertyDefinition records are written before the private list count advances.
The collecting owner is consumed into the completed owner only after all
observable conversion work. The apply phase accepts and consumes only that
completed owner, preventing interleaved conversion and target mutation.

The previous scalar TaggedLocals spelling guard is retired. Existing semantic
controls and new GC Object controls remain required. No compilation or runtime
verification occurred during this dry rewrite; final acceptance requires the
complete source pass and the confirmed 4096 MiB aggregate cgroup budget.
