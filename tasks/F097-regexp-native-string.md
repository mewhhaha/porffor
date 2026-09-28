# F097: Use syntactically valid native function display names

- **Status:** open
- **Owner:** lila-ir builtin catalog and callable_to_string
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F097.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The RegExp legacy static accessor catalog uses get RegExp legacy static as its native name. NativeNamed inserts that string into function NAME() syntax, creating a string that fails the NativeFunction grammar checked by Function.prototype.toString.

## Source evidence

- [crates/lila-ir/src/builtins/catalog.rs:4132](../crates/lila-ir/src/builtins/catalog.rs#L4132): Human debug label is reused as the native function name.
- [crates/lila-ir/src/builtins/callable_to_string.rs:12](../crates/lila-ir/src/builtins/callable_to_string.rs#L12): Name is inserted directly after the function keyword.

## Work

Separate debug labels from grammar-valid native function display names and emit a valid native function representation for these accessors.

## Validation

Run both built-in-function-object executions and validate all builtin toString outputs against the existing harness.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F097.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F097-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:built-ins/Function/prototype/toString/built-in-function-object.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@53046632: Conforms to NativeFunction Syntax: "function get RegExp legacy static() { [native code] }" (%RegExp%.input))
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
