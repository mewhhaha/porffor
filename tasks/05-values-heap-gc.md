# T05 — Value representation, heap, GC and weak reachability

**Status:** In progress — layout registries landed; executable GC and full BigInt remain open

**Parallel group:** Core foundations  
**Depends on:** T02, T04  
**Blocks:** T06, T10, T14, T17, T21 and long-running full-suite stability

## Current repository state

The checked-in heap design and registries document value tags, layouts, roots,
weak edges and collector phases, and allocation grows linear memory safely.
The implementation remains bump-only, the collector contract is metadata-only,
`gc()` is intentionally unsupported, and multi-limb BigInt arithmetic and
conversion are still incomplete. The current linear-memory object model also
does not yet realize this task's Wasm-GC-first target, so that architecture gap
must be resolved rather than treated as completion.

## Objective

Replace ad-hoc linear-memory layouts with a documented, validated runtime data model that can represent every ECMAScript value, grow safely, collect unreachable objects and support weak reachability without changing observable JavaScript semantics.

Design the representation from the experimental Wasmtime lower bound in `AGENTS.md`: Wasm GC structs/arrays, reference types and typed function references are available and should carry the object graph. A hand-written tracing collector over linear memory is not the default plan; it is acceptable only where the checked-in design document justifies it for specific data (for example raw string/buffer bytes). There must be exactly one object model — do not build a parallel non-GC representation for runtimes that lack Wasm GC; those runtimes are rejected at the boundary.

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

- Implement checked allocation, capacity growth and overflow handling for any linear-memory regions the design retains (for example string/buffer byte storage).
- Keep object references stable across memory growth for linear-memory data; GC-managed references are stable by construction.
- Define how host imports borrow memory/references and how re-entrancy is handled.
- Add stress tests near Wasm page boundaries and large sparse allocations.

### Garbage collection

Lean on the runtime's Wasm GC for object lifetime. The task owns: mapping every heap layout onto GC structs/arrays with validated field metadata; keeping any linear-memory side allocations (byte storage, tables) from leaking when their owning GC object dies; and rooting across host calls, suspended frames and pending jobs. If the design document retains a manually collected region, it must define roots from globals, realms, tables, active frames, lexical environments, completion values, host handles and pending jobs — and it must not grow into a second object model.

### Weak semantics

Expose ephemeron/weak-edge support required by WeakMap, WeakSet, WeakRef and FinalizationRegistry. The Wasm GC proposal does not currently provide weak references, so the design document must state explicitly how weak reachability is observed (host-assisted tracking, a dedicated weak-capable region, or a runtime capability) without creating a second general object model. `gc()` used by Test262 must request a real collection cycle; finalization scheduling must remain specification-compatible and not promise collection at an exact instant.

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
