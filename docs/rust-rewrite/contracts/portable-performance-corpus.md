# Portable authoritative Wasmtime-AOT performance corpus

Status: complete source passed the ref105 combined all-target Rust type and
hygiene checks. Explicit idle-machine timing remains pending. No performance
budget or T25 acceptance is claimed.

## One compiled workload

The existing twenty-case workload previously depended on untracked
benchmarks/wasm-aot-20.txt. The global *.txt ignore meant that a clean checkout
could compile the ignored probes but could not run this one. The successor
retains all twenty actual fixture names in the same order through the private
perf/chunk_cases.rs owner. No fixture or workload is invented or simplified.

A fixed [&str; 20] array makes cardinality a Rust type requirement. Each name is
one literal consumed by a private fixture macro, whose include_str! requires
that same actual fixture. Removing or renaming a fixture fails all-target Rust
compilation, including ignored-test compilation. There is no separate mutable
runtime manifest, parser or unvalidated filename string. The corpus is the
actual warmup and measured-loop input, rather than an unused validation list.
The JavaScript source remains existing Rust test fixtures and is executed only
through the ordinary Rust CLI Wasmtime-AOT path when the probe is requested.

## Measurement ownership

The three existing probe names and declared ignored timing policy remain. The
warm exact limit is one second; the warmed twenty-case and cold cache-pruned
limits are five seconds. The twenty-case warmup occurs before the same Instant
measurement span, with the same order in both passes. No source execution or
cache pruning occurs as part of this source batch.

The known-failures ledger describes the compiled corpus and preserves the same
three declared opt-in probes. README and workflow retain historical measured
numbers and explain the portable successor. Timing still needs an idle machine;
compilation or source review is not a timing verdict.

## Verification and remaining work

The complete source, ledger and documentation preceded one grouped all-target
Rust check. That check, formatting, module boundaries and task-plan validation
passed at ref105. Declaration-ledger execution and actual timing remain later
gates. This closes the machine-local manifest dependency; it does not
establish current budgets, a broader benchmark corpus, cache-prune isolation,
CI, differential identity comparison, general generation/reduction or full T25.

## Repeated retained reports

The same single literal list now supplies the exact fixture bytes to the private
CLI performance reporter while retaining the gate's original names and order.
The [reporting contract](performance-reporting.md) describes explicit repeated
whole-invocation measurements, retained artifacts and compatible-baseline
admission. Reporter source and controls are authored; compilation and timing
remain pending. The original three timing tests, limits and ledger are unchanged.
