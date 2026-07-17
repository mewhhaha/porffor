# Wasm AOT Heap Layout

This document records the current Rust Wasm-AOT linear-memory data model. It is
the checked-in design anchor for T05 until the runtime grows a complete
collector.

## Value Encoding

Porffor values cross the Wasm backend as a tag and payload pair. Immediate
values such as `undefined`, `null`, booleans and small static sentinels are
identified by their `ValueKind` tag. Heap-backed values store an object, string,
array, function, environment or backing-store address in the payload. Number
payloads preserve IEEE-754 bits so `NaN` payload behavior and signed zero remain
observable to the abstract operation layer.

`VALUE_ENCODING_SLOTS` is the checked registry for this ABI. It covers all
ECMAScript language types plus backend-specific heap tags such as arrays,
functions, arguments objects and dynamic tag/payload pairs. The IR now preserves
arbitrary-precision BigInt literal text, but the current Wasm-AOT value payload
is still an implementation debt for full T05: the registry marks BigInt as a
hybrid temporary i64-or-heap payload. Small BigInts use the legacy i64 path, and
BigInts that require arbitrary precision are rejected at emission rather than
silently truncated until heap-backed storage and operation support land.

## Allocation

All runtime allocations use the Wasm memory global `HEAP_PTR_GLOBAL_INDEX`.
`FunctionBuilder::emit_heap_alloc_from_local` aligns allocation sizes to eight
bytes, grows memory by whole Wasm pages when the end pointer exceeds the current
memory size, and returns a stable linear-memory address. Existing heap values
therefore remain stable across `memory.grow`.

The allocator is bump-only today. A future collector must either keep objects
non-moving or update every root and interior pointer recorded by the layout
registry before compaction.

## Collector Contract

`HEAP_COLLECTOR_CONTRACT` records the checked collector boundary. The current
contract is a non-moving tracing collector with metadata validation only; it is
not executable. `gc()` must continue to throw until the contract capability is
advanced to executable and every phase required by `HEAP_COLLECTOR_PHASES` is
implemented.

The required phases are stop-the-world, root scan, strong graph marking,
ephemeron processing, WeakRef clearing, finalizer queueing, sweep and resume.
The registry ties those phases to `HEAP_ROOT_SOURCES` and
`HEAP_WEAK_EDGE_SLOTS`, so a future implementation cannot expose `gc()` without
accounting for roots, ephemerons, weak targets and finalizer holdings in one
place.

## Layout Registry

`crates/porffor-aot-wasm/src/heap.rs` owns the layout constants and exposes
`HeapLayoutSlot` registries for the record families that currently have shared
runtime meaning:

- object headers;
- function objects and realm-owned prototype references;
- bound-function records;
- array object headers and array-specific descriptor slots;
- ordinary object property entries;
- dense array entries;
- environment parent and value slots;
- heap string records, whose payload references non-scanned UTF-16 code units;
- heap BigInt records, whose payload references non-scanned fixed-width limbs;
- heap Symbol records, whose description and registry-key payloads are traced
  as tagged references;
- promise, promise-reaction and pending-job records, including the realm,
  callback, result and linked-list edges that must stay live until the job or
  reaction is drained.

Each slot records the record family, name, byte offset, width and whether the
payload may contain a heap pointer. Unit tests assert that registered slots are
eight-byte aligned, remain within their record size and do not collide inside a
record. New heap record families should be added to the registry before their
offsets are consumed by emitters.

Some runtime records are ordinary objects with well-known metadata properties
rather than fixed-offset byte records. `HEAP_NAMED_SLOT_LAYOUTS` records these
properties for ArrayBuffer, DataView, TypedArray, ArrayIterator,
StringIterator, RegExpStringIterator and iterator-helper objects. Named slots mark whether a
property is a strong reference and whether the referenced target is a
tagged/object graph that should be scanned. ArrayBuffer backing-store addresses
are strong references to raw bytes, but the bytes themselves are not scanned as
tagged values.

