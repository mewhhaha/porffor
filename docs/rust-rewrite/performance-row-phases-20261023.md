# Retained-row execution attribution — 2026-10-23

**P001, P002 and P009 are complete as diagnostics.** Construction of 8,192 retained
rows is dominated by collector work; traversal is dominated by object reads,
property lookup and numeric-key conversion. This identifies the next experiments;
it makes no optimization speedup claim.

[Verified analysis](performance-row-phases-20261023.json),
[evidence inventory](performance-row-phases-20261023-evidence.json) and
[portable raw archive](performance-row-phases-20261023.tar.gz) preserve all
40 runner processes, seven workloads, eight profiles, original emitted Wasm,
loaded-code maps, source files and failed initial diagnostics. Every process
passed its checksum, warmup, module-cache and artifact-identity checks.
The archive has 254 byte-verified members; the analysis was produced entirely
from its extracted contents. Normal Node independently checked all seven sources.

The compiler and native runtime are the accepted **`97036e5a8`** product
(source fingerprint `23b924ad7ee8a9b8d2ee88f068bbfb7c6c0cfd24198e44dfaf57c400b6acae94`).
The diagnostic runner is separately identified as SHA-256
`7e0b3225eedeee9e66b7c1ab51cf1683ff5b548639190ad45c50c7cc52adb350`.
It linked in **1.14 seconds** against byte-verified existing release libraries
and native R. This is a runner-only iteration, not a compiler rebuild or fresh
native/conformance acceptance. The ordinary README V8 table and full-suite
chart retain their separate source50 evidence.

Each child uses seven measured invocations after ten warmups, one pinned CPU
on the existing AMD EPYC 9V74 cloud machine, and a fresh Store per invocation.
There were no concurrent build, test or benchmark jobs. The row corpus uses
width four and zero, one or four complete traversal passes. Construction,
traversal and combined spans occur within the same invocation and retained
graph; endpoint row and array identity checks and checksums remain observable.
Zero passes gives the construction control, including its empty loop/marker
boundary. No phase is timed by subtracting separately executed workloads.

| Rows | Read passes | Construction, ms | Traversal, ms | Combined, ms |
| ---: | ---: | ---: | ---: | ---: |
| Empty markers | — | 0.0033 | 0.0013 | 0.0046 |
| 1,024 | 0 | 24.268 | 0.0037 | 24.275 |
| 1,024 | 1 | 23.051 | 5.743 | 28.852 |
| 1,024 | 4 | 27.844 | 25.701 | 53.351 |
| 8,192 | 0 | 1,262.910 | 0.0045 | 1,262.916 |
| 8,192 | 1 | 1,199.886 | 106.023 | 1,301.885 |
| 8,192 | 4 | 1,182.965 | 442.448 | 1,633.907 |

These are medians of two process medians with the observer enabled and perf
disabled. Each column is summarized independently, so displayed medians need
not add up. Every individual observation satisfies construction + traversal +
middle callback = combined. These are **wall-clock durations**, not CPU time or
allocation counters. Construction includes the checksum seed and retained
identity captures. Individual spans exclude endpoint host callbacks; guest
print entry/return work remains, and combined includes the middle callback.
The ordinary full invocation score includes construction, markers and checks.

The zero-pass 8× row increase costs about **52×** as much construction time.
Reported GC capacity starts at 16 MiB and ends at 16 MiB for 1,024 rows or
64 MiB for 8,192 rows. Capacity and sampled process RSS are not live JS bytes,
collection counts or measured pause totals. Raw memory observations remain in
the archive; capacity changes alone do not establish why a collection occurred.

## CPU samples identify different phase costs

Linux perf 499-Hz `cpu-clock:u` sampling starts disabled. The synchronous host
hook enables it only for the selected measured region and disables it at the
endpoint; compilation and ten warmups are excluded. Two independent processes
record each region. All **23,970 samples** match their map's process ID and
original perf sample records, with **zero lost samples**. Perf process metadata
also matches the map PID. R/P Wasm hashes, loaded native-text identities, map
bytes and original perf data are bound in the evidence. Analysis uses sampled
leaf instruction addresses, without inferring complete callers from callchains.

This also completes P009: the maps enumerate loaded R/P function names,
Wasmtime indices, native offsets and native byte lengths. Names/sizes and sample
addresses bind to the exact Wasm, loaded native text and runner executable.
The maps remain serial single-program snapshots, without concurrent mapping
history or exact per-function timing counters. No separate disassembly export
is claimed.

