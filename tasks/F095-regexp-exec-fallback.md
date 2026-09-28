# F095: Use RegExpBuiltinExec when an actual RegExp has noncallable exec

- **Status:** open
- **Owner:** lila-aot-wasm RegExpExec dispatch
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F095.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The fixture assigns null/number/boolean/undefined/string to an actual RegExp exec property; Symbol.match throws value is not callable. RegExpExec should call the property only when callable and otherwise fall back to the internal matcher for a branded RegExp.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/string.rs:13834](../crates/lila-aot-wasm/src/builtins/string.rs#L13834): Existing helper callable check should be shared by all consumers.
- [test262/vendor/test262/test/staging/sm/RegExp/RegExpExec-exec.js:14](../test262/vendor/test262/test/staging/sm/RegExp/RegExpExec-exec.js#L14): Fixture mutates exec on a real RegExp.

## Work

Audit every RegExpExec consumer to use the shared IsCallable plus internal-slot fallback, preserving TypeError for non-RegExp receivers.

## Validation

Run both RegExpExec-exec modes over every noncallable value and ordinary-object negative case.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F095.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F095-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/RegExp/RegExpExec-exec.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@1634016: value is not callable)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
