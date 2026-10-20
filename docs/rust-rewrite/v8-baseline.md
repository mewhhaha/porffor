# Local Wasmtime versus V8 baseline

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

## Measurement and reproduction

`scripts/benchmark-v8.py` records three distinct measurements on the same
machine, validates every result and retains all raw samples and process logs.
It accepts only executables bound to actual default-feature, opt3 release Cargo
producer records. The compiler's normal product path executes generated Wasm
in Wasmtime; Node runs the same JavaScript in V8.

- **Cold start:** a new process and a unique empty scriptw application cache for
  each sample. This includes JavaScript compilation, Wasmtime native module
  work, instantiation and execution. Node uses its normal new-process behavior
  without a persistent compile cache. The OS page cache is not flushed.
- **Hot start:** a new process with a previously populated scriptw disk cache.
  Node is also a repeated new process, with no equivalent persistent code cache
  enabled. This is cache-warm startup, not an already running server.
- **Execution:** compile each identical workload once, then measure host
  monotonic execution spans after explicit warmups. scriptw uses the public
  `Engine::profile_wasm_execution` API on retained Wasm, a fresh Wasmtime Store
  per run, and its ordinary module cache. Node uses one compiled `vm.Script`
  with a fresh `vm.Context` per run. Source compilation and context creation
  are outside the measured span. Output and completion processing are inside.

The startup fixture prints a checked marker. Execution fixtures run 10,000
integer recurrence steps or allocate/traverse 500 objects; their exact expected
checksums are independently specified in the fixture manifest. No HTML or
results are precomputed into the program. The Node output adapter only maps
`print` to host output. These small programs do not establish application-wide
performance. V8's tiering and GC remain enabled. scriptw uses its product
default Wasmtime native policy (Fast, Cranelift `OptLevel::None`), recorded
explicitly rather than silently switching to a different backend for a result.

Both engines inherit the same single allowed CPU during measurement. The report
captures the CPU model, OS/kernel, host logical CPU count, inherited affinity,
cgroup CPU/memory/swap limits, Node/V8/Wasmtime versions, source revision,
executable hashes, source hashes, cache outcomes, raw nanosecond samples and
median/minimum/maximum. This cloud VM is the local baseline, not a controlled
hardware lab or a calibrated T25 acceptance claim.

From the repository root, after activating the Rust tools if necessary:

```sh
mkdir -p target/v8-baseline-build
python3 -B scripts/limited_verification.py --cloud --cloud-cpus auto -- \
  cargo build --locked --offline --release -p scriptw-cli \
  --bin scriptw --example runtime_baseline --jobs 1 --message-format=json \
  > target/v8-baseline-build/cargo.jsonl
python3 -B scripts/limited_verification.py --cloud --cloud-cpus auto -- \
  python3 -B scripts/benchmark-v8.py \
  --scriptw target/release/scriptw \
  --runtime-runner target/release/examples/runtime_baseline \
  --build-log target/v8-baseline-build/cargo.jsonl \
  --output-dir target/v8-baseline-new-run --samples 7 --idle-machine
```

Use a fresh output directory and an idle machine. Startup samples alternate
engine order. The hot runtime warmups are three Wasmtime runs and fifty V8
runs; every warmup is retained and excluded from measured summaries. The
longer V8 warmup allows normal JIT tiering without charging it to execution.
The report retains failed/incomplete observations instead of publishing a
partial run as a completed comparison. A failed checksum is a failed benchmark.
