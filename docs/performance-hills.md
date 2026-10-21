# Performance hills — 2026-10-21

This backlog contains **160 hills, P001–P160**, for compiled JavaScript
on Scriptw's Wasm/Wasmtime path. It is separate from the 100 completed
[maintenance hills](maintenance-hills.md). The progress ledger below distinguishes
experiments from measured improvements. Source inspection supplies
hypotheses; profiling decides which deserve implementation. The cap of 200 is
a ceiling, not a reason to add low-value work.

## Progress ledger

The 2026-10-21 attribution batch adds deterministic, independently checked input
families and raw phase/scaling reports (`scripts/performance_corpus.py` and
`scripts/performance_study.py`). Its retained screening run contains the original
three workloads plus 40 generated cases; every Wasmtime/V8 checksum matches.
Seven-sample confirmation and small-arity/array-width controls are retained under
`target/performance-hills-20261021-attribution`. These are local receipts until a
portable evidence bundle is published, not a new published benchmark score.

User-space CPU-clock sampling of the retained-row child succeeded: 871 samples,
zero lost. Flat self samples attribute approximately 48% to named copying-GC
functions. This profiles the whole child (compilation, warmups and measurements),
so it does not establish that percentage inside the JS execution span. Guest JIT
addresses remain unnamed. P001–P005/P009 have useful evidence but remain open
until the attribution and reproducibility decisions are published.

P040's initial primitive numeric Array own-data experiment is **Rejected: no
reliable gain** under the current Portable/Fast runtime. The seven-sample
read-heavy row case rose from 168.957 to 181.914 ms; ABBA repeats also failed to
show a gain. The implementation is removed, with raw failed-attempt evidence
retained. Its semantic fixtures remain useful generic Array/Get controls.

P091's ordered multiargument concat experiment passes six focused CLI fixtures
and all 43 checked workloads. Its large-part ABBA study (seven observations
after ten warmups in each of four rounds) reduces the medians of round medians
from 24.719 to 7.796 ms for 256 16-unit parts, and from 120.461 to 8.944 ms for
256 128-unit parts, each repeated 64 times. These are focused compiler
`49ac374d7` versus `3b1a3427f` measurements, not a general JS speedup. Small
zero/one/two/four-argument controls and the original workloads are retained.
P091 awaits the broader native/original-case checkpoint before shipment.

P153's matching optimized Portable runtime image and P009's opt-in loaded-code
perf map are the next **unverified experiments**. NativeHost and Portable keep
separate raw keys, packages and v4 manifests, per-profile lazy admission and
the same destination-Engine ownership checks. The map is off by default; use
`SCRIPTW_PERF_MAP_DIR=/tmp` only in serial single-program profiling children.
It is a latest-mapping snapshot, not a general concurrent JIT profiler, and its
hashing/file work belongs to instrumented diagnostics. The other hills remain
open; no full current-source conformance run is claimed here.

## Starting evidence

The [verified baseline](rust-rewrite/v8-baseline.md) and
[raw evidence](rust-rewrite/performance-memory-baseline-20261021.json) bind these
results to compiler `cbbd9b170`, the machine, product policy and executable hashes.
Timing cells contain seven observations; execution follows ten warmups.

| Measurement | Scriptw / Wasmtime | Node / V8 |
| --- | ---: | ---: |
| Cold start | 1,117.891 ms | 31.094 ms |
| Cached start, fresh process | 938.986 ms | 36.474 ms |
| Execute 10,000 integer steps | 19.393 ms | 2.584 ms |
| Execute 500 objects | 51.342 ms | 0.061 ms |
| Execute 8,192 retained rows with strings and arrays | 2,118.118 ms | 3.575 ms |
| Cold-start median child peak RSS | 384.16 MiB | 27.12 MiB |
| Retained-row whole-child peak RSS, one observation | 558.05 MiB | 53.12 MiB |

The retained-row invocation's median sampled Wasmtime RSS maximum is 441.69 MiB;
its GC heap capacity is 64 MiB. Whole-child RSS includes compilation, warmups,
caches and host work. Sampled invocation RSS, V8 endpoints, GC capacity and live
JS heap bytes are different metrics. Wasmtime live-byte, allocation, collection
and pause counters are **unavailable** through the current public profiler.
Internal allocation-site counters would be a separate diagnostic, not a new
claim about those unavailable counters.

Three small workloads establish a starting point, not a representative JS score.
Prioritize absolute time in real workloads, not ratios against a tiny V8 result.
The existing full measurement takes about one minute; the selected integer loop
with a conditioned execution cache takes about three seconds. The initial
39-second cache-conditioning run stays outside those iteration measurements.

There is an important profile distinction to preserve in future comparisons.
The CLI's embedded **NativeHost** runtime is already built with
**SpeedAndSize and Backtracking register allocation**. The execution benchmark's
`profile_script_compilation` emits the **Portable** profile; the embedded image
is deliberately not a candidate for that different runtime key/profile. See
the [producer](../crates/scriptw-engine/src/embedded_runtime/build.rs),
[manifest](../crates/scriptw-engine/src/embedded_runtime/manifest.rs),
[profile gate](../crates/scriptw-engine/src/embedded_runtime.rs) and
[emission entry](../crates/scriptw-engine/src/compilation_profile.rs).
P001/P153 must distinguish these paths. Do not attribute the Portable execution
times to an identical native compilation policy in the CLI, or reuse a
NativeHost runtime for Portable Wasm. P153 investigates an optimized image for
the Portable profile without changing its emitted imports or bundling a parser.

## What already exists

Preserve these working optimizations and measure their remaining costs:

- A shared optimized NativeHost runtime image, immutable runtime packages and
  bounded compiled module caches; compiler/runtime identity and admission
  checks remain required. Embedded image admission is already once per process.
- Ordinary-property hash indexes, sparse array hash indexes and direct own-data
  reads that bypass public descriptor snapshots.
- Static heap-shape analysis, known-number arithmetic, specialized call/builtin
  planning, compact array literal recipes and unused function-type pruning.
