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
- Eliminate regex/source-pattern agent simulations from `test262/harness.js` under T03.

## Wasm/runtime strategy

Document whether shared operations use Wasm shared memory/atomic instructions or typed host imports. Either approach must preserve JavaScript object identity, detachment rules and agent synchronization. Do not claim concurrency coverage from a single-threaded scripted simulation.

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
cargo test -p porffor-aot-wasm typed_array_ --quiet
cargo test -p porffor-spec-exec agent_ --quiet
cargo test -p porffor-test262 agent_ --quiet
cargo test -p porffor-cli wasm_typed_array --quiet
./target/debug/porf test262 run built-ins/ArrayBuffer --execution-backend wasm --timeout-ms 180000 --threads 4
./target/debug/porf test262 run built-ins/TypedArray --execution-backend wasm --timeout-ms 180000 --threads 4
./target/debug/porf test262 run built-ins/Atomics --execution-backend wasm --timeout-ms 180000 --threads 2
```

Run DataView and every concrete typed-array subtree separately during implementation, then execute shared-buffer/agent tests under repeated stress.
