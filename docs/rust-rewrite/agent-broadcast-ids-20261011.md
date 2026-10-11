# Native agent broadcast IDs — 2026-10-11

Test262's agent protocol carries an Int32 or a BigInt. The former scalar bridge
always applied ToNumber and rejected primitive BigInt IDs. The native command
now owns a closed primitive ID: Int32 or a canonical ObservedBigInt. Each delivery
retains its own data; no JavaScript collector root leaves its Store.

The sender validates the SharedArrayBuffer before ID conversion. Primitive
BigInts use the existing primitive string conversion and a GC byte-array wire.
Other inputs retain ToNumber/ToInt32 and their ordinary abrupt completions.
Null bytes select Int32; non-null canonical decimal bytes select BigInt and
require an empty scalar carrier. The two host signatures reside in the runtime
GC recursion group. Native admission retains the module's canonical byte-array
type and rejects ABI lookalikes before linking.

The receiving host validates the shared backing, roots a local ExternRef and,
for a BigInt, allocates its own canonical GC byte array before acknowledging
retrieval. Wasm reconstructs the primitive with the existing UTF-8 reader and
BigInt parser. Mutable BigInt globals/prototype conversion hooks cannot alter
message identity. JavaScript buffer wrappers remain local and distinct across
deliveries; the native backing remains shared. Broadcast's internal function
length now reports both arguments.

The native cohorts cover two workers, seven consecutive IDs, positive/negative
zero and magnitudes beyond 64 bits, negative IDs, Number conversion, shared backing, local prototypes
and distinct wrappers. The separate coercion cohort preserves invalid-buffer
ordering, a throwing object conversion, boxed BigInt and Symbol rejection.
Canonical wire/admission controls retain wrong-arity, wrong-width, nullability,
heap-kind, packed-storage and mutability rejection.

All 22 focused agent controls pass with zero failures or ignores in 55.30 test
seconds (849 unrelated controls filtered). This includes both real native
worker cohorts, canonical ABI admission and all preceding retrieval, parent
deadline, shared-memory, waitAsync and failure-owner controls. The all-feature
Engine test build took 212.670 seconds. Compilation used three bounded cloud
CPUs; the actual test executable inherited one CPU. Joined all-feature/all-target
workspace types and the eleven repository guards also pass. Exact source,
commands, executable and log hashes are in the [JSON receipt](agent-broadcast-ids-20261011.json).

The preceding frozen baseline completed 169 of 245 workspace scopes: 4,228
passes, zero failures and two schematic documentation ignores, including all
867 Engine library controls. It was deliberately interrupted during aot_async
to move the remaining broad checks onto this combined source. Its partial
integration scope has no completed verdict; it is not a full-suite pass. The
private source/termination records and original transcripts remain intact.

The complete current pinned Atomics selection is 764 executions at
`aa55200d1310384c5cf69ea95b2a2ecba457007b`; it still needs a fresh current-image
run. Full workspace, pinned acceptance and task closure remain pending;
publisher totals and task statuses are unchanged. Restoring the inactive Engine
incremental cache preserved its exact file contents and took 2.939 seconds;
this cache operation supplies no general compiler speed claim.
