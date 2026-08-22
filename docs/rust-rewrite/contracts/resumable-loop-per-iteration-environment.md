# Resumable loop per-iteration environment

## Scope

This contract closes one bounded Wasm-AOT lowering gap: a plain async function
containing an array-specialized `for`-`of` loop whose body has one direct
`await` and whose lexical loop binding is captured by a closure.

The lowering already turns the supported storage-only shape into a resumable
`StatementIr::GeneratorLoop`. This extension gives that loop a fresh declarative
Environment Record for every iteration and preserves the active record across
the return to the async job queue. The emitted Wasm remains the product path;
no interpreter, runtime source evaluation, or iterator-protocol shortcut is
introduced.

This contract does not lift the existing explicit rejections for non-array
iterables, destructuring or property targets, captured head-TDZ bindings,
multiple or nested suspension points, `break`/`continue`, `for`-`in`, or
`for`-`await`.

The normative requirements are
[`ForIn/OfHeadEvaluation`](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-runtime-semantics-forinofheadevaluation),
[`ForIn/OfBodyEvaluation`](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-runtime-semantics-forin-div-ofbodyevaluation-lhs-stmt-iteratorrecord-iterationkind-labelset),
and
[`AsyncBlockStart`](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-asyncblockstart).

## Current-pin witnesses and current-head proof

The pinned Test262 revision is
`aa55200d1310384c5cf69ea95b2a2ecba457007b`. The checked-in snapshot-v6
`built-ins/Array/fromAsync` leaf records 95 physical files, 93 passes, and these
two `NotImplemented` failures:

- `built-ins/Array/fromAsync/asyncitems-asynciterator-not-callable.js`;
- `built-ins/Array/fromAsync/asyncitems-iterator-not-callable.js`.

That 2026-08-13 snapshot is pin-current but not a result from the implementation
head. It is selection evidence, not a current pass-count claim. At selection
head `0ef005f0c`, source provided the stronger proof:
`AsyncForOfArrayWalkForm::classify` selected
`CapturedPerIterationBinding` when capture analysis supplied an iteration
environment, and `lower_async_for_of_array_with_body_await` explicitly refused
that form. `StatementIr::GeneratorLoop` carried no environment plan, while the
shared resumable-loop emitter ran the resume, update, test, and next-iteration
phases without entering one.

Both Test262 witnesses capture `v` in an arrow called during the same iteration.
They therefore prove the unsupported path but are not sufficient to prove cell
identity. A wrong implementation that hoists `v` into one activation-record
slot could pass both. The durable CLI oracle
`crates/lila-cli/tests/fixtures/wasm_async_for_of_closure_capture.js` retains six
closures and invokes them only after the loop; the required values are
`1,2,3,4,5,6`, while one shared cell produces `6,6,6,6,6,6`.

## Closed IR domain

Every `StatementIr::GeneratorLoop` carries exactly one
`ResumableLoopIterationEnvironmentIr`:

- `StorageOnly` means the loop owns no runtime per-iteration Environment
  Record; or
- `FreshPerIteration(LexicalEnvironmentIr)` contains the complete environment
  layout that must be allocated once for each entered iteration.

This is a required enum field, not `Option<LexicalEnvironmentIr>` and not a
boolean. Every loop constructor must state the lifecycle it requires, and every
backend consumer must match the domain exhaustively. Adding another lifecycle
becomes a Rust compile error at each consumer rather than a silently omitted
emission step.

The async array-walk classifier converts the captured-binding case into
`FreshPerIteration` only when capture analysis supplied the corresponding
`iteration_environment`. The other rejected forms remain distinct errors.
There is no path that labels a captured binding `StorageOnly`, and no backend
path is asked to infer environment ownership from statements or binding names.

## Runtime lifecycle

For `FreshPerIteration(environment)`, the resumable-loop emitter must implement
this lifecycle:

1. On the entry-state path, evaluate the loop init and test in the enclosing
   environment.
2. After a successful test and before initializing the loop binding, allocate
   one fresh record described by `environment`, chain it to the current
   environment, and make it current.
3. Initialize the iteration binding in that record, then execute the body up to
   its direct suspension.
4. Before returning to the job queue, store the active environment pointer in
   the execution-kind-specific activation environment slot.
5. On the resume-state path, reattach that exact record before compiling the
   post-suspension body.
6. After the iteration finishes, restore the parent environment and publish
   that parent pointer back to the activation before evaluating the update,
   test, or entering the next iteration.
7. A successful next test repeats step 2 and therefore allocates a different
   record. Closures from earlier iterations continue to point at their earlier
   records.

The existing activation environment slots are the intended persistence seam:
plain async functions use `HEAP_ASYNC_ENV_OFFSET`, and async generators use
`HEAP_ASYNC_GENERATOR_LEXICAL_ENV_OFFSET`. Function entry already reloads those
slots. This contract requires the loop lifecycle to update the selected slot as
the current environment changes; it does not require a second environment
pointer field unless implementation proves the existing slot insufficient.

Abrupt completion during binding initialization or the body must not leave a
different current-environment pointer observable to enclosing cleanup. The
environment record may outlive the iteration through closures, but loop
execution relinquishes it exactly once before continuing or exiting normally.

## Ownership boundary

The coherent implementation owns:

- this contract;
- `crates/lila-ir/src/ir.rs` for the closed lifecycle enum and required
  `GeneratorLoop` field;
- `crates/lila-ir/src/lowering_helpers.rs` for classification into a supported
  environment plan or a precise rejection;
- `crates/lila-ir/src/lowering.rs` for carrying the analyzed iteration
  environment into the resumable loop;
- `crates/lila-aot-wasm/src/control_flow.rs` and
  `crates/lila-aot-wasm/src/environments.rs` for allocation, static binding
  scope, persistence, restoration, and release; and
- focused IR/backend structure checks plus the existing CLI oracle.

`crates/lila-aot-wasm/src/emit.rs`, `heap.rs`, and `functions.rs` are verified
dependencies. They change only if the existing activation-environment access
cannot express the lifecycle above. `Array.fromAsync` itself is a witness, not
the semantic owner, so its builtin algorithm and catalog are outside this lane.

## Verification

The integrated current-SHA checkpoint is green:

- `cargo xc`;
- three source-bounded backend structure tests;
- two focused `lila-ir` invariants;
- the existing resumable-loop Wasm module test;
- the six-closure CLI consumer oracle; and
- both exact pinned Test262 witnesses, `4/4` under Wasm-AOT.

Refresh or extend that evidence with:

```sh
cargo fmt --all -- --check
cargo check -p lila-ir --lib
cargo check -p lila-aot-wasm --lib
cargo test -p lila-ir plain_async_for_of_array_body_await --quiet
cargo test -p lila-aot-wasm resumable_loop_iteration_environment --quiet
cargo test -p lila-cli run_wasm_backend_preserves_async_for_of_iteration_environments --quiet
./target/debug/lila test262 run built-ins/Array/fromAsync/asyncitems-asynciterator-not-callable.js --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila test262 run built-ins/Array/fromAsync/asyncitems-iterator-not-callable.js --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila test262 run built-ins/Array/fromAsync --execution-backend wasm --timeout-ms 180000 --threads 1
```

The complete 95-file `Array.fromAsync` leaf was not rerun. The six-closure
oracle is the semantic gate: the two pinned files going green without that
oracle going green is a false fix.
