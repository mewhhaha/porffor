# Collector weak edges

The selected Wasmtime 47.0.0 copying collector has a local weak-edge patch in
[`vendor/wasmtime-47.0.0`](../../../vendor/wasmtime-47.0.0/PATCHES.md).
`WeakAnyRef` and `GcEphemeron` expose Store-scoped generational handles, never
unrooted references. Only the copying collector accepts these registrations.
Owner, target and key must be actual heap references; values may also be i31 or
null. A foreign Store is an error, and retiring a slot increments its generation
without wraparound or reuse at generation exhaustion.
The copyable handles do not deregister on drop. Entry deletion explicitly calls
`remove`; otherwise a surviving owner and key still retain a conditional value.

The registry belongs to the Store and stays outside the strong-root census.
Strong tracing runs first. A conditional value is marked only after its owner
and key are independently live. Newly marked values join the ordinary worklist;
complete ephemeron passes repeat until no additional object is marked. Weak
updates then forward surviving references, clear dead targets and retire dead
owners or keys. ExternRef sweeping follows this fixed point and update phase.
Dropping an in-progress copying collection finishes every phase after the space
flip, including conditional marking and weak updates.

An upgrade reads its slot and creates a Wasmtime root under one `AutoAssertNoGc`
guard. JavaScript's separate keep-during-job rule must retain successful
WeakRef construction and dereference results through the current job, then
release those roots at the real job boundary. Finalization requires weak targets
and tokens, strong holdings, and ordinary cleanup-job scheduling in the correct
Realm. Collector support alone does not implement those JavaScript algorithms.

The current product capability remains `WasmWeakReachabilityCapability::Unavailable`
until all four weak families, their host ABI and job integration are complete.
The earlier
[unavailable-facility contract](weak-unavailable-runtime-boundary.md) still
governs successful product weak operations. No second JavaScript heap or
strong-retaining weak substitute is introduced.

The native collector controls in
[`weak_gc_facility/main.rs`](../../../crates/lila-engine/tests/weak_gc_facility/main.rs)
force actual collections and cover increment-boundary scans, dead targets, moving identity, reverse
ephemeron chains, dead owners with live keys, value/key cycles, stale and foreign
handles, explicit kept roots, ExternRef sweeping and null conditional values.
The vendored unit control drops actual collections at thirteen increment
boundaries and checks both surviving and subsequently dead references. These
controls are separate from complete pinned weak-tree and full-suite acceptance.
