# GC native BinaryData ownership

Status: the MAIN136 whole all-feature/all-target type check and source guards
passed. The focused foundation reached a TypedArray constructor Wasm Boolean
operand failure. The alignment repair below is source-authored; its type, Wasm
validation and Engine controls remain pending. No conformance result is claimed.

Concrete ArrayBuffer, SharedArrayBuffer, BufferOwner, BufferView, DataView and TypedArray records use central typed field authority. Construction completes prototype selection before immutable record publication. Private storage is one GC ByteArray with exact logical BYTE_LENGTH and rounded physical extent; detached means null BYTES and logical zero. Resize replaces the sole edge, copies only the surviving logical prefix, and zeroes new/discarded capacity. Shared storage is a retained native HostResource and shared byte memory; JavaScript identity stays in GC.

DataView22 and the twelve TypedArray constructors use whole values and completion routes. DataView validates cached explicit length before prototype Get, then late detachment/offset/supplied-length bounds. TypedArray scalar length conversion precedes prototype Get; object construction selects prototype before source initialization. Same-kind copying preserves exact bits. From collects iterable values before constructing/mapping; array-like Get and mapping interleave in index order. From/Of custom targets require write admission even when empty. Subarray retains its two-versus-three argument selection. Primitive/object hooks, callable Proxies, original thrown identity and defining/NewTarget Realm authority remain observable.

Atomics14 keeps early integer/view/index admission separate from the late absolute-start revalidation after callbacks. Native shared integer words use aligned no-tear atomic accesses. Notify and both wait modes use one backing registry and FIFO lock. Comparison and registration are indivisible with respect to notify. Async records retain SAB, Promise, positive native identity and deadline; zero means immediate mismatch. Sync records retain a condition-variable signal. Timeout removal races notify through the same lock, Store retirement cancels its async records, and agent-group retirement wakes sync records with a host failure before releasing the list. No second Wasm wait queue remains.

Current [RevalidateAtomicAccess](https://tc39.es/ecma262/multipage/structured-data.html#sec-revalidateatomicaccess) admits an absolute starting byte; [GetValueFromBuffer](https://tc39.es/ecma262/multipage/structured-data.html#sec-getvaluefrombuffer) separately asserts sufficient bytes. The retained rounded extent is an implementation interpretation supported by [HostResizeArrayBuffer](https://tc39.es/ecma262/multipage/structured-data.html#sec-hostresizearraybuffer), which specifies exact logical length without prescribing physical capacity. It keeps admitted accesses within declared GC storage while logical consumers never expose padding; growth cannot expose padding writes. The default fresh exact-sized resize algorithm does not itself resolve this assertion tension. No odd-tail workload or runtime oracle was used.

Current [Atomics.pause](https://tc39.es/ecma262/multipage/structured-data.html#sec-atomics.pause) ignores semantic arguments and returns undefined. Its existing length0/pure metadata stays unchanged. Prior argument-validation fixture bytes are retained in the packet predecessor; the maintained finite fixture now checks ignored Symbol/object operands and no coercion. Any mismatch with the pinned Test262 version remains a reported suite-version obligation, not a skip or count change.

Four finite paired strict/sloppy Engine cohorts and two maintained CLI fixtures are authored. Wait controls use zero timeout or a bounded registered wait followed by notify. All execution remains deferred. The preserved integer-indexed property successor now owns immutable Set, GetOwnProperty and DefineOwnProperty through the shared descriptor kernel; see `gc-integer-indexed-immutability.md`. It preserves the required early Boolean rejection and uncoerced SameValue compatibility instead of adding a blanket late element-write guard. T17 and the whole GC cutover remain unverified.


## TypedArray alignment Boolean boundary

Both ArrayBuffer/SAB constructor alignment guards retain their I64 offset or
logical byte-length remainder and normalize `remainder != 0` with I64Eqz then
I32Eqz before entering the tracked Wasm If. This supplies the required I32
Boolean without truncating the remainder. Misaligned offsets still fail after
ToIndex(byteOffset) and before supplied-length coercion. The fixed-buffer
implicit-length check remains inside its original no-tracking branch; explicit
length and resizable/growable tracking policies retain their separate admission.

The existing third Engine cohort now checks aligned nonzero offsets, empty and
explicit-length views, misaligned offsets suppressing length hooks, fixed
implicit-length failures, explicit views of a partial-tail buffer, and allowed
tracking tails. Finite 1/2/4/8-byte Number and 8-byte BigInt constructors use
both private and shared backing. All old assertions and the four paired
strict/sloppy cohort registrations are retained. These additions are unrun.

ArrayBuffer backing allocation uses the same canonical mutable GC ByteArray
through the private `byte_array_allocate` import. The closed signature accepts
one checked I32 extent and returns that exact nullable array type. The native
callback runs Wasmtime's zero-filled allocator, including its normal collection
and growth attempt. Only `GcHeapOutOfMemory<()>` becomes null; invalid ABI,
schema, and other host errors retain their ordinary failure paths. The Wasm
consumer checks null and creates a RangeError through its existing defining-
Realm Completion before attempting to publish the buffer. This preserves the
existing per-Store memory bound and makes actual backing-store failure catchable
without a manual heap or a fabricated capacity threshold.

Prototype selection still precedes allocation. Resize and transfer continue to
allocate the replacement before mutating or detaching the original owner.
The existing native allocation regression retains its 1 GiB Uint8Array request
and additionally checks Float64Array, observed/abrupt prototype selection,
failed resize and transfer preserving the original bytes, and zeroed successful
allocation. Compilation, Wasm validation and native verification remain pending
the frozen baseline sweep and complete batch application.
