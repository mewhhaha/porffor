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
three Cargo jobs and at most three test threads. Production edits in this batch
have passed 28 Python profile-generation tests and generated profile, range and
kernel-identity checks. Rust compilation and runtime verification are pending.

Exact execution IDs, diagnostics, binary identities and completed-test sets are
in the [machine-readable record](failure-discovery-20260926.json). Candidate
provenance, replay drivers and receipts live under
`target/failure-discovery-20260926/`; watched logs are
`target/watched/failure-discovery-engine.log` and
`target/watched/failure-discovery-candidates.log`. The replay command is
`python3 target/failure-discovery-20260926/replay_candidates.py` in the capped
scope. A full publication requires the separate `test262 publish-status
--execution-backend wasm-aot` workflow.
