# F041: Keep method lexical environments distinct for eval declaration checks

- **Status:** open
- **Owner:** lila-ir function environment analysis; eval instantiation
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 8 executions across 8 physical files (Bug 8, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F041.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Literal direct eval successfully introduces var across an existing method-body lexical binding where SyntaxError is required. Sloppy method bodies need a separate lexical record between the eval lexical and variable environments. The missing conflict also explains the async resolved/double-DONE symptom.

## Source evidence

- [crates/lila-ir/src/analysis/eval_environment.rs:28](../crates/lila-ir/src/analysis/eval_environment.rs#L28): Plans which environment bindings are visible to eval.
- [test262/vendor/test262/test/language/expressions/object/scope-meth-body-lex-distinct.js:48](../test262/vendor/test262/test/language/expressions/object/scope-meth-body-lex-distinct.js#L48): A method lexical x must conflict with literal eval var x.

## Work

Preserve the separate body lexical environment for methods/getters/setters/generators and make EvalDeclarationInstantiation inspect every intervening declarative record.

## Validation

Run all attached method scope fixtures and verify no declaration is created before the SyntaxError.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F041.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F041-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/expressions/object/method-definition/async-gen-meth-eval-var-scope-syntax-err.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1967064: Expected a SyntaxError to be thrown but no exception was thrown at all)
```

- `sloppy-script:language/expressions/object/method-definition/async-meth-eval-var-scope-syntax-err.js` — Bug

```text
[origin:unknown] Test262:AsyncTestFailure:Test262Error: Test262Error: function should not be resolved; Test262:AsyncTestFailure:Error: host harness async $DONE called multiple times
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