- Hashed Map/Set storage with insertion-order records, native JSON/RegExp
  implementations, Ryu/integral number formatting and Intl data/provider reuse.

This backlog does not propose a second object model for non-GC engines, a JS
interpreter, C/native JS output, a REPL, or bundling a parser for general eval.

## Priority, effort and completion

Every entry starts **Proposed**. Order **A** means first-wave investigation or a
candidate directly tied to today's corpus; **B** follows attribution/new coverage;
**C** is a larger or less-evidenced opportunity. Order is not an estimated speedup.
**I** is investigation/measurement, **E** a bounded optimization experiment, and
**A** an architecture experiment requiring an explicit design decision. Effort
**S/M/L** is a rough relative implementation size, not an elapsed-time promise.

All optimization entries depend on a representative benchmark from P001–P015
and a measured cost in the named path. Large representation/ABI changes also
depend on a design, versioned artifact/cache identities and their affected
conformance controls. Do not implement every row before discovering whether it
matters. Close each investigation with retained evidence and a decision:
**Verified improvement**, **Rejected: no gain**, **Deferred**, or **Superseded**
(with the replacement ID). Only the first means a shipped optimization.

Use the [existing benchmark workflow](rust-rewrite/v8-baseline.md#current-performance-and-memory-tooling):

1. Freeze a coherent source batch; build once, then run correctness checks before
   timing on an otherwise idle machine under the cloud limiter. Use the same
   sources, checksums, sample/warmup counts, cache state and CPU for each pair.
2. Use three-sample selected runs to screen candidates. Confirm promising results
   with at least the seven-sample plan and repeated paired runs sufficient to
   distinguish the gain from observed variation. Preserve every raw result.
   Median and tail estimates from seven samples are descriptive, not significance.
3. Keep ordinary product measurements separate from profiler/counter builds.
   Record instrumented policy and overhead. Policy/configuration experiments
   need explicit report support and separately labelled cohorts; the current
   comparison tool correctly rejects incompatible policies or harnesses.
4. Record timing, relevant phase distributions, artifact size and memory under
   the same boundary. Declare any accepted compile-time/RSS/tail-latency tradeoff
   before choosing a winner; do not silently offset one regression with another.
5. Preserve JS results, effects, evaluation order, exceptions, identities and GC
   reachability. Run focused semantic controls and affected original Test262
   cases, then one broad native checkpoint per coherent implementation batch.
   Representation/ABI changes require the broader affected conformance campaign.
   Keep the four tracked original failures visible; no fixture amendments or skips.
6. Re-run the original three workloads for regressions, retain failed attempts,
   and push each verified batch to main. A documentation or diagnostic hill
   cannot claim new Test262 passes or completed release acceptance.

## Recommended first batches

| Batch | Hills | Deliverable and decision |
| --- | --- | --- |
| 1: attribution | P001–P005, P009 | Split construction, traversal and fixed setup; profile the retained-row case; establish scaling and repeatability. Select the dominant measured cost. |
| 2: first execution win | Two to four of P017, P019, P023, P025, P026, P038, P040, chosen from batch 1 | One comparable execution improvement with its original-case regressions checked. Reject experiments that only move cost between phases. |
| 3: compiler policy | P153–P155 | Compare the Portable runtime path with the already-optimized NativeHost image; evaluate matching Portable image reuse and explicit policy choices. Do not rebuild the runtime at application startup or mix source profiles. |
| 4: fixed overhead and memory | P131, then measured candidates P132–P138 and P146–P150 | Reduce startup/host or retention cost while preserving fresh JS state, limits and runtime identity. |
| 5: broader programs | P011–P015, then the matching B/C entries | Establish call, string, collection and synchronous SSR evidence before broad tuning. |

Batches 2–4 may be reordered after attribution. Do not start a shape/array/ABI
rewrite just because it appears in the list. Native and Test262 verification run
outside timing windows, reusing the built artifacts. Luna agents may explore and
review; authored implementation remains with the primary agent.

## Shared implementation boundaries

A hill is a bounded question, not a requirement for a separate PR or mechanism.
One implementation may close several questions with the same linked evidence.
Coordinate these overlapping owners before assigning implementation work:

| Hills | Shared owner and boundary |
| --- | --- |
| P006, P009, P010, P131, P146, P152 | One diagnostic/phase vocabulary; reuse existing spans and receipts instead of adding a profiler per subsystem. |
| P018, P035, P039, P049 | Descriptor/object/array representation design; compare alternatives before changing the shared GC schema or adding another layout. |
| P020, P023, P024, P112 | Canonical key/hash/string ownership; share valid cached facts across property and collection users without confusing hash and equality. |
| P026–P030, P065, P072, P076, P081 | Planning/effect/invalidation facts; extend existing proofs and revoke them across observable mutation. |
| P037–P039, P042, P043 | One fresh-array publication path and compact-recipe contract; batch compatible changes and measure their individual effects. |
| P056, P057, P061, P062, P071, P079 | Value/completion/local lifetime conventions; keep changes coherent and GC roots sound. |
| P077, P078, P083, P086 | Call argument representation; try the current-ABI zero-arity path before committing to a new ABI. |
| P132, P133, P136–P139, P149 | Immutable admission and Engine ownership; caching setup never licenses sharing mutable Store/realm state. |
| P153–P155, P160 | One recorded compiler-policy/build matrix; coordinate source identities and rebuilds rather than multiplying incompatible caches. |

## Attribution and benchmark coverage — P001–P015

Owners: [runner](../scripts/benchmark-v8.py),
[metrics/admission](../scripts/benchmark_metrics.py),
[Wasmtime driver](../crates/scriptw-cli/examples/runtime_baseline.rs),
[runtime phases](../crates/scriptw-engine/src/runtime_profile.rs),
[compilation phases](../crates/scriptw-engine/src/compilation_profile.rs).
These extend existing checked measurements; they do not replace the baseline.

| ID | Order / kind / effort | Hill | Evidence or acceptance |
| --- | --- | --- | --- |
| P001 | A / I / S | Attribute the retained-row runtime with sampling and existing phase spans. | Identify dominant native/Wasm helpers; record profiler availability and overhead separately. |
| P002 | A / I / S | Split retained rows into construction-only, traversal-only and combined cases. | Checked checksums and retained identity; separate allocation from read-loop time. |
| P003 | A / I / S | Sweep row counts and field/array lengths geometrically. | Plot ns/element and capacity boundaries; identify linear, resize or superlinear costs. |
| P004 | A / I / S | Add empty-program, empty-loop and scalar-return controls. | Quantify fixed realm/bootstrap/completion costs without subtracting them from published raw timings. |
| P005 | A / I / S | Establish paired-run repeatability and balanced runtime order across reports. | Retain repeated A/B observations; record an empirical noise range before accepting small gains. |
| P006 | B / I / M | Add optional helper call/probe/allocation-site diagnostic counters. | Count emitted operations without host calls per event; label instrumentation overhead and unavailable GC metrics correctly. |
| P007 | B / I / S | Measure a sustained same-process/same-realm workload separately. | Explicit server-style boundary; preserve the fresh-Store baseline as its own cohort. |
| P008 | B / I / S | Distinguish cold application cache, cold process and OS page-cache effects. | Label each scenario; do not call current warm-OS-cache samples a cold disk boot. |
| P009 | A / I / S | Add external native sampling profiles and emitted-function size/name attribution. | Bind symbols/disassembly to exact Wasm/executable/native image; Wasmtime exposes no per-function timing counter here. |
| P010 | B / I / S | Add separately scoped compiler RSS diagnostics alongside runtime phase sampling. | Existing invocation sampling starts after source emission; align load/execution samples separately and never equate RSS with live JS bytes. |
| P011 | B / I / S | Add direct/indirect/closure/recursion call workloads. | Vary arity, capture and depth; preserve this/arguments and observable call counts. |
| P012 | B / I / S | Add fixed/dynamic key, prototype, accessor and proxy workloads. | Separate fast hits from fallback cost and check every observable trap/coercion. |
| P013 | B / I / M | Add string, RegExp, collection, typed-array and JSON throughput families. | Checked scaled inputs, not one combined score; identify which B-tier experiments are justified. |
| P014 | B / I / M | Benchmark the already-supported pinned synchronous React SSR cases. | Exact Node output, fixed bundle and host surface; keep unsupported streaming/browser entry separate. |
| P015 | B / I / M | Add ESM import-graph and long-lived allocation/retention workloads. | Record compile/startup versus steady state; verify module side-effect order and per-run state isolation. |

## Object allocation and properties — P016–P035

Owners: [allocation](../crates/scriptw-aot-wasm/src/objects/allocation.rs),
[literal publication](../crates/scriptw-aot-wasm/src/objects/object_literal_property.rs),
[property storage](../crates/scriptw-aot-wasm/src/objects/property_storage.rs),
[index](../crates/scriptw-aot-wasm/src/objects/property_storage/index.rs),
[own-data reads](../crates/scriptw-aot-wasm/src/objects/simple_own_data_read.rs),
[read dispatch](../crates/scriptw-aot-wasm/src/objects.rs),
[definition](../crates/scriptw-aot-wasm/src/objects/define_property.rs).
The index and direct read arms already exist. Probe/key/descriptor costs below
need attribution; static shape analysis must be inspected before adding caches.

| ID | Order / kind / effort | Hill | Evidence or acceptance |
| --- | --- | --- | --- |
| P016 | A / E / M | Reduce empty-object header/storage setup where immutable empty components can be shared. | Empty versus populated literals; every object remains fresh and independently mutable. |
| P017 | A / E / M | Allocate known fresh literal property capacity once. | Construction ns/object and growth work; preserve insertion order and computed-property effects. |
| P018 | B / E / M | Reduce descriptor/StoredValue indirections for ordinary data fields. | Read/allocation time and footprint; descriptors, accessors and reflection remain exact. |
| P019 | A / E / M | Give proven fresh literal fields a first-insertion path without replacement lookup. | Probe count and construction time; duplicates and computed keys use full semantics. |
| P020 | B / E / M | Carry a prepared hash through lookup and insertion. | Hash work/write and collision controls; canonical keys and replacement behavior unchanged. |
| P021 | B / E / S | Tune small-object property capacity and growth factors. | Capacity-boundary timing and retained bytes; avoid wasting memory on common one/two-field objects. |
| P022 | B / E / M | Tune property index collision/load policy. | Probe distribution, misses and adversarial keys; retain order independently of the index. |
| P023 | A / E / M | Reuse precomputed hashes for static/interned property keys where absent. | Repeated fixed-key reads/writes; string/symbol equality and collisions stay correct. |
| P024 | B / E / M | Add a proven key-identity shortcut before string content equality. | Interned and equal-but-distinct strings; never treat equal hash as equal key. |
| P025 | B / E / M | Reduce dispatch before the existing ordinary own-data read arm. | Own-data loop latency; exotic objects and accessors must reach their original paths. |
| P026 | A / E / M | Extend existing static heap-shape facts to more safe literal reads. | Inspect current coverage first; overwrite, delete, prototype mutation and escape revoke invalid facts. |
| P027 | B / A / L | Experiment with guarded stable-shape slot caches for unresolved hot reads. | Monomorphic/polymorphic sites, invalidation and memory cost; reuse existing analysis, no unsound shape assumption. |
| P028 | B / A / L | Experiment with guarded writable-data slot stores. | Existing-field writes; setters, receiver differences, attributes and proxies preserve semantics. |
| P029 | B / E / M | Cache proven inherited-read routes with prototype/version guards. | Depth-scaled hits and mutation; invalidate on every relevant chain/descriptor change. |
| P030 | C / E / M | Speed repeated missing-property lookups with sound negative guards. | Miss-heavy workloads; later insertion/prototype mutation must become visible immediately. |
| P031 | B / E / M | Avoid public descriptor materialization in more internal read/write paths. | Audit existing bypasses first; public getOwnPropertyDescriptor still returns fresh observable records. |
| P032 | B / E / M | Specialize primitive property reads beyond existing string length where safe. | Compare boxing cost; null/undefined errors, prototypes and overridden accessors remain observable. |
| P033 | C / E / M | Reduce tombstone history and reindex cost for delete/reinsert objects. | Churn versus steady state; own-key order and active enumeration behavior remain exact. |
| P034 | C / E / M | Reuse sound lookup facts within one has/get/define operation. | Probe counts; do not carry facts across user code, proxy traps or mutation. |
| P035 | C / A / L | Evaluate compact small-object layouts versus today's indexed storage. | Same semantics, ABI/version accounting, RSS and code-size tradeoff; proceed only if simpler targeted work is insufficient. |

## Arrays and indexed operations — P036–P055

Owners: [literal](../crates/scriptw-aot-wasm/src/builtins/array/literal.rs),
[compact recipes](../crates/scriptw-aot-wasm/src/builtins/array/literal/compact.rs),
[sparse index](../crates/scriptw-aot-wasm/src/objects/sparse_array_index.rs),
[array own reads](../crates/scriptw-aot-wasm/src/objects/array_own_data_read.rs),
[array builtins](../crates/scriptw-aot-wasm/src/builtins/array.rs).
Sparse storage already avoids allocation by logical array length. Dense-storage
experiments must preserve huge sparse arrays rather than adding length-sized allocation.

| ID | Order / kind / effort | Hill | Evidence or acceptance |
| --- | --- | --- | --- |
| P036 | A / E / M | Reduce array header and empty-storage initialization. | Empty/four-element allocation cost; identity, prototype and writable length remain exact. |
| P037 | B / E / M | Initialize known literal index capacity once. | Resize count and construction time; holes/spread and huge logical lengths remain bounded. |
| P038 | A / E / M | Batch fresh array-literal index publication. | Four-element/large literal ns/element; preserve evaluation order and abrupt partial construction. |
| P039 | B / E / M | Reduce per-element descriptor/StoredValue allocation. | Retained traversal and allocation-site diagnostics; reflect/define/accessor behavior unchanged. |
| P040 | A / E / M | Avoid repeated numeric-key stringification/parsing on proven numeric indices. | Numeric/string/computed index matrix; -0, nonindices and coercion side effects stay exact. |
| P041 | B / E / S | Give array length its shortest valid existing fast-path route. | Length-heavy loops; named descriptors, receiver/exotic rules and mutations remain correct. |
| P042 | B / E / M | Broaden safe compact literal recipe eligibility. | Equal-size eligible/near-miss literals; preserve holes, element freshness and side effects. |
| P043 | B / E / M | Reduce decoding/publication cost inside existing compact recipes. | Separate artifact-size gains from runtime cost; no skipped element semantics. |
| P044 | B / E / S | Tune sparse index growth thresholds/capacity. | Dense/sparse/collision cases and p95 insert cost; retain bounded memory. |
| P045 | C / E / M | Avoid resize/shrink oscillation in delete-heavy arrays. | Churn and refill latency/RSS; deletes, holes and property ordering remain correct. |
| P046 | B / E / M | Add guarded append/push specialization. | Append-only versus inherited setter/nonwritable-length cases; preserve exact failure timing. |
| P047 | B / E / M | Reduce bounds/key/descriptor checks in proven indexed reads. | Dense/holey/prototype-element cases; invalidate facts when observable mutation can occur. |
| P048 | B / E / M | Reuse validated array facts within a callback-free write loop. | Per-element time; stop reusing across accessors, callbacks, proxies or resizable inputs. |
| P049 | C / A / L | Evaluate an additional dense storage mode for ordinary arrays. | Measured win versus added complexity; holes/accessors/sparse transitions and cache/ABI versions covered. |
| P050 | C / E / M | Reduce indexed own-key snapshot and sorting cost. | Object.keys/Reflect.ownKeys on mixed arrays; ascending index order, symbols and string order exact. |
| P051 | C / E / M | Optimize array slice/copy operations for proven plain data ranges. | Copy throughput and fresh results; holes, species, getters and overlap semantics preserved. |
| P052 | C / E / M | Optimize map/filter/reduce traversal without suppressing callbacks. | Callback count/order, holes and mutation controls; compare per-element overhead. |
| P053 | C / E / M | Improve plain-array iterator next state/access costs. | ns/next and allocation-site diagnostics; iterator identity and live mutations remain visible. |
| P054 | C / E / M | Reduce join output growth and intermediate strings. | Large/string-heavy arrays; getters, coercion order, holes and cycles remain exact. |
| P055 | C / E / M | Optimize sort/toSorted scratch representation and native comparisons. | Stable ordering and memory; comparator calls and source-read ordering remain compliant. |

## Numbers, control flow and bindings — P056–P075

Owners: [operations](../crates/scriptw-aot-wasm/src/operations.rs),
[expressions](../crates/scriptw-aot-wasm/src/expressions.rs),
[value/completion locals](../crates/scriptw-aot-wasm/src/gc_types/value.rs),
[planning](../crates/scriptw-aot-wasm/src/planning.rs),
[control flow](../crates/scriptw-aot-wasm/src/control_flow.rs),
[environments](../crates/scriptw-aot-wasm/src/environments.rs).
Known-number paths and local reuse already exist. Improvements need to show an
actual remaining copy, branch or lookup rather than merely add another wrapper.

| ID | Order / kind / effort | Hill | Evidence or acceptance |
| --- | --- | --- | --- |
| P056 | A / E / M | Keep proven numeric loop intermediates in scalar Wasm locals longer. | Integer-loop time/code; NaN, -0, overflow and externally observed values remain JS-correct. |
| P057 | B / E / M | Avoid redundant tag/payload copies between known-number expressions. | Instruction/size and timing delta; canonical values still reach generic boundaries. |
| P058 | B / E / M | Specialize proven-number bitwise operands before generic setup. | ToInt32/ToUint32 edges and shifts; retain dynamic coercion order. |
| P059 | C / E / M | Improve common finite-number remainder lowering. | Remainder-heavy loops; infinities, NaN, signs and -0 remain exact. |
| P060 | B / E / M | Share numeric-pair classification across conversion and dispatch. | Mixed Number/BigInt and user coercions; preserve TypeError precedence. |
| P061 | A / E / M | Remove unnecessary normal-completion traffic on proven nonthrowing scalar operations. | Scalar loop timing and code size; abrupt-capable operations keep full completion state. |
| P062 | B / E / M | Reduce normal-path completion tuple copying across statements. | Branch-heavy loops; labels, return/throw and completion values remain exact. |
| P063 | B / E / M | Keep loop break/continue routing on direct Wasm control edges where valid. | Nested/labeled loops and UpdateEmpty controls; compare hot-path branch traffic. |
| P064 | B / E / M | Elide uncaptured per-iteration lexical environments where proven unobservable. | for-let cost; captured variables still have distinct iteration bindings. |
| P065 | B / E / M | Hoist pure loop-invariant work using existing effect analysis. | Construction/traversal loops; never hoist coercions, getters, traps or observable throws. |
| P066 | B / E / M | Avoid repeated environment search for resolved local bindings. | Identifier-heavy loops; TDZ, shadowing, with and supported eval visibility remain correct. |
| P067 | B / E / M | Shorten captured-binding access with resolved environment depth/slot facts. | Closure loops; scope escape, mutation and realm ownership remain correct. |
| P068 | C / E / M | Specialize established global lexical slots. | Global-heavy scripts; object-global properties, deletion and dynamic observations retain their distinction. |
| P069 | B / E / M | Reuse numeric increment/update facts across reference load and store. | Prefix/postfix and property updates; old result, coercion and store order exact. |
| P070 | C / E / M | Fold pure constant primitive expressions beyond current analysis coverage. | Emitted code/time; signed zero, BigInt errors and unreachable syntax validation preserved. |
| P071 | B / E / M | Coalesce scratch locals using actual liveness after existing reuse. | Native frame/Wasm local count; preserve rooted references and live operand values. |
| P072 | C / E / M | Eliminate redundant reference tests/casts with retained typed proofs. | Hot mixed-value paths; no unchecked cross-layout reference reinterpretation. |
| P073 | C / E / M | Outline cold exceptional branches from hot arithmetic/property paths. | Code size and execution; preserve exact exception construction and call order. |
| P074 | C / E / M | Improve short-circuit/conditional normal-path value flow. | Branch mixes and side effects; only evaluated arms may perform work. |
| P075 | C / E / M | Optimize uncaptured destructuring binding setup. | Array/object destructuring cost; iterators, defaults, rest, getters and IteratorClose retain semantics. |

## Calls, closures and dispatch — P076–P090

Owners: [function calls](../crates/scriptw-aot-wasm/src/functions.rs),
[function objects](../crates/scriptw-aot-wasm/src/functions/function_object_allocation.rs),
[function entry](../crates/scriptw-aot-wasm/src/function_entry.rs),
[planning](../crates/scriptw-aot-wasm/src/planning.rs),
[resumable blocks](../crates/scriptw-aot-wasm/src/environments/resumable_block.rs).
Direct/tail calls and target discovery already exist. Broadening them must keep
indirect/proxy and overridden builtin behavior correct.

| ID | Order / kind / effort | Hill | Evidence or acceptance |
| --- | --- | --- | --- |
| P076 | B / E / M | Broaden safely resolved direct call targets across simple aliases. | Direct/indirect call loops; mutation, accessor lookup and realm changes invalidate assumptions. |
| P077 | B / A / L | Experiment with a small-arity call ABI avoiding general argv allocation. | Zero/one/two-arg cost, code size and memory; arguments/rest/retention and ABI identities covered. |
| P078 | B / E / M | Specialize the empty argv path within the current ABI. | No-arg calls; canonical empty storage cannot become shared mutable arguments state. |
| P079 | B / E / M | Reduce normal call-result completion copying. | Repeated calls with/without throws; return values and abrupt propagation stay exact. |
| P080 | B / E / M | Avoid redundant function/proxy classification for proven callables. | Monomorphic and changing sites; proxies, bound calls and revoked proxies remain observable. |
| P081 | C / E / M | Extend direct builtin invocation only under valid identity proof. | Builtin loops and monkey-patching controls; preserve this/coercion/realm behavior. |
| P082 | C / E / M | Reduce forwarding storage for bound-function calls. | Bound chains and constructors; prepend order, this, new.target and length/name semantics intact. |
| P083 | C / E / M | Optimize apply/spread for guarded plain argument arrays. | Argument collection time; holes, getters, iterator replacement and abrupt collection stay exact. |
| P084 | B / E / M | Share immutable noncapturing function metadata more widely. | Function-creation loops; every required function object remains distinct with correct reflective properties. |
| P085 | C / E / M | Capture only required environments/slots where existing analysis permits. | Closure retention and call time; escape, dynamic visibility and mutable captures remain sound. |
| P086 | C / E / M | Reduce default/rest parameter setup for simple declarations. | Small-call latency; arguments aliasing, TDZ and default-expression order remain correct. |
| P087 | C / A / L | Evaluate bounded inlining of small proven direct JS functions. | End-to-end speed versus frame/code growth; recursion, exceptions, realm and this semantics preserved. |
| P088 | C / E / M | Extend safe tail-call codegen coverage. | Deep supported tail-call cases and frame size; do not turn ordinary calls into spec-incompatible tail calls. |
| P089 | C / E / M | Reduce generator resume dispatch and state record traffic. | ns/resume and retention; yield/return/throw/finally and delegation remain exact. |
| P090 | C / E / M | Reduce Promise/async job bookkeeping on common fulfilled paths. | ns/job and memory; asynchronous ordering, rejection tracking and microtask boundaries remain exact. |

## Strings, conversions and RegExp — P091–P110

Owners: [string methods](../crates/scriptw-aot-wasm/src/builtins/string.rs),
[coercive add](../crates/scriptw-aot-wasm/src/operations/coercive_add.rs),
[number formatting](../crates/scriptw-aot-wasm/src/operations/number_to_string.rs),
[matcher workspace](../crates/scriptw-aot-wasm/src/builtins/regexp/matcher_workspace.rs),
[regexp compiler](../crates/scriptw-aot-wasm/src/builtins/regexp/compiler.rs),
[range lookup](../crates/scriptw-aot-wasm/src/builtins/regexp/range_search.rs),
[match results](../crates/scriptw-aot-wasm/src/builtins/string/regexp_exec.rs),
[substitution](../crates/scriptw-aot-wasm/src/builtins/string/regexp_substitution.rs).
Native method paths, number-formatting fast paths and compiled RegExp/workspace
machinery exist. Confirm current algorithm/coverage before replacing it.

| ID | Order / kind / effort | Hill | Evidence or acceptance |
| --- | --- | --- | --- |
| P091 | B / E / M | Flatten multi-argument concat output construction where coercion ordering allows. | Bytes copied/time; retain each coercion before subsequent observable work and correct length failures. |
| P092 | B / E / M | Reduce string creation in repeated row-prefix plus integer conversions. | Isolated row-name corpus; preserve number formatting and fresh observable coercion effects. |
| P093 | C / E / M | Improve repeat copying for large counts without extra intermediate strings. | Throughput/RSS; RangeError, maximum lengths and UTF-16 units remain exact. |
| P094 | C / E / M | Construct padStart/padEnd output without avoidable intermediates. | Large gaps/multicode-unit fillers; coercion order and prefix truncation preserved. |
| P095 | C / E / M | Improve long/repetitive-needle string search where current scanning scales poorly. | Absent/late matches and geometric sizes; positions and UTF-16 semantics remain exact. |
| P096 | C / E / S | Shorten already-trimmed string handling and boundary scans. | No-trim/edge-trim latency; all spec whitespace and lone-surrogate cases preserved. |
| P097 | B / E / M | Specialize character access without unnecessary representation conversion. | Sequential/random UTF-16 indices; codeUnit/codePoint/BMP/astral differences remain exact. |
| P098 | C / E / M | Reduce string-iterator state/decode work. | ns/code point; iterator protocol, astral pairing and lone surrogates preserved. |
| P099 | C / E / M | Reduce substring/slice copy cost without unbounded backing-store retention. | Time plus retained-parent memory; coercion/index and UTF-16 behavior preserved. |
| P100 | B / E / M | Avoid generic ToPrimitive/ToString dispatch for proven primitive addition pairs. | Mixed add corpus; BigInt errors, evaluation order and dynamic fallback unchanged. |
| P101 | C / E / M | Tune remaining number-to-string/radix scratch work around existing Ryu/integer paths. | Integer/fraction/exponent/radix cases; exact shortest formatting and signed-zero rules preserved. |
| P102 | C / E / M | Optimize URI encoding/decoding safe ASCII runs. | Bytes/s; invalid surrogate/escape errors and UTF-8 decoding remain compliant. |
| P103 | B / E / M | Reduce RegExp choice/capture workspace growth and copying. | Bounded backtracking workloads; captures, lookarounds, timeout and failure behavior intact. |
| P104 | C / E / M | Improve candidate-start filtering only where compiled matcher coverage lacks it. | Long no-match/late-match inputs; Unicode, sticky and zero-width cases preserved. |
| P105 | C / E / M | Tune Unicode property/range membership lookup. | Mixed-script comparisons/character; preserve full table contents and version identity. |
| P106 | C / E / M | Reduce match-result array/group/indices construction work. | Allocation-site diagnostics/match; required observable result objects remain fresh and complete. |
| P107 | C / E / M | Build RegExp replacement output with fewer intermediate concatenations. | Bytes copied and replace throughput; captures, named groups and replacer callback order exact. |
| P108 | C / E / M | Improve reuse of immutable dynamic RegExp programs within existing cache rules. | Construct-versus-exec timing and retained bytes; flags, source, realm and invalid syntax remain exact. |
| P109 | C / E / M | Reduce raw UTF-16 conversion at regexp/string host boundaries. | Conversion bytes/time; preserve lone surrogates, offsets and canonical ownership. |
| P110 | C / E / M | Specialize literal/non-RegExp split and replacement scanning. | Small/large separator corpus; Symbol protocol overrides, limits and coercion order preserved. |

## Collections, binary data, JSON and Intl — P111–P130

Owners: [collection storage](../crates/scriptw-aot-wasm/src/builtins/collections/storage.rs),
[set operations](../crates/scriptw-aot-wasm/src/builtins/collections/set_operations.rs),
[typed methods](../crates/scriptw-aot-wasm/src/builtins/standard/typed_array_methods.rs),
[typed set](../crates/scriptw-aot-wasm/src/builtins/typed_array_set.rs),
[typed fill](../crates/scriptw-aot-wasm/src/builtins/typed_array_fill.rs),
[JSON parse](../crates/scriptw-aot-wasm/src/builtins/json/parse.rs),
[stringify](../crates/scriptw-aot-wasm/src/builtins/json/stringify.rs),
[quoting](../crates/scriptw-aot-wasm/src/builtins/json/quote.rs),
[Intl number rendering](../crates/scriptw-aot-wasm/src/builtins/intl_numberformat/render.rs),
[collator compare](../crates/scriptw-aot-wasm/src/builtins/intl_collator/compare.rs).
Collection hashing, stable typed sorting and formatter/provider reuse already
exist. These hills tune their remaining work rather than invent those mechanisms.

| ID | Order / kind / effort | Hill | Evidence or acceptance |
| --- | --- | --- | --- |
| P111 | B / E / M | Tune Map/Set bucket load and collision behavior. | Probe distributions across primitive/object keys; SameValueZero and insertion order exact. |
| P112 | C / E / M | Reuse immutable string hashes across collection operations where not already retained. | Repeated key workloads; hashes are never equality and equal strings from distinct origins still match. |
| P113 | C / E / M | Reduce Map/Set resize pauses and repeated growth. | p95 insertion/retained bytes at boundaries; iteration and mutation semantics unchanged. |
| P114 | C / E / M | Reduce collection iteration over deletion history. | Churned/live-entry ratios; active iterators and delete/reinsert order stay compliant. |
| P115 | C / E / M | Improve Set algebra traversal choices under spec constraints. | Skewed cardinalities; size/getter/iterator calls and specified result ordering remain exact. |
| P116 | B / E / M | Use compatible typed-array bulk set/copy paths where legal. | Bytes/s and memory; overlap, conversion, detached/resizable/shared buffers and observable reads covered. |
| P117 | C / E / M | Tune typed fill/copyWithin loops using valid memory operations. | Offset/size matrix; multi-byte fill patterns and overlap cannot be replaced with incorrect byte fill. |
| P118 | C / E / M | Reduce boxed-value scratch in default typed-array sort. | Existing stable sort's memory/time; NaN, -0, BigInt and source-read order preserved. |
| P119 | C / E / M | Specialize native typed numeric comparisons separately from user comparators. | Comparisons/s; stable ties and every comparator call/throw remain exact. |
| P120 | C / E / M | Reduce typed-array callback traversal conversion overhead. | ns/element; preserve callback arguments/order, detachment and resizable-buffer effects. |
| P121 | C / E / M | Batch Base64/hex codec hot loops. | Small/large byte throughput; alphabet, padding, partial writes and syntax errors preserved. |
| P122 | B / E / M | Improve JSON parse scanning of safe ASCII/string/number runs. | Bytes/s by input shape; grammar, duplicate keys, lone surrogates and error boundaries exact. |
| P123 | C / E / M | Reduce JSON parse frame/container allocation work. | Nested/flat input peak RSS and time; depth/error/reviver behavior and fresh objects preserved. |
| P124 | B / E / M | Improve JSON output buffer growth/reservation. | Bytes copied/time; gap, cycles, replacer, toJSON and exception timing preserved. |
| P125 | C / E / M | Copy unescaped JSON string spans in bulk. | ASCII/control/Unicode throughput; exact escaping and well-formed lone-surrogate output preserved. |
| P126 | C / E / M | Reuse JSON property-list/lookup work only between nonobservable steps. | Plain versus replacer/getter cases; snapshot order and mutations stay compliant. |
| P127 | C / E / M | Reduce repeated Intl locale/option normalization around existing retained plans. | Constructor versus reused-formatter workloads; getters, locale fallback and data identity preserved. |
| P128 | C / E / M | Reduce Intl NumberFormat render marshalling/intermediate output. | Values/s plus output equality across options/locales; no dropped precision or parts. |
| P129 | C / E / M | Reduce collator compare conversion/provider overhead around existing reuse. | Comparisons/s and RSS; normalization, sensitivity and locale results remain exact. |
| P130 | C / E / M | Tune DateTimeFormat/Temporal repeated timezone/calendar lookup work. | Reused formatter/calendar corpus; DST transitions, historical rules and all existing failures remain visible. |

## Startup, host setup and caches — P131–P145

Owners: [Engine execution](../crates/scriptw-engine/src/lib.rs),
[runtime loading/linkage](../crates/scriptw-engine/src/wasm_runtime_link.rs),
[embedded images](../crates/scriptw-engine/src/embedded_runtime.rs),
[module cache](../crates/scriptw-engine/src/memory_module_cache.rs),
[Intl admission](../crates/scriptw-engine/src/intl_data_images.rs),
[host linking](../crates/scriptw-engine/src/wasm_gc_host.rs),
[disk caches](../crates/scriptw-engine/src/cache.rs).
Runtime reuse is already working. Module reuse is distinct from reusing mutable
JS state: do not pool a Store/realm just to make a fresh-state benchmark faster.

| ID | Order / kind / effort | Hill | Evidence or acceptance |
| --- | --- | --- | --- |
| P131 | A / I / S | Attribute the remaining approximately one-second startup and cached-process overhead. | Phase distributions for engine/module/Store/linker/instantiate/bootstrap; select a real bottleneck before tuning. |
| P132 | B / E / M | Reduce fresh-process immutable image admission cost after attributing digest/parse work. | Admission already uses OnceLock within a process; preserve manifest/source/native identity and external-byte validation. |
| P133 | B / E / M | Reduce runtime image deserialization/mapping cost on cache misses or evictions. | Ordinary module-cache hits already avoid this work; measure cold/miss RSS and preserve same-Engine ownership. |
| P134 | B / E / M | Replace linear module-cache lookup only if occupancy profiles justify it. | Hit/miss/eviction latency across sizes; exact LRU and retained-image byte bounds preserved. |
| P135 | B / E / S | Tune module cache limits for real workload reuse rather than benchmark-only hits. | Latency/RSS curve and churn; active modules/JS heaps are not counted as retained cache images. |
| P136 | B / E / M | Reuse Engine-bound immutable linker/preinstantiation setup where Wasmtime permits. | Store/linker/instantiate phases; never retain Store-specific roots, mutable state or stale import bindings. |
| P137 | B / E / M | Reduce repeated host import registration work under the same typed import contract. | Setup time/import count; host signatures, required imports and callback state remain exact. |
| P138 | B / E / M | Reduce Intl parse/materialization for standalone artifacts and uncached custom profiles. | Linked runtimes already have a two-profile kernel LRU; target misses/churn and preserve full identity and duplicate-section rejection. |
| P139 | C / E / S | Tune the small Intl-kernel/runtime-plan caches to measured profile diversity. | Cache outcomes and retained bytes; preserve bounded caches and runtime-profile identity. |
| P140 | C / E / M | Reduce cache lock contention without moving compilation under the lock. | Throughput/p95 and duplicate work under concurrent callers; preserve admission and eviction ownership. |
| P141 | B / E / M | Reduce repeated sized-stack worker spawn/join cost for synchronous calls. | Entry overhead and stack commitment; preserve nesting, thread-local context, panic propagation and safe stack headroom. |
| P142 | C / E / M | Reduce cache hit I/O/copy/decoding work without weakening cache keys or validation. | Bytes read, startup time and RSS; atomic publication and corrupt-entry recovery stay exact. |
| P143 | C / E / M | Reduce print-import byte/string copying in output-heavy programs. | Bytes copied and time; shared-memory reads, UTF-8 validation and host hooks preserved. |
| P144 | C / E / M | Avoid duplicate captured output where the selected mode does not need it. | Output memory/time; structured event order and exception/legacy modes remain correct. |
| P145 | C / E / M | Reduce scalar completion decoding/allocation on routine host returns. | Normal modes already use scalar observation; explicit Graph mode, roots, values and exceptions remain correct. |

## Memory, compilation and artifact policy — P146–P160

Owners: [Wasmtime configuration](../crates/scriptw-engine/src/wasmtime_config.rs),
[compilation policy](../crates/scriptw-engine/src/wasm_compilation_policy.rs),
[program initialization](../crates/scriptw-aot-wasm/src/gc_types/value/program_initialize.rs),
[compilation profile](../crates/scriptw-engine/src/compilation_profile.rs),
[runtime package](../crates/scriptw-aot-wasm/src/runtime_artifact/build_package.rs),
[runtime producer](../crates/scriptw-runtime-image/build.rs),
[unused type pruning](../crates/scriptw-aot-wasm/src/native_program_template/unused_function_types.rs).
The 16 MiB initial GC capacity, copying collector, Fast/SizeOptimized policies
and shared package path already exist. The NativeHost build image already uses
SpeedAndSize/Backtracking; ordinary Fast compilation uses None/SinglePass.
Reserved stack/linear memory is not
committed RSS, and a compiler-phase gain is not an execution-time gain.
Runtime/program modules must still share the required Engine and canonical
reference types. NativeHost already admits optimized R into compatible Fast or
SizeOptimized destination Engines. Policy extensions must preserve that checked
producer/destination compatibility, not link modules from different Engines or
bypass native-version checks.

| ID | Order / kind / effort | Hill | Evidence or acceptance |
| --- | --- | --- | --- |
| P146 | B / I / M | Account separately for module images, retained packages, Store memory and host allocator retention. | Phase/RSS/mapping evidence and bounded diagnostics; do not invent a live JS heap by subtraction. |
| P147 | B / E / S | Sweep initial GC heap capacity around the current 16 MiB. | Bootstrap/retained/ephemeral time and RSS; capacity is not live size, GC/weak behavior remains compliant. |
| P148 | B / E / M | Shorten unnecessary GC root/local lifetimes after their last semantic use. | Retention/capacity diagnostics; values needed by closures, exceptions, weak semantics or reentry remain rooted. |
| P149 | B / E / M | Reduce duplicate immutable package/data ownership on compile/load paths. | Count actual copies and RSS; preserve admitted byte lifetimes, immutable sharing and artifact linkage. |
| P150 | C / E / M | Improve arena/scratch reuse on compiler and host allocation paths. | Repeated-compile RSS/time; do not keep prior JS objects or unbounded high-water buffers. |
| P151 | C / I / M | Evaluate stack and linear-memory reservation/guard budgets from measured usage. | Separate virtual reservation from committed pages; preserve large frames, recursion and memory/timeout limits. |
| P152 | B / I / M | Split measured emission cost into runtime admission, codegen, encoding and metadata stages. | Explain the roughly 1.06–1.14-second tiny-program emission observations before changing the pipeline. |
| P153 | A / A / L | Evaluate matching optimized build-image reuse for Portable runtime artifacts. | NativeHost is already optimized; measure Portable helper speed/startup, build/image cost and cache coverage. Preserve Portable imports, source profile and exact R/P/native identities. |
| P154 | B / A / L | Evaluate a speed-focused policy for generated program modules separately from shared runtime policy. | End-to-end compile/startup/execution break-even across short/long programs; retain Fast as a labelled comparison cohort. |
| P155 | B / I / M | Compare native register allocation independently for runtime and program compilation. | NativeHost R already uses Backtracking and Fast P uses SinglePass; measure frame/image size, compilation and execution with exact policy identities. |
| P156 | C / E / M | Specialize/share emitted helper bodies according to measured hotness and duplication. | Code size/compile versus call overhead; preserve per-program/realm metadata and recursion. |
| P157 | C / E / M | Prune additional unreachable entries/data after existing unused-type pruning. | Artifact/compile/instantiate time; all exported, prepared-source and reflective reachability roots preserved. |
| P158 | C / E / M | Share immutable bootstrap metadata while keeping mutable realm initialization fresh. | Instantiation/bootstrap time and RSS; constructors, prototypes and mutable globals isolated per realm. |
| P159 | C / E / M | Reduce repeated parse/analysis/lowering work on cold compilation using validated reusable facts. | Stage timings and memory by source size; syntax errors, parse goal, options and source identity remain exact. |
| P160 | C / I / M | Evaluate Rust release LTO/PGO and artifact partitioning only against measured end-to-end cost. | Distinguish compiler/host improvements from generated Wasm speed; bind profile/training data, binary size and rebuild cost. |

## How this list stays useful

Update the row's state with a linked source-bound receipt when work closes it;
keep IDs stable. Investigations may demonstrate that an existing optimization
already solves the proposed case. Record that evidence and mark the row
Superseded rather than implementing another mechanism. Reorder B/C work using
measured time and representative use, not the number of unchecked rows.

This planning change adds no implementation, new benchmark numbers or Test262
passes. The [current inventory](rust-rewrite/current-inventory.md) continues to
own native/conformance/release status. A later implementation batch should
update that status only with the verification it actually runs.