| Region | Samples in each process | Dominant sample shares |
| --- | ---: | --- |
| Construction | 4,246 / 4,304 | Named collector functions: **83.07% / 84.39%** |
| Traversal | 1,697 / 1,670 | `object_read`: **54.10% / 57.31%**; `ordinary_property_find`: **18.50% / 20.06%**; `number_to_string`: **12.96% / 10.06%** |
| Combined | 6,318 / 5,735 | Named collector functions: **61.02% / 63.61%** |
| Empty markers | 0 / 0 | Too short to attribute; timing controls remain available |

Collector totals include the Wasmtime copying-collector symbols and
`VMGcHeader::kind`; the exact preserved mangled-name selection is `7copying`
or `10VMGcHeader4kind`. The largest are `CopyingHeap::forward`,
`CopyingHeap::scan_field`, `CopyingCollection::collect_increment`, header-kind
inspection and `EphemeronMarking::copied`. Unresolved samples are retained and
are not assigned to GC. These percentages are **shares of sampled instructions**,
not exact fractions of elapsed time or precise GC pause accounting.

## Observer and profiler costs remain visible

The observer controls use the same marked JS and runner in off/on/on/off order.
Thus they compare callback observation, not the cost of adding JS markers to
an unmarked program. Adjacent paired effects vary from **−11.75% to +17.28%**.
The 1,024-row/four-pass control is slower in both observed pairs (**+17.28% /
+10.30%**); the 8,192-row/four-pass control is faster in both (**−5.45% / −2.05%**).
Do not interpret this variation as a precise hook cost or a hook speedup.
All pairs, including slower controls, are retained in the analysis.

Separate profiler off/on/on/off controls keep the observer enabled. Empty
invocation medians are **1.065 / 3.867 / 3.681 / 1.251 ms**: the FIFO wait adds
roughly 2½ ms there. The 8,192-row/four-pass medians are
**1,612.349 / 1,839.882 / 1,683.664 / 1,764.975 ms**, paired effects **+14.11% /
−4.61%**. Sampling and control work are instrumented diagnostics; neither these
scores nor empty-control subtraction replace ordinary benchmark results.
Individual phase timers exclude their acknowledgement callbacks, while the
full invocation includes them. Sampling itself can include boundary-control
CPU work. Empty profiles collected no samples, so they do not measure its
exact CPU share.

## Next experiment and verification

The first target is **P147**, a bounded initial-GC-capacity comparison, retaining
startup, small-workload, memory and forced-collection controls. The current
16-MiB default is unchanged. Wasmtime's retry policy already predicts whether
to collect or grow first; these profiles do not prove that policy is defective.
Scriptw has no current capacity override, so the experiment needs an explicit
diagnostic configuration with a fresh process and matching Engine per variant.
Avoid changing the object representation before testing this smaller hypothesis.

Traversal provides a separate target: investigate the concrete indexed-read
path and numeric-key conversion before another cache or push optimization.
P038's rejected literal batching and P040's earlier push experiment remain
rejected; this analysis does not retroactively accept either implementation.

Four Rust observer tests and seven Python corpus/evidence tests pass. The real
perf FIFO test verifies three measured enable/disable pairs and no warmup
commands. Its first attempt discovered perf's NUL-terminated acknowledgement;
the handler and test were corrected. A later export command used an unsupported
option spelling; the exporter now uses `--hide-call-graph`. Both failed attempts
remain in the archive. Neither required rerunning the 40 measurements.

Generate new measurements with the [row-phase tooling](v8-baseline.md#retained-row-phase-diagnostics).
Reproduce this analysis without a runner, perf, JavaScript execution or builds:

```sh
mkdir -p target/row-phase-evidence
tar -xzf docs/rust-rewrite/performance-row-phases-20261023.tar.gz \
  -C target/row-phase-evidence
python3 -B target/row-phase-evidence/evidence/sources/analyze.py \
  target/row-phase-evidence/evidence/measurements \
  --output target/row-phase-reproduced.json
cmp docs/rust-rewrite/performance-row-phases-20261023.json \
  target/row-phase-reproduced.json
```

The current performance ledger is **29 terminal / six partial / 125 unresolved**.
This closes three diagnostic hills, not the remaining performance or broader
acceptance work.
