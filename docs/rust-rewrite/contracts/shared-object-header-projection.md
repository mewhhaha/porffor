# Shared actual object-header projection

Every generic FunctionBuilder projection from a Value to its OrdinaryObject
header calls the typed `ObjectHeaderProjection` Wasm helper. The actual result
is a non-null GC reference. A schema-owned converter consumes only its
registered `ReferenceHelperResult` to produce the existing GcStackReference;
neither a numeric index nor a raw cast can create that result.

The complete physical dispatch is private to
`gc_types/value/object_header_projection.rs`, beside its sole body compiler.
It walks the original exhaustive `GcLayout::ALL` and `object_projection`
authority. OrdinaryObject projects itself; each composite projects its declared
immutable OBJECT field. No alternate layout list, guessed field ordinal or
parallel object representation is introduced. A non-object reference retains
the original compiler-invariant trap.

Bootstrap data and accessor installers, constructor/global installation,
property operations, descriptor validation, private elements and collection
hashing consume the same helper. Their existing preconditions, ordering,
completion routing and rooted locals remain in their original owners. The
facade takes the original value/function inputs and returns the original stack
reference shape. Its registered Wasm-GC helper-plan requirement is the existing
backend invariant; it does not create a JavaScript failure or a Realm lookup.
The projection executes no user operation and never changes the active Realm.

The original nested Array/request artifact requires one projection body and a
real direct call from Realm bootstrap. It retains the 512 KiB helper/source
ceilings and now requires every encoded body in that fixture to stay at or below
1 MiB, including main and native builtins. Largest-body diagnostics precede size
assertions so a new failure identifies its real source owner. This is an
emitted-size regression requirement, not an assertion that Cranelift memory or
runtime semantics have already passed. The new shared projection is uncompiled
and unexecuted until the parent's capped checkpoint.
