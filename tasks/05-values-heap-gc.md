# T05 — Value representation, heap, GC and weak reachability

**Status:** Blocked on stable T02/T04 interfaces  
**Parallel group:** Core foundations  
**Depends on:** T02, T04  
**Blocks:** T06, T10, T14, T17, T21 and long-running full-suite stability

## Objective

Replace ad-hoc linear-memory layouts with a documented, validated runtime data model that can represent every ECMAScript value, grow safely, collect unreachable objects and support weak reachability without changing observable JavaScript semantics.

## Scope

### Tagged value contract

- Specify payload/tag representation for `undefined`, `null`, Boolean, Number, BigInt, String, Symbol, Object and internal sentinels.
- Preserve all required Number distinctions, including NaN behavior and signed zero.
- Ensure BigInt is arbitrary precision rather than an i64-only semantic model.
- Distinguish JavaScript strings from UTF-8 byte strings; T18 owns algorithms, this task owns storage and identity.

### Heap layouts

Create a central layout registry for:

- ordinary and exotic objects;
- functions/bound functions;
- environments and cells;
- strings, BigInts and Symbols;
- arrays and property tables;
- ArrayBuffer backing stores and views;
- iterator/generator/promise records;
- realm/intrinsic references;
- host handles.

Offsets, sizes, alignment and pointer fields must be generated or asserted in one place. Eliminate unrelated magic offsets scattered through builtin emitters.

### Allocation and memory growth

- Implement checked allocation, capacity growth and overflow handling.
- Keep object references stable across memory growth.
- Define how host imports borrow memory and how re-entrancy is handled.
- Add stress tests near Wasm page boundaries and large sparse allocations.

### Garbage collection

Implement a safe collector suitable for linear Wasm memory. A tracing collector with non-moving objects is acceptable initially if it has a path to compaction or fragmentation control. Define roots from globals, realms, tables, active frames, lexical environments, completion values, host handles and pending jobs.

### Weak semantics

Expose ephemeron/weak-edge support required by WeakMap, WeakSet, WeakRef and FinalizationRegistry. `gc()` used by Test262 must request a real collection cycle; finalization scheduling must remain specification-compatible and not promise collection at an exact instant.

## Design constraints

- No `unsafe` Rust.
- No host pointer values embedded as durable Wasm pointers.
- GC safepoints must not lose values held only in temporary locals or completion records.
- Fast paths must use the same layout metadata as slow paths.
- A thrown JavaScript value remains live through catch/finally and host transitions.

## Acceptance criteria

- A written layout/GC design is checked in beside the implementation.
- Layout collisions and invalid pointer-field declarations fail tests.
- Allocation works across multiple `memory.grow` operations.
- Stress programs can allocate and release cyclic object graphs without exhausting memory at a fixed low threshold.
- Rooting tests cover closures, bound functions, pending promises, generators, exceptions and cross-realm objects.
- Weak collection/finalization primitives can be implemented without semantic hacks.
- Existing Wasm fixtures and representative real Test262 filters do not regress.

## Required tests

```sh
cargo test -p porffor-aot-wasm heap_ --quiet
cargo test -p porffor-engine wasm_ --quiet
cargo test -p porffor-cli wasm_ --quiet
```

Add long-running stress tests behind an ignored or dedicated CI profile, plus focused real tests under `built-ins/WeakRef`, `built-ins/FinalizationRegistry`, weak collections and allocation-heavy Array/Object subtrees.