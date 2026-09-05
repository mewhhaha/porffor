# AOT control-flow emission review

## Scope

This change strengthens the existing Rust JavaScript-to-Wasm compiler. It does
not introduce a Wasm-to-JavaScript backend, restore the retired JavaScript
implementation, or claim completion of the ECMAScript implementation.

The reviewed boundary is `lila-aot-wasm::code_sink`: every emitted function body
passes through it, including raw control instructions emitted outside the
JavaScript control-flow builder. The companion engine tests execute JavaScript
through `ExecutionBackend::WasmAot`, not the debug interpreter.

## Findings and implementation

1. A numerical depth is not a live label identity. Closing a block and opening
   a sibling at the same depth made the old `checked_sub` accept a stale target.
   Targets now carry a unique identity checked against the frame at the recorded
   depth. Foreign-function targets and independently opened clone frames are
   rejected. Cloned bodies intentionally retain their shared live prefix.
2. A final `end` terminates the body. Further instructions, including a new
   block, are rejected before bytes are appended. Capturing a label from a
   finished body is also rejected.
3. `else` is legal only for the current unmatched `if`. It preserves that
   frame's label identity; duplicate `else` and crossing an unclosed nested
   frame are rejected.
4. Relative reference-branch immediates receive the same bounds checks as
   `br`, `br_if` and every `br_table` target. This does not replace reference
   operand/type validation.
5. Rewriting a local declaration preserves live frame identities and the
   finished/open lifecycle instead of reconstructing a fresh control stack.

All guards execute in release builds. Label identities affect compiler-side
validation only; they are not serialized into Wasm. A byte-equivalence test
compares a valid nested body with `wasm_encoder` and validates the resulting
module using `wasmparser`.

## Verification

The independent `AOT control-flow regressions` workflow runs on pull requests
and pushes to main, with read-only repository permissions. Formatting failures
do not suppress subsequent diagnostic test steps and still fail the job.
Emission test discovery checks for a named regression before running the test
filter so a stale filter cannot silently report success with zero tests.

Reproduce the checks with:

```sh
rustfmt --edition 2021 --check crates/lila-aot-wasm/src/code_sink.rs \
  crates/lila-engine/tests/aot_control_flow.rs
cargo test --locked -p lila-aot-wasm --lib code_sink:: -- --test-threads=1
cargo test --locked -p lila-aot-wasm --test product_artifact -- --test-threads=1
cargo test --locked -p lila-engine --test aot_control_flow -- --test-threads=1
```

The engine regressions cover getter throws nested in loops/switches, labeled
break/continue through finally, finally replacing return/throw, short-circuiting
argument evaluation on throw, and successive sibling control regions.

At the review base `c4e15caf53b495a41a9700eadba4ad7429e7a327`, the existing
main CI run `33564514499` failed its known-failure-ledger audit because
`cli_output_ending_structure` was not registered as a `TestTarget`. The Windows
release job also failed checkout. Those pre-existing failures are not bypassed
or converted into expected passes by this change. Consult the PR's checks for
results on the changed code; baseline results are not evidence for the patch.

## Remaining boundaries

This is a bounded compiler-correctness change, not a complete compiler review.
The sink does not validate the Wasm operand stack, JS evaluation semantics,
heap tracing, module linking, builtin conformance, or dynamic-code support.
Those remain the responsibility of their existing typed compiler stages,
Wasm validation, execution regressions and the pinned Test262 gate.

Exception-control forms (`try`, `try_table`, `catch`, `catch_all`, `delegate`,
`rethrow`) remain explicitly rejected by the sink until their label semantics
are implemented. This must not be confused with JS try/catch/finally, which the
current backend lowers through its existing completion representation.

No conformance counts or generated README status artifacts are changed. A
passing focused workflow does not establish full Test262 or ECMAScript
conformance and must not be used to close the project's aggregate gate.
