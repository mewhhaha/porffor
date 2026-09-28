# F046: Honor RegExp prototype getter special cases and generic toString

- **Status:** open
- **Owner:** lila-aot-wasm RegExp getters, toString and realm intrinsics
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 6 executions across 3 physical files (Bug 6, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F046.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Prototype/getter/toString fixtures throw the internal-slot receiver TypeError on valid prototype or generic receiver paths. Current source has a defining-realm prototype special case in getters; the saved failures require tracing actual receiver realm and generic toString dispatch before changing its slot checks.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/string.rs:2053](../crates/lila-aot-wasm/src/builtins/string.rs#L2053): Getter distinguishes defining-realm prototype before requiring slots.
- [test262/vendor/test262/test/staging/sm/RegExp/prototype.js:25](../test262/vendor/test262/test/staging/sm/RegExp/prototype.js#L25): Fixture separates permitted getters from rejecting methods.

## Work

Identify the exact accessor/toString call in each fixture, preserve the intrinsic prototype identity per realm and keep generic property-based toString separate from internal-slot methods.

## Validation

Run prototype, prototype-different-global and toString in both modes, retaining required TypeErrors for exec/test/compile on the prototype.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F046.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F046-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/RegExp/prototype-different-global.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@2518648: RegExp.prototype.exec receiver is not RegExp)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
