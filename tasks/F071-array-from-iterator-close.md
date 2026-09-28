# F071: Close Array.from iterators on mapping or property-creation errors

- **Status:** open
- **Owner:** lila-aot-wasm Array.from iterator protocol
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F071.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The fixture expected iterable.closed=true but it remained false after an abrupt Array.from path. IteratorClose is missing or bypassed on one mapping/CreateDataPropertyOrThrow branch; the exact failing branch is not named by the saved assertion.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/standard.rs:5518](../crates/lila-aot-wasm/src/builtins/standard.rs#L5518): Shared Array.from/TypedArray.from emission owner.
- [test262/vendor/test262/test/staging/sm/Array/from-iterator-close.js:67](../test262/vendor/test262/test/staging/sm/Array/from-iterator-close.js#L67): First abrupt case throws in the mapper and requires close.

## Work

Reduce the first abrupt path, use one owned iterator record across construction/mapping/property creation, and close it with correct original-error precedence.

## Validation

Run both from-iterator-close modes, including mapping throws, proxy defineProperty throws and throwing/noncallable return methods.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F071.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F071-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/Array/from-iterator-close.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1654928: Expected SameValue(«false», «true») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
