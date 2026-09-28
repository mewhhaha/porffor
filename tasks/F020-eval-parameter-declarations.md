# F020: Reject direct-eval var collisions in method parameter environments

- **Status:** open
- **Owner:** lila-ir analysis/eval_environment.rs and direct eval instantiation
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 48 executions across 48 physical files (Bug 48, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F020.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

All attached methods perform direct eval while initializing non-simple parameters and declare arguments into a conflicting parameter environment. The required SyntaxError is missed; async variants consequently call $DONE more than once. The likely cause is an incorrect parameter/variable-environment boundary or arguments declaration classification during eval instantiation, not the async harness itself.

## Source evidence

- [crates/lila-ir/src/analysis/eval_environment.rs:48](../crates/lila-ir/src/analysis/eval_environment.rs#L48): Eval-visible bindings classify parameters and lexical arguments.
- [test262/vendor/test262/test/language/eval-code/direct/meth-no-pre-existing-arguments-bindings-are-present-declare-arguments.js:13](../test262/vendor/test262/test/language/eval-code/direct/meth-no-pre-existing-arguments-bindings-are-present-declare-arguments.js#L13): Literal eval declares arguments during default parameter evaluation.

## Work

Trace the eval environment chain during parameter initialization and reject the declaration before executing eval. Encode arguments/parameter environment roles in the common instantiation path.

## Validation

Run all 48 method, generator-method, async-method and async-generator-method variants; verify SyntaxError and exactly one async completion.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F020.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F020-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/eval-code/direct/async-gen-meth-a-following-parameter-is-named-arguments-declare-arguments-and-assign.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1965328: Expected a SyntaxError to be thrown but no exception was thrown at all)
```

- `sloppy-script:language/eval-code/direct/async-meth-a-following-parameter-is-named-arguments-declare-arguments-and-assign.js` — Bug

```text
[origin:unknown] Test262:AsyncTestFailure:Error: host harness async $DONE called multiple times
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
