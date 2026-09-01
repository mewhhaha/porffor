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

The passive linear-heap weak-edge inventory now uses seven closed
`HeapWeakEdge` identities and derives retention from one closed
`HeapWeakEdgeKind` domain. One private exhaustive projection owns every
identity's record, slot name and kind, so an arbitrary row cannot pair weak-map
or finalization strings with an unrelated kind. Weak keys, targets and
unregister tokens do not retain their targets; ephemeron values are
conditionally retained when their keys are reachable through the fixpoint;
and finalizer holdings remain strong until cleanup. This is an inventory
invariant only: it does not execute tracing, clear a weak target, queue cleanup
or make the current records weak in practice.

The host decoder for the linear heap's BigInt sign word now parses `-1`, `0`
and `1` once into the private `HeapBigIntSign` domain. One exhaustive projection
derives both the `num_bigint::Sign` and whether the magnitude must be zero, so
those two meanings cannot drift while the raw word remains available only for
boundary diagnostics. Invalid words and inconsistent sign/magnitude records
retain their evidence-bearing errors. On 2026-08-25, the bounded structure
target passes `3/3`, the focused decoder unit tests pass `7/7`, the structured
heap-BigInt observation witness passes `1/1`, and the normal/throw legacy
rendering witnesses pass `2/2`. The shared workspace compile and every
repository policy gate pass. All 648 Wasm-golden artifacts remain present; the
host-only decoder invariant adds no emitted delta beyond the shared Iterator
realm repair recorded under T06/T15. No broader Test262 run was performed for
this decoder-only invariant.

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
- `HeapWeakEdge` closes the seven passive weak-edge identities and exhaustively
  derives each record, slot name and `HeapWeakEdgeKind`; the kind then
  exhaustively derives non-retaining, ephemeron-conditional or
  strong-until-cleanup retention.

The private `FunctionModuleState::{Main, Internal}` construction authority now
derives no incidental capabilities. Exactly one main and four internal
constructors move it into the function builder; return ABI, main-frame cache
bindings and both GC-root lifecycle gates borrow it through four exhaustive
matches. The focused
[contract](../docs/rust-rewrite/contracts/function-module-state.md) and
recursive structure guard record the five producers and exact projections.
This source-equivalent ownership closure is expected to leave emitted Wasm
byte-identical. The structure target passes `3/3`, and both exact GC-root unit
witnesses pass `1/1`; it does not change the GC schema, collector or root
lifecycle. Independent dry review is clean, and the shared format, `cargo xc`,
diff, module-boundary and task-plan checkpoint is green with the workspace's
existing warnings.

The 50 registered named iterator slots now select one closed
`HeapNamedSlotStorage::{StrongReference, Scalar}` class instead of storing
independently writable strength and tracing Booleans. Two exhaustive
projections derive both meanings from the same class, so metadata cannot scan a
scalar target or omit a strong target from tracing. Their six layout families
now use the capability-free `HeapNamedSlotFamily` domain instead of a registry
of arbitrary slice references. One exhaustive projection owns every exact
family-to-slice mapping and the typed registry preserves Array, String, RegExp,
Helper, Concat, Zip order. The focused
[contract](../docs/rust-rewrite/contracts/heap-named-slot-storage.md) and
structure regression pin both domains, all projections, exact family registry
and the complete 30-strong/20-scalar producer census. This remains passive heap
inventory: it does not emit tracing, implement collection or change an object
layout. The strengthened structure guard passes `4/4`, and the exact family
registry, Zip and Concat witnesses each pass `1/1` with only the workspace's
existing warnings. Targeted formatting and diff checks pass, and the shared
`cargo xc` checkpoint is green. Golden and conformance execution do not apply
to this passive metadata-only closure.

The three retained linear-memory side-storage spans now use the closed
`LinearSideStorage::{ArrayBufferBackingStore, StringCodeUnits, BigIntLimbs}`
identity domain instead of rows that independently combine record and
length-source strings with an element. One private exhaustive metadata
projection owns all three exact mappings. The projected
`LinearSideStorageElement::{Byte, Utf16CodeUnit, BigIntLimb}` domain remains the
sole width/reference authority: exhaustive projections derive the respective
`1`, `2` and `8` byte widths and classify all three as non-reference storage.
The focused
[contract](../docs/rust-rewrite/contracts/heap-side-storage-element.md) and
recursive structure regression pin both domains, the sole metadata projection,
both element projections, exact mappings and registry order. This is passive
side-storage inventory only: it does not change allocation, emitted Wasm,
semantic object storage, collection or weak reachability. The strengthened
side-storage and adjusted value-encoding guards each pass `4/4`, and the exact
heap owner witness passes `1/1` with only the workspace's existing warnings.
Targeted formatting and diff checks pass, and the shared `cargo xc` checkpoint
is green. Golden and conformance execution do not apply to this passive
metadata-only closure.

