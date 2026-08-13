# T25 — Differential testing, fuzzing and performance discipline

**Status:** Bounded campaign in progress — versioned replay, one deterministic Add/Sub generator/reducer and an additive observed-execution seam exist; structured value comparison, broader generation/reduction, performance budgets and CI remain open

**Parallel group:** Validation lane  
**Depends on:** T01, T02, T03, T04  
**Blocks:** Confidence and performance gates in T26

## Current repository state

T28 removed the retired JavaScript fuzz, benchmark and execution surfaces. The
durable starting points are ignored Rust-owned Wasm-AOT performance probes,
snapshot determinism tests and an explicitly feature-gated spec-exec oracle.
One performance probe still expects a machine-local untracked manifest, so it
is not a portable benchmark corpus. The new `lila differential replay` command
consumes a schema-v1 Rust-owned corpus entry, always runs Wasm-AOT first, and
can run the off-by-default spec-exec oracle only when both the cargo feature
and explicit `--oracle spec-exec` flag are present. Its JSON report has a stable
case fingerprint and mismatch signature. Replay compiles both backends with the
ordinary product host-surface policy; schema-v1 probes do not silently gain
Test262-only globals.

The Rust-owned `integer-arithmetic-v1` campaign is the first non-decorative
consumer of that replay path. `lila differential generate-arithmetic` uses a
stable SplitMix64-v1 stream to build one or more self-checking Script probes
from a closed Add/Sub grammar. Leaves are integers in `-32..=32`; the public
plan types admit only 1–32 checks and depths 1–4, and expected results are
accepted only while they remain exact safe integers. The reducer admits a
typed non-zero budget of 1–512 replays and only constructs candidates whose
`(check count, node count, sum of absolute literals)` complexity decreases
strictly. It removes contiguous check ranges, replaces binary expressions by
children, and shrinks literals toward zero. A candidate is retained only when
it preserves both the mismatch direction and the failing backend phase; it
does not compare source-dependent mismatch fingerprints.

The generator command is protected by the same two independent oracle gates
as replay. It writes a schema-v1 case that the existing replay command consumes
only after both backends complete, or after a mismatch has been reduced; shared
failures and observation-contract failures are reported but are not persisted
as corpus entries. The committed seed-1/checks-4/depth-2 case pins the PRNG,
grammar rendering and schema encoding.

This first observation protocol is intentionally smaller than the objective
below. The engine exposes backend identity, a typed normal-or-throw completion,
and an execution-scoped `print` event channel for both backends. Schema v1
deliberately projects that richer result back to self-checking, no-output
probes: it compares only normal-versus-error disposition and verifies that both
transcripts are empty. Reports say semantic equivalence is `not_established`
and enumerate every value/identity observation capability they still omit. A
shared failure or observed no-output contract violation is red, not a match.
The committed foundation and generated cases plus feature-gated end-to-end
contract tests make this slice durable, but they do not satisfy the full
structured differential, full-corpus replay, layered generator, general AST
reducer, fuzz, performance, or CI requirements.

## Objective

Build automated methods that discover semantic divergences, crashes, hangs and pathological code generation before they become one-off Test262 investigations. Maintain performance without adding test-specific or observably incorrect fast paths.

## Bounded observed-execution contract

The first structured boundary is additive. `Engine::observe_script` and
`Engine::observe_module` return an owned observation whose completion is
either `Normal(value)` or `Throw(value)`. Compiler diagnostics, module-loader
failures, host failures, Wasmtime traps, timeouts and invalid backend ABI data
remain `EngineError`. Existing `run_*` entry points preserve their public API
and abrupt-completion shape: Wasm legacy execution turns a JavaScript throw
into an `EngineError`, while spec-exec keeps its separate legacy execution path.
Throw classification and spec-exec job/print side effects therefore do not
change as an incidental consequence of differential work.

One Wasm string edge case is an intentional correctness change rather than a
compatibility claim. The observed core decodes the runtime's bounded UTF-8/WTF-8
payload directly to UTF-16, so a normal or thrown String containing a lone
surrogate is observable. Legacy `run_*` execution now succeeds for a normal
lone-surrogate completion with a generic non-scalar String note where its old
strict Rust UTF-8 renderer returned `EngineError`; ordinary scalar diagnostics
retain their legacy rendering.

