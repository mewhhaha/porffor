# Reflect consumes the shared descriptor owner

Source authored 2026-10-04 in the atomic T05 draft.

Native Reflect invokes emit_to_property_descriptor once, after ToPropertyKey. ReservedPropertyDescriptorLocals supplies a borrowed validated descriptor to emit_object_define_entry_validated; the owner retains converted roots through the operation and then clears roots and flags. Reflect has no second Proxy invariant algorithm or raw descriptor layout. Normal false and whole Throw remain distinct.

Existing aot_reflect_completed_descriptors, aot_proxy_target_descriptors and Realm controls remain required. Three new GC entry controls cover identity, order, admission and borrowed Realms.

The obsolete scalar source spelling guards are retired. All controls are unrun
for this source; Rust types, Wasm validation, runtime and conformance proofs are
null. Complete the all-task source batch before executable verification. Later
checks require a confirmed 4096 MiB aggregate kernel cgroup cap, zero swap,
grouped OOM termination, one CPU and serial workers.
