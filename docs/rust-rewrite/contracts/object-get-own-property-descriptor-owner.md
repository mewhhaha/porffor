# Object.getOwnPropertyDescriptor compiler owner

The 2026-10-05 source draft keeps the native entry in the private
`builtins/object/get_own_property_descriptor.rs` owner and Proxy processing in
its private `proxy.rs` child. The standard dispatcher calls the entry directly.

The entry completes ToObject before ToPropertyKey. Non-Proxy objects use the
single shared non-Proxy GetOwnProperty storage kernel, including Array,
Arguments, String boxes, TypedArray, functions and module namespaces. The
nullable typed PropertyDescriptor result represents absence. Present records
are published through the sole FromPropertyDescriptor owner as fresh ordinary
objects with the executing Realm's Object prototype. Values and getter/setter
identities remain complete GC values.

Proxy processing keeps the normative trap, target and conversion order. It
acquires GetMethod and calls the trap with the handler receiver. An invalid
primitive trap result is rejected before target observations. Target
GetOwnProperty completes recursively before IsExtensible and observable
ToPropertyDescriptor. An undefined trap result with an absent target descriptor
returns without IsExtensible. Present trap descriptors are completed once,
validated by the shared compatibility kernel and the remaining nonconfigurable
and nonwritable invariants, and published as fresh objects. Missing traps
forward through the private NativeObjectAlgorithm call authority. No public
Object/Reflect property lookup or scalar argument adapter supplies that call.

The execution-Realm selector also serves generated internal-method helper
bodies that have no ordinary callable entry. Their nearest defining environment
or active Realm supplies the native callable; normal builtin bodies use their
actual FunctionContext Realm.

The historical 2026-08-28 ownership checkpoint verified the previous source.
Its results do not verify this GC rewrite. The authored
`lila-engine/tests/aot_gc_object_entries.rs` controls cover recursive Proxy step
order, fresh results and original Throw identity; existing semantic descriptor
controls remain required. Obsolete source-spelling and linear-heap guards are
retired. The draft has had no compilation, emitted-Wasm validation or execution.
Run the complete implementation checkpoint under the confirmed 4096 MiB process
tree cap before assigning acceptance or changing published conformance counts.
