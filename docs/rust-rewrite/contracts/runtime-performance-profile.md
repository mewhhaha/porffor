# Product runtime performance observations

Status: invocation-scoped RSS sampling and version-two report admission pass the
combined workspace type check. Three Engine sampler controls and all 24 CLI
performance controls pass. Native runtime-profile failure/lifetime controls and
benchmarks remain pending. No status refresh ran for this slice; these results
do not establish calibrated T25 budgets or sustained acceptance.

After the coherent source batch is verified, the opt-in command is:

```sh
python3 scripts/limited_verification.py -- \
  ./target/release/lila performance runtime \
  --output-dir target/performance/runtime-candidate --samples 7 \
  --machine-label dedicated-benchmark-host --idle-machine
```

The original fixed twenty-case corpus supplies the exact ordered filenames and
embedded bytes. Every selected fixture must match those bytes. The reporter
uses the existing identity readers, one compilation worker and serial execution.
The output directory must be fresh. `--fixture-root PATH` may select another
copy of the same corpus; its canonical filename root participates in identity.
There are no new generated workloads, input ceilings or timing budgets.
`--gate` belongs to the original whole-CLI report and is rejected here.

Each case compiles once through the product parse, IR, lowering, Wasm emission
and validation pipeline, outside the runtime samples. Its actual Wasm bytes,
SHA-256 and validated section footprint are retained. One unmeasured warmup
precedes 3..100 measured repetitions of those same bytes. Each repetition calls
`Engine::profile_wasm_execution` and constructs a fresh original Wasmtime Store.
The runtime's normal native-module retention and disk/function caches remain
enabled, and each sample records the actual module-memory-cache outcome. A
warmup does not promise a cache hit; capacity limits or ambient configuration
can cause misses. No reporter cache or alternate runtime is introduced.

Function-stencil and program-Wasm disk hits refresh recency through the same
`CacheStore::get` owner, including on `noatime` filesystems. Timestamp updates
are best effort: failure cannot discard successfully read bytes. Cranelift's
stencil/target/version identity and the configured cache limits remain intact.
All sixteen cache checks pass in `tasks-dry-closure-foundation2`, including
pruning after both actual entry points read a file whose access time stays old.

Pooled string slots now retain their checked insertion indices. Initializers,
ordinary literal reads and collected source diagnostics use that same stored
index; duplicate interning adds no slot, and literal lookup uses the existing
ordered map directly. An unrelated source string can no longer renumber the
builtin literals already collected before it. Collection order, UTF-16 data,
static offsets and native cache identity stay owned by their existing paths.
This does not promise stable late-collected literals or static table offsets
across arbitrary programs.

The retained native4 primitive artifacts expose the predecessor cost: their
type/import/function/export/element sections match, but 203 builtin bodies and
24 shared helpers differ. The 888,484-byte Duration round body differs at 13
pooled-string-index bytes. The existing two-emission literal-data control now
also checks byte-identical shared coercion code across source strings that sort
on opposite sides of builtin spellings. Slot/data correspondence, duplicates,
source-diagnostic resolution and the u32 table-length boundary have three unit
controls. All three and the artifact control pass with the joined workspace
type check on 2026-10-08. The actual shared helper bytes match across changed
source literals. Measured native cache/timing gains remain unverified, and the
original five-second campaign remains open.

Global identifier reads now share four typed Completion helpers for ordinary
Get/typeof and sloppy/strict source contexts. The caller selects its existing
source Global Environment and execution Realm, then transports those roots with
the source name and caller Environment. Only the registered helper parameter
can establish this execution-Realm authority. The private helper reuses the
original ResolveBinding/GetBindingValue/Reference-release owners: both object
HasProperty checks, live lexical delegate refresh, TDZ behavior and exact Throw
values remain ordered. References retained across RHS evaluation keep their
existing separate lifecycle; the helper does not own a source Environment or
manufacture a callable context.

The retained native5 expanded Switch artifact measured a 1,638,171-byte sloppy
main and a 1,201,087-byte strict main before this extraction. The sloppy native
case passed; strict compilation was deliberately stopped. The existing captured
Switch artifact control now emits that exact combined sloppy source and checks
one declaration of every read helper, actual Value/typeof call edges from main,
and the unchanged 1-MiB body threshold. The existing native global-read cohort
also covers typeof TDZ and four foreign-Realm read/error cases. These extraction
checks and any resulting compile-time improvement are pending verification.

