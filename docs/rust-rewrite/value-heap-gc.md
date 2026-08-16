# Value, heap and garbage-collection architecture

This document is the source of truth for T05. It describes the object model
Lila is moving to; it does not describe the current linear heap as complete.
The migration must preserve one product object model at every commit.

## Ground truth

The current Wasm-AOT path represents a JavaScript value as integer payload/tag
parts. Identity-bearing values are integer addresses into a bump-allocated
linear-memory heap. `heap.rs` contains extensive layout, root, weak-edge and
collector tables, but those tables do not drive an executable collector. The
current `gc()` path is unsupported, and the current weak-reference records hold
ordinary strong integer addresses. They are useful inventory, not proof of GC
or weak semantics.

The engine is pinned to Wasmtime 38.0.4. Every product engine is now built from
one `WasmtimeRuntimePolicy`: reference types, typed function references, Wasm
GC and exception handling are required explicitly, and the collector is
explicitly `Collector::DeferredReferenceCounting`. The product feature graph
contains `gc-drc` and no longer contains `gc-null`, so there is no null-collector
fallback. Wasmtime states that DRC cannot collect cycles; unreachable cycles
remain until the Store is dropped. Therefore the current lower bound cannot
meet T05's cyclic-graph acceptance criterion even though Lila emits the GC
capability anchor.

The product runtime policy independently records
`WasmWeakReachabilityCapability::Unavailable`. This is separate from DRC's
cycle limitation: Wasm GC exposes strong references but no weak-reference or
ephemeron operations. Both capability facts flow through runtime-policy
reporting and typed engine-setup error context. The unavailable variant is a
boundary truth, not a weak implementation for the current linear records.

This is an explicit runtime-capability blocker, not a reason to add a tracing
collector over Lila's current linear-memory object graph. Before T05 can close,
the lower bound must expose a cycle-capable Wasm-GC collector and the engine
must select and require it. Until then, GC emission is architectural work, not
an executable-GC completion claim.

Primary references for the pinned facts are Wasmtime 38.0.4's `Collector`
documentation and DRC implementation, plus the WebAssembly GC proposal:

- <https://github.com/bytecodealliance/wasmtime/blob/v38.0.4/crates/wasmtime/src/config.rs>
- <https://github.com/bytecodealliance/wasmtime/blob/v38.0.4/crates/wasmtime/src/runtime/vm/gc/enabled/drc.rs>
- <https://github.com/WebAssembly/gc/blob/main/proposals/gc/Overview.md>

## Non-negotiable invariants

1. Every identity-bearing ECMAScript object is a Wasm-GC reference. No object,
   environment, property table or closure is represented sometimes by a GC
   reference and sometimes by a linear-memory integer handle.
2. A GC reference is never cast to, packed into, or recovered from an integer.
   The Wasm type checker and Rust schema must retain the distinction through
   locals, fields, globals, calls, returns, exceptions and host transitions.
3. Linear memory contains bytes, limbs and transient host-I/O buffers only. A
   linear address is not an object identity and cannot participate in the
   JavaScript reference graph.
4. Every dynamic linear span has exactly one statically named GC owner and no
   owning aliases. Interior views borrow an owner plus a checked range; they do
   not own the backing allocation.
5. Strong GC fields, weak edges and external resources are different domains.
   A missing weak-reference facility cannot be approximated with a strong
   `GcRef`, and a host resource handle cannot become a second object model.
6. All roots are real Wasm references in typed locals, fields, globals, tables,
   exception payloads or host rooting scopes. A parallel integer root registry
   is not part of the target architecture.
7. A module that requires this ABI fails at the runtime boundary when Wasm GC
   or the required collector capability is absent. There is no non-GC backend.

These invariants imply an atomic semantic cutover. Gradually teaching a few
builtins to return GC references while the rest consume integer heap handles
would create two object models and require a bridge expressly forbidden by
invariants 1 and 2.

## Value representation

SSA computation uses typed value parts:

- a closed JavaScript value tag;
- scalar bits for `undefined`, `null`, Boolean, Number and small internal
  sentinels; and
- a nullable, typed GC-reference slot for String, BigInt, Symbol, Object and
  internal records.

The active slot is determined by the closed tag. Rust builders must construct
and consume the whole value, so a reference-bearing tag without a reference is
not expressible at an emitter call site. The reference slot remains a Wasm
reference in function signatures and locals; it is not squeezed into today's
`i64` payload ABI.

Stored values use a central GC layout carrying the same tag/scalar/reference
parts. This may box a value when it crosses from SSA into a property,
environment or job record, but it keeps primitive fast paths allocation-free
inside an expression. Layout-specific records may use narrower typed fields
when the ECMAScript specification fixes the field's domain.

