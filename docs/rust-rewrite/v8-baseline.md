# Local Wasmtime versus V8 baseline

## Release packaging checkpoint — published 2026-10-22

Measured compiler: `b59303ebe953e5f07ad7aebba467211dd2ac88ff`, source fingerprint
`db6e85fc67799d1f95ea938c63631cfa3e89299db30c8533d2db9ee7c36c9d9a`.
The [README table](../../README.md#benchmarks-vs-v8) uses its actual release
Cargo CLI and runner, seven samples and ten execution warmups. The
[portable evidence](performance-kept-packaging-20261022.json) retains ordinary
reports, source-frozen native acceptance, compiled semantic controls, paired
execution controls, Cargo producers/timings and reproduction harnesses. Each
embedded gzip+base64 asset is independently length/digest verified.

The CLI is **290,489,840 bytes (277.03 MiB)**, **202.07 MiB smaller** than the
preceding accepted release. Rust debug information is stripped from the CLI and
examples; symbols and compiled panic locations remain. Full loaded-executable
SHA-256 and independently validated compiler/profile-bound cache identities are
preserved. The product remains statically linked to its runtime image archive;
the test-only shared linker is used only for libtest executables.

Ordinary cold/cached medians are **1,059.284 / 818.785 ms**, versus V8
**33.660 / 35.440 ms**. These are separate ordinary reports, not paired effects.
The earlier allocated-section-equal strip experiment below isolates the startup
benefit. One 1,645.712 ms cached-start outlier remains in the raw samples.
Process memory is essentially unchanged: cold/cached medians **382.83 / 349.91
MiB**, retained-row whole-child peak **392.82 MiB** and median invocation-sampled
RSS **373.34 MiB**. V8's retained whole-child peak is **52.88 MiB** and its
invocation endpoint median is **49.13 MiB**. GC capacity is still 64 MiB;
none of these measures is live JavaScript heap usage.

The three ordinary execution medians are **8.363 / 18.966 / 1,470.230 ms**.
Short-run variation prompted separate seven-sample/ten-warmup case-wise ABBA
controls against the prior frozen runner. Integer cohort medians are
**10.038→7.933 ms** and object medians **19.654→18.174 ms**. These controls do
not show a consistent execution regression; they do not establish an isolated
algorithm gain or statistical significance. The README retains the actual
ordinary scores rather than substituting favorable paired observations.

Native acceptance is **3,568 passed / 0 failed / 0 ignored**: AOT 444, engine
974, builtins 541, language 324, realm/modules 428, CLI 854 and runtime-image 3.
Source and linker remained unchanged throughout; all ignored controls were
explicitly enabled. Seven product/test image payloads and manifests are verified
byte-identical. Six actual compiled CLI semantic fixture families pass. The
**9,411-execution Test262 replay returns 9,407 Success / 4 Bug / 0 Crash /
0 NotImplemented**, with exactly the original four upstream-tracked failures.
[Final replay evidence](performance-kept-packaging-replay-20261022.json) supplements
the earlier immutable native/measurement snapshot, which records replay pending.
Four workers ran after native verification drained, with existing per-case
deadlines retained. This adds 204 array-expression executions to the preceding
9,207-case selection. This affected replay and scoped native checkpoint do not
refresh full-suite counts; all four tracked failures remain open.

The product build took **8m44s**. Cargo timings identify repeated AOT LLVM work
and two Intl feature/profile variants as major costs; runtime-image production
was only **5.04s** with its exact-input native cache warm. Overeager detachment
of unchanged preserved Intl outputs caused about **143s** of avoidable rebuild
work. A next-batch transparent rustc wrapper has passed a tiny real-Cargo
fresh/rebuild/fresh probe: it preserves unrelated fresh dependencies and detaches
only an actually rebuilding unit's verified output aliases. Bounded four-slot
LLVM compilation is also available. Neither prototype has yet been timed on a
Scriptw product build. CLI Intl tests independently repeat SDK/CLI P emission;
existing exact raw-R caches can be shared only within matching profile/selection
identities. Slow test spans have not been attributed to startup alone.

The next replacement experiment has twelve source fixtures in
`crates/scriptw-cli/tests/fixtures/v8_replacements`. They use the existing
`benchmark-v8.py --fixtures … --skip-startup` interface and exact-output checks;
whole-workload timings include receiver/template construction and output
checksum. Callback and plain/dollar-template cohorts separate whole-result
assembly from substitution work; deletion/no-match/single-match controls check
the small cases. No measurements or acceptance are claimed for that source yet.

The opt-in `scripts/test-shared-runtime-images/preserve-output-rustc` wrapper
composes the existing quota-bounded LLVM jobserver without altering rustc
arguments. `SCRIPTW_PRESERVED_OUTPUT_INVENTORY` names a JSON object with absolute
`cargo_target_dir` and `files` entries (`original`, `storage`, `sha256`);
`SCRIPTW_OUTPUT_PROTECTION_LOG` names an append-only intent journal. Every owned
output alias for the rebuilding rustc unit is validated before any is detached.
The preserved payload stays intact, unrelated dependency aliases stay attached,
and queries remain literal. The inventory must contain only task-owned aliases.
Build-script-produced runtime image outputs still require separate protection;
this wrapper only intercepts rustc. `SCRIPTW_TEST_CODEGEN_JOBS=auto` requires
serial Cargo and a finite inherited cloud budget. Twelve ownership/argument controls and
a tiny real-Cargo probe pass; an actual Scriptw build has not yet measured the
iteration-time benefit.
The existing native `test-rustc` wrapper also admits that optional inventory;
its existing test/incremental policy stays intact. This avoids protecting product
outputs at the cost of stale test-graph dependencies or the reverse. Metadata-only
compilation leaves the unit's unwritten linked outputs attached.

## Second execution batch — published 2026-10-22

Measured compiler: `86518905134e401bd9533697e1d93007558e54cf`, product source
fingerprint `7e96c784036365a6f8f7da1a84bcbdb841e17feb847ceda76800714cc8e93266`.
This source's seven-sample, ten-warmup Wasmtime/V8 report is retained as the
preceding baseline. The [portable evidence](performance-second-wave-20261022.json)
retains complete
ordinary reports, the 61-case corpus, paired controls, rejected variants,
Cargo producers, native acceptance and host profiles. Assets retain the verified
gzip+base64 format described below; executable images, archives and private
caches are excluded.

Native acceptance is **3,567 passed / 0 failed / 0 ignored** across AOT library
444, engine library 974, builtins 541, language 324, realm/modules 428, CLI 853
and runtime-image library 3. All ignored controls were explicitly enabled.
The two checkpoint receipts have identical source maps and unchanged source;
product/test raw, native, compressed and manifest image bytes match. This is
scoped default-feature verification, not full-workspace/all-feature acceptance.
Ten compiled semantic fixture families and all 61 ordinary corpus cases pass.
The final-source **9,207-execution Test262 replay returns 9,203 Success / 4 Bug /
0 Crash / 0 NotImplemented**, with exactly the original four upstream-tracked
failures and no new regressions in this campaign.
[Final replay evidence](performance-second-wave-replay-20261022.json) supplements
the earlier native/measurement snapshot. The intermediate F replay remains
historical. The README graph still describes its existing reconciled full-suite
checkpoints; this affected-case campaign does not refresh full-suite counts.

The kept batch shares immutable empty private storage, shortens concrete array
length and normal-call completion paths, bulk-copies repeat/pad/partial slices,
reuses full Strings, and shares identical-operand descriptor snapshots. Binding
payload flattening, signed-range conversion and runtime Number-tag branches
were rejected and removed. Layout remains 15. Paired short controls overlap:
F→final cohort medians are integer 7.775→7.667 ms, objects 19.384→17.732 ms and
retained rows 1,506.528→1,469.976 ms. These descriptive diagnostics use a separate
cohort from the README; they do not establish isolated per-change contributions.

Here C denotes `621ba89ca` and F denotes `a5b4a6c5d`.
The preceding C→F ABBA string controls establish the larger changes: repeat
169.226→19.993 ms, padEnd 4.460→1.498 ms, trimStart 24.120→1.617 ms, trimEnd
24.129→1.390 ms, full slice 6.691→1.475 ms and partial slice 6.860→2.618 ms.
The final ordinary corpus records 23.834 / 1.847 / 1.472 / 1.600 / 1.384 / 2.352 ms
for those cases. These are separate source-bound cohorts; do not multiply ratios
or call them a direct C→final paired effect. Final repeat/partial-slice observations
are slower than the matching F ordinary report. No universal speedup is claimed.

Portable native storage now occupies **21,713,669 bytes**, decoding to
**80,992,360 bytes**. NativeHost/raw package bytes and independent manifest,
digest and actual destination-Engine checks remain required. A weak lease shares
concurrent loads and releases decoded storage after Wasmtime copies it into its
own Module mapping. Final CLI size is **502,376,224 bytes**, **57.12 MiB smaller**
than the preceding accepted batch's 562,274,800 bytes.

The retained-row run records median invocation process RSS **373.68 MiB** and a
separate whole-child high-water peak **392.80 MiB** (one observation). The preceding
accepted report recorded 439.91 / 439.73 MiB; F's retained decoded buffer recorded
454.86 / 454.66 MiB. These are combined-batch process measurements with different
boundaries, including runtime/caches and, for the whole child, compilation and
warmups. They are not live JS heap sizes. GC capacity remains 64 MiB. V8's final
invocation RSS endpoint is 49.38 MiB; its whole-child peak is 53.12 MiB. Those
boundaries must not be merged into one heap comparison.

Four separately instrumented cached CLI profiles attribute **64.38–74.32%** of
65–74 user CPU samples per child to SHA-256, with no lost samples. Full loaded
executable binding currently hashes approximately 500 MB once per fresh process.
This is whole-child host attribution, not ordinary startup timing or a JS-span
profile. A separately labelled debug-stripped copy is prepared, with allocated
ELF code/data equality and distinct full executable identity verified. The
[completed startup diagnostic](performance-startup-strip-20261022.json) records
14 fresh processes per variant/mode in ABBA order (seven per round):

| Boundary | Full parent | Derived debug-stripped copy |
| --- | ---: | ---: |
| Identity binding | 376.835 ms | 221.256 ms |
| Cold application start | 1,171.645 ms | 1,018.480 ms |
| Cached application start | 923.743 ms | 739.889 ms |

Both variants retain full loaded-executable hashing, separate independently
primed application caches and embedded-native admission. All allocated ELF
sections match. Size falls 502,376,224→289,166,424 bytes (203.33 MiB); process RSS
does not materially change. These derived-copy diagnostics are separate from
ordinary V8 scores. Rust DWARF line tables disappear; global symbols, compiled
panic locations and Wasm code/data remain. Ordinary producer/hash checks and
compiler-bound cache identity remain intact.

The next source batch keeps release CLI/example debug stripping. Its combined
P017/P038 literal experiments were built and measured, then removed because they
did not establish a throughput gain. [Exact decisions and 216 verified assets](performance-literal-decisions-20261022.json)
retain the 14-case screen, ten-case ABBA confirmation, six compiled semantic
families, actual Cargo producers and 24 unique Wasm programs. Object nine-key and
33-key medians rise 17.823→19.393 / 47.447→50.464 ms; four-element arrays are
9.917→10.109 ms and retained rows 1,369.244→1,395.373 ms. These descriptive
Wasmtime-only cohorts are separate from ordinary V8 scores and do not isolate
each combined change. The packaging-only product and scoped native acceptance
and its completed expanded Test262 replay are now published above.


## First execution batch — published 2026-10-22

Measured compiler: `621ba89ca04a3522192b97afdd2c9fae617009a7`, product source
fingerprint `31caadf8de6aa83eb3581a6ba0a49e9d8147d4f2cc43c490abb72b0dc306bf76`.
The historical seven-sample, ten-warmup table is retained in the evidence.
[Portable evidence](performance-first-wave-20261022.json)
retains the ordinary reports, checked comparisons, rejected experiments, exact
Cargo producers, native checkpoint, original-case replay and instrumented profiles.
Assets use the same verified gzip+base64 format described below; ELF images and
private cache payloads are excluded. Report sources remain their actual tested
commits, independently of this later documentation commit.

NativeHost and Portable now have **separate** matching optimized runtime R
images, SpeedAndSize/Backtracking, with profile-specific raw keys and v4 manifests.
Generated P keeps Fast/None/SinglePass. Admission is lazy per profile and always
uses the actual destination Engine; no mutable Store or realm is shared.
The startup prime receipt proves NativeHost admission. The separate Portable
trace and runtime-key proof bind the execution corpus to actual Portable image
admission; the startup flag alone does not prove that path.

In the matched seven-sample confirmation against `3b1a3427f`, integer execution
fell from 17.481 to 8.109 ms, 500 objects from 44.053 to 18.275 ms, and retained
rows from 1,904.600 to 1,385.741 ms. These describe the combined concat/runtime
batch, with no general JS score or significance claim. A separate focused ABBA
concat study retains the earlier compiler's 3.17× and 13.47× gains on large-part
inputs. The numeric Array own-data experiment was rejected under the preceding
unoptimized Portable runtime policy and removed; its raw failed attempt remains.

The image tradeoff is explicit: the CLI grew from 440,628,904 to 562,274,800 bytes
(**116.01 MiB**). In a same-session before/current/current/before startup check,
14 observations per image/mode gave medians 1,137.973 → 1,195.838 ms cold and
904.560 → 926.627 ms cached. This accepts a measured startup/image cost in return
for execution gains; it does not call startup improved. Samples retain ranges
and round order. The main table is the separate paired Wasmtime/V8 run.

Retained-row whole-child peak RSS was **439.73 MiB** versus V8 **53.21 MiB**,
one observation per process. The invocation's median sampled process maximum
was **439.91 MiB**; V8's median endpoint was **48.96 MiB**. Different measurement
boundaries and kernel/proc sampling need not agree exactly. GC capacity remains
64 MiB and is not live JS memory. No allocation, live-byte, collection or pause
counters are claimed.

The separate loaded-code profile collected **670 user CPU samples, zero lost**.
About **59%** of self samples name copying-GC functions; ordinary property
read/find helpers account for about 3.58% each. Actual R/P function names now
resolve through the opt-in map, bound to loaded text and Wasm hashes.
This is whole-child attribution including compile/warmup/hash work, not a claim
that 59% of the execution span is GC. Four map-off/on/on/off observations retain
instrumentation controls; their overlap does not establish zero profiler cost.

The ordinary native checkpoint passes **2,687 tests**: engine library 973,
builtins 541, language 322 and CLI 851, including their ignored controls.
The source-bound Test262 replay is **182 Success / 4 Bug** across 186 original
executions, with the same four upstream-tracked failures and no new failures.
The replay compiler is `4fe934c4b`, with the same product fingerprint as `621ba89ca`;
the latter changes a Rust test only. This is neither a fresh full Test262 run
nor full-workspace/all-feature acceptance. The separate 85-workload Portable
correctness run competed with native verification: its timings are excluded.

## Performance and memory baseline — 2026-10-21

Measured source: `cbbd9b17039c595d470b5bc140b4ab7a1f2beea9`. Ordinary default-feature opt3 release;
seven measured observations per timing cell after ten execution warmups.
Both runtimes used one pinned CPU on the AMD EPYC 9V74 cloud machine, with a
32 GiB memory limit and four-CPU quota. Linux 6.18.55 x86_64 / glibc 2.41;
Wasmtime 47.0.0; Node 24.19.0 / V8 13.6.233.17-node.51.

| Measurement | scriptw / Wasmtime median | Node / V8 median |
| --- | ---: | ---: |
| Cold start | 1,117.891 ms | 31.094 ms |
| Cached start | 938.986 ms | 36.474 ms |
| Execute: 10,000 integer steps | 19.393 ms | 2.584 ms |
| Execute: 500 objects | 51.342 ms | 0.061 ms |
| Execute: 8,192 retained objects | 2,118.118 ms | 3.575 ms |

| Whole-child high-water RSS | scriptw / Wasmtime | Node / V8 |
| --- | ---: | ---: |
| Cold start (median) | 384.16 MiB | 27.12 MiB |
| Cached start (median) | 350.95 MiB | 27.12 MiB |
| Retained-object run (one process) | 558.05 MiB | 53.12 MiB |

Every checksum matched. Every measured Wasmtime execution hit the native module
cache, and startup priming confirmed admission of the embedded precompiled
runtime. Wasmtime uses a fresh Store; V8 uses a fresh context. Source compilation
and Store/context setup are outside the execution timer. The execution cache
was conditioned before this run; cold application startup caches remain unique
and empty. OS caches are not flushed.

Startup measures the CLI's NativeHost profile, whose embedded runtime is already
optimized with SpeedAndSize and Backtracking register allocation. The execution
driver emits Portable artifacts through `profile_script_compilation`; that
different source-compilation profile does not use the embedded NativeHost image.
Its runtime follows ordinary native compilation/cache admission for its profile.
Keep these boundaries distinct when attributing execution costs or comparing
compiler policies; the [performance backlog](../performance-hills.md) records
the matching-image experiment as P153.

[Source-bound evidence](performance-memory-baseline-20261021.json) contains the
full report, every warmup and measured observation, all child logs, workload and
emitted Wasm bytes, Cargo producers/build log, harness bytes, machine limits and
validation logs. Assets use deterministic gzip+base64 with original and compressed
SHA-256. Executable ELF images and private caches are excluded; actual executable
hashes and producer records remain in the reports.

Whole-child peaks include compilation, warmups, caches and host work. During the
retained-object invocations, Wasmtime's median sampled process RSS maximum was
**441.69 MiB**; V8's median process RSS endpoint was
**48.88 MiB**. A sampled maximum and an endpoint have different boundaries.
Wasmtime GC heap capacity was **64 MiB**, while V8 process-wide used heap was
**12.87 MiB**; those are different metrics, not comparable live JS heap sizes.
The runner explicitly reports unavailable Wasmtime live-byte, allocation and
collection/pause counters rather than inventing them.

The complete run took **59.6 seconds**. Two matched three-sample/three-warmup
integer runs took **3.0 and 3.2 seconds** after the initial **39.1-second**
execution-cache conditioning report. Offline comparison and publication
round-trip validation pass. The matched pair uses the same compiler source:
it demonstrates the short iteration workflow and observed variation, not an
optimization gain. The 8,192-object workload, at about 2.12 seconds per invocation,
is the largest absolute execution-time target in this small corpus.

Validation includes the completed release build, 13 standalone Rust fingerprint
controls, final module-boundary checks and three source-guard mutation controls.
The full 703-test tooling checkpoint predates the build-coherence guard; its exact
source and logs are retained alongside the subsequent focused controls. These
measurements do not establish full native, Test262 or calibrated T25 acceptance.

The initial `target/performance-memory-baseline-20261021` run remains a diagnostic,
not this healthy baseline. A shared Cargo target had reused an old custom-build
executable with a stale compiler-input inventory, rejecting the embedded runtime
and recompiling it at cold startup. Rebuilding only that custom-build executable
restored source/image identity. Current build scripts attest their fingerprint
implementation against the checkout. Earlier process RSS accounting could also
inherit the Python launcher's high-water floor; the current runner uses a small
GNU time supervisor to measure the actual runtime child.

To unpack and verify the published assets for offline report comparison:

```sh
python3 - <<'PY'
import base64, gzip, hashlib, json
from pathlib import Path
bundle = json.loads(Path('docs/rust-rewrite/performance-memory-baseline-20261021.json').read_text())
root = Path('target/performance-evidence-20261021')
root.mkdir(parents=True, exist_ok=False)
for asset in bundle['assets']:
    packed = base64.b64decode(asset['data'], validate=True)
    assert hashlib.sha256(packed).hexdigest() == asset['gzip_sha256']
    raw = gzip.decompress(packed)
    assert len(raw) == asset['bytes'] and hashlib.sha256(raw).hexdigest() == asset['sha256']
    path = root / asset['path']
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(raw)
PY
python3 -B scripts/benchmark-v8.py compare \
  target/performance-evidence-20261021/runs/iteration-before/report.json \
  target/performance-evidence-20261021/runs/iteration-after/report.json
```

## Measured cloud baseline — 2026-10-20

Measured source: `c5053c9d4ed1ce961f66eff8cd0d721b39108ffd`. Ordinary default-feature opt3 release,
with no product oracle dependency; seven measured samples in every cell.
AMD EPYC 9V74 80-Core Processor; 32 GiB cloud memory limit; 4-CPU quota.
Both runtimes used the same one pinned CPU.
Node 24.19.0 / V8 13.6.233.17-node.51; Wasmtime 47.0.0.
Kernel/platform: `Linux-6.18.55-x86_64-with-glibc2.41`. Architecture: `x86_64`.

| Measurement | scriptw median [min, max], ms | V8 median [min, max], ms |
| --- | ---: | ---: |
| Cold start | 1221.570 [1124.999, 1449.517] | 32.789 [32.268, 64.987] |
| Hot start (disk cache) | 920.607 [869.768, 1024.726] | 35.565 [32.464, 74.314] |
| Execute: 10,000 integer steps | 17.020 [16.040, 17.523] | 2.400 [1.684, 2.552] |
| Execute: 500 objects | 44.490 [42.013, 62.516] | 0.044 [0.041, 0.067] |

All checksums matched. Integer output: `730532261`; object output: `149600`;
startup output: `scriptw-startup-ok`. Every measured Wasmtime execution hit
the in-process native module cache. [Source-bound evidence](v8-baseline-20261020.json)
contains the complete measurement report, all nanosecond samples and warmups,
process logs, Cargo producers, harness/fixture bytes and machine limits.
Assets use deterministic gzip+base64 with original and compressed SHA-256.
Executables are hash-bound to their actual Cargo producers; ELF images are
not embedded. The first rejected build and its fixed-width wire-format repair
are retained separately, along with the subsequent Intl identity repair and
sparse-fixture admission failure. The main baseline completed without failed samples.

The shared runtime image is compiled during the Rust build and reused even
for a cold application cache. The generated program modules use Wasmtime's
ordinary Fast/Cranelift `OptLevel::None` policy. The CLI endpoint includes its
outcome diagnostic, while Node only prints the checked program output.
Startup spans include the driver's receipt/log setup and child-process wall time.
Execution includes Scriptw realm/bootstrap/completion work; fresh Store and
V8 context setup sit outside the measured span. Warmups on fresh V8 contexts
do not guarantee the same JIT tier in every measured context.
These boundaries make this an explicit product baseline, not isolated
instruction-throughput or calibrated T25 acceptance.

## Current performance and memory tooling

The [performance backlog](../performance-hills.md) owns proposed optimizations,
their priorities, semantic controls and the first measurement batches.

The v2 runner extends the historical timing harness with memory observations,
phase timings, retained emitted Wasm, workload selection and checked comparisons.
The 2026-10-20 table is the historical v1 result. The 2026-10-21 results above
use v2 instrumentation; these schemas cannot serve as each other's performance
baseline.

Requires Linux, GNU `time`, Node and the Rust toolchain. Use `--time PATH` for
a GNU time executable outside PATH. Use the release/default-feature product CLI
and the companion runtime runner.
Commit the coherent source batch before building: both executables must identify
that exact commit. Keep the Cargo JSON build log. Run builds and tests before
benchmarking, then measure on an otherwise idle machine. The cloud launcher uses
the inherited machine limits; the benchmark pins both runtimes to one allowed CPU.

```sh
mkdir -p target/performance-build
python3 -B scripts/limited_verification.py --cloud --cloud-cpus auto -- \
  cargo build --locked --offline --release -p scriptw-cli \
  --bin scriptw --example runtime_baseline --jobs 1 --message-format=json \
  > target/performance-build/cargo.jsonl
python3 -B scripts/limited_verification.py --cloud --cloud-cpus auto -- \
  python3 -B scripts/benchmark-v8.py \
  --scriptw target/release/scriptw \
  --runtime-runner target/release/examples/runtime_baseline \
  --build-log target/performance-build/cargo.jsonl \
  --output-dir target/performance-before --samples 7 --warmups 10 --idle-machine
```

If `CARGO_TARGET_DIR` is set, use its actual executable paths in these commands.
The output directory must be new. Default startup priming enables compilation
trace outside the measured samples and requires a confirmed embedded-runtime hit.
A stale compiler/image source stamp fails here with retained diagnostics, before
spending time on cold samples. Both build-script producers also verify that their
compiled fingerprint implementation matches the current checkout. If that check
rejects a shared target directory, remove only the affected release build artifacts
with `cargo clean --release -p scriptw-engine -p scriptw-runtime-image`, then rebuild;
there is no need to clean third-party compiler dependencies. The runner keeps an incomplete report and logs
on failure; a failed checksum, trap or timeout fails the benchmark. It records
machine/cgroup limits, one-CPU affinity, runtime versions, source and executable
identities, exact workload bytes and output checksums. JavaScript still follows
the ordinary Rust compiler to generated Wasm and the product Wasmtime policy.

For a shorter optimization loop, select an existing manifest entry and omit
startup. Use the same options for before/after comparisons:

```sh
python3 -B scripts/limited_verification.py --cloud --cloud-cpus auto -- \
  python3 -B scripts/benchmark-v8.py \
  --scriptw target/release/scriptw \
  --runtime-runner target/release/examples/runtime_baseline \
  --build-log target/performance-build/cargo.jsonl \
  --output-dir target/performance-objects-before \
  --case retained-objects.js --skip-startup --samples 3 --warmups 3 \
  --execution-cache target/performance-execution-cache --idle-machine
```

The explicit execution cache reuses the ordinary checked runtime/function/module
caches across runs. It does not affect isolated cold/cached startup measurements.
Compilation and warmup observations remain outside the execution summaries.

Generate the attribution corpus to vary retained row counts, widths and read
passes, and cover call/property/string/collection/JSON families:

```sh
python3 -B scripts/performance_corpus.py --output-dir target/performance-corpus \
  --rows 256,1024,4096 --widths 4 --passes 0,1,4
```

Pass `--fixtures target/performance-corpus` to the benchmark, then select entries
such as `--case rows-n1024-w4-p0.js --case rows-n1024-w4-p4.js`.
These are whole-invocation measurements: construction remains included in every
case. Increasing read passes supports marginal-cost attribution; it is not a
separately timed traversal-only span. Generated checksums are independently
validated in normal V8. Use `--first-runtime v8` to reverse the recorded runtime
order in paired repeatability runs without changing workload or sample plans.
After a complete report, derive checked phase attribution with
`python3 -B scripts/performance_study.py REPORT --output NEW_STUDY.json`.
The runtime driver now records the Portable profile and linked runtime key/hash
explicitly, alongside emitted-program and compiler identities.

After a compiler change, commit and rebuild with a new build log. Repeat the
same measurement with a fresh output directory and optionally
`--baseline target/performance-objects-before/report.json`. Existing reports
can also be compared without rerunning JavaScript:

```sh
python3 -B scripts/benchmark-v8.py compare \
  target/performance-objects-before/report.json \
  target/performance-objects-after/report.json
```

The runner writes a readable `summary.md`, raw `report.json`, child stdout/stderr
under `logs/`, exact sources and filtered manifest under `workload/`, and compiled
artifacts under `wasm/`. Preserve the entire directory when moving a baseline;
comparison verifies retained bytes and recomputes summaries from raw observations.
Different hardware/affinity, sample plans, workload bytes, runtime versions,
measurement policy, release profile or harness bytes reject comparison. Compiler
commits, compiler fingerprints and emitted Wasm may differ as part of optimization.
Ratios are observations, not a statistical-significance verdict. V1 reports remain
historical and are rejected as v2 baselines.

## What is measured

- **Cold start:** a fresh child process with a unique empty Scriptw application
  cache. The shared runtime is already compiled into the release build. Node
  starts normally with its persistent compile cache disabled. OS caches stay warm.
- **Cached start:** a fresh child process after priming the Scriptw disk cache.
  Node remains a new process without an equivalent persistent code cache. This
  does not measure a running server.
- **Execution:** each case has its own process. Source compiles once, then both
  runtimes use the same explicit warmup count and measured count. Wasmtime uses
  a fresh Store and V8 a fresh vm.Context per invocation. Context/Store setup and
  source compilation sit outside the execution span; output/completion processing
  is inside. V8 tiering and natural GC stay enabled; equal warmup counts do not
  prove equal steady-state/JIT tiers. Measured Wasmtime module-cache hits are
  required. Warmup observations remain in raw data and are excluded from summaries.
- **Phases and bytes:** Wasmtime reports native-module/cache work, Store/linker,
  instantiation, export lookup, execution and total. Preparation/lowering/emission/
  validation spans are single compilation diagnostics, not repeated benchmarks.
  The exact emitted Wasm and body/data footprint identify code-size changes.
- **CPU and process memory:** GNU `time` gives the runtime child's kernel high-water RSS,
  excluding the Python launcher's inherited pre-exec footprint. Linux `wait4`
  gives CPU time for the small native supervisor and its waited child. Startup has repeated child observations;
  execution has one whole-child observation per runtime/case, including compilation,
  warmups and retained runtime caches. Blocking wait avoids timeout-polling rounding
  in short startup spans. Driver receipt/log writes sit outside the wall timer;
  the GNU time supervisor, process launch, watchdog setup and scheduling stay inside. A watchdog kills the
  owned process group at the explicit timeout.
- **Invocation memory:** Wasmtime's existing profiler samples process RSS every
  10 ms plus endpoints around runtime admission through completion. Short peaks
  can be missed; it includes process-wide caches, host work and sampler overhead.
  V8 reports process memory endpoints before context creation, before execution
  and after execution. Earlier warmups and contexts can contribute to those process
  totals until normal GC runs. These are not per-workload live bytes or peak samples.
- **Heap observations:** Wasmtime exposes GC heap *capacity* before/after execution.
  V8 exposes process-wide used/reserved heap, external and ArrayBuffer endpoints.
  They have different meanings. Do not subtract process RSS to invent a JS heap,
  equate Wasmtime capacity with V8 used heap, or interpret natural GC deltas as
  allocation totals. Wasmtime live bytes, allocation totals, collection counts,
  GC pauses and peak GC heap stay explicitly unavailable. No forced GC is added.

The default corpus retains the original 10,000-step integer and 500-object
workloads and adds 8,192 retained objects, strings and four-element arrays with
32,768 indexed reads. Each prints an independently specified checksum. These
small kernels help find costs; they do not establish application-wide performance.
Use `--fixtures PATH` for a directory with the same manifest schema, a checked
startup entry and nonempty execution entries (`file`, `stdout`, optional `work`).
Case files must have distinct `.js` basenames. An entry may name any self-contained
script supported by both runtimes; `print` is the common output interface. The
benchmark rejects incorrect output instead of accepting a faster wrong result.

For the existing fixed twenty-case Wasmtime compiler/runtime reports, see the
[compiler profile](contracts/compiler-performance-profile.md),
[runtime profile](contracts/runtime-performance-profile.md) and
[whole-CLI reporter](contracts/performance-reporting.md). Those reports use their
own fixed Test262 host corpus and policies; do not mix their timing spans with
this normal-product/V8 comparison. Wider calibrated T25 acceptance remains open.