Realm intrinsic initialization now has one emitted
`helper::realm_initialize_intrinsics` body, shared by entry bootstrap and created
Realms. The existing ordered initialization algorithm remains its sole producer.
Its typed arguments carry the selected Realm and caller Environment; native
function construction retains its explicit selected-Realm context and null
lexical/private captures. Six nullable OrdinaryObject results carry Reflect,
Math, JSON, Atomics, Temporal and Intl in that order. The helper places all six
references on the Wasm result stack before clearing its local owners; the caller
roots all six before reloading the same Realm's Object/Function prototypes.
The shared bootstrap plan determines absent namespaces on both sides.

Global publication stays at the caller: entry declarations and the created
Realm's canonical-only globals keep their original separate paths and ordering.
Agent-wide literal/symbol roots still initialize before entry bootstrap. The
zero-length Array prototype uses a fixed-zero entry to the same private array
allocation body, avoiding a JavaScript completion return inside the reference
helper; dynamic array lengths retain their existing range check and throw.
No GC layout, backend, cache identity, resource cap or native optimization
threshold changes. The exact optional-alias fixture supplies an authored artifact
guard for one shared helper, both direct call paths, and the existing 1-MiB body
threshold. The capped serial `tasks-realm-bootstrap1` checkpoint passes the module
guard, workspace all-feature/all-target type check, six runtime-helper controls
and ten selected AOT controls, with no failures or ignored tests. Its validated
artifact has one 662,558-byte initializer helper; main falls from 1,196,893 to
534,595 bytes and the ShadowRealm constructor from 686,818 to 24,632 bytes. Every
body is below 1 MiB, and both direct call edges pass. `native3` passes expanded
namespace isolation and the exact optional-alias fixture in both modes. The
optional-alias test takes 194.27 seconds versus the prior 364.18; native
compilation takes 75.157 / 72.615 seconds versus 160.616 / 140.921 for sloppy /
strict modes. The emitted sloppy module is 41,458,110 bytes. These are single
observed runs with cache differences, not calibrated performance acceptance.
The wider native queue and original five-second campaign remain open.

The SDK shares the actual product execution method, required copying collector,
imports, limits, epoch timeout mechanism, host work and structured completion
decoder. Only a requested runtime profile starts the memory sampler; ordinary
execution has no sampling thread or procfs reads. The default report supplies
no new timeout or acceptance budget.
Runtime phase spans measure engine lookup/setup, native-module compilation or
cache lookup, Store/linker setup, instantiation, export lookup and execution
through host work and completion decoding. Total starts at runtime artifact
admission and ends after completion roots are dropped and the sampler is joined.
Its sampler setup/retirement overhead is included. It excludes source
compilation, worker-thread creation, Store destruction, evidence writes and
summary/checkpoint work. Module start code, if present, belongs to instantiation.
These exact spans are retained separately; they are not general throughput,
operation-count or statistical-significance claims.

Pinned Wasmtime 47 exposes `Store::gc_heap_capacity()`. The SDK samples that
actual byte capacity after instantiation/before the main call and after
completion decoding/host-root release. The report names it **capacity**, in byte
units, and retains both snapshots for every warmup and sample. Capacity can be
zero when the Store has not initialized a GC heap. It can differ between
repetitions as the real allocator/collector behaves. It is neither allocated
bytes, live bytes, resident process memory nor a peak.

The selected runtime has no public per-Store allocation-count, allocated-byte,
live-byte, collection-count or pause-time counter. Those fields explicitly say
`wasmtime-has-no-public-counter`. Peak GC heap still says
`no-per-invocation-peak-sampler`. No collection is forced before or after a sample.
Explicit program `gc()` calls and automatic collection keep their original
product semantics; this profile does not turn observed explicit calls into a
total collector count.

