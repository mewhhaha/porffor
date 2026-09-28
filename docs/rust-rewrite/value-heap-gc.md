# Value, heap and garbage-collection architecture

This document is the source of truth for T05. It describes the object model
Lila is moving to; it does not describe the current linear heap as complete.
The migration must preserve one product object model at every commit.

## Ground truth

The current Wasm-AOT path represents a JavaScript value as integer payload/tag
parts. Identity-bearing values are integer addresses into a bump-allocated
linear-memory heap. `heap.rs` contains extensive layout, root, weak-edge and
collector tables, but those tables do not drive an executable collector. The
weak-reference records hold ordinary strong integer addresses. They are useful
inventory, not proof of JavaScript heap reclamation or weak semantics. The
`gc()` host hook now invokes Wasmtime's native collector; its scope is described
below.

The engine is pinned to Wasmtime 49.0.1, built with Rust 1.97.1. Every
product engine uses one `WasmtimeRuntimePolicy`: reference types, typed
function references, Wasm GC and exception handling are required explicitly.
The policy selects `Collector::Copying`, and the product feature graph includes
`gc-copying` rather than `gc-drc` or `gc-null`. This collector can reclaim
unreachable native Wasm GC cycles while the Store remains alive.

This runtime upgrade does not migrate Lila's integer-addressed semantic values.
Semantic-object reclamation and JavaScript weak reachability remain unfinished.
The policy independently records
`WasmWeakReachabilityCapability::NativeCopyingExtension`: the vendored runtime
now processes native weak references, ephemerons and finalization registrations.
This is a Lila extension to Wasmtime, not a standard Wasm-GC capability. Its
API accepts native references only; it cannot collect the compiler's remaining
integer-addressed JavaScript objects. The semantic-reference migration and
language integration remain required, without adding a parallel tracing heap.

The focused runtime test
`wasmtime_policy::tests::product_collector_reclaims_cycles_and_preserves_live_roots`
creates self-referential native GC structs containing finalization witnesses.
It checks that an unreachable cycle is collected while a global-rooted cycle
survives, then that clearing the global makes the second cycle collectible.
This tests the runtime collector, not JavaScript heap reclamation.

### Explicit host collection

Test262's `INTERPRETING.md` defines `$262.gc()` as a wrapper around the host's
collection invocation mechanism. Wasmtime provides `StoreContextMut::gc(None)`.
The AOT host builtin invokes it through the optional `lila_host.gc: () -> ()`
import and returns JavaScript `undefined` after collection succeeds. Errors from
the runtime propagate. The hook does not clear weak tables, emulate finalization,
or substitute a no-op for collection.

This mechanism is useful before the complete semantic-object migration: native
argument vectors, deferred invocation frames and other Wasm references already
need collection and correct stack roots. A native runtime regression calls the
same registered host hook with a self-referential struct live in a Wasm local,
checks the reference after collection, and then verifies that the unreachable
cycle is reclaimed before Store teardown. JavaScript integration regressions
exercise calls, deferred arguments, generator continuations, Promise jobs,
cross-Realm calls and pending exceptions across the hook.

The cross-Realm regression also exposed a call-ABI defect in the created
`$262` record. Its `gc`, `createRealm` and `detachArrayBuffer` methods were
initialized with self-backed function handles despite their lexical-environment
ABI. Their materializer now consumes the same closed environment classification
as ordinary calls and stack guards, selecting the defining Realm's global
environment for those methods.

The thirteen remaining executions in the required-fixes inventory were blocked
at the absent host invocation. Their subsequent results must be reported
separately from semantic-heap collection: none establishes reclamation of a
JavaScript object, weak-target clearing or ephemeron processing. The complete
native-reference migration and JavaScript weak-builtin integration below remain required.

Primary references are the pinned runtime implementation and Wasm GC proposal:

- <https://github.com/bytecodealliance/wasmtime/blob/v49.0.1/crates/wasmtime/src/config.rs>
- <https://github.com/bytecodealliance/wasmtime/blob/v49.0.1/crates/wasmtime/src/runtime/vm/gc/enabled/copying.rs>
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

These invariants require a coherent cutover for JavaScript object identities.
An internal, unobservable argument carrier can migrate earlier: native GC arrays
can transport the existing tagged scalar values without introducing another
JavaScript object representation. Carriers must remain typed Wasm references;
observable argument arrays and suspended linear records need explicit value
snapshots, never integer-encoded GC references. This reduces internal allocation
without claiming semantic-object collection.

