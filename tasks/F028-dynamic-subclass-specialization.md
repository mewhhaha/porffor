# F028: Compile finite Function constructor calls through subclass super

- **Status:** open
- **Owner:** lila-ir dynamic source planning; lila-aot-wasm function construction
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 16 executions across 8 physical files (Bug 0, NotImplemented 16, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F028.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

These fixtures use Function or GeneratorFunction subclasses with statically supplied constructor text. The saved execution reaches the runtime no-specialization guard through subclass construction. This is a missing finite-source/newTarget specialization, not evidence that a runtime JavaScript parser is required.

## Source evidence

- [crates/lila-ir/src/dynamic_source.rs:177](../crates/lila-ir/src/dynamic_source.rs#L177): The recorded guard means no compiled source/environment specialization was selected.
- [test262/vendor/test262/test/language/statements/class/subclass/builtin-objects/Function/instance-length.js:19](../test262/vendor/test262/test/language/statements/class/subclass/builtin-objects/Function/instance-length.js#L19): Finite constructor subclass source is in the fixture.

## Work

Trace the inherited constructor and super call through source/environment planning, precompile the finite bodies, and retain subclass newTarget/prototype semantics. Do not add a runtime JS evaluator.

## Validation

Rerun all attached subclass cases in both script modes and assert name, length, prototype, and constructed call behavior.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F028.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F028-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/statements/class/subclass/builtin-objects/Function/instance-length.js` — NotImplemented

```text
[origin:unknown] unsupported dynamic-source operation `Function` selected during Wasm execution without a compiled source/environment specialization
```

- `sloppy-script:language/statements/class/subclass/builtin-objects/GeneratorFunction/instance-length.js` — NotImplemented

```text
[origin:unknown] unsupported dynamic-source operation `GeneratorFunction` selected during Wasm execution without a compiled source/environment specialization
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
