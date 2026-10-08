# TypedArray intrinsic identity and prototype publication

## Concrete GC constructor bodies — 2026-10-07

Every compiled concrete constructor now owns the body for its
`TypedArrayElementKind`. Planning, emission, function counts and diagnostic names
consume the same compiled-builtin list; there is no second list that collapses
these distinct bodies. Existing emitted-entry checks require every body's actual
index to agree with its planned entry. Shared unsupported stubs retain their
separate declared role.

The old planner reused the first concrete constructor's Wasm entry for all
twelve kinds. After the GC constructor cutover, that entry embedded its element
kind: `new Int32Array(new SharedArrayBuffer(8))` produced Float64 storage of
length one while retaining the requested Int32 prototype. A bounded native
diagnostic observed that brand, length, shared-buffer identity and rejection by
both Atomics.load and waitAsync. The correction retains the actual specialized
bodies, without deriving element kind from mutable properties or `newTarget`.

The new native storage control covers all twelve kinds in entry and created
Realms, scalar-length and shared-buffer construction, element conversion and
Reflect.construct with a different newTarget prototype. Its joined checkpoint
also reruns the original waiter progress control. `tasks-constructor-progress1`
passes the workspace type check and both native controls, plus the corrected
all-Symbol catalog control, under one CPU and 4096 MiB with none ignored.
Earlier prototype results below retain their historical scope.

Status: implemented for the entry- and created-Realm hidden `%TypedArray%`
constructor identities and their prototype graphs.

## Boundary

`%TypedArray%` receives its spec-owned prototype through
`FunctionPrototypeMaterialization::BootstrapSupplied`. Bootstrap then appends
the constructor's own `prototype` property once with attributes
`{ writable: false, enumerable: false, configurable: false }`. The constructor
header and every accessor and method publication refer to the same
`TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX` object in the entry Realm and the same
Realm-local object in a created Realm.

Automatic prototype materialization is forbidden at this boundary. It would
create a non-configurable `prototype` property and a separate prototype object
before bootstrap knows the intrinsic object. Replacing that property through
ordinary `ValidateAndApplyPropertyDescriptor` is correctly rejected and must
not be made possible by weakening descriptor validation.

The generator-function intrinsic bootstrap is the neighboring authority for
this birth-time publication pattern.

`StandardBuiltinId::TypedArrayConstructor` is the only compiler identity for
the hidden constructor. It is absent from the global catalog, has native name
`TypedArray`, length zero and an ordinary call-and-construct protocol. Its
`ALWAYS_THROWS` catalog fact records the separate semantic invariant that both
direct call and direct construction end abruptly. This distinction is
required: the value must pass `IsConstructor` when used only as the `newTarget`
of another constructor.

`Object.getPrototypeOf` on the closed concrete typed-array constructor family
returns this exact target and its constructable function shape. Normal-result
joins omit catalog targets marked `ALWAYS_THROWS`; they do not invent an
`undefined` completion for the hidden constructor. The Wasm body always throws
a `TypeError` through the current-function-Realm route.

Created-Realm allocation uses the same bootstrap-supplied prototype policy,
then gives the function a self-backed Realm environment carrying that Realm's
`TypeError.prototype`. The function's internal prototype is the created
Realm's `%Function.prototype%`; its own `prototype` property is
non-writable/non-enumerable/non-configurable, and
`%TypedArray.prototype%.constructor` is writable/non-enumerable/configurable.

## Evidence

`typed_array_intrinsic_prototype_publication_structure.rs` pins the
bootstrap-supplied allocation, initial descriptor flags, absence of an ordinary
redefinition, and the shared prototype publication target. The existing
`wasm_typedarray_iterators.js` CLI fixture observes
`Object.getPrototypeOf(Uint8Array).prototype`, its iterator method descriptors,
and the `@@iterator` alias before exercising iteration.

On 2026-08-27, the structure target passed `3/3`, the CLI fixture passed `1/1`,
and the pinned `%TypedArray%.prototype`, `constructor`, `values`, `keys` and
`entries` descriptor leaves passed both variants (`10/10`) with every failure
bucket at zero. The following shared 683-dump semantic golden passed `2/2` in
655.10 seconds, adding and removing none. After accounting normalization, 682
of 683 retained summaries are equal; the sole structural change is the
independently expanded Array iterator corruption witness.

`typed_array_constructor_identity_structure.rs` additionally pins the hidden
catalog row, planner dependency, preallocated entry-Realm identity, defining-
Realm throw body, closed string-pool entry and created-Realm materialization.
`wasm_typedarray_intrinsic_identity.js` observes all eleven concrete
constructors in the entry and one created Realm. It checks distinct Realm-local
identities, native source, name/length/prototype/constructor descriptors,
direct-call and direct-construct errors, and both `Reflect.construct` target
and `newTarget` behavior. On 2026-08-31, the focused IR regressions passed
`4/4`, the identity structure target passed `5/5`, and the Wasm-AOT fixture
passed `1/1`. The six exact pinned `name`, `length`, `invoked`, `prototype`,
`prototype/constructor` and concrete `Uint8Array/proto` leaves pass both
variants (`12/12`) with every non-success bucket at zero.

## Deferrals

This contract does not claim the complete T17 surface. In particular, it does
not close all static/prototype algorithms, resizable-buffer observations,
species/default-constructor selection, Float16 support, SharedArrayBuffer or
Atomics/agent behavior. Created-Realm publication of the hidden constructor's
`from`, `of` and `@@species` members also remains separate surface-parity work.