Gradually teaching a few
builtins to return GC references while the rest consume integer heap handles
would create two object models and require a bridge expressly forbidden by
invariants 1 and 2.

## Call-boundary preparation

The current emitter passes result destinations and pre-evaluated arguments as
`TaggedLocals` through `store_call_results`, `store_call_results_to`, and
`emit_pre_evaluated_arg_vector`. Callers must supply the complete current value
pair at these boundaries. This removes tuple-shaped API variations before adding
reference-bearing values; it does not change the emitted ABI or reclaim memory.
The existing constructor still accepts two local indices, so this step alone
does not make payload/tag transposition impossible.

A thirteen-fixture emitted-Wasm comparison passed byte-for-byte for this
refactor (`target/watched/abi-final-v21.log`, 2026-09-27). All 460 backend
unit tests, 54 selected engine integration tests, and ten engine stack tests
also pass. The artifact comparison covers ordinary and prepared calls, Proxy
apply/construct, async
and generator lifecycles, Array callbacks, BigInt coercion, JSON replacers,
Temporal conversions, RegExp callbacks, and Promise reactions. The refactor is
not counted as a repair to any of the remaining GC/heap Test262 failures.

### Shared call layout (verified)

`abi/call.rs` declares the ordinary and prepared-script parameter slots and the
completion result slots once. Module signatures, parameter counts and indices,
standard completion stores/returns, and stack-guard result locals consume that
contract. Completion-local binding matches every result slot exhaustively, so
adding a slot requires a corresponding local binding at compile time.

This batch preserves the existing integer ABI. Helper operands that only share
its physical Wasm type remain distinct from JavaScript parameter roles. Ordinary
call-operand emission, local allocation, semantic layouts and host rooting still
need the native-reference cutover; this contract alone does not add GC roots or
reclaim JavaScript allocations. Verification includes existing runtime checks
and comparison against the thirteen recorded Wasm artifacts from the prior
frozen compiler. The v27 checkpoint passes 461 backend tests, 66 focused engine
integration tests, and all 785 engine library tests with no failures or ignored
tests. All thirteen artifacts remain byte-identical; the required-fixes receipt
records the frozen compiler hash and log.

### Native argument carriers (verified)

The next batch replaces the hidden Array allocated for each ordinary call with
an internal Wasm-GC array of tagged scalar cells. The carrier has no JavaScript
identity or `ValueKind`: JavaScript objects still use the existing representation.
Argument evaluation captures values immediately into typed reference locals,
which remain roots across nested calls and branches. A reference temporary can
never be reused as a scalar temporary, or stored into linear memory.

Proxy traps receive ordinary observable argument arrays. Async and generator
activation records retain explicit value snapshots in canonical Arrays and
reconstruct a GC carrier on resume. Dynamic spread and array-like application
must preserve the existing iterator/getter order before normalizing their values.
These boundaries copy values, never convert a GC reference into an integer.

The frozen v27 compiler passes the six original behavioral regression fixtures.
Four million fixed two-argument arrow calls trap in its linear argument-array
allocator under the unchanged 1 GiB limit. The native-carrier version passes
that regression and all nine new integration tests. The combined carrier and
arguments-elision batch passes 466 backend tests and 67 existing integration
tests. Its fresh original-inventory replay passes both strict TypedArray stress
cases; both sloppy variants still exhaust the linear heap.

The initial stress fixture used an ordinary function and exposed a separate
allocation: ordinary functions eagerly construct their JavaScript `arguments`
object and, for mapped parameters, their environment on every call. Its
[reproducer](../../crates/lila-engine/tests/fixtures/gc-argv/ordinary-calls-still-allocate-arguments.js)
is retained. The arrow fixture isolates carrier lifetime; it does not establish
bounded memory for ordinary functions. Sloppy `function.arguments` and property
descriptor reads can observe these objects, so eliminating them needs a proven
use analysis or a lazy materialization protocol that preserves those observations.
This is an internal allocation migration; the semantic `gc()` hook, object and
closure lifetimes, and weak reachability remain separate implementation work.

### Unobserved arguments objects (verified)

A second allocation batch uses source-analysis evidence to avoid constructing a
JavaScript `arguments` object that cannot be observed. The proof rejects direct
references, raw or aliased `arguments` bindings, captured environment storage,
and eval-visible environments. Synthetic IR starts conservatively. The backend
also requires the same legacy-reflection barrier classification used at function
entry; sloppy ordinary functions retain eager `.arguments` materialization.

