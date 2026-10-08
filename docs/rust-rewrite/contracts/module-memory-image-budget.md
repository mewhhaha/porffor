# In-process native module image budget — source batch, 2026-10-03

The engine's process-local native Wasmtime module cache has one private owner
for its LRU entries, measured image sizes and aggregate byte count. The normal
retaining engine/CLI path uses that owner for lookup and post-compilation
admission. Cache keys still include the input-Wasm SHA256 and native compiler
mode; Wasmtime compilation stays outside the cache mutex. The current-thread,
agent and native-compiler fallback bypass policies are unchanged.

The default limits are 64 entries and 512 MiB of retained compilation images.
`LILA_MODULE_MEMORY_CACHE_ENTRIES` keeps its existing positive entry override;
`LILA_MODULE_MEMORY_CACHE_LIMIT_BYTES` supplies the positive byte override.
Both accept decimal `usize` values after trimming whitespace. Unset, invalid,
blank, zero, negative or overflowing values use their respective defaults.
The process snapshots the limits when this cache is first initialized. Positive
limits use `NonZeroUsize`; outside callers cannot mutate limits, entries or the
aggregate independently.

Each admitted module is measured from immutable
[`Module::image_range`](https://docs.wasmtime.dev/api/wasmtime/struct.Module.html#method.image_range).
The pinned [Wasmtime 47 implementation](https://github.com/bytecodealliance/wasmtime/blob/v47.0.0/crates/wasmtime/src/runtime/module.rs)
provides this API and includes code, data and debug information in that image.
The engine observes pointer addresses and subtracts them with checked integer
arithmetic. It neither dereferences nor changes the image and does not serialize
the module or use input-Wasm byte length as a proxy. An unmeasurable range
bypasses retention while leaving the execution module intact.

The retained byte total is the exact sum of the image sizes on cache entries.
A hit promotes the same entry to MRU and preserves its accounting. A miss
compiles outside the mutex and checks again on admission: if another worker
already retained that key, the existing entry is promoted and reused without a
second image charge. The initial lookup still determines Hit/Miss tracing.

A module larger than the entire byte budget executes normally and does not
enter the cache or evict hot entries. Other admissions evict LRU entries until
both limits permit the new image. Subtraction-based available capacity avoids
overflow even with a `usize::MAX` override; checked subtraction on eviction and
addition on admission retain exact totals. A valid image exactly filling the
remaining budget is admitted. The entry cap still bounds zero-byte images.

This addresses the documented entry-only in-process retention gap. Disk-cache
budgets, emitted-Wasm caching, compiler selection and error behavior retain
their existing owners. The budget limits images referenced by this cache.
Active module/instance clones can keep evicted images alive; compilation work,
other Wasmtime allocations, JavaScript/Wasm heaps and process RSS are not bounded
by this cache policy. The change does not establish that earlier OOM runs now
complete or that all process memory is capped.

Seven authored deterministic unit tests cover positive parsing/defaults,
capacity at integer boundaries, entry eviction, multiple byte evictions to an
exact boundary, hit/miss LRU behavior, oversized nonretention preserving hot
entries and post-miss duplicate reuse/accounting. Their isolated cache instances
derive budgets from actual compiled image ranges. Small empty/passive-data
modules are created only when these tests are later executed; the tests do not
instantiate modules or execute guest code. The existing current-thread bypass
test observes the actual owner through its read-only test method; the compiler
mode key test is unchanged.

The batch is authored source only. Source inspection, direct formatting and
ordinary patch correspondence are hygiene, not runtime evidence. No compilation,
Wasm validation, test execution, memory/RSS experiment, CLI suite, product
generator or conformance/status refresh ran. Rust checks, the new owner tests,
existing compiler-mode/bypass controls and combined focused/broad acceptance
remain pending. Root owns the shared README, task and batch-workflow updates.
