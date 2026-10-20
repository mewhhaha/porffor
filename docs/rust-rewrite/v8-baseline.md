# Local Wasmtime versus V8 baseline

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
