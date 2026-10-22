# Current project inventory — 2026-10-22

This is the maintained status entry point. Results belong to their measured
source and executable, not automatically to today's checkout. The
[earlier inventory](inventory-history-20261021.md) retains all detailed historical
counts and links. [Workflow](batch-workflow.md) owns operating instructions;
[maintenance hills](../maintenance-hills.md) owns cleanup status.

## Current work

The first eight maintenance hills are complete: obsolete product routes are
retired, runner setup is clearer, LDML parsing has one owner, and current guidance
is separated from history. Implementation is in `ab7093a46`, `30065bbb5` and
`e7532de91`; [verification and exact source limits](maintenance-first-wave-20261021.md)
are recorded with [artifact identities](maintenance-first-wave-20261021.json).
The following [tooling/documentation wave](maintenance-tooling-wave-20261021.md)
completes 28 more hills. The [CLI/reporting wave](maintenance-cli-wave-20261021.md)
completes nine more. The [conformance/publication wave](maintenance-conformance-wave-20261021.md)
completes another 16. The [compiler/runtime/Intl wave](maintenance-compiler-intl-wave-20261021.md)
completes the final 39: **all 100 authorized maintenance hills are done**.
The final checkpoint covers all 109 all-feature native scopes after two complete
test-owner replays, plus 685 tooling controls and all 15 Intl identities.
107 default and 109 all-feature planning scopes remain unchanged. Maintenance
adds no published Test262 passes and does not establish release acceptance.

