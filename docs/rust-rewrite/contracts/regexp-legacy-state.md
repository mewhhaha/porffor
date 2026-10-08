# RegExp legacy state and Realm ownership

The pinned `legacy-regexp` tests use the
[TC39 legacy RegExp proposal](https://github.com/tc39/proposal-regexp-legacy-features).
The constructor accessors now read actual Realm-owned state. An initially empty
GC String differs from the null internal marker used for invalidated state.
A closed captured slot selects each getter; only input has a setter. Construction
requires the original Realm and the immutable legacy-enabled choice for both
source literals and dynamically constructed RegExp objects.

Successful builtin matching publishes the input, full match, UTF-16 contexts,
first nine captures and final capture. Failed matches preserve the prior record.
A successful subclass match invalidates the executing Realm's state; borrowed
cross-Realm operations preserve unrelated state. The input setter validates its
receiver before ToString and preserves abrupt identity. `compile` rejects a
foreign or legacy-disabled receiver before mutation.

Two Engine controls run both source modes through actual Wasm AOT. They cover
unmatched captures, a final capture beyond nine, surrogate-pair contexts,
reentrant and abrupt conversion, native descriptors, actual called-Realm errors,
subclass invalidation and recovery. This source batch is uncompiled and
unexecuted until the combined checkpoint; no pinned aggregate is refreshed here.
