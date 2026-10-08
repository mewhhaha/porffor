# Retained whole-CLI performance reports

Status: source authored, including private statistics, baseline-admission and
artifact controls. Compilation, control execution and idle-machine timing are
pending. This does not establish any timing budget, subsystem measurement or
T25 acceptance.

## Explicit invocation

After building the actual default-feature CLI, run on an idle machine:

```sh
python3 scripts/limited_verification.py -- \
  ./target/release/lila performance report \
  --output-dir target/performance/candidate \
  --samples 7 --machine-label dedicated-benchmark-host --idle-machine
```

The output directory must not exist; its parent must exist. Samples are limited
to 3..100 per selected gate. `--gate warm-exact|warm-chunk|cold-exact|all`
defaults to all three. `--fixture-root PATH` selects an existing copy of the
compiled fixture corpus; the default is the CLI package's tests/fixtures build
path. Every source must match its embedded bytes. A relocated CLI can select a
checkout or retained corpus directory explicitly. Cold timing prunes the usual
Lila caches before each sample, as the existing gate does. This is an opt-in
measurement command with that real cache effect.

Compare a later build against a retained complete report:

```sh
python3 scripts/limited_verification.py -- \
  ./target/release/lila performance report \
  --output-dir target/performance/later \
  --samples 7 --machine-label dedicated-benchmark-host --idle-machine \
  --baseline target/performance/candidate/report.json
```

`--idle-machine` records an operator acknowledgment. It cannot prove that the
machine stayed idle or that another process did not affect cache contents.
Run this separately from compilation, conformance or other timing work. The
outer launcher retains the confirmed aggregate 4096 MiB/no-swap/one-CPU policy;
there is no nested launcher or uncapped timing fallback.

## Actual measured owners

The original twenty literals in tests/perf/chunk_cases.rs produce both names
and include_str fixture bytes. The existing gate reads the same names, and the
reporter reads those names and bytes. No fixture text, order, timing test,
ignored-test declaration or known-failure ledger row changes.

Each warm-exact repetition runs wasm_functions.js once for warmup, then measures
one ordinary in-process CLI Wasm invocation. Each warm-chunk repetition warms
all twenty fixtures in original order, then measures that ordered serial loop.
Each cold-exact repetition invokes the actual loaded CLI image's cache prune
outside the timing span, then measures its subprocess run of wasm_host_output.js.
The reporter verifies that selected image's compiler identity. There is no
interpreter path or detached timing model.

The aggregate span covers the complete invocation or chunk loop. Nested
Instants record real per-case durations; artifact writes and source-identity
checks occur outside that span. Timer and capture bookkeeping remain part of
the measurement policy. Warmup invocations are retained separately. These are
whole CLI wall-clock measurements, including whichever compilation, cache,
engine startup and execution work the ordinary CLI actually performs. They do
not expose separate parse/lower/emit/native-compile/validation/runtime/GC costs.
The original ignored gates remain independently runnable; a report does not
claim they ran. Their limits remain one second for warm exact and five seconds
for the chunk and cold exact. The command exits red if any measured repetition
exceeds its gate's existing limit, retaining all completed evidence.

The separate [runtime profile](runtime-performance-profile.md) now has authored
source for actual emitted-Wasm execution spans and real Store GC capacity
snapshots. It uses the same fixed corpus and identity readers. Allocation,
collection, pause and peak-memory fields explicitly remain unavailable where
the pinned runtime provides no observation. Its controls and timings are unrun;
this whole-CLI command and its three budgets retain their original spans.

## Identity and comparison admission

The report binds the actual running compiler's validated embedded source
fingerprint, source revision and loaded executable SHA-256. The CLI build script
captures the compiling rustc's verbose version, Cargo target/host/profile,
optimization/debug settings and encoded rustflags. Linked oracle-feature state
is explicit even though measurements select only Wasm-AOT.

The platform record contains the actual OS, architecture, kernel, CPU model,
available logical CPU count and physical memory; Linux also records process
CPU affinity and visible native cgroup2 limits. Linux and macOS have real
platform readers. Other platforms reject reporting before timing rather than
inventing hardware data. The caller's machine label, actual compilation jobs,
default product host surface, working directory, cache paths/limits and relevant runtime/cache
configuration are retained. Unexposed cgroup limits and unrelated ambient
process activity remain limits of this evidence, not assertions of isolation.

Ordered workload identity includes every name, exact byte length and SHA-256,
and one length-framed corpus digest. The canonical fixture root also participates
in compatibility because the ordinary compiler cache key includes filenames.

Baseline admission requires the same schema, measurement policy, gate order,
repetition count, workload bytes/order, filename root, platform, toolchain,
linked features and configuration/resource settings. Compiler revisions and
images may differ, and both identities are shown in the comparison. Different
optimization profiles or hardware require separate baselines. A failed,
interrupted, reordered, truncated or missing-log report cannot become a
baseline. Admission checks retained log and workload bytes and recomputes all
summaries from raw samples; changed summary numbers are rejected.

Each per-case and aggregate summary retains sample count, min/max, median and
nearest-rank p90/p95. Even medians average the two central integer nanosecond
samples, rounding down below one nanosecond. Comparisons show signed median and
p95 differences and candidate/baseline ratios; a zero baseline has an explicit
null ratio. All outliers remain in raw evidence. A ratio is an observed timing
comparison, not a throughput or statistical-significance claim.

## Retained artifacts and boundaries

report.json is atomically checkpointed before work and after completed prefixes.
It remains Running after process interruption; orderly errors produce Failed.
Neither gets completed summaries or a baseline verdict. Every captured warmup,
measured invocation and cache-prune stdout/stderr is retained under logs with
its size and SHA-256. workload contains all twenty exact source snapshots.
When comparing, baseline.json retains the admitted baseline report bytes and
its digest; the original baseline directory remains the retained owner of its
logs and workload snapshots.

A complete report can still exceed budgets. Completion only means the planned
successful invocations finished. Full conformance is always explicitly false.
No generated workload, URI failure workload, silent case skip, platform fallback,
subsystem dashboard or broad performance claim is introduced here. The three
existing opt-in gates and wider T25 performance/fuzz/reduction/CI obligations
remain separate acceptance work.
