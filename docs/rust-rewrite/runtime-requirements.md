# Wasm runtime requirements

Scriptw's tested execution path is its Rust CLI/library with the project's
vendored **Wasmtime 47.0.0**. The host policy is defined in
[wasmtime_policy.rs](../../crates/scriptw-engine/src/wasmtime_policy.rs).
The same proposal and collector policy applies to both native compilation modes.

## Required feature set

| Wasm capability | Required by the backend |
| --- | --- |
| GC structs and arrays | JavaScript objects and the semantic heap |
| Reference types and typed function references | Values, closures and typed calls across modules |
| Exception handling | JavaScript abrupt completion and exception flow |
| Tail calls | Enabled as part of the required backend target |
| Multi-memory | Program/runtime memory arrangements |
| Threads and shared memory | Required proposal support and host shared-memory allocation |

The native adapter explicitly selects Wasmtime's copying collector and verifies
GC, reference types, typed function references, exceptions, threads and shared
memory before admitting its engine. Multi-memory and tail calls are enabled
in the same policy. This does not claim that Scriptw supplies browser Workers
or Node's worker APIs. SIMD is not an additional requirement of this policy.

## Host compatibility

An emitted program is linked to Scriptw's runtime module and host imports,
including the `scriptw_host` namespace. Matching runtime bytes, Intl data and
host adapters are part of the execution environment.

- **Scriptw CLI / Rust API:** the supported integration with vendored Wasmtime
  47.0.0, including the copying collector and project host bindings.
- **Browsers, Node's Wasm API and other engines:** no compatible Scriptw adapter
  or end-to-end portability result is established. An engine implementing the
  same Wasm proposals still needs the host ABI, data and collector capabilities.
- **Bare WASI / a standalone Wasmtime CLI:** WASI does not provide Scriptw's
  JavaScript runtime or its custom host imports. The tested path is Scriptw's
  embedding, not loading the emitted file as an arbitrary WASI command.

These are requirements of the current backend, not a promise of support for
every engine or a fallback implementation for engines missing Wasm GC.

## Dynamic source: eval and Function constructors

Dynamic code generation is **limited**, rather than wholly absent:

- The compiler can prepare finite known `eval` and Function-constructor sources
  ahead of time, with dispatch that checks the source and execution context.
- The native host can parse, lower and compile additional supported runtime
  strings into Wasm. See the actual
  [direct-eval compiler](../../crates/scriptw-engine/src/wasm_native_direct_eval_compiler.rs)
  and [Function compiler](../../crates/scriptw-engine/src/wasm_native_function_compiler.rs).
- A portable Wasm artifact does not bundle a parser, interpreter or JavaScript
  VM. Runtime strings outside its prepared sources require a suitable native
  compiler host; unsupported syntax, lowering or contexts remain capability gaps.

This does not establish full ECMAScript `eval`, `new Function` or cross-realm
dynamic-source compatibility. See the [current inventory](current-inventory.md)
for verification and remaining acceptance work.

## Out-of-scope tests

Standalone Wasm dynamic-source cases that would require bundling a JavaScript
parser, interpreter or VM into the emitted artifact are **outside product
scope**, rather than features planned for implementation. This includes such
uses of `eval`, `new Function` and cross-realm Function constructors.

This is not a blanket exclusion of those APIs: finite prepared sources and
native-host compilation remain supported paths. A parser/lowering bug in a
supported path stays a failure or unimplemented feature. A test's filename,
use of `eval` or `Function`, or `Unsupported` failure kind alone cannot decide
that it is out of scope.

The README chart reserves blue for out-of-scope cases. The publication
does not include audited per-test scope decisions, so its blue category has an **unknown
count** and no numeric bar height. The three measured categories retain their
original counts and full denominator; some may include cases later classified
out of scope. No cases are subtracted or silently skipped. A numeric split
requires case-level evidence from the same suite pin and run, with a reason
for each excluded case, before it can change the chart.

## Refreshing the README chart

`python3 -B scripts/generate-test-graph.py` refreshes the README caption and
[vertical bar chart](assets/test262-status.svg) from the canonical published
Test262 snapshot. `--check` checks both outputs without writing them.
Green is Success, red is Bug plus Crash, gray is NotImplemented, and blue is
the explicitly unclassified out-of-scope category. Unrun tests are not
reclassified as unimplemented. The date and pin remain visible;
this chart is a dated measurement of its identified compiler and suite, not full ECMAScript conformance.

The previous 12,400-line README is preserved verbatim beneath a short provenance
header in [development history](../../DEVELOPMENT_HISTORY.md). Its body and source
revision are recorded in [the frozen history registry](scriptw-frozen-history.json).
