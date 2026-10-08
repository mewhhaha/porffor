# Compiler stage and artifact profiles

`lila performance compile --output-dir PATH --samples 3..100 --machine-label LABEL
--idle-machine` measures the existing twenty-source performance corpus serially.
Each source gets one full warmup, followed by the requested measured compiles.
The original whole-CLI timing gates remain a separate `performance report`.

The SDK's `Engine::profile_script_compilation` calls the actual product
preparation, lowering and Wasm emission methods. Preparation includes parsing,
syntax errors and dependency/prelude admission; lowering includes spec IR and
coded early/resolution rejection. Validation uses Wasmtime with the original
product capability policy. Worker startup, runtime engine setup and artifact
inventory analysis are outside those four spans. The compiler profile does not
read the program artifact cache or execute JavaScript. It requires the product
experimental Wasmtime capability set without a second validation policy.

The report retains raw nanosecond spans, medians and nearest-rank percentiles
for every source and stage. Its hardware, compiler/image, toolchain, profile,
cache/environment and resource identities use the existing reporting owners.
Only compiler/image identity may differ in a baseline comparison. Debug and
optimized reports retain their actual build profiles and are not treated as
interchangeable baselines. Comparisons include stage deltas and both artifact
footprints; they do not inherit the whole-CLI gate's time budgets.

Every completed sample retains the actual emitted Wasm by content digest. The
inventory records module size, imported/defined function counts, encoded code
and largest body size, exact duplicate encoded bodies, data payload and every
named custom-section payload. Identical encoded bodies are a size diagnostic;
they do not prove that different function indices can be merged. Custom-section
rows expose the actual carried data/metadata footprint without estimating it
from locale or source counts. Warmup and measured artifacts must agree exactly.

The fresh output directory begins incomplete and checkpoints after each warmup
and measured compile. A failed stage, publication or deterministic-artifact
check leaves a failed report with prior evidence. Baseline admission checks the
same complete source inventory and measurement contract before reading retained
files, then validates hashes, product Wasm admission, artifact-derived footprints
and recomputed summaries. Missing or changed evidence cannot supply a comparison.
Output reuse is rejected. Distinct artifacts are written once and kept alongside
the exact workload and optional baseline JSON.

SDK controls compare the profiled bytes with the original compile/emit path and
reject malformed retained Wasm and parse failures. Report controls cover missing
samples/warmup, changed artifacts, stage distributions and incompatible CLI
options. These controls are authored and unrun. Idle-machine timing, runtime
throughput, peak memory/GC metrics, subsystem budgets and full T25 acceptance
remain separate work; no measured result or conformance claim is made here.
