# Remaining Temporal and staging failures: repair plan

Starting integration revision: `647b24005` on
`feat/test262-upstream-20260924`. The user requested implementation with parallel
agents, with the integrator responsible for review and verification.

## Work allocation

| Lane | Scope | Owner |
|---|---|---|
| Temporal conversions | Branded PlainDateTime/ZonedDateTime conversion, observable operation order, Instant grammar and constructor time-zone validation | `temporal_conversion_fixes` (GPT-6-Sol, xhigh) |
| Duration | Rounded-duration validation, day carry and exact total arithmetic | `temporal_recovery` (resumed agent; model settings unavailable) |
| TypedArray and realms | Primitive iteration, created-realm methods, detach host plumbing and runtime DataView argument validation | `integration_review` (resumed agent; inherited model settings unavailable) |
| Shared integration | Math.log10, stale SharedArrayBuffer preflight gating, realm harness, review, failure inventory and verification | primary agent |
| Next independent chunks | Generator loop/branch suspension; TypedArray sort complexity; allocation/collection; remaining extension and Intl capabilities | primary agent triage, then freed lanes |

The platform accepted one new Sol/xhigh thread and rejected additional new
threads. Two existing agents were resumed rather than silently claiming their
unexposed model configurations match the requested override.
As those lanes finished, the platform accepted new GPT-6-Sol/xhigh agents
`math_log10_fix` and `typedarray_sort_fixes`. They own the numeric kernel and
stable-sort complexity respectively. `temporal_conversion_fixes` also completed
a read-only design for the next generator-lowering batch.
`generator_nested_yields` (GPT-6-Sol, xhigh) is implementing that batch in the
isolated `generator-nested-yields` worktree while the first batch is verified.
`failure_batch_review` independently reviewed the first batch and corrected
Array.from primitive boxing, length snapshotting and inherited indexed reads.

The preserved Temporal comparison has now completed: baseline **8,976/9,210**,
after **9,172/9,210**. Its 38 remaining failing executions across 19 files and the
65 staging failures are the repair inventory. This evidence uses the earlier
saved binaries, not the new unverified batch.

Shared files are coordinated explicitly. Duration owns its method emitter;
conversion work owns other Temporal emitters and the relevant constructor/parser
regions. TypedArray owns the Array.from/TypedArray.from arm of `standard.rs`,
created-realm installation in `host.rs`, and DataView static call classification.
The primary owns the Test262 harness and numeric kernel. Changes to sorting in
`standard.rs` require a separate coordinated region.

## Acceptance criteria

1. Read the pinned original tests, identify the actual semantic failure and add
   compiled-path regressions. No fixture-specific branches, no-op host GC,
   weakened expectations or test-source rewrites.
2. Keep exact arithmetic, ordering and validated domains in code invariants
   where practical. Compile JavaScript to Wasm; do not add a runtime interpreter.
3. Finish each coherent implementation batch before compilation. The integrator
   reviews diffs, compiles once, runs focused regressions and original failing
   Test262 executions, then sequential broad integration checks.
4. Compare complete execution-ID sets and classify every remaining failure. A
   newly exposed later failure is not a passing test. Partial sweeps are not
   complete verdicts.
5. Dynamic source generation requiring a bundled parser/interpreter remains an
   explicit Wasm-AOT limitation under AGENTS.md. Missing GC, Intl or generator
   capabilities require real implementation; they cannot be faked to make the
   staging tests pass.

Cargo and conformance commands use memory-capped systemd scopes (`MemoryMax=10G`,
`MemorySwapMax=0`), `nice -n 5`, at most three cargo jobs and three test threads.
Long runs use `scripts/run-watched.sh`. Frozen input snapshots and later receipts
are machine-local under `target/failure-batch-20260925/`. The saved Temporal sweep is complete. The current failure inventory contains
103 failing executions across 54 files.

This document is a work plan, not evidence of completed fixes or a conformance
publication. The published full-suite status remains unchanged.

## First implementation checkpoint

The focused engine targets contain 29 tests. The first run exposed a LIFO
compiler-temporary release bug in the new log10 emitter; it was corrected before
the full focused run. That run passed 27 of 28 tests and exposed a missing Intl
import declaration in PlainDate conversion. All five ToTemporalDate callers now
declare the dependency. A new separate-program regression prevents another
Temporal method from masking a missing import. The affected conversion and
wide-duration targets then passed all 13 tests. Combined with the unaffected
focused results, every one of the 29 focused tests has a passing result.

This is initial verification plus focused correction checks, not a claim of a
complete broad rerun. Logs: `target/watched/failure-batch-focused-corrected.log`
and `target/watched/failure-batch-import-fix.log`. Formatting, diff whitespace,
module boundaries, host ABI, repository identity, legacy retirement and the
named-zone/time-zone-name/Intl identity generation checks passed. The original
Test262 failure rerun and broad verification are still pending.
