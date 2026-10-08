# Reflect optional arguments use presence

Source authored 2026-10-04 in the atomic T05 draft.

The closed ReflectReceiverOperation selects receivers: omission copies the target; explicit Undefined retains that receiver. Construct validates its target first, then defaults only an omitted newTarget. An explicit Undefined newTarget fails IsConstructor before CreateListFromArrayLike reads length. Apply validates IsCallable before that list operation. The shared argument-list owner preserves ordered Gets, whole thrown values and element reference identity.

The new aot_gc_reflect_entries controls cover target/newTarget admission before list reads and omitted versus explicit receivers. Existing semantic controls remain mandatory.

The obsolete scalar source spelling guards are retired. All controls are unrun
for this source; Rust types, Wasm validation, runtime and conformance proofs are
null. Complete the all-task source batch before executable verification. Later
checks require a confirmed 4096 MiB aggregate kernel cgroup cap, zero swap,
grouped OOM termination, one CPU and serial workers.
