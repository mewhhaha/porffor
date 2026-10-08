# Atomics access revalidation

Current source status, 2026-10-05: the atomic Wasm-GC rewrite is authored only. Compilation, emitted Wasm, focused controls, real agents and full pinned conformance remain unverified. No status counts changed.

The retained PendingAtomicAccess is minted by concrete integer brand/kind, entry-view and index admission. All value/replacement hooks finish before RevalidatedAtomicAccess acquires the sole current backing. The final comparison is the approved absolute starting byte against exact logical BYTE_LENGTH, after ValidateTypedArrayBounds; it is not a new floored element-index check. Shared integer operations use aligned width atomics, while private bytes remain in the sole GC store. The rounded physical extent is separate from logical admission and is never used by getters, view lengths or copies. Resize clears the discarded/padding region and growth copies only the logical surviving prefix. This retained-capacity interpretation reconciles current RevalidateAtomicAccess with GetValueFromBuffer's sufficient-byte assertion; it does not claim that the default exact-sized Data Block resize algorithm removes that normative assertion tension.

Four paired strict/sloppy finite Engine cohorts in `aot_gc_binary_data_entries.rs` cover native buffers, DataView, TypedArray construction/statics/species and Atomics/Realm lifecycle. Existing CLI semantic fixtures remain; obsolete raw-spelling guards are retired rather than replaced with mirrors. The historical implementation and receipts below do not certify this batch.

## Historical record before the atomic GC rewrite


The nine integer Atomics operations retain the absolute byte index approved by
ValidateAtomicAccess against the entry witness. Index, value and replacement
coercions may execute user code. After those complete normally, one fresh view
observation rejects detachment or an out-of-bounds fixed/tracking view with
TypeError, then rejects an approved starting byte beyond the current backing
length with RangeError. A coercion's own abrupt completion takes precedence.

This follows [ECMAScript 2026 RevalidateAtomicAccess](https://tc39.es/ecma262/2026/multipage/structured-data.html#sec-revalidateatomicaccess).
The last comparison uses the absolute starting byte rather than recomputing a
floored element index; an odd-byte resize may leave that byte in bounds. The
current backing pointer is used only after both rejection paths close.

The actual load/store/RMW/compareExchange emitters require an opaque,
non-copyable RevalidatedAtomicAddressLocal minted by this operation. Existing
method-entry and property/accessor witnesses share its observation law. DoWait
has its own real Int32/BigInt64 entry load because its algorithm has no such
post-coercion revalidation step; wait/notify semantics are not changed here.

One consumed CLI fixture covers the nine operations, Number/BigInt coercions,
fixed/tracking bounds, transfer, compareExchange order, abrupt identity, odd
length, and private/shared growth. Existing source checks are maintained for
the refactor. No compilation, runtime or test execution has occurred. Whole
T17 acceptance, waiter lifecycle and GC migration remain open.
