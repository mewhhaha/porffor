# Current project inventory — 2026-10-21

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
The remaining 92 hills are candidates. This batch adds no conformance passes and
does not establish complete workspace, release or Test262 acceptance.

## Test262: complete baseline and focused repair

| Evidence | Source / scope | Observed result |
| --- | --- | --- |
| Complete original pinned suite, 2026-10-20 | Compiler `c5053c9d4`; 102,994 executions | 102,978 Success, 16 Bug, 0 Crash, 0 NotImplemented |
| Original failure replay, 2026-10-21 | Compiler with global-prelude repair `cbb07c6cf`; 16 exact IDs | 12 Success, 4 Bug |
| Remaining-failure recheck | Four original IDs and ten nearby controls | Four failures retained; all ten controls pass |
| Maintenance revalidation | Fresh default compiler `e7532de91`; original 16 IDs, then four plus ten controls | 12 Success/4 Bug; then 10 Success/4 Bug; zero Crash/NotImplemented |

[Complete baseline](test262-current.md) and [focused replay](failed-case-replay-20261021.md)
bind exact pins, compiler fingerprints, executable hashes and commands. The
canonical README chart still represents the complete pre-repair baseline.
There has been no complete fixed-source campaign, so subtracting targeted fixes
from published totals is not authorized.

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
and preserves the original failure-replay results. The earlier optimized release
at `c5053c9d4` also builds without the oracle. Three synchronous React 19.3.1 SSR
outputs match Node at that earlier source; the public browser entry
fails. [SSR evidence](react-ssr.md) and [local Wasmtime/V8 measurements](v8-baseline.md)
are focused demonstrations, not full compatibility or calibrated T25 acceptance.

The host requires the [documented Wasm features and Scriptw imports](runtime-requirements.md).
Portable Wasm does not bundle a parser/VM for general eval or Function constructors.
Finite prepared sources and native-host compilation are distinct supported paths.
Scope classification of Test262 dynamic-code cases remains unaudited.

## Native verification and release gates

At `e7532de91`, complete engine (973), CLI library (82) and CLI integration (856)
scopes pass with zero failures or ignored tests; the replacement runtime-image
anchor has zero tests. The CLI inventory is covered exactly once by four serial
partitions. Engine Intl's 299 tests pass on the preceding frozen batch source.
The record separates these receipts and documents the fixture-only correction
and formatting-only restoration between them. All 653 tooling checks pass, as do
locked all-target/all-feature Cargo check and full NumberFormat regeneration.
These affected scopes do not replace the unfinished broader campaign below.

The stopped pre-rename campaign at `cbf7cc71f` completed 49/113 scopes with
7,560 passes, one failure and zero ignored tests. Sixty-four scopes remain
uncompleted, including 2,094 expected native bodies and documentation tests.
Its original failure is:

`scriptw-test262:differential_generated_campaign::builtin_scenarios_use_actual_selected_workers_for_all_six_stateful_families`

The seed-0 array-buffer Wasm worker timed out at the original 5,000 ms deadline.
The oracle completed. Its cause is unresolved; a diagnostic pass does not replace
it. The failed executable and receipts remain in the
[source-bound inventory](current-inventory-20261020.json).

The old coordinators were absent after restoration. Their saved disk wait is
historical, not a current capacity or live-process observation. Reconcile original
receipts before continuing uncompleted scopes in a fresh namespace.

At the earlier source, all ordinary library scopes and Engine Intl integration
checks passed. Later renamed-release and tooling checks have separate receipts;
none transfers an older full-suite result to the maintenance batch.

Outstanding gates include:

- Complete current-source all-feature/default native suites and ordinary
  optimized release acceptance.
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