Iterator.zip helpers keep their mutable state in an inaccessible ordinary
state object. A helper with the `OBJECT_INTERNAL_BRAND_ITERATOR_ZIP_HELPER`
brand holds that state's pointer in its `boxed_payload` header slot while its
`boxed_kind` remains `NONE`; the state object's eight named slots are registered
as `HEAP_ITERATOR_ZIP_STATE_NAMED_SLOTS`. This branded header reference is a
strong edge, and the state object owns the iterator, next-method, open-array
and padding-array edges that must be scanned.

Iterator helper instances use one of the six exact-object internal brands for
zip, map, filter, flatMap, take and drop. These unforgeable header brands select
the shared `%IteratorHelperPrototype%` methods without consulting observable
properties on the helper object.

Raw byte spans are represented separately from pointer-bearing records.
`HEAP_RAW_BYTE_SPAN_LAYOUTS` records ArrayBuffer backing stores, string
code-unit storage and BigInt limb storage as non-pointer spans. A collector
must keep those spans alive through the owning ArrayBuffer/DataView/TypedArray,
String or BigInt object graph, but it must not scan the span contents as tagged
values.

## Rooting Model

`HEAP_ROOT_SOURCES` records the root categories that a collector must scan at
each safepoint. The current registry covers:

- globals that hold heap payloads, including realm and intrinsic references;
- active frame locals that contain tag/payload values;
- lexical environment chains and environment slots;
- function object environment handles and prototype references;
- object, array, accessor and descriptor payload slots marked as pointers;
- completion records that carry thrown, returned, break or continue values;
- bound-function target/this/argument-array records;
- array object descriptor payloads, present-index tables and named-property
  tables;
- iterator source, next-method, mapper/predicate and nested flatMap iterator
  helper slots;
- promise result, reaction lists, reaction handlers, pending job callbacks,
  job arguments, job realms and job queue links;
- ArrayBuffer backing stores through their owning ArrayBuffer metadata objects;
- generator, additional binary-data view records and host-handle records once
  those records are represented in the heap registry.

Transient sources such as active locals, completion records and host-borrowed
values are still roots while control can re-enter user code or the host. They
cannot be optimized away at a GC safepoint merely because their values are held
in temporary locals.

The `pointer` flag on `HeapLayoutSlot` is intentionally conservative. Some
payload slots are only pointers for object/string/symbol/function tags; the
collector must still inspect the paired tag before tracing the payload.

`wasm_heap_rooted_closure_exception.js` is the focused runtime proof for the
current bump allocator: it keeps an ArrayBuffer/DataView graph reachable through
a closure stored on a thrown object, then forces `memory.grow` and reads through
both the captured and newly allocated views. This does not prove collection yet,
but it does pin the current root and stable-address behavior that a collector
must preserve.

Additional focused fixtures cover bound-function and generator graphs across
the same `memory.grow` boundary. These fixtures intentionally prove stable
addresses and visible reachability only; collection, weak clearing and finalizer
scheduling remain guarded by the non-executable collector contract.

## Weak Reachability

`HEAP_WEAK_EDGE_SLOTS` records the weak and ephemeron edge families required by
WeakMap, WeakSet, WeakRef and FinalizationRegistry. WeakMap values are modeled
as ephemeron values that become live only when their corresponding key is live.
WeakRef and FinalizationRegistry targets are weak targets and must not keep the
target object alive through ordinary tracing. FinalizationRegistry holdings stay
strongly live after the target becomes collectible so cleanup callbacks can
receive them.

The records are a contract for the future collector; the collector itself is
not executable yet. When `gc()` is wired to a real collection cycle, it must use
these weak-edge kinds and advance `HEAP_COLLECTOR_CONTRACT` rather than
test-specific shortcuts.

## Host Boundary

`HEAP_HOST_BOUNDARY_CONTRACT` records the import/export ownership rule that
heap tests enforce. The contract rejects durable host pointers in Wasm payloads,
limits memory borrows to the import call, and links re-entrant host calls to the
`host-borrowed-values` transient root source.

Host imports may borrow Wasm memory only for the duration of the import call.
They must not store raw host pointers as durable Wasm payloads. Re-entrant host
calls must treat all tag/payload locals and completion values as live until the
call returns or throws.
