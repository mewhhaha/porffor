# AOT control-flow emission and suspension review

## Scope

This change strengthens the existing Rust JavaScript-to-Wasm compiler. It does
not introduce a Wasm-to-JavaScript backend, restore the retired JavaScript
implementation, or claim completion of the ECMAScript implementation.

The principal boundary is `lila-aot-wasm::code_sink`: every emitted function
body passes through it, including raw control instructions emitted outside the
JavaScript control-flow builder. The companion engine tests execute JavaScript
through `ExecutionBackend::WasmAot`, not the debug interpreter.

## Emission invariants

A numerical depth is not a live label identity. Closing a block and opening a
sibling at the same depth made the old depth subtraction accept a stale target.
Targets now carry a unique identity checked against the frame at the recorded
depth. Foreign-function targets and independently opened clone frames are
rejected; cloned bodies intentionally retain their shared live prefix.

A final `end` terminates the body. Further instructions, including a new block,
are rejected before bytes are appended. Capturing a label from a finished body
is also rejected. An `else` must belong to the current unmatched `if`; it
preserves that frame's identity. Duplicate `else` and crossing an unclosed
nested frame are rejected.

Reference-branch immediates receive the same bounds checks as `br`, `br_if` and
every `br_table` target. Rewriting a local declaration preserves live frame
identities and the finished/open lifecycle instead of creating a fresh stack.

All guards execute in release builds. Label identities are compiler-side
validation metadata and are not serialized into Wasm. A byte-equivalence test
compares one valid nested body with `wasm_encoder` and validates the resulting
module using `wasmparser`. This is not a corpus-wide byte-equivalence claim.

## Failures exposed by the full backend run

The complete backend run exposed an actual suspension-storage defect: an
uncaptured lexical for-await head could lose its value across a yield because
its per-iteration environment was elided and its alias was not allocated in
the root activation. For-await lowering now explicitly retains that binding
when there is no materialized iteration environment. The existing allocation
helper deduplicates already-owned bindings. Captured heads keep their single
per-iteration lexical cell instead of acquiring a competing activation slot.

The IR regressions check unique slots for `let`, `const` and `var` heads,
separation from a shadowed outer binding, and non-duplication of captured
iteration cells. The Wasmtime regressions use multiple yields and observable
reads after resumption, checking the complete output sequence for two loop
iterations and final iterator completion.

Independent baseline defects also blocked verification:

- The runtime-error literal table had two adjacent messages out of order. The
  entries are now sorted; the strict ordering/uniqueness test remains intact.
- Heap ownership audits stopped at a test-only import rather than the test
  module. They now inspect the entire implementation. The async-resume audit
  accounts for the canonical layout entry and verifies its offset, width and
  non-pointer role. The Promise router audit ends at its own method boundary
  rather than a removed neighboring method. The ownership and dispatch
  assertions remain enforced.
- `cli_output_ending_structure` and `test262_verdict_command_structure` were
  absent from the CLI test-target registry. Both are now closed `TestTarget`
  variants with parsing and stem mappings. No expected-failure ledger entry or
  ignored test was added.
- An unhandled-rejection diagnostic had been inserted before the fixed string
  seeds, moving the comma payload and breaking both literal-layout and RegExp
  append-only data checks. Interning the diagnostic after the fixed seeds
  restores their offsets without deleting the diagnostic or changing those
  tests' expected values. A new production-pool regression also verifies the
  diagnostic payload resolves to its exact bytes.

## Verification commands and CI design

The read-only `AOT control-flow regressions` workflow runs on pull requests and
pushes to main. Its focused job checks formatting, the sharding runner, IR
activation planning, emission invariants, the product artifact and Wasmtime
execution. The emission filter first discovers a named regression, preventing
a renamed filter from silently selecting zero tests.

```sh
cargo fmt --all -- --check
python3 -m unittest discover -s scripts/tests -p test_run_aot_unit_shard.py -v
cargo test --locked -p lila-ir --test async_for_of_activation
cargo test --locked -p lila-aot-wasm --lib code_sink:: -- --test-threads=1
cargo test --locked -p lila-aot-wasm --test product_artifact -- --test-threads=1
cargo test --locked -p lila-engine --test aot_control_flow --test aot_async_for_of -- --test-threads=1
cargo test --locked -p lila-cli --test cli -- known_failures:: --test-threads=2
cargo test --locked -p lila-cli --test cli_output_ending_structure --test test262_verdict_command_structure
```

The full backend library is partitioned into eight deterministic, disjoint
shards from the compiled test inventory, not a hand-maintained allowlist. Each
test runs in a fresh process to bound retained compiler memory. A failing,
ignored, missing or timed-out execution fails its shard. Test discovery checks
its declared total against the unique names, and successful execution must
report exactly one passing test. Runner tests verify complete partition
coverage and rejection of vacuous or incomplete results.

```sh
# The whole inventory in one local shard:
python3 scripts/run_aot_unit_shard.py 0 1
# CI uses every index from 0 through 7 with this shard count:
python3 scripts/run_aot_unit_shard.py 0 8
```

The original single-process backend step exceeded its 15-minute deadline
before completing the inventory. Sharding changes the scheduling, not the
required test set. The existing main CI remains responsible for repository
contracts, workspace compilation, the CLI ledger and both fake Test262 suites.
Consult the PR checks and verification comment for results on the exact head;
results from an earlier revision do not establish success for a later one.

## Remaining boundaries

This is a bounded compiler-correctness review. The sink does not validate the
Wasm operand stack, reference operand types, JS evaluation semantics, heap
tracing, module linking or general builtin conformance. Existing typed stages,
Wasm validation, execution regressions and the pinned Test262 gate retain those
responsibilities.

This review originally left exception-control forms explicitly rejected.
Their native instruction accounting is now implemented and covered by the
[exception-control follow-up](aot-native-exception-control.md). This does not
migrate JS try/catch/finally away from the existing completion representation.
Suspended materialized loop/body environments remain a separate backend
boundary; the captured-head IR test is an ownership check, not a claim of
complete runtime support for that case.

The full CLI/engine suites, the before/after CLI-fixture golden comparison,
performance comparison and current-pin real Test262 aggregate are outside the
checks added here. Sorting the error-message pool can change encoded data
positions, so the single-body byte-equivalence test does not imply unchanged
whole-program bytes. No generated conformance counts are edited. Passing
these focused and backend checks does not establish full ECMAScript or Test262
conformance.
