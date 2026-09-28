# F005: Handle oversized RegExp replacement allocations through JavaScript completion

- **Status:** open
- **Owner:** lila-aot-wasm RegExp replacement and string allocation
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 0, NotImplemented 0, Crash 2)

[Backlog](README.md) · [Exact execution list](cases/F005.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

replace-math requests a 2^36-character result and explicitly catches allocation failure, but helper::heap_alloc traps with Wasm unreachable instead of propagating a JavaScript exception. The failure mechanism is uncatchable allocation/growth failure during oversized string construction.

## Source evidence

- [test262/vendor/test262/test/staging/sm/String/replace-math.js:26](../test262/vendor/test262/test/staging/sm/String/replace-math.js#L26): Fixture asks for 64 Gi characters and permits a caught OOM.
- [crates/lila-aot-wasm/src/heap.rs:222](../crates/lila-aot-wasm/src/heap.rs#L222): Growth failure leads to unreachable and cannot be caught by JavaScript.

## Work

Check string/result size and allocation overflow before materialization and report an appropriate catchable JavaScript failure; avoid building enormous intermediate replacements.

## Validation

Run both original replace-math cases and verify either a valid result within supported limits or a caught exception, never a Wasm trap.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F005.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F005-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/String/replace-math.js` — Crash

```text
[origin:unknown] wasmtime execution trapped: error while executing at wasm backtrace:
    0:  0x1409c - lila!helper::heap_alloc
    1: 0x2a3f48 - lila!builtin::RegExp.prototype[Symbol.replace]
    2: 0x2f9986 - lila!builtin::String.prototype.replace
    3: 0x423333 - lila!lila::main

Caused by:
    wasm trap: wasm `unreachable` instruction executed

```

- `strict-script:staging/sm/String/replace-math.js` — Crash

```text
[origin:unknown] wasmtime execution trapped: error while executing at wasm backtrace:
    0:  0x1409c - lila!helper::heap_alloc
    1: 0x2a138e - lila!builtin::RegExp.prototype[Symbol.replace]
    2: 0x2f6dcc - lila!builtin::String.prototype.replace
    3: 0x42085d - lila!lila::main

Caused by:
    wasm trap: wasm `unreachable` instruction executed

```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