Elision preserves the implicit binding and its mapped/unmapped classification,
including mapped-slot validation. It initializes the unused binding's storage to
`undefined`, so declaration initializers still have a destination. This removes
an allocation only; it does not collect objects or change their representation.
The frozen GC-carrier compiler still traps on four million strict ordinary calls.
Four semantic control fixtures pass before elision. The current implementation
passes all five expanded arguments-allocation tests and the nine native-carrier
tests, including both four-million-call loops. The fresh original-inventory
replay passes 16/31, adding strict large sorting to the previous 15 passes. All
1,179 IR tests, 466 backend tests, 50 structure checks and 67 existing runtime
integration tests pass. The complete engine library rerun also passes 785/785,
with no failed or ignored tests (`arguments-binding-broad-v43.log`).

The expanded controls exposed existing binding bugs: an uncaptured
`var arguments` could start as undefined, and an arrow could capture the wrong
binding when parameter expressions create a separate body environment. Source
reference resolution now preserves the nearest binding for reads and writes;
shared bindings also share conservative type facts. Runtime checks cover
initializers, closures, named parameters and functions, reassigned values, and
the supplied-argument path that skips a default initializer.

### Deferred legacy arguments (verified)

For a sloppy ordinary function with simple parameters and a source proof that
its implicit `arguments` binding is unobserved, the invocation now retains a
private native-GC frame instead of constructing the observable object at entry.
The frame roots the call vector and validated mapping and links to its parent
frame. It retains the current scalar environment and function context without
encoding a GC reference into linear memory.

Property reads, own descriptors and descriptor validation materialize the
canonical mapped object at the first observation of the function's legacy
`arguments` data property. The materializer invokes no source code. Recursive
invocations resolve the innermost matching function identity; normal and abrupt
returns restore the previous properties and native frame root. Source-observed
bindings and non-simple parameters retain their established construction path.

This reduces allocation but does not collect semantic objects or parameter
environments. The two original sloppy TypedArray failures now pass in the frozen-compiler
replay, bringing the original inventory to 18/31. All 467 backend tests, 50
structure checks, 86 focused runtime tests and 785 engine library tests pass,
including four million sloppy calls and cross-Realm materialization. The semantic
GC hook was still absent at that checkpoint, leaving thirteen required failures.
The explicit host collection section above records the subsequent hook repair;
semantic-object migration and weak integration remain unfinished.

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
`WasmGcCapability::CopyingWithCycleCollection` value for collector configuration,
trace reporting and typed engine-setup failure context. Runtime collection and
the semantic-object cutover have separate acceptance criteria; selecting the
collector does not satisfy the latter.

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

The vendored facility now implements ephemeron fixed-point processing, weak
target clearing after strong tracing, strong holdings for live finalization
registries, weak unregister tokens, and an explicit cleanup queue. No cleanup
callback runs inside collection. Test262's `gc()` hook invokes the same copying
collector; it never directly clears a weak table or simulates finalization.

### Native copying-collector extension (implemented)

The per-Store `GcWeakStore` contains actual native GC references to owners,
targets, keys, values and tokens. Weak owners and endpoints are excluded from
ordinary root tracing. Runtime address keys are lookup caches only, rebuilt
after movement; neither addresses nor handles are exposed to the compiler.
The public API takes `AnyRef` values and returns scoped `Rooted<AnyRef>` values.
It validates Store ownership and rooting, rejects immediate i31 owners, targets,
keys and tokens, and rejects collectors that lack the extension.

The copying collector drains strong roots and fields, then traces conditional
edges. It retains an ephemeron value only if both owner and key are already
live; it retains finalization holdings only for a live registry. New copied
objects send collection back through the ordinary worklist and conditional
scan. Weak clearing occurs only after this fixed point, so a value-to-key
cycle cannot make itself live. Dead owners and entries are pruned, and every
surviving endpoint is updated before from-space is reused.

`GcWeakRef::initialize` and successful `get` calls add the target to the native
kept-object set. `GcWeakHeap::clear_kept_objects` releases that set at the host's
job boundary, not at `gc()`. Finalization cells preserve holdings and weak
tokens; unreachable targets of live registries enqueue cleanup records.
Pending cleanups root their owner and holding across subsequent collections.
`GcFinalizationRegistry::unregister` removes every matching active and queued
cell. `GcWeakHeap::take_cleanup` roots the result before relinquishing the
queue's references. A running cleanup drains that registry one holding at a time
with `GcFinalizationRegistry::take_holding`, preserving other registries' queues
and allowing callbacks to unregister cells that have not been delivered yet.
The host remains responsible for scheduling callbacks after the current job and
handling their exceptions.

