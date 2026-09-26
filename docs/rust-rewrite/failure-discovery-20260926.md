# Additional failure discovery — 2026-09-26

Baseline: `3979f0116`. This is a targeted discovery and repair batch, not a
full pinned Test262 publication. The prior 34 failures remain recorded in the
[integrated checkpoint](failure-batch-integrated-20260925.md).

Nine previously uncovered engine integration targets passed all **74 tests**,
with no failures or ignored cases. Saved broader Test262 results supplied
12 further candidate files. Fresh execution on the existing release binary
confirmed all **19 failing executions**, with no execution-ID overlap with the
original inventory. The known unresolved count at this discovery checkpoint is
therefore **53**, not an estimate of all failures in Test262.

| Additional area | Executions | Observed failure |
|---|---:|---|
| Import bytes | 5 | Host rejects bytes module requests |
| TypedArray locale formatting | 2 | Element calls lose locales and options |
| DateTimeFormat | 12 | Calendar alias/data, range formatting, Dangi parts, Temporal calendar, Japanese hour cycle, absent Intl constructor |

Only dynamic source generation through `eval`/`Function` is an intentional
unsupported category. GC, heap exhaustion, catchable stack overflow, Intl,
module loading and function reflection remain implementation requirements.
No missing capability is counted as a pass or silently skipped.

## Repair ownership and verification

- Root owns Array/TypedArray locale argument forwarding and integration.
- The Intl lane adds Japanese locale data, Dangi and Islamic Civil conversion,
  the `islamicc` alias, and a sparse canonical range-pattern overlay. Collapsed
  ranges retain scalar patterns; noncollapsed ranges use checked CLDR records.
- The function lane owns sloppy activation-based `caller`/`arguments` behavior,
  including recursion, unwinding, strict/native boundaries and direct eval.
- [Import bytes](contracts/import-bytes.md) uses intrinsic initialization and
  preserves module provenance through lowering, admission and artifact caching.
  Coverage includes Script, Module, top-level await and mixed source-phase graphs.
- GC/heap work remains the required object-model/collector implementation,
  subject to the project's Wasm-GC architecture contract.

Implement coherent changes before compilation, then run focused regressions,
original cases and affected families before the combined integration checkpoint.
All runtime commands use the existing watched 10 GiB systemd scope, no swap,
three Cargo jobs and at most three test threads. On 2026-09-26, the combined
122-execution replay passed **87**, retained all **69** earlier passes, and
confirmed **18 additional fixes**: three original function-reflection failures
and 15 of the 19 additional candidates. The original inventory is now **72/103**;
the additional candidates are **15/19**. The remaining **35** include **12**
intentional dynamic-source exclusions and **23** required implementation gaps.

Focused verification passes 452 AOT unit tests, nine activation-structure tests,
19 module-structure tests, three graph-cache tests, 77 engine integration tests
and the corrected CLI locale fixture. Earlier Intl checks passed 251 tests;
profile generation passed 28 Python tests and both generated data and identity
checks. The first broad core run passed 4,510 tests and found 12 failures. Its
size regressions, structural inventories and wildcard-panic issue are corrected;
the final pre-reset core checkpoint passed all 4,522 tests. An environment
reset interrupted engine verification and removed ignored `target/` artifacts.
The [worktree recovery checkpoint](worktree-recovery-20260926.md) records the
subsequent integrated verification after the recovered capture fix: 4,522 core,
395 selected engine integration, 778 engine library and 806 main CLI tests pass;
one existing CLI allocation stress test remains ignored.

Review also repaired stale module-cache resolution identity, duplicated legacy
activation cleanup in every abrupt guard, and activation restoration before
proper tail calls. These have focused runtime/structure regressions. The compiler
used for the combined replay is `02a93376a`; exact binary identity is in the JSON
record. No full-suite publication is claimed.

The affected-family replay completes **598/610**, including DateTimeFormat
**478/490**, Intl Array/TypedArray locale calls **6/6**, core Array locale calls
**22/22**, core TypedArray locale calls **78/78**, bytes imports **5/5**, and nine
neighboring function-reflection regressions **9/9**. The DateTimeFormat result
retains all 466 passes from a historical 490-ID snapshot and reduces its
failures from 24 to 12. That historical snapshot records the same Test262 tree
and execution IDs, but no compiler revision or binary hash; it is comparison
evidence, not an exact `3979f0116` baseline.

Eight remaining calendar executions from that broader replay are outside the
122-ID inventory. Across both runs, **713 distinct executions** were checked;
**43 fail**, of which **12** are intentional dynamic-source exclusions and
**31** are required implementation work. The eight additional failures cover
`formatToParts/compare-to-temporal-lunisolar.js`,
`formatToParts/dangi-calendar-dates.js`, `formatToParts/era.js`, and
`resolvedOptions/calendar.js`, each in strict and sloppy mode. This remains a
targeted inventory, not a full Test262 failure count.

The [machine-readable record](failure-discovery-20260926.json) records execution
IDs, diagnostics, binary identities and replay summaries. Complete affected-family
execution sets are in the per-family receipts. Candidate provenance, replay
drivers and receipts originally lived under
`target/failure-discovery-20260926/`; these ignored artifacts were lost in the
environment reset. Their recorded results remain in this committed JSON. The
original watched log paths were
`target/watched/failure-discovery-engine.log` and
`target/watched/failure-discovery-candidates.log`. The original replay driver was
`python3 target/failure-discovery-20260926/replay_candidates.py` in the capped
scope; it must be reconstructed from the committed case inventory after the
environment reset. A full publication requires the separate `test262 publish-status
--execution-backend wasm-aot` workflow.

## Required work beyond this repair batch

These remain implementation requirements, not intentional unsupported cases:

- Calendar support needs a shared, typed calendar arithmetic boundary used by
  Temporal and DateTimeFormat. The two `compare-to-temporal` tests need 13
  calendars absent from Temporal; seven are also absent from DateTimeFormat:
  Coptic, Ethiopian, Amete Alem, Islamic Tabular, Umm al-Qura, Japanese and Hebrew.
  Supporting them requires real conversion, era/month-code rules, property-bag
  handling and arithmetic. Generated names must admit calendar-specific month
  counts and era inventories instead of assuming 12 months and two eras.
- Missing Collator, PluralRules and RelativeTimeFormat require their actual
  operations and data. Validation-only constructor stubs would hide the first
  failure without implementing the service.
- Catchable recursion exhaustion needs a compiler-owned resource guard with
  justified frame costs and error/unwind headroom. Translating a Wasmtime trap
  after the stack unwinds cannot make an inner JavaScript catch execute. The
  pinned recursion case then calls `$262.gc()`, so fixing catchability alone
  cannot pass that case.
- GC and heap reclamation require the Wasm-GC object-model migration described
  in [the heap contract](value-heap-gc.md). A copying-collector runtime upgrade
  removes one prerequisite; it does not collect the current integer-addressed
  JavaScript heap or implement weak reachability and finalization by itself.

The read-only audits originally lived in
`target/failure-discovery-20260926/remaining-implementation-audits.json`; that
ignored file was lost in the environment reset. The required-work summary
above and per-failure owners in the committed JSON remain available.
