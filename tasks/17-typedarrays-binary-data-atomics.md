# T17 — ArrayBuffer, DataView, TypedArray, SharedArrayBuffer and Atomics

**Status:** In progress — broad binary-data support exists; GC/agents and full-tree closure remain

**Parallel group:** Feature lane; split internally by API family  
**Depends on:** T03, T04, T05, T06, T10; iterator paths use T15; `waitAsync` uses T14  
**Blocks:** Binary-data and concurrency portions of T26

## Current repository state

ArrayBuffer, SharedArrayBuffer, DataView, TypedArray and Atomics have dedicated
backend implementations, including resizable/growable backing-store and
`waitAsync` work with focused fixtures. Binary-data-specific harness rewrites
remain, real GC is unavailable, and the shortcut-free real-agent/full-tree
acceptance criteria have not been demonstrated on a current complete matrix.

The cross-instance async-waiter transport now shares the closed
`lila-runtime::AgentHostOperation` wire domain with the rest of the Wasm agent
ABI. Registration, polling, notification and cancellation are typed at every
AOT producer and exhaustively dispatched by the engine; their stable wire
values remain 10 through 13. This prevents producer/consumer opcode drift but
does not by itself prove waiter semantics or multi-agent stress safety.

Resizable-buffer observation now has a typed AOT seam for callback and
search/access consumers. A private TypedArray view record keeps the stored fixed
byte extent immutable, while a fresh buffer witness derives out-of-bounds
state, element length and an element-aligned index bound from one cached
backing-store length. Its closed use domain distinguishes validated TypedArray
method entry, generic Array length snapshots, live integer-indexed property
observations and the three-kind view-accessor projection. The callback families
shared with T16 use that seam,
including both `reduce` property checks; so do `at`, the generic Array index
searches and the non-generic TypedArray search methods. TypedArray search length
is validated and snapshotted once at method entry, while generic Array search
keeps its `LengthOfArrayLike` and live integer-indexed behavior. Focused
contracts cover fixed-view out-of-bounds/regrow behavior and the Uint16
odd-byte floor.

The witness is not yet the universal integer-indexed exotic protocol. The
shared indexed `Get` implementation and other binary-data consumers still use
older emitters, and no Test262 resizable-buffer rewrite has been retired. The
TypedArray iterator boundaries are migrated separately below; ordinary Array
iterators do not require a TypedArray backing-store witness.
Constructor/subclass and BigInt variants represented by those rewrites remain
separate closure work. The shared `at` emitter encodes its generic-array-like
versus validated-TypedArray receiver policy as a closed enum; the old raw
boolean can no longer route a new caller to the wrong incompatible-receiver
behavior.

ArrayBuffer slicing now has a closed late-source-observation seam. The three
builtin operations project exhaustively to detachable-bounded, shared-bounded,
or detachable-exact-final copy policy. The sole copy writer rechecks ordinary
detachment and reloads current source length and data after observable work.
Ordinary `slice` bounds the copy by the bytes still available from the initially
normalized start, so a species-provided target suffix remains untouched.
`sliceToImmutable` instead rejects a current length below the resolved final
bound before allocating its target, then copies the exact requested length.
Shared sources keep their distinct non-detachable bounded branch. The focused
[slice source re-observation contract](../docs/rust-rewrite/contracts/array-buffer-slice-source-reobservation.md)
and CLI fixture cover detachment during coercion/species, ordinary bounded
resizable shrinkage, and `sliceToImmutable` detach-versus-short-source error
precedence; this is not yet a claim of complete ArrayBuffer or shared memory
correctness.

The three `%TypedArray%.prototype` view accessors now share the same live
buffer-witness seam as the migrated Array/TypedArray consumers. A closed
`TypedArrayAccessorKind` makes `byteLength`, `byteOffset`, and `length` explicit
projections; each builtin delegates with one variant, and the accessor compiler
cannot directly read backing length, data, or the length-tracking slot. The
single witness therefore owns detached/out-of-bounds zeroing, fixed-view
regrowth, and whole-element flooring for odd-byte length-tracking buffers.
The focused
[accessor buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-accessor-buffer-witness.md)
and existing accessor fixture pin those rules. This closes the accessor
duplication, not the older shared indexed `Get`, constructor, or
remaining binary-data consumers, and it does not retire a Test262 rewrite.

TypedArray iterator creation and stepping now use that same live buffer witness
instead of reconstructing private view slots through the older raw validator.
Both boundaries select the closed `ValidatedMethodEntry` projection: creation
consumes validation, while `next` consumes the length derived from the one
cached backing-store observation. Detached and out-of-bounds errors route
through the current function Realm, including created-Realm TypedArray methods
and their Realm-owned `%ArrayIteratorPrototype%.next`. The focused
[iterator buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-iterator-buffer-witness.md)
and existing iterator fixture pin Realm identity, detach/shrink timing, current
resizable length, whole-element flooring and permanently-done behavior. The remaining raw TypedArray
validators and full integer-indexed/iterator closure remain open; this is a
source-invariant correction and does not claim a new baseline pass.