The schema vocabulary in `crates/lila-aot-wasm/src/gc_types.rs` starts these
compile-time distinctions:

- `GcTypeIndex<T>` prevents indices for different layouts from being swapped;
- `GcField<Owner, Value, Mutability, Nullability>` binds every field ordinal to
  its owner and complete storage contract;
- nullable scalar fields do not type-check;
- `GcRef<T>` is a zero-sized strong-reference schema marker with no integer
  representation; and
- `GcRootGlobal<T>` names only a mutable, nullable Wasm global carrying a
  strong reference to `T`; it cannot name a scalar global or contain a linear
  address; and
- `LinearAddr<Owner>` and validated `LinearSpan<Owner>` cannot be substituted
  for GC references or for another layout's side storage.

The capability-anchor subset of this vocabulary is now wired through the
central type-section registry, not an individual builtin. The remaining
semantic layouts stay schema-only until the atomic object-model cutover.

The schema module is also the sole raw Wasm-GC encoder boundary. Type-index and
field-ordinal construction/extraction, plus typed GC-root construction and
extraction, stay private there. Module assembly first emits every fixed and
dynamic scalar global into one open builder. One consume-once finalization owns
the type registry and that builder: it derives the root slot from the encoded
section's actual length, appends the typed root, and returns an opaque,
non-cloneable package containing the finalized sections and their private
`RuntimeModuleSchema`. There is no planned raw root index or copyable schema for
another caller to recompute, supply or pair with a different section. A
dedicated main-compilation transition consumes that exact package, compiles main
internally against its private lifecycle, and retains the main body in the
package's code-section builder. After the remaining bodies are supplied, the
sealed compiled package has one consuming module-assembly operation; it emits
its type, global and code sections around the other owned core sections in Wasm
section order. No independent main/type/global/code append surface exists, so
two finalized packages cannot be split and recombined through normal assembly.
Function emission cannot extract interchangeable `u32` indices or construct
`struct.new`/`struct.get` instructions itself. The typed accessor boundary pairs
a field with its owner and, for reference fields, with its target type through
Rust generics before the final `wasm_encoder` call.

## Runtime GC anchor

`RuntimeGcAnchor` and `RuntimeGcAnchorHolder` are the first executable schema
types. Neither is a JavaScript object. The anchor's single field is an
immutable, non-nullable `i32` ABI version; the holder's single field is an
immutable, non-null `GcRef<RuntimeGcAnchor>`. The emitter now:

1. appends the anchor and then the holder to the module's type section, retaining
   both typed indices;
2. encodes the holder field through a typed reference-field builder that
   consumes `GcTypeIndex<RuntimeGcAnchor>` and derives its nullability and
   mutability from the `GcField` type;
3. appends one unexported, mutable, nullable `GcRootGlobal<RuntimeGcAnchor>`
   after every pre-existing fixed and dynamic global, so no established global
   index moves;
4. constructs the anchor and holder before any other main instruction,
   traverses the holder field, and stores the recovered reference in that
   typed root;
5. keeps the root live across main initialization, calls, source execution and
   the final job checkpoint; and
6. on every real main return, loads and non-null-checks the root, reads the
   anchor's ABI-version field, traps if it differs from
   `RuntimeGcAnchorSchema::ABI_VERSION`, then clears the root to null.

That sequence makes the Wasm validator and runtime exercise a concrete strong
GC edge, a real Wasm global root and struct construction/field traversal,
without introducing a live semantic object or changing the current heap.
`ModuleTypeRegistry` owns the section and assigns both indices in dependency
order. Consuming the type registry and open global-section builder is the only
way to obtain the finalized runtime package: the same operation binds the root
to the section's actual next index, appends it, and seals the section against
further globals. The private schema is neither `Copy` nor exposed. The only main
compiler input is a closed plan constructed by the emitter; the main-compilation
transition consumes it internally against the package's exact globals and
immediately stores the resulting main in package-owned code. The resulting
compiled package is the only owner of those type, global and code sections, and
a single consume-once append operation emits all three. That package owns raw
declaration, access, lifecycle and assembly as one opaque operation surface, so
a holder field cannot be paired with the anchor type, a type index cannot be
used as a global index, and neither a separately predicted root index/schema nor
a main compiled against another package can drift from the completed global
section.

