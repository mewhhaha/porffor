# Local Wasmtime versus V8 baseline

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

The v2 runner extends the historical timing harness with memory observations,
phase timings, retained emitted Wasm, workload selection and checked comparisons.
The 2026-10-20 table is the historical v1 result. The 2026-10-21 result above
uses v2 instrumentation; these schemas cannot serve as each other's performance
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
