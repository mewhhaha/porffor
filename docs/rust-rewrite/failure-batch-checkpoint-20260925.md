# Failure repair checkpoint — 2026-09-25

Implementation revision: `3c715f46f`. This checkpoint concerns the **103 previously failing executions across 54 files**, not a full Test262 publication. The [plan](failure-batch-plan-20260925.md) records the work allocation and focused review corrections.

## Original-case rerun

The fresh release compiler passed **57/103**; **46 remain failing**. All 38 Temporal and both Atomics failures passed. TypedArray source/realm fixes and the sorting algorithm account for the other 17 passes. No original failing execution was omitted. The complete ID sets, owner tasks and reasons are in the [machine-readable checkpoint](failure-batch-checkpoint-20260925.json).

The DataView constructor repair exposes a later dynamic-eval limitation in `staging/sm/extensions/dataview.js`; that test is still failing. The large counting-sort staging test progressed beyond its old timeout but now exhausts the heap; that is not a pass.

| Remaining work | Executions | Owner |
|---|---:|---|
| dynamic | 12 | [T13](../../tasks/13-dynamic-source-evaluation.md) |
| generator | 12 | [T15](../../tasks/15-generators-iterators-resource-management.md) |
| heap | 4 | [T05](../../tasks/05-values-heap-gc.md) |
| gc | 12 | [T05](../../tasks/05-values-heap-gc.md) |
| extension | 3 | [T09](../../tasks/09-functions-classes-private-elements.md) |
| intl | 2 | [T23](../../tasks/23-intl402.md) |
| stack | 1 | [T09 / T05](../../tasks/09-functions-classes-private-elements.md) |

## Every remaining file

Paths below are relative to `staging/sm/`. Modes and complete diagnostics are retained in the machine-local snapshots; execution IDs are also in the committed JSON.

| File | Failed executions | Owner / reason |
|---|---:|---|
| `TypedArray/Tconstructor-fromTypedArray-byteLength.js` | 2 | T13: Dynamic eval/Function has no compiled source/environment specialization; explicitly unsupported by the Wasm-AOT contract. |
| `TypedArray/constructor-ArrayBuffer-species-wrap.js` | 2 | T13: Dynamic eval/Function has no compiled source/environment specialization; explicitly unsupported by the Wasm-AOT contract. |
| `TypedArray/constructor-buffer-sequence.js` | 2 | T15: Nested yield regions and resumable for-of require the in-progress generator implementation. |
| `TypedArray/element-setting-converts-using-ToNumber.js` | 2 | T05: Execution reaches heap_alloc via array_alloc and exhausts the current linear bump heap; raising its limit is not a collector. |
| `TypedArray/set-detached.js` | 2 | T15: Nested yield regions and resumable for-of require the in-progress generator implementation. |
| `TypedArray/slice-bitwise-same.js` | 2 | T15: Nested yield regions and resumable for-of require the in-progress generator implementation. |
| `TypedArray/slice-detached.js` | 2 | T15: Nested yield regions and resumable for-of require the in-progress generator implementation. |
| `TypedArray/sort-negative-nan.js` | 2 | T15: Nested yield regions and resumable for-of require the in-progress generator implementation. |
| `TypedArray/sort_large_countingsort.js` | 2 | T05: Execution reaches heap_alloc via array_alloc and exhausts the current linear bump heap; raising its limit is not a collector. |
| `TypedArray/sort_small.js` | 2 | T15: Nested yield regions and resumable for-of require the in-progress generator implementation. |
| `extensions/ArrayBuffer-slice-arguments-detaching.js` | 2 | T05: The test calls host gc, which requires a real collector. In the detachment cases this occurs during argument coercion before the expected TypeError. |
| `extensions/DataView-construct-arguments-detaching.js` | 2 | T05: The test calls host gc, which requires a real collector. In the detachment cases this occurs during argument coercion before the expected TypeError. |
| `extensions/DataView-set-arguments-detaching.js` | 2 | T05: The test calls host gc, which requires a real collector. In the detachment cases this occurs during argument coercion before the expected TypeError. |
| `extensions/arguments-property-access-in-function.js` | 1 | T09: SpiderMonkey-specific dynamic sloppy function .arguments/.caller behavior is not implemented; these extension expectations remain visible failures. |
| `extensions/dataview.js` | 2 | T13: Dynamic eval/Function has no compiled source/environment specialization; explicitly unsupported by the Wasm-AOT contract. |
| `extensions/destructure-accessor.js` | 2 | T13: Dynamic eval/Function has no compiled source/environment specialization; explicitly unsupported by the Wasm-AOT contract. |
| `extensions/expression-closure-syntax.js` | 2 | T13: Dynamic eval/Function has no compiled source/environment specialization; explicitly unsupported by the Wasm-AOT contract. |
| `extensions/function-caller-skips-eval-frames.js` | 1 | T09: SpiderMonkey-specific dynamic sloppy function .arguments/.caller behavior is not implemented; these extension expectations remain visible failures. |
| `extensions/function-properties.js` | 1 | T09: SpiderMonkey-specific dynamic sloppy function .arguments/.caller behavior is not implemented; these extension expectations remain visible failures. |
| `extensions/keyword-unescaped-requirement.js` | 2 | T13: Dynamic eval/Function has no compiled source/environment specialization; explicitly unsupported by the Wasm-AOT contract. |
| `extensions/quote-string-for-nul-character.js` | 2 | T23: Intl.Collator is absent, so supportedLocalesOf yields TypeError before invalid-option validation. Later assertions also require other Intl capabilities. |
| `extensions/recursion.js` | 1 | T09 / T05: Recursive calls trap at the Wasm stack limit before a catchable JS error; the test also subsequently requires host gc. |
| `extensions/regress-650753.js` | 2 | T05: The test calls host gc, which requires a real collector. In the detachment cases this occurs during argument coercion before the expected TypeError. |
| `extensions/typedarray-set-detach.js` | 2 | T05: The test calls host gc, which requires a real collector. In the detachment cases this occurs during argument coercion before the expected TypeError. |
| `extensions/weakmap.js` | 2 | T05: The test calls host gc, which requires a real collector. In the detachment cases this occurs during argument coercion before the expected TypeError. |

