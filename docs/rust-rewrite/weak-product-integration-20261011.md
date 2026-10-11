# JavaScript weak integration — 2026-10-11 verified selection

All sixteen WeakMap, WeakSet, WeakRef and FinalizationRegistry routes now use
actual GC brands and the patched copying collector. Native resources hold weak
slot identities; WeakMap values are conditional complete GC StoredValues.
WeakRef construction and dereference keep successful targets through the job.
The script/module epilogue and each dequeued job clear that kept-root set.

Finalization registries have strong holdings, weak targets and weak unregister
tokens. Completed-job polling discovers cleared cells and appends a cleanup job
in the registry's captured Realm. Each cleanup removes a cell before calling
user code and reads the list again afterward. Callback throws stop that cleanup
invocation, produce a distinct ReportedException host event and preserve the
script completion. The CLI reports these errors on stderr. The pinned
[ECMA authority](https://github.com/tc39/ecma262/blob/5345883164f463e87f8b40aca4956157ecba8783/spec.html#sec-cleanup-finalization-registry)
permits this boundary polling policy and makes callback invocation abrupt via
HostCallJobCallback.

Seven new native product controls cover actual collections and job boundaries,
conditional chains and cycles, dead collection owners, cleanup queue ordering,
unregistering cleared cells, reentrant removal and callback errors. Existing
created-Realm/early-error controls now require successful weak construction.
Nineteen focused native controls pass with zero failures or ignores: ten weak
product/Realm/early-error controls, three runtime-policy controls, two honest
Test262 positive/negative-accounting controls and four CLI controls. Joined
all-feature/all-target workspace types and all twelve repository guards pass.
Native execution uses one CPU,
one libtest worker, include-ignored and the original fixture deadlines.

The complete weak-feature source census contains 339 files and 677 execution
IDs: 603 IDs in the four builtin directories plus 74 tagged cross-family,
subclassing, object and staging IDs. All 677 pass in Wasm AOT, as do all 70
Error stack proposal IDs: 747 exact IDs, zero failures and zero timeouts.
The execution keeps four case workers, one compilation job, the original
60,000 ms execution timeout and the existing bounded cold compilation allowance.
The first outer watcher interrupted the run because child checkpoints went to
private log files. Its 240 successful WeakMap checkpoints were retained and
revalidated by resume against the same compiler, sources and exact case set;
the retry forwards progress to the watcher. This is a complete selected run
with validated reuse. Full suites and task closure remain open; the canonical
pinned-suite denominator is unchanged.

The [receipt](weak-product-integration-20261011.json) retains the successful
executable digests, exact native inventories, unchanged-source identities and
preceding failed compiler/link attempts. The native link crashed near a full
disk; after obsolete derived caches were pruned, its retry built in 2.534 seconds
and all ten product controls passed. The typed root fixes and exhaustive output
consumers are included in the verified source.

See the [collector contract](contracts/collector-weak-edges.md) and the separately
[verified collector foundation](weak-collector-foundation-20261011.md).