The two Temporal.Instant layout rows now use the capability-free
`TemporalInstantHeapSlot::{EpochNanosecondsTag, EpochNanosecondsPayload}`
identity domain instead of free-form records that independently select names,
offsets, widths and pointer Booleans. One private exhaustive metadata
projection owns both exact rows and the typed registry preserves tag-then-
payload order. The tag remains a scalar 8-byte word and the payload remains a
strong-reference 8-byte word at their existing offsets. The focused
[contract](../docs/rust-rewrite/contracts/temporal-instant-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form producer. This remains passive layout inventory: it does not change
allocation, emitted Wasm, Temporal semantics, collection or weak
reachability. The structure target passes `4/4`, the exact identity owner
witness passes `1/1`, and the adjusted collision/pointer registry witnesses
pass `2/2` with only the workspace's existing warnings. Targeted formatting
and diff checks pass, and the shared `cargo xc` checkpoint is green. Golden and
conformance execution do not apply to this passive metadata-only closure.

The two WeakRef record rows now use the capability-free
`WeakRefHeapSlot::{TargetTag, TargetPayload}` identity domain instead of
free-form records that independently select names, offsets, widths and pointer
Booleans. One private exhaustive metadata projection owns both exact rows and
the typed registry preserves tag-then-payload order. Both words remain
non-pointers, while the independent `HeapWeakEdge::WeakRefTarget` identity and
`HeapWeakEdgeKind::WeakTarget` remain the sole semantic non-retention
authority. The focused
[contract](../docs/rust-rewrite/contracts/weak-ref-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change allocation, emitted Wasm, WeakRef behavior, collection or weak
reachability execution. The structure target passes `4/4`, the exact
identity/non-retention owner witnesses pass `2/2`, and the adjusted
collision/pointer registry witnesses pass `2/2` with only the workspace's
existing warnings. Targeted formatting and diff checks pass, and the shared
`cargo xc` checkpoint is green. Golden and conformance execution do not apply
to this passive metadata-only closure.

The two private-environment layout rows now use the capability-free
`PrivateEnvironmentHeapSlot::{Parent, ClassScope}` identity domain instead of
free-form records that independently select names, offsets, widths and pointer
Booleans. One private exhaustive metadata projection owns both exact rows and
the typed registry preserves parent-then-class-scope order. The parent remains
the traced edge while the class-scope identifier remains scalar. The focused
[contract](../docs/rust-rewrite/contracts/private-environment-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change allocation, emitted Wasm, private-name lookup, class semantics or
collector execution. The structure target passes `4/4`, the exact identity
owner witness passes `1/1`, and the adjusted collision/pointer registry
witnesses pass `2/2` with only the workspace's existing warnings. Targeted
formatting with child module traversal disabled and diff checks pass, and the
shared `cargo xc` checkpoint is green. Golden and conformance execution do not
apply to this passive metadata-only closure.

The three ordinary Set entry rows now use the capability-free
`SetEntryHeapSlot::{Present, ValueTag, ValuePayload}` identity domain instead
of free-form records that independently select names, offsets, widths and
pointer Booleans. One private exhaustive metadata projection owns all three
exact rows and the typed registry preserves present-tag-payload order. The
presence and tag words remain scalar while the value payload remains the sole
traced edge. The focused
[contract](../docs/rust-rewrite/contracts/set-entry-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change allocation, emitted Wasm, Set behavior or collector execution. The
structure target passes `4/4`, the exact identity owner witness passes `1/1`,
and the adjusted collision/pointer registry witnesses pass `2/2` with only the
workspace's existing warnings. Targeted formatting with child module traversal
disabled and diff checks pass, and the shared `cargo xc` checkpoint is green.
Golden and conformance execution do not apply to this passive metadata-only
closure.

The three WeakSet entry rows now use the capability-free
`WeakSetEntryHeapSlot::{Present, ValueTag, ValuePayload}` identity domain
instead of free-form records that independently select names, offsets, widths
and pointer Booleans. One private exhaustive metadata projection owns all
three exact rows and the typed registry preserves present-tag-payload order.
All three words remain non-pointers, while the independent
`HeapWeakEdge::WeakSetValue` identity and `HeapWeakEdgeKind::EphemeronKey`
remain the sole semantic non-retention authority. The focused
[contract](../docs/rust-rewrite/contracts/weak-set-entry-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change allocation, emitted Wasm, WeakSet behavior or collector execution.
The structure target passes `4/4`; the exact identity and weak-edge owner
witnesses each pass `1/1`; and the adjusted collision/pointer registry
witnesses pass `2/2`, with only existing workspace warnings. The shared
`cargo xc`, workspace formatting, diff, module-boundary and task-plan checks
pass. Golden and conformance execution do not apply to this passive
metadata-only closure.

The four Map iterator record rows now use the capability-free
`MapIteratorHeapSlot::{MapPayload, NextIndex, Kind, CursorState}` identity
domain instead of free-form records that independently select names, offsets,
widths and pointer Booleans. One private exhaustive metadata projection owns
all four exact rows and the typed registry preserves payload-index-kind-cursor
order. The Map payload remains the sole traced word; the iteration index, kind
and cursor state remain scalar. The focused
[contract](../docs/rust-rewrite/contracts/map-iterator-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change allocation, emitted Wasm, Map iteration behavior or collector
execution. The structure target passes `4/4`, the exact identity owner witness
passes `1/1`, and the adjusted collision/pointer registry witnesses pass `2/2`
with only existing workspace warnings. Targeted formatting and diff checks
pass. The shared `cargo xc`, module-boundary and task-plan checks pass. Golden
and conformance execution do not apply to this passive metadata-only closure.

The four Set iterator record rows now use the capability-free
`SetIteratorHeapSlot::{SetPayload, NextIndex, Kind, CursorState}` identity
domain instead of free-form records that independently select names, offsets,
widths and pointer Booleans. One private exhaustive metadata projection owns
all four exact rows and the typed registry preserves payload-index-kind-cursor
order. The Set payload remains the sole traced word; the iteration index, kind
and cursor state remain scalar. The focused
[contract](../docs/rust-rewrite/contracts/set-iterator-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change allocation, emitted Wasm, Set iteration behavior or collector
execution. The structure target passes `4/4`, the exact identity owner witness
passes `1/1`, and the adjusted collision/pointer registry witnesses pass `2/2`
with only existing workspace warnings. The shared `cargo xc`, formatting,
diff, module-boundary and task-plan checks are green. Golden and conformance
execution do not apply to this passive metadata-only closure.

The four Set record rows now use the capability-free
`SetRecordHeapSlot::{EntriesPointer, EntriesLength, EntriesCapacity,
LiveCount}` identity domain instead of free-form records that independently
select names, offsets, widths and pointer Booleans. One private exhaustive
metadata projection owns all four exact rows and the typed registry preserves
entries-pointer-length-capacity-live-count order. The entries pointer remains
the sole traced word; length, capacity and live count remain scalar. The
focused
[contract](../docs/rust-rewrite/contracts/set-record-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change allocation, emitted Wasm, Set behavior or collector execution. The
canonical structure target passes `4/4`, the exact identity owner passes `1/1`,
and the adjusted collision/pointer registry witnesses pass `2/2`. The shared
`cargo xc`, formatting, diff, module-boundary and task-plan checks are green.
Golden and conformance execution do not apply to this passive metadata-only
closure.

The four WeakSet record rows now use the capability-free
`WeakSetRecordHeapSlot::{EntriesPointer, EntriesLength, EntriesCapacity,
LiveCount}` identity domain instead of free-form records that independently
select names, offsets, widths and pointer Booleans. One private exhaustive
metadata projection owns all four exact rows and the typed registry preserves
entries-pointer-length-capacity-live-count order. The record's entries-storage
pointer remains traced, while the closed WeakSet entry layout remains entirely
non-pointer and `HeapWeakEdge::WeakSetValue` remains the weak-retention
authority. The focused
[contract](../docs/rust-rewrite/contracts/weak-set-record-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change allocation, emitted Wasm, WeakSet behavior or collector execution.
The structure target passes `4/4`, and the exact identity owner and collision
witnesses each pass `1/1` with only existing workspace warnings. The shared
`cargo xc`, formatting, diff, module-boundary and task-plan checks are green.
Golden and conformance execution do not apply to this passive metadata-only
closure.

The four WeakMap record rows now use the capability-free
`WeakMapRecordHeapSlot::{EntriesPointer, EntriesLength, EntriesCapacity,
LiveCount}` identity domain instead of free-form records that independently
select names, offsets, widths and pointer Booleans. One private exhaustive
metadata projection owns all four exact rows and the typed registry preserves
entries-pointer-length-capacity-live-count order. The record's entries-storage
pointer remains traced, while the closed WeakMap entry layout remains entirely
non-pointer and `HeapWeakEdge::{WeakMapKey, WeakMapValue}` remain the ephemeron
authority. The focused
[contract](../docs/rust-rewrite/contracts/weak-map-record-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change allocation, emitted Wasm, WeakMap behavior or collector execution.
The structure target passes `4/4`, and the exact identity owner and collision
witnesses each pass `1/1` with only existing workspace warnings. The shared
`cargo xc`, formatting, diff, module-boundary and task-plan checks are green.
Golden and conformance execution do not apply to this passive metadata-only
closure.

The four ordinary Map record rows now use the capability-free
`MapRecordHeapSlot::{EntriesPointer, EntriesLength, EntriesCapacity,
LiveCount}` identity domain instead of free-form records that independently
select names, offsets, widths and pointer Booleans. One private exhaustive
metadata projection owns all four exact rows and the typed registry preserves
entries-pointer-length-capacity-live-count order. The entries pointer remains
the sole traced record word, while the closed Map entry layout retains its two
strong key/value payload edges. The focused
[contract](../docs/rust-rewrite/contracts/map-record-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change allocation, emitted Wasm, Map behavior or collector execution. The
recursive structure target passes `4/4`, the exact identity owner witness
passes `1/1`, and the adjusted collision/pointer registry witnesses pass `2/2`
with only existing workspace warnings. Targeted formatting and diff checks
pass. The shared `cargo xc`, formatting, diff, module-boundary and task-plan
checks are green. Golden and conformance execution do not apply to this passive
metadata-only closure.

The five ordinary Map entry rows now use the capability-free
`MapEntryHeapSlot::{Present, KeyTag, KeyPayload, ValueTag, ValuePayload}`
identity domain instead of free-form records that independently select names,
offsets, widths and pointer Booleans. One private exhaustive metadata
projection owns all five exact rows and the typed registry preserves present-
key-tag-key-payload-value-tag-value-payload order. The key and value payloads
remain the only two traced edges, deliberately contrasting with the closed
WeakMap entry layout's five non-pointer words. The focused
[contract](../docs/rust-rewrite/contracts/map-entry-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change allocation, emitted Wasm, Map behavior or collector execution. The
structure target passes `4/4`, the exact identity owner witness passes
`1/1`, and the adjusted collision/pointer registry witnesses pass `2/2` with
only existing workspace warnings. Targeted formatting and diff checks pass.
The shared `cargo xc`, module-boundary and task-plan checkpoints are green.
Golden and conformance execution do not apply to this passive metadata-only
closure.

The five WeakMap entry rows now use the capability-free
`WeakMapEntryHeapSlot::{Present, KeyTag, KeyPayload, ValueTag, ValuePayload}`
identity domain instead of free-form records that independently select names,
offsets, widths and pointer Booleans. One private exhaustive metadata
projection owns all five exact rows and the typed registry preserves present-
key-tag-key-payload-value-tag-value-payload order. All five words remain
non-pointers, while the independent `HeapWeakEdge::{WeakMapKey, WeakMapValue}`
identities and their ephemeron kinds remain the sole semantic retention
authority. The focused
[contract](../docs/rust-rewrite/contracts/weak-map-entry-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change allocation, emitted Wasm, WeakMap behavior or collector execution.
The structure target passes `4/4`, and the exact identity owner, ephemeron
authority and collision witnesses each pass `1/1` with only existing workspace
warnings. Targeted formatting and diff checks pass. Broad workspace, golden
and conformance verification do not apply to this passive metadata-only
closure; the shared `cargo xc`, module-boundary and task-plan checks pass.

The async-generator object's sole raw layout row now uses the capability-free
`AsyncGeneratorObjectHeapSlot::Activation` identity. One private exhaustive
metadata projection owns the exact `async-generator-object` record name,
`activation` slot name, existing activation offset, 8-byte width and traced
pointer classification; the typed registry contains only that identity. The
focused
[contract](../docs/rust-rewrite/contracts/async-generator-object-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. Runtime allocation and activation access continue
to use the existing offset. This remains passive layout inventory: it does not
change allocation, emitted Wasm, async-generator behavior or collector
execution. The recursive structure target passes `4/4`, the exact identity
owner witness passes `1/1`, and the adjusted collision/pointer registry
witnesses pass `2/2` with only existing workspace warnings. The shared
`cargo xc`, formatting, diff, module-boundary and task-plan checks are green.
Golden and conformance execution do not apply to this passive metadata-only
closure.

The three raw environment layout rows now use the capability-free
`EnvironmentHeapSlot::{Parent, BindingTag, BindingPayload}` identity domain.
One private exhaustive metadata projection owns the exact environment and
binding-slot record names, slot names, offsets, 8-byte widths and pointer
classifications; the typed registry preserves parent-tag-payload order. The
environment parent and each binding payload remain traced, while binding tags
remain scalar. The focused
[contract](../docs/rust-rewrite/contracts/environment-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change environment allocation, emitted Wasm, binding access or collector
execution. The recursive structure target passes `4/4`, the exact owner
witness passes `1/1`, and the collision/pointer registry witnesses pass `2/2`.
The shared `cargo xc`, formatting, diff, module-boundary and task-plan checks
are green. Golden and conformance execution do not apply.

The four raw String record layout rows now use the capability-free
`StringHeapSlot::{CodeUnitsPointer, ByteLength, CodeUnitLength, InternId}`
identity domain. One private exhaustive metadata projection owns the exact
record and slot names, offsets, 8-byte widths and pointer classifications; the
typed registry preserves code-units-pointer, byte-length, code-unit-length and
intern-id order. The code-units address remains the sole pointer-classified
word, while both lengths and the intern identity remain scalar. The existing
`LinearSideStorage::StringCodeUnits` identity continues to own UTF-16
side-storage element representation. The focused
[contract](../docs/rust-rewrite/contracts/string-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change String allocation, emitted Wasm, code-unit storage, interning or
collector execution. The recursive structure target passes `4/4`, the exact
owner witness passes `1/1`, and the collision/pointer registry witnesses pass
`2/2`. The shared `cargo xc`, formatting, diff, module-boundary and task-plan
checks are green. Golden and conformance execution do not apply.

The four raw BigInt record layout rows now use the capability-free
`BigIntHeapSlot::{Sign, LimbsPointer, LimbsLength, LimbsCapacity}` identity
domain. One private exhaustive metadata projection owns the exact record and
slot names, offsets, 8-byte widths and pointer classifications; the typed
registry preserves sign-pointer-length-capacity order. The sign, length and
capacity remain scalar, while the limbs address remains the sole
pointer-classified word. The existing `LinearSideStorage::BigIntLimbs`
identity continues to own the non-reference limb element representation. The
focused
[contract](../docs/rust-rewrite/contracts/bigint-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change BigInt allocation, emitted Wasm, sign decoding, limb storage,
arbitrary-precision readiness or collector execution. The recursive structure
target passes `4/4`, the exact heap owner witness passes `1/1`, and the
collision/pointer registry filter passes `2/2`. The shared `cargo xc`
checkpoint is green. This is a passive layout migration, so no semantic golden
or Test262 rerun was performed.

The four raw Symbol record layout rows now use the capability-free
`SymbolHeapSlot::{DescriptionTag, DescriptionPayload, RegistryKeyPayload,
SymbolId}` identity domain. One private exhaustive metadata projection owns
the exact record and slot names, offsets, 8-byte widths and pointer
classifications; the typed registry preserves description-tag,
description-payload, registry-key-payload and symbol-id order. The tag and
symbol identity remain
scalar, while the description and registry-key payloads remain
pointer-classified. The focused
[contract](../docs/rust-rewrite/contracts/symbol-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change Symbol allocation, emitted Wasm, description or registry semantics, or
collector execution. The recursive structure target passes `4/4`, the exact
heap owner witness passes `1/1`, and the collision/pointer registry filter
passes `2/2`. The shared `cargo xc` checkpoint is green. This is a passive
layout migration, so no semantic golden or Test262 rerun was performed.

The four raw TypedArray iterator record layout rows now use the capability-free
`TypedArrayIteratorHeapSlot::{TypedArrayPayload, NextIndex, Kind, Done}`
identity domain. One private exhaustive metadata projection owns the exact
record and slot names, offsets, 8-byte widths and pointer classifications; the
typed registry preserves typed-array-payload, next-index, kind and done order.
The TypedArray payload remains the sole pointer-classified word while all
iterator progress and state words remain scalar. The focused
[contract](../docs/rust-rewrite/contracts/typed-array-iterator-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change TypedArray iterator allocation, emitted Wasm, iterator stepping,
detachment or resizable-buffer semantics, or collector execution. The recursive
structure target passes `4/4`, the exact heap owner witness passes `1/1`, and
the collision/pointer registry filter passes `2/2`. The shared `cargo xc`
checkpoint is green. No semantic golden or Test262 rerun applies to this
passive layout-only migration.

The four raw Temporal.PlainDate record layout rows now use the capability-free
`TemporalPlainDateHeapSlot::{IsoYear, IsoMonth, IsoDay, CalendarPayload}`
identity domain. One private exhaustive metadata projection owns the exact
record and slot names, offsets, 8-byte widths and pointer classifications; the
typed registry preserves ISO-year, ISO-month, ISO-day and calendar-payload
order. The three numeric date fields remain scalar while the calendar payload
remains the sole pointer-classified word. The focused
[contract](../docs/rust-rewrite/contracts/temporal-plain-date-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change Temporal.PlainDate allocation, emitted Wasm, date or calendar semantics,
or collector execution. Dry source review pins the exact four rows, the
three-scalar/one-pointer census, typed registry order and unchanged runtime
offset consumers. The recursive structure target passes `4/4`, the exact heap
owner witness passes `1/1`, and the collision/pointer registry filter passes
`2/2`. The shared `cargo xc` checkpoint is green. No semantic golden or Test262
rerun applies to this passive layout-only migration.

The four raw DisposableStack record layout rows now use the capability-free
`DisposableStackRecordHeapSlot::{State, EntriesPointer, EntriesLength,
EntriesCapacity}` identity domain. One private exhaustive metadata projection
owns the exact record and slot names, offsets, 8-byte widths and pointer
classifications; the typed registry preserves state, entries-pointer,
entries-length and entries-capacity order. Lifecycle and accounting words
remain scalar while the entries-storage pointer remains the sole
pointer-classified word. The focused
[contract](../docs/rust-rewrite/contracts/disposable-stack-record-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change allocation, stack lifecycle, capability transfer, resource retention,
disposal order, emitted Wasm or collector execution. Dry source review pins
the exact four rows, the three-scalar/one-pointer census, typed registry order
and unchanged runtime offset consumers. Scoped formatting and diff checks pass.
The recursive structure target passes `4/4`, the exact heap owner witness
passes `1/1`, and the collision/pointer registry filter passes `2/2`. The
shared `cargo xc` checkpoint is green. No semantic golden or Test262 rerun
applies to this passive layout-only migration.

The four raw AsyncDisposableStack record layout rows now use the capability-free
`AsyncDisposableStackRecordHeapSlot::{State, EntriesPointer, EntriesLength,
EntriesCapacity}` identity domain. One private exhaustive metadata projection
owns the exact record and slot names, offsets, 8-byte widths and pointer
classifications; the typed registry preserves state, entries-pointer,
entries-length and entries-capacity order. Lifecycle and accounting words
remain scalar while the entries-storage pointer remains the sole
pointer-classified word. The focused
[contract](../docs/rust-rewrite/contracts/async-disposable-stack-record-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change allocation, stack lifecycle, asynchronous disposal, resource retention,
await completion, emitted Wasm or collector execution. Dry source review pins
the exact four rows, the three-scalar/one-pointer census, typed registry order
and unchanged runtime offset consumers. At the 2026-08-28 Batch W checkpoint,
the recursive structure target passes `4/4`, the exact heap owner witness
passes `1/1`, the collision/pointer registry filter passes `2/2`, and the
shared `cargo xc` checkpoint is green. No semantic golden or Test262 rerun
applies to this passive layout-only migration.

The five raw DisposableStack entry layout rows now use the capability-free
`DisposableStackEntryHeapSlot::{Kind, ValueTag, ValuePayload, MethodTag,
MethodPayload}` identity domain. One private exhaustive metadata projection
owns the exact record and slot names, offsets, 8-byte widths and pointer
classifications; the typed registry preserves kind, value-tag, value-payload,
method-tag and method-payload order. The kind and tags remain scalar while the
resource value and acquired method payloads remain pointer-classified. The
focused
[contract](../docs/rust-rewrite/contracts/disposable-stack-entry-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change allocation, stack lifecycle, capability transfer, resource or method
retention, disposal order, emitted Wasm or collector execution. Dry source
review pins the exact five rows, the three-scalar/two-pointer census, typed
registry order and unchanged runtime offset consumers. Scoped formatting, diff
and task-plan checks pass. At the 2026-08-28 Batch X checkpoint, the recursive
structure target passes `4/4`, the exact heap owner witness passes `1/1`, the
collision/pointer registry filter passes `2/2`, and the shared `cargo xc` and
module-boundary checks are green. No semantic golden or Test262 rerun applies
to this passive layout-only migration.

The five raw AsyncDisposableStack entry layout rows now use the capability-free
`AsyncDisposableStackEntryHeapSlot::{Kind, ValueTag, ValuePayload, MethodTag,
MethodPayload}` identity domain. One private exhaustive metadata projection
owns the exact record and slot names, offsets, 8-byte widths and pointer
classifications; the typed registry preserves kind, value-tag, value-payload,
method-tag and method-payload order. The kind and tags remain scalar while the
resource value and acquired method payloads remain pointer-classified. The
focused
[contract](../docs/rust-rewrite/contracts/async-disposable-stack-entry-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change allocation, stack lifecycle, asynchronous disposal, resource or method
retention, await completion, emitted Wasm or collector execution. Dry source
review pins the exact five rows, the three-scalar/two-pointer census, typed
registry order and unchanged runtime offset consumers. At the 2026-08-28 Batch
Y checkpoint, the recursive structure target passes `4/4`, the exact heap owner
witness passes `1/1`, the collision/pointer registry filter passes `2/2`, and
the shared `cargo xc`, formatting, diff, module-boundary and task-plan checks
are green. No semantic golden or Test262 rerun applies to this passive
layout-only migration.

The five raw pending-completion record layout rows now use the capability-free
`PendingCompletionHeapSlot::{Next, Payload, Tag, Kind, Aux}` identity domain.
One private exhaustive metadata projection owns the exact record and slot
names, offsets, 8-byte widths and pointer classifications; the typed registry
preserves next, payload, tag, kind and auxiliary order. The linked-record and
completion payload words remain pointer-classified while the tag, completion
kind and auxiliary state remain scalar. The focused
[contract](../docs/rust-rewrite/contracts/pending-completion-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change allocation, finally restoration, async disposal, emitted Wasm, root
scanning or collector execution. Dry source review pins the exact five rows,
the three-scalar/two-pointer census, typed registry order and unchanged runtime
offset consumers. At the 2026-08-28 Batch Z checkpoint, `cargo xc` is green,
the recursive structure target passes `4/4`, the exact heap owner witness passes
`1/1`, and the collision/pointer registry filter passes `2/2`. Runtime execution
and semantic goldens were not rerun; no Test262 claim applies to this passive
layout-only migration.

The six raw Atomics async-waiter layout rows now use the capability-free
`AtomicsAsyncWaiterHeapSlot::{State, Address, PromiseRecord, DeadlineNanos,
Next, HostId}` identity domain. One private exhaustive metadata projection owns
the exact record and slot names, offsets, 8-byte widths and pointer
classifications; the typed registry preserves state, address, Promise-record,
deadline, next-link and host-identity order. The Promise record and waiter-list
link remain pointer-classified, while state, the linear-memory wait address,
monotonic deadline and opaque host identity remain scalar. The focused
[contract](../docs/rust-rewrite/contracts/atomics-async-waiter-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change waiter allocation or traversal, timeout processing, host-agent calls,
Promise settlement, emitted Wasm, root scanning or collector execution. Dry
source review pins offsets 0, 8, 16, 24, 32 and 40, the four-scalar/two-pointer
census, typed registry order and unchanged runtime offset consumers. At the
Batch AA checkpoint, `cargo xc` is green, the structure target passes `4/4`,
the exact layout-owner unit passes `1/1`, and the heap registry filter passes
`2/2`. Runtime, semantic-golden and Test262 checks do not apply to this passive
layout-only migration and were not run.

The six raw bound-function layout rows now use the capability-free
`BoundFunctionHeapSlot::{TargetPayload, TargetTag, ThisPayload, ThisTag,
ArgumentsPayload, SelfPayload}` identity domain. One private exhaustive
metadata projection owns the exact record and slot names, offsets, 8-byte
widths and pointer classifications. The typed registry preserves the existing
target-payload, target-tag, bound-this-payload, bound-this-tag,
arguments-payload and self-payload order. Target and bound-this tags remain
scalar, while their payloads, the arguments storage and bound-function self
payload remain pointer-classified. The focused
[contract](../docs/rust-rewrite/contracts/bound-function-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second
free-form layout producer. This remains passive layout inventory: it does not
change allocation, call or construct behavior, Realm lookup, `instanceof`,
emitted Wasm, root scanning or collector execution. Dry source review pins
offsets 8, 0, 24, 16, 32 and 40 in registry order, the two-scalar/four-pointer
census, typed registry order and unchanged runtime offset consumers. At the
Batch AB checkpoint, `cargo xc` is green, the structure target passes `4/4`,
the exact layout-owner unit passes `1/1`, and the heap registry filter passes
`2/2`. Runtime, semantic-golden and Test262 checks do not apply to this passive
layout-only migration and were not run.

The five raw FinalizationRegistry record layout rows now use the capability-free
`FinalizationRegistryRecordHeapSlot::{CleanupCallbackTag,
CleanupCallbackPayload, CellsPointer, CellsLength, CellsCapacity}` identity
domain. One private exhaustive metadata projection owns the exact record and
slot names, offsets, 8-byte widths and pointer classifications; the typed
registry preserves cleanup-callback-tag, cleanup-callback-payload,
cells-pointer, cells-length and cells-capacity order. The callback tag and cells
accounting words remain scalar while the callback payload and cells-storage
pointer remain pointer-classified. The focused
[contract](../docs/rust-rewrite/contracts/finalization-registry-record-heap-slot-authority.md),
recursive structure regression and bounded heap owner witness pin the closed
domain, reject incidental identity capabilities and exclude a second free-form
layout producer. This remains passive layout inventory: it does not change
allocation, cleanup callback invocation, cell registration or removal,
weak-edge retention, cleanup scheduling, emitted Wasm, root scanning or
collector execution. Dry source review pins offsets 0, 8, 16, 24 and 32, the
three-scalar/two-pointer census, typed registry order and unchanged runtime
offset consumers. At the shared Batch AD checkpoint, `cargo xc` is green, the
recursive structure target passes `4/4`, the exact heap-owner witness passes
`1/1`, the `heap_layout_registry_` filter passes `2/2`, and
`finalization_registry_cells_keep_only_holdings_strongly_reachable` passes
`1/1`. The FinalizationRegistry runtime builtin remains unchanged. No CLI,
Test262 or semantic-golden verification applies to this passive layout-only
migration, and none was run.

The seven raw FinalizationRegistry cell layout rows now use the capability-free
`FinalizationRegistryCellHeapSlot::{State, TargetTag, TargetPayload,
HoldingsTag, HoldingsPayload, UnregisterTokenTag, UnregisterTokenPayload}`
identity domain. One private exhaustive metadata projection owns the exact
record and slot names, offsets, 8-byte widths and pointer classifications; the
typed registry preserves state followed by the target, holdings and unregister-
token tag/payload pairs. Holdings payload remains the sole pointer-classified
word. Target and unregister-token payloads remain non-pointers because their
non-retaining semantics belong to the closed weak-edge registry, while holdings
remains strong until cleanup. The focused
[contract](../docs/rust-rewrite/contracts/finalization-registry-cell-heap-slot-authority.md),
recursive structure regression and bounded heap owner witnesses pin the closed
domain, reject incidental identity capabilities, exclude a second free-form
layout producer and preserve the weak/strong relation. This remains passive
layout inventory: it does not change cell allocation or growth, registration
or unregistration, persisted cell-state admission, weak reachability, cleanup
scheduling, emitted Wasm, root scanning or collector execution. Dry source
review pins offsets 0, 8, 16, 24, 32, 40 and 48, the six-scalar/one-pointer
census, typed registry order and unchanged runtime offset consumers. At the
shared Batch AE checkpoint, `cargo xc` is green. The cell-layout, cell-state and
weak-edge-retention structure targets pass `4/4` each (`12/12` combined). The
exact typed-owner witness passes `1/1`, the `heap_layout_registry_` filter passes
`2/2`, and the only-holdings retention witness passes `1/1` (`4/4` combined).
The FinalizationRegistry runtime builtin remains unchanged. No CLI, Test262 or
semantic-golden verification applies to this passive layout-only migration,
and none was run.

The six raw Promise capability layout rows now use the capability-free
`PromiseCapabilityHeapSlot::{PromiseTag, PromisePayload, ResolveTag,
ResolvePayload, RejectTag, RejectPayload}` identity domain. One private
exhaustive metadata projection owns the exact record and slot names, offsets,
8-byte widths and pointer classifications; the typed registry preserves the
promise, resolve and reject tag/payload pair order. The three tags remain
scalar, while all three payloads remain pointer-classified so the existing
async-generator retention consumer cannot silently lose the Promise, resolve
function or reject function edge. The focused
[contract](../docs/rust-rewrite/contracts/promise-capability-heap-slot-authority.md),
recursive structure regression and bounded heap owner witnesses pin the closed
domain, reject incidental identity capabilities, exclude a second free-form
layout producer and preserve the retention relation. This remains passive
layout inventory: it does not change capability allocation or initialization,
Promise settlement, reactions or jobs, async-generator behavior, emitted Wasm,
root scanning or collector execution. Dry source review pins offsets 0, 8, 16,
24, 32 and 40, the three-scalar/three-pointer census, typed registry order and
unchanged Promise and Array runtime offset consumers. At the shared Batch AF
checkpoint, `cargo xc` is green. The Promise-capability structure target passes
`4/4`, exact
`heap::tests::promise_capability_heap_slot_identities_own_layout_metadata`
passes `1/1`, exact
`heap::tests::async_generator_records_expose_queue_activation_and_promise_edges_to_gc`
passes `1/1`, and the `heap_layout_registry_` filter passes `2/2`. No CLI,
Test262 or semantic-golden verification applies to this source/type-only heap
ownership change, and none was run.

The seven raw pending Promise-job layout rows now use the capability-free
`PendingJobHeapSlot::{CallbackTag, CallbackPayload, ArgumentTag,
ArgumentPayload, Realm, Next, Kind}` identity domain. One private exhaustive
metadata projection owns the exact record and slot names, offsets, 8-byte
widths and pointer classifications; the typed registry preserves callback and
argument tag/payload pairs followed by Realm, next and kind. The two tags and
kind remain scalar, while both payloads, Realm and next remain pointer-
classified so queued work cannot silently lose its callback record, argument,
evaluation Realm or following FIFO node. The focused
[contract](../docs/rust-rewrite/contracts/pending-job-heap-slot-authority.md),
recursive structure regression and bounded heap owner witnesses pin the closed
domain, reject incidental identity capabilities, exclude a second free-form
layout producer and preserve the exact next-edge retention check. This remains
passive layout inventory: it does not change job construction, queue ordering,
job dispatch, Promise or async-generator behavior, emitted Wasm, root scanning
or collector execution. The separately typed pending-jobs root source remains
unchanged. Dry source review pins offsets 0, 8, 16, 24, 32, 40 and 48, the
three-scalar/four-pointer census, typed registry order and unchanged runtime
enqueue/drain offset consumers. At the shared Batch AG checkpoint, `cargo xc`
is green, the recursive structure target passes `4/4`, exact
`heap::tests::pending_job_heap_slot_identities_own_layout_metadata` passes
`1/1`, and the `heap_layout_registry_` filter passes `2/2`. No CLI, Test262 or
semantic-golden verification applies to this source/type-only ownership
change, and none was run; the runtime enqueue/drain code is byte-untouched.

The six raw class-function-context layout rows now use the capability-free
`ClassFunctionContextHeapSlot::{LexicalEnvironment, ActiveFunction,
HomeObjectPayload, HomeObjectTag, FieldKeys, PrivateEnvironment}` identity
domain. One private exhaustive metadata projection owns the exact record and
slot names, offsets, 8-byte widths and pointer classifications; the typed
registry preserves lexical environment, active function, the home-object
payload/tag pair, field keys and private environment order. Home-object tag
remains the sole scalar, while all five ownership edges remain pointer-
classified so class contexts cannot silently lose their lexical environment,
function, home object, computed field keys or private environment. The focused
[contract](../docs/rust-rewrite/contracts/class-function-context-heap-slot-authority.md),
recursive structure regression and bounded heap owner witnesses pin the closed
domain, reject incidental identity capabilities, exclude a second free-form
layout producer and preserve the sole-home-object-tag-scalar relation. This
remains passive layout inventory: it does not change class-context allocation,
method or field initialization, `super` resolution, private-name lookup,
emitted Wasm, root scanning or collector execution. Dry source review pins
offsets 0, 8, 16, 24, 32 and 40, the one-scalar/five-pointer census, typed
registry order and unchanged runtime consumers in `emit.rs`, `functions.rs`
and `objects/private_elements.rs`. At the shared Batch AH checkpoint, `cargo xc`
exits `0`, the `class_function_context_heap_slot_structure` target passes
`4/4`, exact
`heap::tests::class_function_context_heap_slot_identities_own_layout_metadata`
passes `1/1`, and the shared `heap_layout_registry_` filter passes `2/2`. No
CLI, Test262 or semantic-golden verification is needed for this source/type-
only ownership change, and none was run. Final formatter, diff,
module-boundary, task-plan and 240-entry shortcut-inventory gates are green.

The six raw private-element-entry layout rows now use the capability-free
`PrivateElementEntryHeapSlot::{Next, Receiver, Token, Kind, ValueTag,
ValuePayload}` identity domain. One private exhaustive metadata projection owns
the exact record and slot names, offsets, 8-byte widths and pointer
classifications; the typed registry preserves linked-list node, receiver,
private-name token, kind and value tag/payload order. Kind and value tag remain
the two scalars, while next, receiver, token and value payload remain pointer-
classified so private-element entries cannot silently lose their linked-list,
identity or stored-value edges. The focused
[contract](../docs/rust-rewrite/contracts/private-element-entry-heap-slot-authority.md),
recursive structure regression and bounded heap owner witnesses pin the closed
domain, reject incidental identity capabilities and exclude a second free-form
layout producer. The existing `PrivateElementEntryLocals` and
`PrivateElementHeapKind` authorities remain the independent owners of legal row
contents and kind wire words. This remains passive layout inventory: it does
not change entry construction, linked-list publication or traversal, private-
name lookup, field/method/accessor semantics, emitted Wasm, root scanning or
collector execution. Dry source review pins offsets 0, 8, 16, 24, 32 and 40,
the two-scalar/four-pointer census, typed registry order and unchanged runtime
offset consumers. At the shared Batch AI checkpoint, `cargo xc` exits `0`, the
`private_element_entry_heap_slot_structure` target passes `4/4`, the unchanged
`private_element_entry_protocol_structure` target passes `5/5`, exact
`heap::tests::private_element_entry_heap_slot_identities_own_layout_metadata`
passes `1/1`, and the `heap_layout_registry_` filter passes `2/2`. No CLI,
Test262 or semantic-golden verification is needed for this source/type-only
ownership change, and none was run because the runtime is byte-untouched. Final
formatter, diff, module-boundary, task-plan and 240-entry shortcut-inventory
gates are green.

The six raw Temporal.ZonedDateTime layout rows now use the capability-free
`TemporalZonedDateTimeHeapSlot::{EpochNanosecondsTag,
EpochNanosecondsPayload, TimeZoneTag, TimeZonePayload, CalendarTag,
CalendarPayload}` identity domain. One private exhaustive metadata projection
owns the exact record and slot names, offsets, 8-byte widths and pointer
classifications; the typed registry preserves the epoch-nanoseconds, time-zone
and calendar tag/payload pair order. The three tags remain scalar, while all
three payloads remain pointer-classified so a ZonedDateTime record cannot
silently lose its epoch-nanoseconds, time-zone or calendar edge. The focused
[contract](../docs/rust-rewrite/contracts/temporal-zoned-date-time-heap-slot-authority.md),
recursive structure regression and bounded heap owner witnesses pin the closed
domain, reject incidental identity capabilities and exclude a second free-form
layout producer. The existing Temporal Instant, Temporal PlainDate and
ZonedDateTime algorithm enums remain independent semantic authorities. This
remains passive layout inventory: it does not change ZonedDateTime allocation
or access, epoch-nanoseconds representation, time-zone or calendar semantics,
emitted Wasm, root scanning or collector execution. Dry source review pins
offsets 0, 8, 16, 24, 32 and 40, the three-scalar/three-pointer census, typed
registry order and unchanged Temporal runtime offset consumers. Shared
`cargo xc` passes, the recursive structure target passes `4/4`, the exact heap
owner passes `1/1`, and the registry filters pass `2/2`. No runtime CLI,
Test262 or semantic-golden verification is needed for this passive metadata
change with byte-untouched Temporal consumers.

The seven raw Promise-reaction layout rows now use the capability-free
`PromiseReactionHeapSlot::{Capability, HandlerTag, HandlerPayload, Realm, Next,
Type, CallbackKind}` identity domain. One private exhaustive metadata
projection owns the exact record and slot names, offsets, 8-byte widths and
pointer classifications; the typed registry preserves capability,
handler-tag, handler-payload, Realm, next, type and callback-kind order.
Capability, handler payload, Realm and next remain pointer-classified, while
handler tag, type and callback kind remain scalar, so one reaction cannot
silently lose a live edge or trace a wire word as an address. The focused
[contract](../docs/rust-rewrite/contracts/promise-reaction-heap-slot-authority.md),
recursive structure regression and bounded heap owner witnesses pin the closed
domain, reject incidental identity capabilities and exclude a second free-form
layout producer. The existing `PromiseReactionType` and
`PromiseReactionCallbackKind` remain independent wire/semantic authorities.
This remains passive layout inventory: it does not change Promise-reaction
allocation, list linking, fulfillment or rejection dispatch, handler
tag/payload representation, Realm selection, queued jobs, emitted Wasm, root
scanning or collector execution. Dry source review pins offsets 0, 8, 16, 24,
32, 40 and 48, the three-scalar/four-pointer census, typed registry order and
byte-untouched Promise runtime consumers. Batch AK shared `cargo xc` is green,
the recursive structure target passes `4/4`, the exact heap owner passes `1/1`,
both collision/pointer registry tests pass `2/2`, and the neighboring
async-generator retention witness passes `1/1`. This passive metadata change
requires no runtime CLI, Test262 cohort or semantic golden. Final formatter,
diff, module-boundary, task-plan and 240-entry shortcut-inventory gates are
green.

The five raw Intl.Locale layout rows now use the capability-free
`IntlLocaleHeapSlot::{TagPayload, LanguagePayload, ScriptPayload,
RegionPayload, BaseNamePayload}` identity domain. One private exhaustive
metadata projection owns the exact record and slot names, offsets, 8-byte
widths and pointer classifications; the typed registry preserves tag,
language, script, region and base-name payload order. All five slots remain
pointer-classified, so an initialized Locale record cannot silently lose any
materialized string edge, including optional script and region payloads when
present. The focused
[contract](../docs/rust-rewrite/contracts/intl-locale-heap-slot-authority.md),
recursive structure regression and bounded heap owner witnesses pin the closed
domain, reject incidental identity capabilities and exclude a second free-form
layout producer. The existing `IntlLocaleStringSlot` remains the independent
runtime authority for accessor offsets and optional-result policy. This
remains passive layout inventory: it does not change Locale allocation,
initialization, accessors, optional script or region semantics,
canonicalization, emitted Wasm, root scanning or collector execution. Dry
source review pins offsets 0, 8, 16, 24 and 32, the zero-scalar/five-pointer
census, typed registry order and byte-untouched Intl runtime consumers. At the
Batch AL checkpoint, `cargo xc` is green, the new structure target passes
`4/4`, its existing string-slot neighbor passes `3/3`, the bounded heap owner
passes `1/1`, and the registry checks pass `2/2`. No runtime CLI, Test262 leaf
or semantic golden was required or run for this passive metadata change.
Final formatter, diff, module-boundary, task-plan and 240-entry
shortcut-inventory gates are green.

The eight raw ordinary object-entry layout rows now use the capability-free
`ObjectEntryHeapSlot::{Key, DescriptorKind, DataTag, DataPayload, GetterTag,
GetterPayload, SetterTag, SetterPayload}` identity domain. One private
exhaustive metadata projection owns the exact record and slot names, offsets,
8-byte widths and pointer classifications; the typed registry preserves key,
descriptor kind and the data, getter and setter tag/payload pair order. Key
and all three payloads remain pointer-classified, while descriptor kind and
all three tags remain scalar, so an entry cannot silently lose a live edge or
trace a wire word as an address. The focused
[contract](../docs/rust-rewrite/contracts/object-entry-heap-slot-authority.md),
recursive structure regression and bounded heap owner witnesses pin the closed
domain, reject incidental identity capabilities and exclude a second free-form
layout producer. `DescriptorWord`, `StoredDescriptorKind` and the stored
descriptor local types remain independent semantic authorities. This remains
passive layout inventory: it does not change property allocation, lookup,
descriptor creation or update, accessor invocation, Array or Object builtins,
emitted Wasm, root scanning or collector execution. Dry source review pins
offsets 0, 8, 16, 24, 32, 40, 48 and 56, the four-scalar/four-pointer census,
typed registry order and byte-untouched runtime consumers. At the Batch AM
checkpoint, `cargo xc` is green, the new structure target passes `4/4`, both
descriptor neighbors pass `4/4`, the bounded heap owner passes `1/1`, and the
registry checks pass `2/2`. No runtime CLI, Test262 leaf or semantic golden was
required or run for this passive metadata change.
Final formatter, diff, module-boundary, task-plan and 240-entry
shortcut-inventory gates are green.

The nine raw Realm-record layout rows now use the capability-free
`RealmRecordHeapSlot::{RealmId, AgentId, GlobalObject, GlobalThis,
GlobalEnvironment, Intrinsics, HostHooks, ModuleRegistry, PrivateElements}`
identity domain. One private exhaustive metadata projection owns the exact
record and slot names, offsets, 8-byte widths and pointer classifications; the
typed registry preserves both scalar ids followed by the seven Realm ownership
edges. Realm id and Agent id remain scalar, while global object, global this,
global environment, intrinsics, host hooks, module registry and private
elements remain pointer-classified, so a Realm cannot silently lose live state
or trace an identity word as an address. The focused
[contract](../docs/rust-rewrite/contracts/realm-record-heap-slot-authority.md),
recursive structure regression and bounded heap owner witnesses pin the closed
domain, reject incidental identity capabilities and exclude a second free-form
layout producer. `RealmRecordLocal`, Realm-id allocation and created-Realm
publication policies remain independent lifetime and semantic authorities.
This remains passive layout inventory: it does not change Realm allocation,
initialization, lookup, intrinsic publication, global-environment behavior,
host hooks, module loading, private elements, emitted Wasm, root scanning or
collector execution. Dry source review pins offsets 0, 8, 16, 24, 32, 40, 48,
56 and 64, the two-scalar/seven-pointer census, typed registry order and
byte-untouched runtime consumers. At the Batch AN checkpoint, `cargo xc` is
green, the new structure target passes `4/4`, the Array and Promise Realm
neighbors pass `3/3` and `5/5`, the bounded heap owner passes `1/1`, and the
registry checks pass `2/2`. No runtime CLI, Test262 leaf or semantic golden was
required or run for this passive metadata change. Final formatter, diff,
module-boundary, task-plan and 240-entry shortcut-inventory gates are green.

The six passive Temporal.PlainTime layout rows now use the capability-free
`TemporalPlainTimeHeapSlot::{Hour, Minute, Second, Millisecond, Microsecond,
Nanosecond}` identity domain. One private exhaustive metadata projection owns
the exact record and slot names, offsets 0, 8, 16, 24, 32 and 40, 8-byte widths
and the six-scalar/zero-pointer census; typed registry order cannot diverge from
the component layout. The focused
[contract](../docs/rust-rewrite/contracts/temporal-plain-time-heap-slot-authority.md),
recursive structure regression and bounded heap owner pin that closed passive
authority. `TemporalTimeUnit` remains the runtime authority for field indexes,
offset selection and component ranges. Temporal.PlainTime allocation, emitted
Wasm, time arithmetic, Intl formatting, root scanning and collector execution
remain byte untouched. At the Batch AO checkpoint, `cargo xc` is green, the
new and neighboring structure targets pass `4/4` each, the bounded heap owner
passes `1/1`, and the registry witnesses pass `2/2`. No runtime CLI, Test262
leaf or semantic golden was required or run for this passive metadata
migration.

The ten passive Temporal.PlainDateTime layout rows now use the capability-free
`TemporalPlainDateTimeHeapSlot` identity domain. One private exhaustive
projection owns nine scalar ISO date/time fields at offsets 0 through 64 and
the traced calendar payload at offset 72; typed registry order cannot diverge
from their layout metadata. The focused
[contract](../docs/rust-rewrite/contracts/temporal-plain-date-time-heap-slot-authority.md),
recursive guard and bounded heap owner pin that authority. Allocation, field
reads, arithmetic, calendar behavior, emitted Wasm, root scanning and collector
execution remain untouched. Batch AP's recursive structure target passes
`4/4`, the exact heap-slot identity unit passes `1/1`, both heap-layout
registry controls pass `2/2`, and `cargo xc` is green. No runtime CLI, Test262
leaf or semantic golden is required.

The passive Temporal.Duration heap layout now uses a closed
`TemporalDurationHeapSlot` identity domain. One private exhaustive projection
owns the exact years-through-nanoseconds record names, offsets, widths and
untraced classifications; typed registry order cannot diverge from that
metadata. The focused
[contract](../docs/rust-rewrite/contracts/temporal-duration-heap-slot-authority.md),
recursive guard and bounded heap unit pin that authority. Allocation, field
reads, arithmetic, emitted Wasm, root scanning and collector execution remain
untouched. At the 2026-08-28 Batch AQ checkpoint, `cargo xc` is green, the
recursive structure target passes `4/4`, the exact heap-slot identity unit
passes `1/1`, and both heap-layout registry controls pass `2/2`. No runtime
CLI, Test262 leaf or semantic golden is required. This source-equivalent
passive metadata migration claims no new Temporal behavior.

The passive Intl.DateTimeFormat heap layout now uses a closed
`IntlDateTimeFormatHeapSlot` identity domain. One private exhaustive projection
owns all twenty-three names, offsets, widths and pointer classifications; the
typed registry fixes their order and preserves the six-pointer/seventeen-scalar
census. The former 165-line table and new 257-line owner have SHA-256
`7c2284a3fc1325cf43f042d1df6240f96b1c273836bada75c9cd2a8410d7d6a9`
and `4679b3d4ffae6088c8dca5c580b8356278e91b64624e965b6ef6270a9cb5dd59`.
The focused
[`contract`](../docs/rust-rewrite/contracts/intl-date-time-format-heap-slot-authority.md)
and recursive guard pin the passive metadata migration. Allocation, field
access, formatting, emitted Wasm, root scanning and collector execution remain
untouched. At the 2026-08-28 Batch AR checkpoint, `cargo xc` is green, the
structure target passes `4/4`, the focused slot-identity unit passes `1/1`, and
both heap-layout registry controls pass `2/2`. No runtime CLI, Test262 leaf or
semantic golden is required. This source-equivalent passive metadata migration
claims no new Intl behavior.

The passive collector's seven root sources now use one closed
`HeapRootSource` identity and one closed
`HeapRootKind::{PersistentNonTagged, PersistentTaggedValues,
TransientTaggedValues}` classification. A single private exhaustive metadata
projection owns every source's diagnostic name, owner and kind; the inventory
can no longer combine independently writable tagged-value and transient
Booleans. The closed host-boundary policy projects the typed
`HeapRootSource::HostBorrowedValues` variant instead of a free-form string, so a
misspelled or renamed host root cannot compile. The focused
[contract](../docs/rust-rewrite/contracts/heap-root-source-authority.md) and
structure regression pin the exact domains, seven meanings, registry and typed
host producer. This remains passive inventory: it does not trace roots, change
emitted Wasm or make the collector executable. Independent dry review is
clean, the standalone root-source structure guard passes `4/4`, the adjusted
named-slot guard remains green at `3/3`, and targeted formatting and diff
checks pass. Package owner witnesses and compilation remain deferred to the
shared batch checkpoint.

The collector contract's eight required phases now use one closed
`RequiredHeapCollectorPhase` domain instead of rows containing an arbitrary
name, a separately chosen kind and a writable `required_for_gc_builtin`
Boolean. One exhaustive projection owns the exact diagnostic names, while the
ordered registry and `HeapCollectorPolicy::required_phases()` accept only the
required phase type. A phase can no longer be misnamed or marked optional, and
adding a variant requires an explicit name choice. The focused
[contract](../docs/rust-rewrite/contracts/heap-collector-phase-authority.md)
and structure regression pin the exact eight variants, projection, registry
and typed contract producer. This remains passive metadata and does not
implement collection or expose `gc()`. Independent dry review is clean, the
standalone phase guard passes `4/4`, the adjusted weak-edge guard remains green
at `2/2`, and targeted formatting and diff checks pass. Package owner witnesses
and compilation remain deferred to the shared batch checkpoint.

The host-memory boundary now uses the sole closed
`HeapHostBoundaryPolicy::ImportCallOnlyWithTransientTaggedRoots` variant
instead of an arbitrary name, a single-variant duration and independently
writable durable-pointer and re-entrancy-root Booleans. Exhaustive projections
derive the diagnostic name and typed
`HeapRootSource::HostBorrowedValues` identity; its existing root metadata keeps
the transient tagged classification authoritative. Durable host pointers and
unrooted re-entrancy have no representable policy state. The focused
[contract](../docs/rust-rewrite/contracts/heap-host-boundary-policy.md) and
structure regression pin the domain, projections, producer, forbidden-field
absence and typed heap-owner consumption. This remains passive metadata and
does not alter host imports, root execution or emitted Wasm. Independent dry
review is clean; the standalone host-policy, root-source and collector-phase
guards each pass `4/4`, and targeted formatting and diff checks pass. The
package owner witness and compilation remain deferred to the shared batch
checkpoint.

The passive value ABI registry now uses twelve closed `HeapValueEncoding`
identities instead of rows that independently combine a `ValueKind`, payload
and Number-bit/arbitrary-precision Booleans. Four exhaustive no-wildcard
projections derive those meanings, so adding an identity requires explicit
choices for every representation claim. Number remains the sole
`Ieee754Bits`/bit-preserving identity, while BigInt remains
`I64TemporaryOrHeapPointer` and explicitly not arbitrary-precision-ready. The
unused standalone `I64Temporary` payload variant is deleted. The focused
[contract](../docs/rust-rewrite/contracts/heap-value-encoding-authority.md)
and recursive structure regression pin the exact domains, projections,
mappings, registry order and heap-owner witnesses. This remains passive
metadata and does not change the emitted value ABI or close the BigInt gap.
Independent dry review is clean; the standalone value-encoding,
collector-phase and host-policy guards each pass `4/4`, the exact package owner
witness passes `1/1`, and targeted formatting and diff checks pass. Broader
workspace and conformance verification remain deferred to the shared batch
checkpoint.

The passive weak-edge registry now stores seven closed `HeapWeakEdge`
identities instead of rows independently combining record and slot strings
with `HeapWeakEdgeKind`. One private exhaustive no-wildcard metadata projection
owns all seven exact mappings, while the existing exhaustive kind projection
remains the sole retention authority. The collector policy returns only the
typed identity slice, so a WeakMap, WeakSet, WeakRef or FinalizationRegistry
slot cannot silently select an unrelated edge kind. The focused
[contract](../docs/rust-rewrite/contracts/heap-weak-edge-identity-authority.md)
and recursive structure guard pin both domains, both projections, every
mapping, exact registry order, retired-row absence and the typed collector
consumer. This remains passive metadata: it does not implement tracing,
ephemeron processing, weak clearing, cleanup scheduling or executable `gc()`.
The recursive structure guard passes `4/4`, the adjusted named-slot guard
remains green at `3/3`, and the exact registry, WeakMap, WeakRef,
FinalizationRegistry and unsupported-collector owner witnesses each pass
`1/1` with only the workspace's existing warnings. Targeted formatting and
diff checks pass. Broad workspace and conformance verification remain deferred
to the shared batch checkpoint.

The passive collector selection now uses the sole closed
`HeapCollectorPolicy::NonMovingMetadataChecked` identity instead of a contract
that independently stored an arbitrary name, movement Boolean, capability and
three registry slices. Six exhaustive no-wildcard projections preserve the
exact `non-moving-tracing-collector` name, non-moving behavior, root sources,
weak edges, required phases and non-executable state. The old capability type
and its unused documented-only/executable states are deleted, so exposing a
real collector requires a new explicit policy identity and complete projection
coverage rather than one field edit. The focused
[contract](../docs/rust-rewrite/contracts/heap-collector-policy-authority.md)
and recursive structure guard pin the closed domain, every projection, heap
delegation and host unsupported boundary. This remains passive metadata and
does not implement or expose collection. The policy, phase and weak-edge
guards each pass `4/4`, the adjusted named-slot guard remains green at `3/3`,
and the exact phase-inventory, unsupported-policy and emitted host-GC throw
witnesses each pass `1/1` with only the workspace's existing warnings.
Targeted formatting and diff checks pass. Broad workspace and conformance
verification remain deferred to the shared batch checkpoint.

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

Promise internal builtin closures now store their immutable algorithm capture
in a dedicated function-header pointer slot instead of overloading the lexical
environment handle. The function-object size is 312 bytes; both the shared and
direct bound-function allocators initialize the GC-visible slot. Promise
closures self-back their environment handle so Realm/error header reads remain
well-typed. This is a linear-heap layout/root census update, not semantic GC or
weak-reachability closure.

Ordinary async-function activations now have a 144-byte registered layout with
a traced defining-Realm slot at offset 136. Captured async reactions and direct
async Promise allocation retain that edge across suspension. Async-generator
activations reuse their existing traced function-object edge and add no
duplicate Realm slot. This remains passive linear-heap pointer metadata; it
does not implement collection or close T05 rooting. The shared 663-dump
semantic golden adds the three focused Realm witnesses, removes none and
preserves all 660 retained structural summaries after expected code-size and
local-accounting fields are normalized.

Realm intrinsic records now include a traced canonical `%Promise%` constructor
edge at offset 416 and occupy 424 bytes. Entry and created-Realm bootstrap write
the exact constructor identity into that slot; async-generator requests load it
without adding a duplicate Realm edge to their existing 56-byte request record.
This extends the linear-heap root census only and does not close T05 collection
or weak-reachability work. The subsequent 664-dump semantic golden passes
`2/2`, adds only the Temporal field-mode fixture, removes none and changes no
retained non-accounting summary except the strengthened async Realm witness's
five intentional internal/name entries.

The standard `Promise.all`, `Promise.allSettled` and `Promise.any` outer
result/error Array now consumes the existing opaque current-function Realm
Array-prototype proof. The change adds no heap field or trace edge: it reuses
the Realm intrinsic Array slot and the combinator shared context's existing
values pointer. Missing nonentry catalog state traps instead of substituting an
entry prototype. This remains a Realm ownership correction, not collection or
weak-reachability closure. The following 665-dump semantic golden passes `2/2`,
adds only the RegExp result-mode witness and removes none. All retained dumps
except the intentionally expanded Promise witness preserve their non-accounting
summary, confirming that the Realm correction adds no heap-layout surface.

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

Observed BigInt values now cross the shared engine boundary as an
`ObservedBigInt` with private canonical decimal text. Its evidence-bearing
parser accepts only `0`, nonzero unsigned decimals and negative nonzero
decimals, so malformed text, leading zeros and negative zero cannot enter the
typed observation or differential protocol. Both Wasm producers and the
spec-exec producer parse before construction, while the differential projection
can only read canonical text. The bounded structure target passes `3/3`, the
runtime grammar target passes `1/1`, and the focused engine, spec-exec and two
differential witnesses pass `1/1` each. The shared workspace compile and every
repository policy gate pass, and all 648 Wasm-golden artifacts are byte-identical
to the post-Iterator baseline. No broader conformance run was performed.

A Wasmtime setup failure now retains one optional `WasmtimeRuntimePolicy`
instead of independently optional GC and weak-reachability capabilities. Both
public capability accessors project exhaustively from that single policy, so a
setup error cannot expose only half of the required product policy or combine
capabilities from different policies. The bounded structure target passes
`3/3`, the focused setup/ordinary-error projection witness passes `1/1`, and
the existing dual-native-engine policy witness and Wasm `gc()` capability CLI
witness each pass `1/1`. Broad workspace, golden, policy and Test262 gates
remain deferred to the centralized verification pass.
