# T05 — Value representation, heap, GC and weak reachability

**Status:** In progress — GC and weak runtime limits are explicit; cyclic collection and real weak reachability remain blocked

**Parallel group:** Core foundations  
**Depends on:** T02, T04  
**Blocks:** T06, T10, T14, T17, T21 and long-running full-suite stability

## Current repository state

[`docs/rust-rewrite/value-heap-gc.md`](../docs/rust-rewrite/value-heap-gc.md)
is now the checked-in architecture and phased cutover contract. The new
`gc_types.rs` vocabulary distinguishes GC type indices, field owners, storage
values, mutability, nullability, strong GC references and owner-typed linear
spans at compile time. The central `ModuleTypeRegistry` appends
`RuntimeGcAnchor` and then `RuntimeGcAnchorHolder` from that typed schema. The
holder has one immutable, non-null `GcRef<RuntimeGcAnchor>` field. Consuming the
type registry and open global-section builder derives one unexported typed root
from the section's actual next index, appends it after every existing global,
and returns one opaque, non-cloneable package containing both finalized
sections and the root lifecycle. A dedicated consuming transition compiles main
internally against that exact package and retains it in package-owned code; no
arbitrary callback can substitute another package. Once the remaining bodies
are supplied, a single consuming assembly operation emits the package's type,
global and code sections inseparably in Wasm section order. Callers cannot
predict a raw root index, extract or copy its schema, clone its raw global
section, append a later global, split two packages across main/type/global/code,
or give an internal function the root lifecycle. Main constructs both structs
before any call, transfers the holder's edge into the root, keeps it through
source execution and the final job checkpoint, then verifies the anchor ABI and
clears the root on every real main exit. The schema module is the sole raw
Wasm-GC encoder boundary: module assembly consumes one opaque compiled package,
and function emission consumes opaque initialization/cleanup operations instead
of extracting interchangeable type, field and global `u32` indices.

The product implementation is still the bump-allocated linear-memory object
model. Its layout, root, weak-edge and collector tables remain passive metadata;
`gc()` is unsupported and current weak records are strong in practice. The GC
anchor and its global root are only a runtime-capability/lifecycle witness; no
JavaScript semantic value has moved from the linear heap and there is no
integer/reference bridge. This does not establish semantic roots for calls,
exceptions, suspended frames or pending jobs.

The engine is pinned to Wasmtime 38.0.4. One typed runtime policy now configures
both product Wasmtime engines, explicitly requires reference types, typed
function references, Wasm GC and exceptions, and explicitly selects DRC. The
product dependency retains `gc-drc` and removes `gc-null`, so engine creation
cannot silently fall back to the null collector. DRC cannot collect cycles;
T05's cyclic-graph acceptance criterion therefore requires a cycle-capable
lower-bound update, not a hand-written parallel collector.

The same product policy records weak reachability separately as
`WasmWeakReachabilityCapability::Unavailable`. Wasm GC and DRC do not expose
the weak-reference or ephemeron operations T21 needs, and typed engine-setup
errors retain that fact independently of the collector selection. This is an
explicit blocker, not a claim that the current linear weak records have weak
semantics.

The central feature-enabled CLI compile covers both `lila-aot-wasm` and
`lila-engine`. The complete resource-bounded engine inventory and the complete
620-test default CLI inventory instantiate and execute product modules through
the shared main prologue under the explicit DRC policy, so the typed
strong-edge/ABI probe is runtime-verified. The new global-root lifecycle still
requires its focused and inventory verification. Even once verified, it proves
only the lower-bound feature, schema edge and one capability root—not semantic
heap migration, reclamation, cycle collection or weak reachability.

## Landed foundation

- One-object-model invariant and atomic semantic cutover are explicit.
- `GcTypeIndex<T>` and `GcField<Owner, Value, Mutability, Nullability>` prevent
  the main schema/index/field-shape mixups before encoding.
- `GcRef<T>` deliberately has no integer representation and denotes only a
  strong edge; no false weak-reference marker exists.
- `GcRootGlobal<T>` names only a mutable, nullable Wasm global containing a
  strong reference to `T`; consuming the type/global builders derives the root
  from the actual section length, appends it, and returns one opaque finalized
  package. Its closed main compiler consumes that package into package-owned
  code, and the sealed result exposes no independent main/type/global/code
  assembly operations.
- `LinearAddr<Owner>` and checked `LinearSpan<Owner>` distinguish byte storage
  from object identity and name its sole GC owner.
- `RuntimeGcAnchorSchema` fixes one immutable `i32` ABI-version field; its type
  index is assigned by the central builder rather than a numeric constant and
  is carried into the main-function initialization as typed module state.
- `RuntimeGcAnchorHolderSchema` fixes one immutable, non-null
  `GcRef<RuntimeGcAnchor>` field. Ordered central registration consumes the
  anchor's typed index when encoding that field. The main probe transfers that
  edge into the typed root before any call and verifies/clears it only at the
  shared main exit; the final job checkpoint deliberately does not clear it.
- Raw GC type-index and field-ordinal construction/extraction, typed root
  construction/extraction and `struct.new`/`struct.get` instructions are
  private to `gc_types.rs`. No module-assembly API accepts a planned root slot;
  consuming the actual global section creates its private, paired
  `RuntimeModuleSchema`. Only the opaque package exposes complete lifecycle and
  consume-once assembly operations, so owner/field/target/root-index and
  cross-package main/type/global/code pairing mistakes fail at the Rust boundary
  instead of surviving until Wasm validation.
- `WasmGcCapability::DeferredReferenceCountingWithoutCycleCollection` is the
  closed collector truth consumed by configuration, trace reporting and typed
  `EngineError` context; both native compiler profiles share that policy.
- `WasmWeakReachabilityCapability::Unavailable` independently closes the
  product weak/ephemeron capability domain, so changing the collector cannot
  silently imply weak-reference support.

## Remaining implementation sequence

1. Generate the complete GC layout/type/accessor registry.
2. Close the value, completion, exception and host ABIs over scalar plus real
   Wasm-reference parts.
3. Atomically switch every semantic-object producer/consumer and delete the
   corresponding linear object model in the same batch.
4. Raise the lower bound to a cycle-capable collector and close rooting,
   side-storage and cyclic stress cases.
5. Select a real weak/ephemeron facility and replace the explicit unavailable
   capability before implementing weak collections, WeakRef,
   FinalizationRegistry and Test262 `gc()`.

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
cargo test -p lila-aot-wasm heap_ --quiet
cargo test -p lila-engine wasm_ --quiet
cargo test -p lila-cli wasm_ --quiet
```

Add long-running stress tests behind an ignored or dedicated CI profile, plus focused real tests under `built-ins/WeakRef`, `built-ins/FinalizationRegistry`, weak collections and allocation-heavy Array/Object subtrees.
