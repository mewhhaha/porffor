# Continue Lila — 2026-10-17

The task is open. Delete this file and `tasks/` only after every acceptance gate
passes. Native subsets and focused Test262 inventories do not establish full
pinned conformance. Push verified progress to main continuously.

Current focused checkpoint (2026-10-17): all230 original stress executions have
completed with the current compiler:197Success/16Bug/17Crash, zero skipped.
The remaining203 contribute171Success/16Bug/16Crash; all26 earlier stress passes
are retained. These are a focused inventory, not full pinned Test262 acceptance.
All74 tests in the repaired `modules_realms` Cargo target pass unfiltered.
The current compiler also has965 Engine,407 AOT and8 template integration passes;
the1,599-test IR evidence is retained after exact input checks, not rerun.
[Literal outcomes and raw evidence](docs/rust-rewrite/current-conformance-230-and-module-realms-20261017.json) preserve every failure. The next compiler
repairs target oversized Unicode declaration bodies and repeated native eval
exhausting Wasmtime's10,000-instance Store limit. Task closure remains open.

The current product compiler fingerprint is
`9a5ccae64da4b8ef001b37da6ee75e0c5a33abd732c00caa25fc0aa74478cc6b`.
The native support fingerprint is unchanged:
`b4f121eff0513714278d34ae4e89ad214c28934c8fb3521d145034c847b87c30`.

## Known current failures

The exact230-case inventory contains33 nonpasses:

- 13 Unicode identifier executions fail Wasm function-body validation. A raw
  fixture control emits a2.50MB main body and validates; measure the actual
  production-materialized case before attributing the larger failing body.
- 14 dynamic eval stress executions exhaust the Store instance count at10,001.
  Compiled-module reuse still creates a fresh Instance for every invocation.
  Any instance-sharing repair must preserve every source's immutable data,
  fresh ProgramInstance/Realm semantics, live functions and nested callbacks.
- 2 regexp executions time out on the same repeated native-source path.
- 1 strict class execution still times out in the mixed campaign, although the
  isolated strict/sloppy class pair passes. Its reliable repair remains open.
- 2 Temporal Johnston link executions conflict with the pinned backzone/spec
  mapping. CLDR aliases and a cross-country remap are not accepted authority.
- 1 staging Annex B labelled-function fixture conflicts with the accepted
  label-chain boundary. Keep its literal failure pending authoritative resolution.

The unchanged deadline is60000ms, compile allowance300000ms, and worker count1.
The official Annex B implicit-arguments regression now passes. The typed empty
strict-body compiler retains entry guards, source activation tracking and cleanup.

## Next work

1. Repair the two shared compiler/runtime causes above without increasing
   deadlines or losing any of the197 passing literal IDs. Preserve independent
   full-library and integration verification at each changed compiler identity.
2. Finish current-source native coverage. Cargo metadata discovers84 default
   integration targets; the earlier selected21 affected targets are not all84.
   Build the current CLI library/executable once, and admit the missing Boa/spec-
   exec closure separately. Full ordinaryCargo default/all-feature/all-target
   and actualCLI acceptance remain open. The current compiler passed the CI
   workspace check; the latest assertion/format fixes still need complete CI.
3. Replay all5,365 historical IDs from
   `target/historical-backlog-replay-input-20261012/executions` (SHA256
   `95d46445f4c3fda986c7cdf390dccdddfb8ab370b8a831e3f77ea7f387a1a43e`),
   retaining every earlier success and every nonpass in the denominator.
4. Complete every task criterion: full-manifest shard/thread and interruption/
   resume equivalence, current-source performance and idle campaigns, and two
   fresh unfiltered complete pinned Wasm-AOT families.
5. Preserve actual supported-host artifacts. Native Intl run39045297297 passed
   Ubuntu, macOS and cross-host comparison at this compiler; actual artifact IDs
   remain hosted in that run, and local download is not yet admitted. Other host
   acceptance remains open. Resolve T22 Apia against accepted authority; issues
   and draft proposals do not authorize changing the expectation.
6. Generate canonical real-suite status through the publisher, commit exact
   pins and source-bound closure artifacts, then remove this file and `tasks/`.

## Verification discipline

Use `scripts/limited_verification.py --cloud` under the inherited finite machine
budget, serial workers and bounded caches. Admit enough scratch capacity before
compilation. Reuse optimized current libraries and archive only inactive outputs
after checking every decoded byte. Never mutate a source or driver bound to an
active run. Earlier checkpoint passes keep their actual compiler identities.

Rejected cache-size and native-parallelism trials remain in
`docs/rust-rewrite/rejected-native-cache-experiments-20261017.json`; do not repeat
them without new measurements. T00/T27/T28/T29 are complete. T01–T25 remain in
progress and T26 remains blocked until their actual acceptance criteria pass.
