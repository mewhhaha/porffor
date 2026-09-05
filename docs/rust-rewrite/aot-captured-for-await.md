# Captured for-await head bindings across suspension

This continues the suspended-Reference work with a real Wasm AOT implementation
of captured `let`/`const` head bindings in async-generator `for await` loops.
It does not add an interpreter, duplicate activation layout, or shadow copy of
the captured binding. The existing `ForInOfEnvironmentIr` still owns the one
per-iteration cell; uncaptured heads keep their existing activation storage.

## Lifecycle

A successful `next()` result enters one fresh environment and initializes the
head once. Before the body can yield, its current environment is published to
the activation's existing lexical-environment field. Closures created before
the yield and reads/writes after resumption therefore address the same record.

Function re-entry reloads that published child. Only body resume states detach
it temporarily while loop bookkeeping addresses the parent activation layout.
The activation continues rooting the child. The body then reattaches the exact
saved pointer without allocating or initializing a new cell. Entry, `next()`
resumption, exhausted iterators and close resumption do not detach a child.

Non-copyable saved/active carriers bind reattachment and cleanup to the same
activation and environment field. An inner cleanup frame records the **child**
environment depth: abrupt branches reach it without also unwinding the record.
Normal completion, break, continue, return and throw converge on one leave,
which publishes the parent before dispatching to the existing iterator-close
machinery. A real suspension returns directly and does not execute that leave.
This matters because iterator closing can itself await and then re-enter the
function; the completed iteration must not remain its current environment.

## Verification

`crates/lila-engine/tests/aot_captured_for_await.rs` executes compiled JavaScript
through Wasmtime, requires `ExecutionBackend::WasmAot`, and compares exact traces.
The cases cover fresh const cells over multiple yields, bidirectional closure
and body mutation of let cells, shadowing, yielded closures, queued requests,
interleaved activations, break/continue, throw/return resumption, asynchronous
iterator closing, close-rejection precedence, rejected yield values, and a throw
before the first body yield. Catch/finally handlers yield again to detect stale
activation pointers rather than checking only the first completion.

The existing IR activation tests continue to require one captured iteration
cell and **no** duplicate root slot. The activation-layout structure tests also
require the exhaustive environment-offset projection and consuming cleanup
authority. The AOT regression workflow requires those tests and the new execution
target alongside the existing control-flow, suspended-Reference,
product-artifact and complete-inventory backend tests. No tests are skipped or
marked expected-failure, and no generated conformance counts are edited.

```sh
cargo fmt --all -- --check
cargo test --locked -p lila-ir --test async_for_of_activation -- --test-threads=1
cargo test --locked -p lila-aot-wasm --test for_await_activation_layout_structure
cargo test --locked -p lila-engine --test aot_captured_for_await -- --test-threads=1
cargo test --locked -p lila-engine --test aot_async_for_of --test aot_suspended_references --test aot_control_flow -- --test-threads=1
cargo test --locked -p lila-aot-wasm --test product_artifact -- --test-threads=1
```

Check the PR verification record for the exact revision and commands executed.
This batch does not claim full generator or pinned Test262/T26 conformance.
Additional materialized lexical scopes in the body, nested for-await and the
other unsupported dispatcher shapes remain open. Dynamic-source execution
retains the explicit product policy in AGENTS.md.

## Specification

- [ForIn/OfBodyEvaluation](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-runtime-semantics-forinofbodyevaluation-lhs-stmt-iteratorrecord-iterationkind-lhskind): a fresh lexical environment per iteration, with restoration before close.
- [AsyncIteratorClose](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-asynciteratorclose): asynchronous close and abrupt-completion precedence.