The [performance backlog](../performance-hills.md) contains 160 investigations
and optimization experiments. Its first verified execution batch ships batched
concat (P091) and matching optimized Portable runtime images (P153); the numeric
Array experiment (P040) was rejected and removed. Execution improves in the
measured corpus, with an explicit 116.01 MiB executable and startup-time cost.
[Results, exact source limits and evidence](v8-baseline.md#first-execution-batch--published-2026-10-22)
record 2,687 passing ordinary native tests and a 186-execution Test262 replay:
182 Success / the same four Bug results. This is not a fresh full Test262 or
all-feature workspace checkpoint; published conformance counts are unchanged.

The [second performance batch](v8-baseline.md#second-execution-batch--published-2026-10-22)
is accepted at `865189051` for the measured corpus and **3,567 scoped native
tests**, with zero failures/ignored tests and unchanged source. Product and test
runtime image bytes match. It retains shared immutable empty storage, bulk
repeat/pad/trim/slice operations, shorter array-length/normal-call routes,
identical-operand descriptor snapshots and transient decoded Portable images.
The CLI shrinks by 57.12 MiB against the preceding accepted batch; retained-row
invocation process RSS is 373.68 MiB and the separate whole-child peak is
392.80 MiB. These are process measurements, not live GC bytes. The README now
uses the new seven-sample Wasmtime/V8 report.

Its [final 9,207-execution Test262 replay](performance-second-wave-replay-20261022.json)
returns **9,203 Success / 4 Bug / 0 Crash / 0 NotImplemented**, preserving exactly
the original four upstream-tracked failures. No new regression appears in the
campaign. The intermediate F replay is historical; the final affected-case
campaign does not refresh the full-suite graph. Rejected layout-16 binding and numeric-branch
experiments remain removed, with their evidence retained. Layout remains 15 and
bindings retain immutable StoredValue snapshots. Performance work remains open.

Startup host sampling identifies SHA-256 as roughly 64–74% of sampled user CPU.
A separately identified debug-stripped copy is prepared with allocated ELF
sections verified equal, full executable hashing preserved and separate caches;
its paired timing waits for correctness work to finish. No ordinary default
compiler-policy or integrity-admission change follows from that preparation.

The CLI/reporting wave passes 478 Test262 library tests, 89 CLI unit tests and
14 affected CLI integration controls after correcting two batch test/diagnostic
issues and ten preexisting rename-related expectations. Its record retains the
earlier full 856-ID CLI checkpoint and adjacent checks with their source limits.

The conformance/publication wave passes 493 Test262 library tests, all 856 CLI
integration tests, 91 CLI unit tests and 29 adjacent native tests across its
corrected source-bound checkpoints. It also passes 669 tooling controls and 53
structure checks, and admits all 53,910 pinned Test262 files (102,994 execution
IDs). Failed attempts and the test-only changes between checkpoints are retained
in its record; that earlier selected checkpoint implied no complete workspace
or conformance refresh. The final maintenance checkpoint is recorded below.

## Test262: complete baseline and focused repair

| Evidence | Source / scope | Observed result |
| --- | --- | --- |
| Complete original pinned suite, 2026-10-20 | Compiler `c5053c9d4`; 102,994 executions | 102,978 Success, 16 Bug, 0 Crash, 0 NotImplemented |
| Original failure replay, 2026-10-21 | Compiler with global-prelude repair `cbb07c6cf`; 16 exact IDs | 12 Success, 4 Bug |
| Remaining-failure recheck | Four original IDs and ten nearby controls | Four failures retained; all ten controls pass |
| Maintenance revalidation | Fresh default compiler `e7532de91`; original 16 IDs, then four plus ten controls | 12 Success/4 Bug; then 10 Success/4 Bug; zero Crash/NotImplemented |

[Complete baseline](test262-current.md) and [focused replay](failed-case-replay-20261021.md)
bind exact pins, compiler fingerprints, executable hashes and commands. The
canonical publication retains the complete pre-repair baseline. The README chart
now reconciles it with those exact replay repairs and an audited standalone
dynamic-source scope partition; it does not claim a fresh full rerun.
There has been no complete fixed-source campaign, so the canonical publication
retains its original counts. The explicitly reconciled chart partitions those
102,994 IDs as 102,561 in-scope passes, four failures, zero NotImplemented and
429 audited dynamic-source executions outside standalone Wasm scope.

**User decision: keep all four original disagreements as failures.** The README
lists reasons and [upstream tracking](../../test262/errata/proposals/upstream-status-20261021.md):
arguments has an exact open issue/PR; Johnston has related open discussion;
labelled functions have related discussion but no exact issue found in the search.
Local amended-fixture and specification proposals are held. They do not change
original results, and no report was posted upstream. The
[resolution evidence](final-four-resolution-20261021.md) remains historical.

## What the product can compile and run

Scriptw compiles JavaScript through parsing, early errors, spec IR, lowering IR
and Wasm code generation. Wasm-AOT is the default; the feature-gated interpreter
is an engineering oracle. C/native JavaScript output and a REPL are out of scope.

Verified representative programs include closures and private fields, object
properties/Proxy/Reflect, collections, typed arrays/BigInt/Unicode/JSON, Promise
jobs and generators, filesystem ES modules with top-level await, and selected
Intl/Temporal operations. The CLI provides `print`; it does not supply Node or
browser globals such as `console`.

`run file.mjs` and `run --module file.js` use the Module goal. `build wasm file.js`
compiles a Script and can emit a program/runtime Wasm pair through
`SCRIPTW_WASM_DUMP`; a dedicated module-artifact build command is not exposed.

The ordinary default-feature debug CLI at `e7532de91` builds without the oracle
and preserves the original failure-replay results. The default-feature optimized
release at `cbbd9b170` builds and completes the new performance/memory corpus;
its recorded timings, child RSS, compilation
phases and emitted Wasm are in the [current measurement](v8-baseline.md).
The earlier optimized release at `c5053c9d4` also builds without the oracle.
Three synchronous React 19.3.1 SSR outputs match Node at that earlier source;
the public browser entry fails. [SSR evidence](react-ssr.md) and
[local Wasmtime/V8 measurements](v8-baseline.md)
are focused demonstrations, not full compatibility or calibrated T25 acceptance.

The host requires the [documented Wasm features and Scriptw imports](runtime-requirements.md).
Portable Wasm does not bundle a parser/VM for general eval or Function constructors.
Finite prepared sources and native-host compilation are distinct supported paths.
The chart now has an audited 429-execution dynamic-source subset; the scope
inventory is conservative and does not claim to classify every incidental
mention of eval or Function.

## Native verification and release gates

The final [compiler/runtime/Intl checkpoint](maintenance-compiler-intl-wave-20261021.md)
covers every one of the 109 all-feature native scopes: **9,712 passed, zero
remaining failures, zero ignored**. The frozen workspace run passed 107 scopes
and exposed two stale source-layout assertions. Only those two test files changed;
complete frontend (181) and engine structure (33) replays then passed with the
same dependency closures. The zero-test feature anchor adds no tests.
[Exact receipts and source boundaries](maintenance-compiler-intl-wave-20261021.json)
retain both original failures and all inventories.

This includes all 973 engine unit tests, 299 Engine Intl integration tests,
856 CLI integration tests and the differential campaigns at their original
budgets. The final full tooling checkpoint passes all 685 controls with zero
skips under TMPDIR=/tmp. It includes three new mutation controls for standalone
source guards corrected after the native replays; no compiler or native-test
source changed. Earlier failed boundary checks are retained in the record.
Locked all-target/all-feature Cargo check and all 15 Intl identity checks pass.
Generated semantic payloads remain byte-identical. This is all-feature debug
native verification, not default/release or a complete refreshed Test262 run.

The stopped pre-rename campaign at `cbf7cc71f` remains historical: 49/113 scopes,
7,560 passes, one failure and zero ignored tests, with 64 scopes unfinished.
It is superseded for current all-feature coverage by the complete 109-scope
checkpoint above, not silently resumed or relabelled. Its original failure was:

`scriptw-test262:differential_generated_campaign::builtin_scenarios_use_actual_selected_workers_for_all_six_stateful_families`

The seed-0 array-buffer Wasm worker exceeded its original 5,000 ms deadline.
That control now passes at the unchanged deadline in the new complete native
checkpoint. This does not prove the old timeout's cause; its failed executable
and receipts remain in the [historical source-bound inventory](current-inventory-20261020.json).

Outstanding gates include:

- Complete current-source default native and ordinary optimized release
  acceptance; current all-feature native coverage is complete as recorded above.
- Original 230-case/prior 219-case retention and real serial/shard/resume checks.
- Complete debug/optimized differential and robustness campaigns, idle samples,
  supported-host reproducibility and calibrated T25 performance.
- Two fresh complete current-pin Wasm-AOT families, deterministic backlog
  generation and canonical publication with all non-successes visible.
- Historical 5,365-failure replay: its original aggregate is absent from this
  cloud checkout and was likely local/unpushed. Do not manufacture that input.

## Task acceptance

T00, T27, T28 and T29 remain recorded complete; T01–T25 remain in progress and
T26 remains blocked. These are permanent technical areas, distinct from the
maintenance hills. [Ownership](conformance-ownership.md) and
[acceptance criteria](conformance-acceptance.md) are authoritative.

Keep CONTINUE and tasks until every criterion passes. The decision to retain
upstream failures documents their status; it does not establish a green suite
or waive release acceptance. The [continuation queue](../../CONTINUE.md) records
next actions without duplicating this inventory.