The holder becomes unreachable as soon as its edge is transferred to the
global. The anchor then remains live only through the global until the shared
main exit verifies and clears it. This is an executable root-lifecycle witness,
not a JavaScript value. It proves neither reclamation nor cyclic collection,
does not establish roots for semantic values in calls, exceptions, suspended
frames or pending jobs, and adds no weak edge. A Wasm trap before the shared
main exit may retain the witness until Store teardown; Store teardown remains
the owner of that exceptional cleanup.

The matching engine seam uses the closed
`WasmGcCapability::DeferredReferenceCountingWithoutCycleCollection` value for
collector configuration, trace reporting and typed engine-setup failure
context. With Wasmtime 38.0.4 the only available collecting choice is DRC, so
this makes today's limitation explicit but cannot satisfy cyclic collection.
T05 closure requires raising the pinned lower bound to a cycle-capable
implementation.

## GC layout families

One central registry will declare all struct/array types and their recursion
groups. The registry, not builtin-local offsets, assigns type indices and field
ordinals. The required families are:

- the stored JavaScript value record;
- ordinary objects, property descriptors and indexed/property tables;
- functions, bound functions, executable code identities and closures;
- declarative/object/module environments and mutable binding cells;
- strings, BigInts and Symbols;
- Arrays, ArrayBuffers, views and typed arrays;
- iterator, generator, async activation, Promise and job records;
- realms and intrinsic tables; and
- host/external resource handles.

Prototype, environment, closure, property-value, pending-job and completion
links are strong typed references. The registry must generate both
`wasm_encoder` field declarations and the typed accessors used by emitters; a
separate descriptive table is not sufficient. Adding a layout without its
field schema, or a field without an exhaustive encoder mapping, must fail
`cargo check`.

## Linear side storage

The default representation for dynamically owned semantic data is a Wasm-GC
array: packed code units, BigInt limbs, property entries and unshared buffer
bytes can then die with their owner without a finalizer protocol. Immutable
compiler data may stay in linear memory for the lifetime of the instance, and
host calls may use checked, call-scoped linear buffers.

Dynamic linear side storage is allowed only after its reclamation mechanism is
real. Wasm GC currently provides no destructor callback for a collected struct,
so merely storing `LinearAddr<Owner>` in a GC object would leak. Until a
cycle-capable runtime also supplies a suitable resource/finalization facility,
the cutover must not emit dynamically owned `LinearSpan<Owner>` values.
`LinearSpan` exists now to make ownership and memory32 bounds explicit for the
remaining static/transient uses, not to claim lifetime integration.

If a future host-owned resource is necessary (for example a shared backing
store), the Wasm-GC object remains the sole JavaScript identity. Its field holds
a typed external-resource reference. The host resource owns bytes only, has no
properties/prototype/environment, and is released by runtime-supported
resource lifetime—not by a second JavaScript heap or an integer handle table.

Host imports may borrow linear memory only for the dynamic extent of the call.
Re-entrancy must establish a host rooting scope for every reference passed out
of Wasm. No host pointer or unrooted Wasmtime reference survives a call.

## Weak reachability

The current WebAssembly GC surface has no weak-reference or ephemeron field.
Consequently:

- `GcRef<T>` always denotes a strong edge;
- WeakMap/WeakSet keys, WeakRef targets and FinalizationRegistry targets cannot
  use it without changing observable reachability;
- DRC's inability to reclaim cycles independently blocks ordinary cyclic
  garbage; and
- the current linear weak-edge tables and records are inventory only.

Correct weak semantics require a runtime capability that can observe the
Wasm-GC graph and provide weak/ephemeron processing, or a Wasm proposal/runtime
extension with equivalent semantics. A host sidecar that merely stores object
IDs cannot learn that a Wasm reference is unreachable, and rooting references
in that sidecar makes them strong. Neither is acceptable.

The eventual facility must support ephemeron fixpoint processing, clearing weak
targets after strong tracing, holding finalizer holdings strongly, treating
unregister tokens according to their specified reachability, and queueing
cleanup jobs without promising when collection occurs. Test262's `gc()` hook
must request a real full collection cycle; it may not clear tables directly or
schedule finalizers deterministically as a substitute.

The engine boundary now encodes that capability as explicitly unavailable.
Weak builtins remain blocked until a real facility is selected and replaces
that variant. This is a truthful unsupported capability, not a silent skip and
not permission to preserve the current strong behavior.

The passive linear-heap weak-edge inventory keeps its retention vocabulary
closed even while that facility is unavailable. `HeapWeakEdgeKind` exhaustively
derives one of three meanings: an edge that does not retain its target, an
ephemeron value retained only when its key is reachable through the fixpoint,
or finalizer holdings retained strongly until cleanup. A slot cannot separately
attach a Boolean strength claim that contradicts its kind. This makes the
future collector obligation precise; it does not make the inventory executable
or give the current linear records weak semantics.

