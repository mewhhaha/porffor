# Checked native Wasm exception-control emission

## Boundary

This completes the exception-control instruction accounting that the previous
[AOT control-flow review](aot-control-flow-review.md) left explicitly rejected.
Every compiler-emitted function still passes through `code_sink::Function`.
There is no new public emitter, interpreter, dynamic-source fallback or second
runtime object model.

JavaScript `try`/`catch`/`finally` still lowers through the existing completion
representation. Supporting a native Wasm instruction at the sink boundary is
not a claim that JavaScript exceptions have been migrated to `exnref`, nor that
suspended materialized environments or general builtin conformance are closed.

## Structural invariants

`try_table` checks every catch clause against the **enclosing** label stack
before pushing its own frame. This follows the pinned wasmparser validator's
ordering. Checking after the push would silently accept a one-past-the-end
handler target. The match covers `catch`, `catch_ref`, `catch_all` and
`catch_all_ref` exhaustively, including clauses that follow a catch-all. Once
inside the body, ordinary branches include the new try-table label normally.

Legacy `try` has distinct body, tagged-catch and catch-all states. Each handler
changes the arm without changing label identity or depth. Tagged handlers may
repeat, but catch-all is terminal. A catch cannot cross an unclosed nested
block or attach to a modern try-table.

`delegate` is legal only in a legacy try body and substitutes for its `end`.
Its target is checked against the remaining enclosing stack before the frame
is removed. Ordinary blocks and the implicit function label are legal targets.
`rethrow` instead indexes the current stack and must name a live legacy catch
or catch-all frame, not just an in-range numerical depth.

All malformed instructions are rejected before changing either encoded bytes
or the frame stack. Existing stale-label, foreign-label, clone, local-rewrite
and finished-body rules are retained. Checks execute in release builds too.
Tag signatures, operand types and reference assignability remain the Wasm
validator's responsibility; the sink does not pretend to validate them.

The legacy instruction checks do not introduce a legacy execution backend.
The required runtime execution tests use modern `try_table`/`exnref` with
Wasmtime exceptions explicitly enabled. There is no fallback or capability
skip when that feature is unavailable.

## Evidence and verification

The 18 additional sink tests cover valid transitions, invalid depths, malformed
handler ordering, transactional rejection, label lifecycle, local rewrites,
exact wasm-encoder byte equivalence and wasmparser validation. Legacy encoding
fixtures enable the legacy validator feature explicitly; modern fixtures do
not enable it.

Ten modern fixtures are shared between the sink tests and the existing engine
control-flow target. The sink constructs their complete modules and compares
every byte against the shared constants. The Wasmtime tests execute those
same constants, avoiding a second hand-written runtime-only implementation.
They check tagged payloads, both reference-rethrow forms, catch-all, unmatched
tag propagation, branching to a try-table, catching to the function label,
first-handler ordering, normal completion and a trap bypassing catch-all.
Execution is fuel-bounded; the trap test requires the precise unreachable trap,
not an arbitrary error or exhausted fuel.

These are native backend instruction fixtures, not JavaScript compiler outputs.
The pre-existing JavaScript Wasm-AOT control-flow and suspension execution
regressions remain part of the same CI gate.

```sh
cargo fmt --all -- --check
cargo test --locked -p lila-aot-wasm --lib code_sink:: -- --test-threads=1
cargo test --locked -p lila-engine --test aot_control_flow -- --test-threads=1
cargo test --locked -p lila-engine --test aot_async_for_of -- --test-threads=1
cargo test --locked -p lila-aot-wasm --test product_artifact -- --test-threads=1
```

The existing full backend sharding workflow discovers the new tests from the
compiled inventory. No ignore, expected-failure entry, selector shortcut or
conformance-count edit is introduced. Consult the PR checks for results on the
exact head; the current-pin Test262 aggregate and T26 release gate remain open
until independently verified.

## Specification references

- [Modern exception handling](https://github.com/WebAssembly/exception-handling/blob/main/proposals/exception-handling/Exceptions.md)
- [Legacy instruction structure and target semantics](https://github.com/WebAssembly/exception-handling/blob/main/proposals/exception-handling/legacy/Exceptions.md)
- [Pinned wasmparser validator](https://github.com/bytecodealliance/wasm-tools/blob/v1.245.1/crates/wasmparser/src/validator/operators.rs)
