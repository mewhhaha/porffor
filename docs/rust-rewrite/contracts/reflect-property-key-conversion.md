# Reflect property keys retain their value and order

Source authored 2026-10-04 in the atomic T05 draft.

Native property methods receive ReflectObjectTarget only in the successful object branch. They then use the sole shared ToPropertyKey authority, preserving String-hint coercion and Symbol identity. DefineProperty converts attributes after the key. Get/Set retain a separate whole receiver. Shared object owners retain Proxy and exotic behavior.

The new aot_gc_reflect_entries controls cover rejection before key conversion, unchanged throw identity, single Symbol conversion and key-before-descriptor ordering. Existing completed descriptor and Proxy controls remain mandatory.

The obsolete scalar source spelling guards are retired. All controls are unrun
for this source; Rust types, Wasm validation, runtime and conformance proofs are
null. Complete the all-task source batch before executable verification. Later
checks require a confirmed 4096 MiB aggregate kernel cgroup cap, zero swap,
grouped OOM termination, one CPU and serial workers.
