# Integrated failure repair — 2026-09-25

Code checkpoint: `e1b872c93`. This batch repairs the 103 previously failing
executions across 54 files recorded in the [plan](failure-batch-plan-20260925.md).
It is a targeted repair checkpoint, not a full Test262 status publication.
Published full-suite counts remain unchanged.

## Implementation

- Temporal branded conversions read internal slots and preserve option/getter
  order; duration string rounding validates the rounded result, carries through
  days, and handles rounded zero. Time-zone parsing distinguishes constructor
  identifiers from accepted conversion strings and validates numeric offsets.
- Math.log10 uses a real numeric kernel, including IEEE edge cases and subnormals.
- Array.from and TypedArray.from preserve primitive iterator receivers and
  ordinary indexed reads. Created realms install the required methods and host
  hooks. TypedArray sorting is stable O(n log n), continues comparisons after
  detachment, and preserves result coercion and live-view writes.
- Synchronous generators support recursively nested yielding loops and branches
  with checked continuation plans and activation-owned Iterator Records. Async
  and generator for-of share one complete iterator protocol emitter.
- TypedArray.set coerces offsets before detached/out-of-bounds validation,
  retains the required early immutable-buffer rejection, and observes target and
  source effects before rejecting positive infinity or oversized offsets.

Nested yielding try/finally, captured classic-for lexical heads, and other
unsupported generator suspension forms still fail explicitly. This is not a
claim of complete generator conformance. The sorting fixture was corrected to
transfer its buffer once; the set fixture now requires offset coercion before
detached-target rejection. No skips or expected-failure rules were added.

## Verification

| Check | Result |
|---|---|
| First complete original-case rerun (`3c715f46f`) | 57/103 pass |
| Generator complete original-case rerun (`699b69067`) | 67/103 pass; no regressions from first checkpoint |
| Original set-detached case after its fix | 2/2 pass |
| Final combined 103-case rerun | 69/103 pass; 34 remain; no regressions from earlier checkpoints |
| Six full affected families at first checkpoint | 326/326 pass |
| Full TypedArray.prototype.set family | 220/220 pass |
| Combined generator/async-for-of/async-binding execution tests | 33/33 pass |
| Updated generator structural tests | 38/38 pass across 7 targets |
| TypedArray.set expanded CLI regression | 1/1 pass |
| TypedArray.set structural tests | 3/3 pass |
| Updated TypedArray witness census | 5/5 pass |
| Generator module refactor golden comparison | 746 fixture records; empty diff |
| Set offset module refactor golden comparison | 746 fixture records; empty diff |
| Formatting, whitespace, module boundaries, host ABI, retirement, identity | Pass |
| Broad core, engine integration, engine library and CLI checkpoint | Pending |

Golden captures record emitted hashes, byte counts, diagnostics and debug dumps;
they are not 746 conformance passes. Both refactors preserved all captured output.
Removing each new module in isolation made the boundary guard reject the tree;
all files were restored. Independent review found no correctness concern in the
TypedArray.set ordering change.

## Remaining failures and limits

The final run passed **69/103** with **34 failures**. No original execution was
omitted. Exact passing/failing IDs, diagnostics and owner tasks are in the
[machine-readable integrated checkpoint](failure-batch-integrated-20260925.json).
The [first checkpoint](failure-batch-checkpoint-20260925.md) preserves the earlier
57/103 result. All 38 Temporal, both Atomics and all 12 formerly generator-blocked
executions now pass.

| Remaining category | Executions | Owner |
|---|---:|---|
| Unsupported dynamic source generation | 12 | T13 |
| Host GC requiring a real collector | 12 | T05 |
| Heap exhaustion | 4 | T05 |
| SpiderMonkey function extensions | 3 | T09 |
| Missing Intl methods | 2 | T23 |
| Catchable stack overflow, then GC | 1 | T09 / T05 |
 A no-op GC hook or larger heap would not fix the
collector gap; it needs the [planned object-model cutover](value-heap-gc.md).

## Reproduction and evidence

Run from the integration checkout. Cargo and Test262 work used watched systemd
scopes with a 10 GiB memory cap, no swap, at most three Cargo jobs and three test
threads. The pinned harness was unchanged; every original execution ID must be
present, completed, and reconciled with the reported verdict and exit status.

The final original-case log is `target/watched/failure-batch-original-cases-final.log`;
the set-family log is `target/watched/failure-batch-set-upstream.log`.
Machine-local receipts, frozen inputs and driver scripts are under
`target/failure-batch-20260925/`. Logs are under `target/watched/` with the
`failure-batch-` prefix. The original-case driver is `rerun_failures.py`; family
drivers are `rerun_families.py` and `rerun_set_family.py`. Broad verification uses
`run_broad.py core engine-integration engine-lib`, then `cli_chunks.py` and
`audit_cli_results.py`.

The full-suite publication refresh is deliberately separate; use the repository
`test262 publish-status --execution-backend wasm-aot` workflow to refresh it.
