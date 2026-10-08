# Reflect descriptor objects retain the executing Realm

Source authored 2026-10-04 in the atomic T05 draft.

Reflect.getOwnPropertyDescriptor invokes the sole native object descriptor algorithm through NativeObjectAlgorithm::GetOwnPropertyDescriptor. The call materializes checked native code in the executing function Realm, retains whole arguments and completion, and reads no public method property. Fresh descriptor objects use that Realm Object prototype. Proxy defineProperty descriptors come from the shared internal-method owner and its executing context. The duplicate scalar prototype loader is retired.

Existing reflect_descriptor_object_realm and aot_reflect_completed_descriptors controls remain required. The three new aot_gc_reflect_entries controls check identity, admission, errors and borrowed-Realm descriptor/Array results.

The obsolete scalar source spelling guards are retired. All controls are unrun
for this source; Rust types, Wasm validation, runtime and conformance proofs are
null. Complete the all-task source batch before executable verification. Later
checks require a confirmed 4096 MiB aggregate kernel cgroup cap, zero swap,
grouped OOM termination, one CPU and serial workers.