## Atomic cutover plan

Each phase has an invariant gate. Phases 0–3 add no second semantic object path;
phase 4 is the single product-model switch.

### Phase 0 — schema and measured baseline (landed)

- Check in this architecture and the typed schema vocabulary.
- Record the current collector selection, cycle blocker and weak-edge blocker.
- Keep the current emitter and engine behavior unchanged until phase 1.

Gate at the phase-0 boundary: source checks showed no `GcRef` integer payload
and no GC instructions emitted by the new module. Phase 1 has now superseded
the latter condition with the capability anchor below.

### Phase 1 — explicit runtime capability anchor (implementation landed)

- Centralize engine GC configuration and explicitly select the supported
  collector; reject missing Wasm-GC/reference capabilities (landed).
- Encode weak-reference and ephemeron availability independently of the
  collector as `WasmWeakReachabilityCapability::Unavailable` (landed; a real
  facility remains phase 6 work).
- Remove `gc-null` from the product feature graph (landed).
- Emit and traverse the `RuntimeGcAnchorHolder -> RuntimeGcAnchor` strong edge
  through the central type registry, including the anchor ABI assertion
  (landed; runtime-boundary verification remains).
- Transfer that edge into a typed nullable Wasm global before main can call or
  allocate, retain it through the final job checkpoint, and verify/clear it on
  every shared main exit without moving existing global indices (landed;
  runtime-boundary verification remains).
- Keep raw GC type-index and field-ordinal construction/extraction, typed-root
  construction/extraction and struct instructions inside the schema module;
  consuming the type/global builders derives and appends the sole root from the
  actual encoded section length, and main borrows only that exact non-cloneable
  package's opaque lifecycle operations (landed; compile and focused runtime
  verification remain).

Gate: a module containing the anchor/holder/root probe validates and executes
on the pinned lower bound, and fails clearly when GC is disabled. This proves
typed strong-edge and global-root feature wiring, not reclamation, cycle
collection, semantic call/frame/job roots, weak semantics or JavaScript heap
migration.

### Phase 2 — complete generated layout registry

- Declare every layout family, recursion group, field and array element once.
- Generate encoder declarations and typed field accessors from that registry.
- Choose GC arrays for all dynamic data lacking a real side-storage release
  mechanism.

Gate: every planned semantic record has an exhaustive typed schema; no product
emitter consumes it yet, and layout additions cannot omit an encoder mapping.

### Phase 3 — closed value and host ABIs

- Define the scalar/reference value parts and stored-value record.
- Define typed function, completion, exception, global/table and host-call
  signatures.
- Make rooting scopes and external resources explicit at the Rust boundary.

Gate: all producers and consumers have a compile-time migration mapping. There
is no conversion from a linear object address to a GC reference.

### Phase 4 — atomic semantic switch

In one coherent batch, change every JavaScript-value producer and consumer:
script functions, runtime helpers, builtins, objects, environments, realms,
jobs, exceptions, host imports/exports and result decoding. At the same time,
delete linear allocation/layout code for identity-bearing semantic records and
remove their integer root/weak metadata.

Gate: the emitted module contains no semantic-object allocation through
`heap_alloc`; every reference-bearing value is carried as a Wasm reference;
there is no bridge or fallback representation. Static/transient byte allocation
may remain under the side-storage rules above.

### Phase 5 — lifetime and stress closure

- Require a cycle-capable collector at the engine boundary.
- Exercise roots across calls, exceptions, suspended frames, generators,
  promises, realms and host re-entry.
- Stress allocation and release of cyclic graphs under a fixed low limit.
- Verify memory32 boundary behavior for every retained linear region.

Gate: cyclic stress stabilizes rather than growing until Store teardown, and
all T05 strong-root acceptance cases pass.

### Phase 6 — weak capability and finalization

- Replace the typed unavailable capability with the selected runtime
  weak/ephemeron facility.
- Implement weak collections, WeakRef and FinalizationRegistry against it.
- Wire `gc()` to a real collection request and cleanup jobs to the job queue.

Gate: focused real Test262 weak suites pass without direct table clearing,
strong substitutes, deterministic-finalization promises or expected failures.

## Completion boundary

T05 is complete only when phases 1–6 are implemented and verified. The schema
and anchor are foundations. Enabling `wasm_gc`, validating `struct.new`, or
passing acyclic allocation tests alone does not satisfy executable GC, cyclic
collection, side-storage reclamation or weak reachability.