## Affected-family regression check

All six full selected families passed: **326/326 executions**, with no omissions or failures.

| Family | Passed / total |
|---|---:|
| `built-ins/Math/log10` | 10 / 10 |
| `built-ins/Temporal/Duration/prototype/toString` | 88 / 88 |
| `built-ins/TypedArray/prototype/sort` | 72 / 72 |
| `built-ins/TypedArray/prototype/toSorted` | 24 / 24 |
| `built-ins/TypedArray/from` | 42 / 42 |
| `built-ins/Array/from` | 90 / 90 |

The log is `target/watched/failure-batch-families.log`; each snapshot and binary-hashed receipt is under `target/failure-batch-20260925/families/`. These families overlap the original-failure rerun and must not be added to its fixed-execution count.

## CLI fixture correction

Read-only review of the CLI sort fixture found a comparator that transferred
its ArrayBuffer on every call while expecting sorting to finish normally.
Repeated comparison after detachment is valid; the second transfer correctly
throws TypeError. A direct run with the new compiler reproduced that error.
The fixture now detaches once and retains its assertions that comparator-result
coercion runs after detachment and that the view stays detached. The corrected
fixture executed successfully through Wasm AOT (`boolean(true)`), recorded in
`target/watched/failure-batch-sort-cli-fixture-corrected.log`. This corrects the
fixture's dependence on the old premature sort abort, not the product semantics.

## Evidence and limits

Frozen inputs and rerun receipts are under `target/failure-batch-20260925/`; the run log is `target/watched/failure-batch-original-cases.log`. Each selection ran through the unmodified pinned harness in sloppy/strict mode as declared, with a 60-second case timeout, three workers, and the bounded process settings in the plan. The runner verified every original failing ID completed and reconciled pass/failure counts with process exit status.

The generator implementation and broad integration checkpoint are still in progress. No skips or expected-failure rules were added. GC requires the [planned object-model cutover](value-heap-gc.md); this patch does not substitute a no-op collection hook or increase heap limits. Published full-suite counts are unchanged.
