# DisposableStack synchronous lifecycle with semantic GC

The 2026-10-05 source draft uses an actual GC DisposableStack brand and a
separate private DisposableResourceStack List. The object holds the List edge;
the List owns its mutable typed resource array and complete-entry count. A
successful native pending-state check creates the only PendingDisposableStack
witness accepted by registration and move. Disposed registration/move throws
ReferenceError before callback or resource acquisition.

Registration retains the original List before observable GetDisposeMethod.
This matters when a Symbol.dispose getter moves the original stack: the new
resource must append to the List already transferred to the destination.
Growing its backing array changes the same List's edge, preserving identity.
Each immutable resource record retains the whole value/method and closed call
kind; the complete record is written before the count advances. Nullish use
registers no resource. Adopt and defer validate callability once and retain their
respective undefined-this argument conventions.

Move allocates a destination in the executing method's Realm, detaches the
original List, installs a fresh empty List on the source, and marks the source
disposed. Its private non-Copy transfer owner is consumed exactly once by the
pending destination finalizer. No public constructor/prototype property lookup
or linear-memory record/pointer transfer participates.

Dispose marks the object disposed before callbacks and consumes an owned LIFO
walker. Each entry is detached before invocation. Normal return values are
ignored, including thenables/Promises; synchronous stack disposal performs no
Await or assimilation. Original whole thrown values are retained and successive
failures create actual SuppressedError instances in the executing Realm, with
the latest error and previous suppressed completion in the required order.
The original private List is cleared after walking even when disposal failed.
Reentrant or repeated dispose returns undefined.

The actual List capture/append behavior follows
[AddDisposableResource](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-adddisposableresource)
and the native [use operation](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-disposablestack.prototype.use).

Three authored GC controls cover getter-triggered move/List identity, grown
LIFO storage, whole callback receiver/arguments, reentrant disposal, ignored
thenables, SuppressedError identity/order, state/error precedence, constructor
newTarget and borrowed-Realm allocation/errors. Existing semantic controls remain
required. Two obsolete native spelling guards are retired; the old slot-layout
retirement remains with the central heap batch. No compilation, emitted-Wasm,
runtime or conformance has run in this source pass. Verification waits for all
remaining tasks and a confirmed 4096 MiB aggregate cgroup cap with one worker.