In Wasmtime 49.0.1, the narrow collection seam is `GcStoreTraceState` and
`GcStore::gc` in `vendor/wasmtime-49.0.1/src/runtime/vm/gc.rs`, plus the
`CopyingCollection` worklist and forwarding phases in
`vendor/wasmtime-49.0.1/src/runtime/vm/gc/enabled/copying.rs`. Ordinary GC
fields remain unconditionally strong through `GcStructLayoutField`,
`TraceInfo`, and the copying collector's inline reference bitmap. The registry
does not change Wasm field semantics or type canonicalization. The added
`ProcessWeakEdges` phase also participates in cancellation completion: dropping
an in-progress collection must finish weak processing before the mutator runs.
The product explicitly selects the copying collector and requires extension ABI
version 1 at compile time. Other collectors reject weak registration.

JavaScript integration still requires phase 4's atomic reference migration. Today
the AOT ABI passes values as payload/tag integers, identity-bearing values are
addresses in the linear heap, and the WeakRef/WeakMap/FinalizationRegistry
records in `heap.rs` are passive layout/edge metadata. The runtime GC anchor is
not a JavaScript object owner. Do not register weak edges against linear
addresses. Every registered owner, key, value and root must already be a native
Wasm reference. The implemented collector API does not make current JavaScript
WeakRef operations weak.

The JavaScript layer must connect constructor/deref operations and job boundaries
to the kept-object API, retain eligibility checks for objects and non-registered
symbols, use native finalization cells and schedule cleanup callbacks from its
job queue. Diagnostics explicitly label the available capabilities `native-*`;
they do not claim completed JavaScript weak semantics.

The native extension passes 17 acceptance tests on 2026-09-27
(`target/watched/native-weak-gc-v71.log`). They check live Wasm stack roots,
explicit and allocation-triggered collection, weak cycles, repeated movement,
dead owners, ephemeron chains and replacement, finalization holdings and queued
roots, weak tokens, unregistering during cleanup, and invalid API inputs. Drop witnesses assert
reclamation while the Store remains alive. Refresh with:

```sh
cargo test -p lila-engine --release --locked -j3 --lib gc_weak_tests -- --test-threads=3
```

Final verification passes all 803 engine-library tests, with no failures or
ignored cases (`native-weak-gc-v71.log`, 778.41 seconds for the full library).
The preceding `native-weak-gc-v70.log` passes 29 focused AOT integration tests
and three runtime-policy structure checks. Refresh those checks with:

```sh
cargo test -p lila-engine --release --locked -j3 --test engine_error_wasmtime_policy_authority_structure --test aot_host_gc --test aot_gc_call_arguments --test aot_deferred_legacy_arguments --test aot_recursion_guard --test aot_typed_array_sources_and_realms -- --test-threads=3
cargo test -p lila-engine --release --locked -j3 --lib -- --test-threads=3
```

These runs verify the native extension and regressions in existing behavior.
They do not verify JavaScript object reclamation or JavaScript weak-builtin
integration. Neither the original Test262 inventory nor the full pinned suite
was replayed for this runtime-extension checkpoint.

The passive linear-heap weak-edge inventory keeps its retention vocabulary
closed while JavaScript integration is pending. `HeapWeakEdgeKind` exhaustively
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
  collector (landed; initially `Unavailable`, now `NativeCopyingExtension`
  following the native runtime tests above).
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
  weak/ephemeron facility (native extension implemented and independently tested).
- Implement weak collections, WeakRef and FinalizationRegistry against it.
- Connect weak processing and cleanup jobs to the job queue. The existing
  `gc()` request must then include this integrated semantic graph.

Gate: focused real Test262 weak suites pass without direct table clearing,
strong substitutes, deterministic-finalization promises or expected failures.

## Completion boundary

T05 is complete only when phases 1–6 are implemented and verified. The schema
and anchor are foundations. Enabling `wasm_gc`, validating `struct.new`, or
passing acyclic allocation tests alone does not satisfy executable GC, cyclic
collection, side-storage reclamation or weak reachability.
