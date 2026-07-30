# T25 — Differential testing, fuzzing and performance discipline

**Status:** Early in progress — benchmarks exist; differential/reduction/CI framework remains open

**Parallel group:** Validation lane  
**Depends on:** T01, T02, T03, T04  
**Blocks:** Confidence and performance gates in T26

## Current repository state

The repository has legacy fuzz inputs, benchmark programs, ignored Wasm-AOT
performance tests, snapshot determinism tests and an explicitly feature-gated
spec-exec oracle. It does not yet expose the task's grammar-aware differential
generator, structured mismatch corpus, AST reducer and replay CLI, nor the
described nightly fuzz/performance artifact pipeline. Treat performance
fixtures as a starting point, not completion of this validation lane.

## Objective

Build automated methods that discover semantic divergences, crashes, hangs and pathological code generation before they become one-off Test262 investigations. Maintain performance without adding test-specific or observably incorrect fast paths.

## Differential execution framework

Create a runner that can execute the same generated or minimized program through:

- Lila Wasm-AOT (the product under test);
- Lila `spec-exec` (the internal Boa-based oracle — this differential role is the only permitted use of an interpreter in the project);
- the legacy JavaScript implementation when it provides useful independent evidence;
- an optional external standards-oriented engine configured explicitly for developer testing.

The framework must compare structured observations rather than only process exit status:

- printed/output event sequences;
- normal result value and type, including NaN/signed zero/BigInt/String/Symbol-safe rendering;
- completion kind and arbitrary thrown value;
- error constructor/prototype realm where observable;
- property descriptors, own-key order and prototype identities for selected probes;
- side-effect logs for coercion/property/trap/evaluation order;
- timeout, panic, Wasm validation failure and host crash.

The reference backend is an oracle candidate, not infallible truth. Store disagreements with all observations so maintainers can determine which engine is wrong.

## Input generation

Implement layered generators:

1. **Grammar-aware programs:** derive valid syntax from supported AST constructors and feature flags.
2. **Negative syntax/early-error inputs:** mutate bindings, contexts and grammar constraints while tracking expected phase.
3. **Operation sequences:** create values/objects/proxies and invoke shared abstract operations with side-effecting hooks.
4. **Builtin scenarios:** generate receiver/argument combinations, descriptors, subclasses, species constructors and cross-realm objects.
5. **Stateful API programs:** arrays, typed buffers, iterators, promises, generators, modules, collections and Temporal sequences.
6. **Metamorphic transformations:** rename bindings, insert semantically neutral blocks, compare equivalent loops, clone realms and reorder unobservable declarations.

Seed generation from T01's failure metadata and newly green Test262 cases, but do not copy expected answers into compiler code.

## Reduction and replay

- Build an AST-aware reducer that preserves the observed mismatch/crash/timeout.
- Reduce statements, expressions, bindings, object properties, pattern features, numeric/string values and harness setup.
- Preserve parse goal, strictness, feature flags and async/module execution mode.
- Store minimized cases in a versioned regression corpus with the seed, engine versions, compiler commit and mismatch signature.
- Provide one command to replay the entire corpus and one command to reproduce a single case.

Every fixed crash or semantic mismatch should add the minimized case to a focused crate/CLI test when practical.

## Robustness fuzzing

Fuzz the following boundaries independently:

- JavaScript parser and early-error classifier;
- AST-to-IR lowering;
- IR validation and optimization passes;
- Wasm emitter and module validation;
- value serialization/debug reporting;
- Test262 frontmatter/prelude/snapshot parsers;
- RegExp, JSON, URI, numeric and Temporal parsers;
- module resolver/loader path handling.

No arbitrary input may panic the Rust process, invoke undefined behavior, allocate without limits, or emit invalid Wasm without returning a structured error.

## Performance measurement

Create reproducible benchmarks and tracked budgets for:

- parse, lower, emit and Wasm validation time;
- generated Wasm byte size, function count, helper duplication and static data size;
- runtime throughput/latency for representative language and builtin workloads;
- peak linear memory, allocation rate and GC pause/retained size;
- full Test262 node duration, timeout count and slowest cases;
- Intl/Unicode/time-zone data footprint;
- debug vs optimized compiler builds.

Record hardware/toolchain metadata and compare medians or robust percentiles. Do not treat a noisy single run as a regression.

## Optimization rules

- Correctness comes first; every optimization needs a semantic guard and slow/fallback path.
- Static shapes, constant folding, builtin direct calls and allocation elision must deopt/fallback when prototypes, accessors, proxies, realms, species or coercions make them observable.
- Performance fixes may not branch on Test262 paths/source text.
- A faster timeout is not a pass; timeouts must become completed correct executions.
- Add differential tests specifically for every optimization guard.

## CI tiers

Define three practical tiers:

- **PR-fast:** deterministic unit tests, regression corpus, small fuzz seed set, fake suite and focused changed-family tests.
- **Nightly:** longer differential/fuzz campaigns, sanitizer-like Wasm validation, stress GC/agents and performance smoke comparisons.
- **Full conformance/release:** complete pinned matrix, full corpus, repeated determinism and benchmark report.

CI artifacts must retain minimized failures, random seeds and comparison reports so failures are reproducible locally.

## Acceptance criteria

- Differential runs produce stable machine-readable mismatch signatures.
- The reducer turns seeded complex mismatches into meaningfully smaller reproductions.
- The regression corpus is deterministic and green on both debug and optimized builds where applicable.
- Parser/lowering/emitter fuzz targets run for a sustained campaign with zero unhandled panics or invalid-memory behavior.
- Performance dashboards identify compile/runtime/size regressions by subsystem and pin.
- Every enabled optimization has a test that forces both fast and fallback paths.
- Full-suite timeout counts trend as explicit performance debt and reach zero before T26 closes.

## Required tests

```sh
cargo test --workspace --quiet
cargo test -p porffor-ir fuzz_regressions --quiet
cargo test -p porffor-aot-wasm fuzz_regressions --quiet
cargo test -p porffor-test262 snapshot_determinism --quiet
./target/debug/porf differential replay --corpus tests/differential
```

The exact CLI may differ, but equivalent seed/replay/reduce commands, CI jobs and artifact retention are required before closing this task.
