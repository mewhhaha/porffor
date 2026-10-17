# Continue Lila — 2026-10-17

The task is open. Delete this file and `tasks/` only after every acceptance gate
passes. Native subsets and focused Test262 inventories do not establish full
pinned conformance. Push verified progress to main continuously.

Current recovery work is checkpointed to `codex/cloud-recovery-20261017`.
The declaration-table repair passes all13 original Unicode failures through the
actual Cargo-built public CLI at compiler identity
`e74f8d553509a1dc15d6cd8923af9b0a10ac1b4189f59dba5a00efb53ef974be`.
The197-prior-pass replay was deliberately stopped at a saved110-case checkpoint:
105Success/5deadlineCrash,87 IDs unaccepted. Every input remains in the next
full197 replay; this is not a new230 aggregate or accepted retention result.
[Exact CLI checkpoint](docs/rust-rewrite/direct-eval-cli-checkpoint-20261017.json).

Four complete native targets pass: AOT411, IR1599, IR integration740 and
Engine965. The fullRealm target completed427passes/1failure in a new incorrect
constructor-name assertion; its corrected six-case function passes focused
native execution. The original rejected receipt is preserved in
[native evidence](docs/rust-rewrite/rejected-prelude-native-20261017.json).
Separate complete Front177 and Runtime33 native tests also pass. Current-source
ordinaryCargo/wholeRealm verification remains required after the next batch.

The coherent next batch is applied: shared immutable full bootstrap plans,
indexed local declarations, omission of unused native-factory Main, the prelude
assertion correction, and counted loops for identical live identifier reads.
Type checking and18 non-executing emission/validation controls pass; the largest
body for five10,000-read variants falls from1.9MB to under6KB while preserving
the original reads and completion path. Runtime Wasm bytes are unchanged.
Actual Cargo/native/CLI and timing acceptance remain pending. Build selected
native scopes together once, then run complete affected scopes and all13+197
literal CLI IDs before promoting this compiler batch to main. Avoid an extra
wholeRealm rebuild for the already-rejected intermediate compiler.

The recovered f9ff55f5a probe passed all 1,599 IR unit and 740 IR integration
checks, retaining every earlier integration ID. AOT was red: 407 passed and the
new 1,200-name body-size guard failed. Diagnostic emission measured 907,642 body
bytes for bare declarations and 971,170 bytes / 4,838 locals with a strict
directive. The Realm target was not executed; the rejected batch stopped at a
completed IR dependency boundary, preserving its outputs. The current repair
uses P-owned packed name tables for the three ordered phases and also permits
literal strings in the no-frame proof. Its fresh native/CLI verification is
pending; it has not repaired any real-suite verdict yet.

The 3f33caf46e native probe passed the 1,200-name size guard, all 1,599 IR
unit tests and all 740 IR integration tests. AOT completed 408 passes and two
failures because sloppy direct eval's runtime global branch had no collected
table. Engine reproduced that defect before the rejected run was stopped;
Realm and CLI replay were unexecuted. The collector now includes sloppy direct
eval without changing its caller-local frame proof. A dedicated strict/sloppy
collection control and fresh native/CLI verification are pending.
[Rejected-probe evidence](docs/rust-rewrite/rejected-packed-global-plan-probe-20261017.json)
preserves the failure and partial-run scope.

Last published focused checkpoint (2026-10-17): all230 original stress executions have
completed with the current compiler:197Success/16Bug/17Crash, zero skipped.
The remaining203 contribute171Success/16Bug/16Crash; all26 earlier stress passes
are retained. These are a focused inventory, not full pinned Test262 acceptance.
All74 tests in the repaired `modules_realms` Cargo target pass unfiltered.
The current compiler also has965 Engine,407 AOT and8 template integration passes;
the1,599-test IR evidence is retained after exact input checks, not rerun.
[Literal outcomes and raw evidence](docs/rust-rewrite/current-conformance-230-and-module-realms-20261017.json) preserve every failure. The next compiler
repairs target oversized Unicode declaration bodies and repeated native eval
exhausting Wasmtime's10,000-instance Store limit. Task closure remains open.

The last published baseline compiler fingerprint is
`9a5ccae64da4b8ef001b37da6ee75e0c5a33abd732c00caa25fc0aa74478cc6b`.
The native support fingerprint is unchanged:
`b4f121eff0513714278d34ae4e89ad214c28934c8fb3521d145034c847b87c30`.

## Known failures and unaccepted scopes

The last completed230-case baseline contained33 nonpasses. Its current-source
subsets have the following status; do not manufacture a new aggregate:

- All13 Unicode identifier executions now pass at the identity recorded above.
  Retain them when verifying the next compiler identity.
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
- Five previously passing executions timed out in the interrupted197 replay:
  both modes of `destructuring-array-default-simple.js` and
  `object-literal-__proto__.js`, and sloppy `regress-563221.js`. The throughput
  and large-body repairs are applied but their actual retention is unverified.

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