On Linux, a private invocation owner reads current `VmRSS` from
`/proc/self/status` at entry, at a nominal 10-ms interval, and at retirement after
completion decoding/root release. Each bounded read admits only the current
RSS field in kernel kB units and converts it to bytes with checked arithmetic.
The first and final reads count even when an invocation finishes before the
first interval. The report retains initial/final RSS, maximum observed RSS,
sample count, nominal interval, actual sampling span and the closed
`runtime-invocation-process-rss` scope. The periodic loop waits at least one
interval between reads; scheduling and read time can lengthen that interval.
The observed maximum is a sampled lower bound on the true peak, not an exact
peak. Short-lived allocations can occur between samples.

This is whole-process residency during one invocation. Retained native modules,
other threads, the Rust host and the sampler itself contribute; it does not
attribute RSS growth to JavaScript, one Store or the collector. The sampler has
one 256-KiB-stack thread and bounded 64-KiB procfs records with a one-byte
oversize probe, retains only aggregate
observations, and owns its stop/join through RAII on success, error and unwind.
No lifetime `VmHWM`/`ru_maxrss`, cgroup-wide peak, reserved GC capacity or capacity
difference is substituted for an invocation observation. Unsupported platforms,
unreadable/malformed procfs, thread start/panic and sample-counter failures have
typed unavailable results; failed sampling never becomes numeric zero or a
partial successful maximum.

Linear-memory capacity is not added in this slice: the profiling API accepts
admitted artifacts beyond the normal product memory layout, and one exported
memory is not a complete owner of every private/shared memory. GC capacity
continues to use the two original snapshots described above.

Every successful invocation retains the original typed completion's category
and debug representation plus verbatim captured print events. Object/Symbol
observations remain category-only; no user coercion or backend address is used.
Different normal observations are retained independently because real programs
may observe time. An ECMAScript throw, trap, host failure or compiler failure
marks the report Failed and retains all completed prefixes. Interruptions leave
Running. Neither is admitted as a baseline or receives complete summaries.
An ECMAScript throw also retains its current completion and print observation
before failure; the failure reason names that file. Trap/host errors retain the
original product error and already completed measurements rather than inventing
a completion for an execution that never reached the observation boundary.

To compare a later compiler, add `--baseline PATH/report.json`. Baseline
admission precedes candidate compilation/execution. It requires the same exact
ordered workload, filename root, machine, toolchain/build, environment/resource
configuration, sample count, runtime capability report and measurement policy.
The compiler revision/image may differ and both identities are retained.
Admission rechecks retained source, Wasm and observation digests, validates Wasm
under the product runtime, recomputes its footprint and all summaries from raw
samples, and rejects unsupported metric claims or totals smaller than the sum
of their disjoint phases. Original baseline evidence remains in its directory;
candidate `baseline.json` retains the admitted report bytes.

The runtime report is version two and binds this sampling policy and scope.
Version-one reports have no equivalent memory observation or sampler overhead
and are refused as compatible baselines. Each retained invocation observation
includes its RSS result; admission checks its equality with the aggregate sample,
endpoint/max ordering, kernel byte units, interval/count bound and sampling span
within total invocation time. Existing source/Wasm/output evidence checks remain.
RSS summaries exclude warmup and exist only when every measured invocation in
that case has valid sampling. Otherwise typed reasons remain and no numeric RSS
budget metric or delta is published; successful raw samples are still retained.

Timing summaries use the existing min/max, median and nearest-rank p90/p95
rules. Capacity summaries use the same statistics in explicitly named byte
fields. Comparisons show signed timing and capacity changes and original/candidate
Wasm footprints. Available sampled-RSS maxima use byte medians/p95 and the
`sampled-process-rss` budget metric under an explicit policy. This metric is not
an allocation or live-GC budget. Timing ratios preserve explicit null for zero baselines. No
green/red performance limit is inferred from completing the planned samples.
The existing one-second/five-second whole-CLI gates and wider T25 runtime,
allocation, GC, fuzz and CI obligations remain separate acceptance work.

Run this command alone under the existing one-CPU/4096-MiB/no-swap outer
launcher. `--idle-machine` records operator acknowledgment, not proof of
isolation or absence of other activity. There is no nested launcher or uncapped
fallback. The report checkpoint and workload/artifact/observation files use the
same fresh-write and retained-evidence rules as the compiler/whole-CLI owners.