`%TypedArray%.prototype.join` now uses the validated-method-entry projection of
that same buffer witness. Its compiler performs the receiver-brand check first,
loads one immutable view record, and consumes the witness's element length
directly instead of reconstructing private slots, calling the legacy raw
validator and dividing byte length itself. Detached and out-of-bounds failures
therefore use the executing builtin's Realm, including when a created Realm's
`join` is borrowed onto an entry-Realm receiver. Separator coercion remains
after the initially captured length, and later integer-indexed reads remain
live. The focused
[join buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-join-buffer-witness.md)
and CLI fixture pin Realm identity, fixed and tracking resize behavior, BigInt,
and whole-element flooring. Remaining raw validators, the shared indexed
`Get`, Test262 rewrites and full binary-data closure remain separate work.

## Objective

Implement the complete binary-data stack, integer-indexed exotic semantics and real agent/Atomics behavior. Replace rejection-only SharedArrayBuffer behavior and harness simulations with general backing-store and host concurrency support.

## Backing stores and ArrayBuffer

- Model detachable, resizable, growable/shared and fixed backing stores separately from view objects.
- Implement `ArrayBuffer` construction, `byteLength`, `maxByteLength`, `resizable`, `resize`, `slice`, `transfer`, `transferToFixedLength`, detachment and species behavior.
- Preserve backing-store identity across views and define safe host access during memory growth/detachment.
- Implement `SharedArrayBuffer` and growable shared buffers where present; they must not be detachable.

## DataView

Complete constructor validation and every get/set method, including:

- ToIndex/offset ordering;
- detached/out-of-bounds checks before and after observable coercion;
- endian handling;
- integer, Float16, Float32/64 and BigInt64/BigUint64 conversion;
- resizable/growable buffer behavior;
- realm/species/custom-new-target descriptors.

## Typed arrays

Implement all concrete typed-array constructors and `%TypedArray%` semantics:

- construction from length, buffer/offset/length, typed arrays and iterables/array-likes;
- integer-indexed exotic internal methods and canonical numeric index strings;
- fixed vs length-tracking views over resizable/growable buffers;
- BigInt/Number element-kind separation, clamping, Float16 and NaN/signed-zero rules;
- all static/prototype methods, iterators, species and subclassing;
- detachment/out-of-bounds validation at exact spec points;
- generic Array method borrowing where allowed and non-generic TypedArray methods where required.

## Atomics and agents

- Implement all Atomics operations with correct element-kind validation and sequentially consistent behavior required by ECMAScript.
- Provide host-managed shared backing stores and actual agent threads/workers for Test262.
- Implement wait queues, `wait`, `notify`, `waitAsync`, timeouts, `isLockFree`, blocking restrictions and monotonic timing.
- Integrate job completion for `waitAsync` with T14.
- Eliminate regex/source-pattern agent simulations from the embedded
  `lila-test262` local harness under T03.

### Resolved CLI hang and remaining concurrency debt

`binary_data::run_wasm_backend_succeeds_for_atomics_wait_core_fixture` used to
hang the CLI suite. The bounded known-failure machinery detected when it began
passing in batch 6, and its hang row, `should_panic` annotation and compile-time
ledger assertion were removed together. It is now an ordinary passing test and
the current CLI ledger contains no declared hang. The suite must run without an
`atomics_wait_core` skip.

That focused result proves only that the fixture's non-equal waits return; it
does not prove the real-agent acceptance criteria below. Host-managed agents,
wait queues, notifications, timeouts and `waitAsync` job integration remain
open until the real Test262 agent trees pass without source-pattern simulation.
The generic per-invocation timeout and watched-run safeguards remain useful for
detecting the next hang and are not evidence of an expected failure.

## Wasm/runtime strategy

The backend uses a hybrid design. Shared scalar memory operations use Wasm
shared memory and atomic instructions. Host-managed agent orchestration and
the cross-instance `waitAsync` waiter registry use the typed `agent_call`
import, because waiters and reports must cross independently instantiated Wasm
modules. The host operation is decoded into a closed Rust enum before semantic
dispatch; an unknown wire value is a visible host error. Both paths must still
preserve JavaScript object identity, detachment rules and agent
synchronization. Single-threaded scripted simulation is not concurrency
coverage.

## Acceptance criteria

- Complete pinned trees for ArrayBuffer, SharedArrayBuffer, DataView, TypedArray constructors/prototypes and Atomics are green.
- Integer-indexed exotic descriptor/key/proxy cases pass.
- Resizable/growable buffer tests pass before, during and after coercion/callback mutation.
- BigInt and Number typed arrays reject mixed values correctly.
- Real multi-agent wait/notify/report tests pass without source pattern matching.
- Detached/out-of-bounds checks occur at spec-required times.
- No data races or host panics under repeated agent stress tests.

## Required tests

```sh
cargo test -p lila-aot-wasm typed_array_ --quiet
cargo test -p lila-spec-exec agent_ --quiet
cargo test -p lila-test262 agent_ --quiet
cargo test -p lila-cli wasm_typed_array --quiet
./target/debug/lila test262 run built-ins/ArrayBuffer --execution-backend wasm --timeout-ms 180000 --threads 4
./target/debug/lila test262 run built-ins/TypedArray --execution-backend wasm --timeout-ms 180000 --threads 4
./target/debug/lila test262 run built-ins/Atomics --execution-backend wasm --timeout-ms 180000 --threads 2
```

Run DataView and every concrete typed-array subtree separately during implementation, then execute shared-buffer/agent tests under repeated stress.