Spec-exec compatibility is behavioral, not merely a matching return type. Its
legacy path still returns immediately after a top-level Script throw,
does not drain that Script's queued jobs, and uses its historical stdout host
printer. The observed path has its own host checkpoint: it drains queued jobs
after capturing the primary completion, while never replacing a primary throw
with a later job failure. Module observation stages parse, load, link and
evaluate separately: parse/load/link failures are engine failures, whereas a
rejected evaluation promise is a JavaScript throw.

The shared value domain is deliberately smaller than a serializer:

- `undefined`, `null`, Boolean, Number, String and BigInt retain owned value
  data. Numbers canonicalize NaN while preserving signed zero, and Strings
  retain UTF-16 code units.
- Symbol is type-only in this batch. Description, registry membership and
  identity remain explicit observation gaps.
- Object is type-only. The observer must not call user coercion hooks, inspect
  properties, publish a backend heap handle or guess an object class.

Each observation also owns the ordered `print` lines emitted by its root
execution context. Those lines are the result of the program's actual host
`print` operation, including its ordinary argument coercions; collecting an
event must not add a second coercion. The event sink is scoped to one execution
and shared with the job checkpoint and module evaluation performed by that
execution. It is not a process-global transcript. Legacy execution selects a
closed delegate-only mode and streams to its existing host hook without
retaining a duplicate transcript.

Diagnostic text stays inside the same boundary. An opaque Symbol or Object
throw receives only a type label; Boa debug rendering and Wasm heap handles are
kept out of public observed outcomes (including their `Debug` output).
Structured Wasm execution never reads the throw-diagnostic globals or invokes
the legacy renderer. A separate legacy mode retains its bounded historical
human diagnostic, using a generic placeholder for valid non-scalar UTF-16.

Corpus and report schema v1 remain the self-checking, no-output protocol. The
differential runner consumes the common event channel and projects the typed
completion back to v1's normal-versus-error disposition. It does not serialize
or compare the new values yet, and matching projected dispositions still do
not establish semantic equivalence.

This seam does not yet carry a partial transcript inside `EngineError`,
identify Symbols or Objects, expose Symbol descriptions, classify Error
objects/realms/prototypes, or isolate a backend panic/host crash. Agent-produced
output is excluded from the common typed-transcript contract in this batch:
Wasm worker stores run delegate-only and the root typed outcome does not capture
their lines, while spec-exec's shared observed session can currently surface
agent lines. Their presence and ordering are therefore not backend-comparable
and consumers must not rely on them. Differential schema v1 shadows the existing
realm output hook so it can still report output emitted before an `EngineError`;
the typed outcome is authoritative for completed normal and throw executions.

## Differential execution framework

Create a runner that can execute the same generated or minimized program through:

- Lila Wasm-AOT (the product under test);
- Lila `spec-exec` (the internal Boa-based oracle — this differential role is the only permitted use of an interpreter in the project);
- one or more optional external standards-oriented engines configured
  explicitly for developer testing.

The retired JavaScript implementation is neither an oracle nor a runnable
backend. Git history may explain an old behavior, but differential tooling must
not restore or execute that product surface.

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
cargo test -p lila-ir fuzz_regressions --quiet
cargo test -p lila-aot-wasm fuzz_regressions --quiet
cargo test -p lila-test262 snapshot_determinism --quiet
cargo test -p lila-test262 --features spec-exec-oracle differential::generated_arithmetic::tests::committed_generated_arithmetic_case_replays_through_both_backends -- --exact
cargo run -p lila-cli --features spec-exec-oracle -- differential generate-arithmetic /tmp/lila-t25-arithmetic.json --seed 1 --checks 4 --depth 2 --max-replays 64 --oracle spec-exec
cargo run -p lila-cli --features spec-exec-oracle -- differential replay /tmp/lila-t25-arithmetic.json --oracle spec-exec
```

The exact CLI may differ, but equivalent seed/replay/reduce commands, CI jobs and artifact retention are required before closing this task.
