# F001: Stop scratch/side-storage exhaustion in repeated allocation loops

- **Status:** open
- **Owner:** lila-aot-wasm heap allocation and GC side storage
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 18 executions across 9 physical files (Bug 0, NotImplemented 0, Crash 18)

[Backlog](README.md) · [Exact execution list](cases/F001.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The recorded Date DST loops and nullish-coalescing hot loop trap in helper::heap_alloc. This allocator is a monotonic linear-memory bump allocator with overflow/memory.grow failure traps; it does not reclaim inside the helper. Repeated transient allocations likely exhaust side storage, but the snapshots do not distinguish growth failure from overflow and require a reduced reproducer.

## Source evidence

- [crates/lila-aot-wasm/src/heap.rs:165](../crates/lila-aot-wasm/src/heap.rs#L165): Monotonic HEAP_PTR allocation and three explicit unreachable guards.
- [test262/vendor/test262/test/staging/sm/expressions/nullish-coalescing.js:48](../test262/vendor/test262/test/staging/sm/expressions/nullish-coalescing.js#L48): Fixture repeats allocations 100,000 times.

## Work

Measure transient allocation/root retention in these loops, route collectible language values through the GC representation, and bound/reuse side storage with explicit lifetimes rather than introducing a second object model.

## Validation

Rerun the attached 18 executions under the existing resource caps; confirm steady-state memory and inspect which heap_alloc guard fires.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F001.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F001-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/Date/dst-offset-caching-1-of-8.js` — Crash

```text
[origin:unknown] wasmtime execution trapped: error while executing at wasm backtrace:
    0:  0x2104c - lila!helper::heap_alloc
    1:  0x215ac - lila!helper::plain_object_alloc
    2: 0x252d10 - lila!builtin::Date
    3: 0x83db57 - lila!helper::proxy_construct
    4:  0x7b2d4 - lila!js::tzOffsetFromUnixTimestamp#f17
    5:  0x7ee3e - lila!js::clearDSTOffsetCache#f18
    6:  0x8d566 - lila!js::runDSTOffsetCachingTestsFraction#f8
    7: 0x8c3594 - lila!lila::main

Caused by:
    wasm trap: wasm `unreachable` instruction executed

```

- `sloppy-script:staging/sm/Date/dst-offset-caching-2-of-8.js` — Crash

```text
[origin:unknown] wasmtime execution trapped: error while executing at wasm backtrace:
    0:  0x2104c - lila!helper::heap_alloc
    1:  0x215ac - lila!helper::plain_object_alloc
    2: 0x252d10 - lila!builtin::Date
    3: 0x83db57 - lila!helper::proxy_construct
    4:  0x7b2d4 - lila!js::tzOffsetFromUnixTimestamp#f17
    5:  0x7ee3e - lila!js::clearDSTOffsetCache#f18
    6:  0x8d566 - lila!js::runDSTOffsetCachingTestsFraction#f8
    7: 0x8c3595 - lila!lila::main

Caused by:
    wasm trap: wasm `unreachable` instruction executed

```

- `sloppy-script:staging/sm/expressions/nullish-coalescing.js` — Crash

```text
[origin:unknown] wasmtime execution trapped: error while executing at wasm backtrace:
    0:  0x26c04 - lila!helper::heap_alloc
    1:  0xd1412 - lila!js::shouldBe#f9
    2:  0xe4839 - lila!js::testBasicCases#f12
    3: 0x98691d - lila!lila::main

Caused by:
    wasm trap: wasm `unreachable` instruction executed

```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
