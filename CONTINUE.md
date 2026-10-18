Full-BMP RegExp Engine diagnostic — 2026-10-18: all12 unchanged fixture/mode
combinations now pass through the frozen628 public Wasm-AOT Engine in40.6–51.5s
per case, with the original60000ms execution allowance, one CPU and fresh bounded
caches. Every full65536-unit loop and original assertion is retained. This is a
compiler diagnostic, separate from ordinary CLI acceptance, the full workspace
and the full pinned suite. [Exact sources, outputs and build provenance](docs/rust-rewrite/regexp-bmp-kernel-diagnostic-20261018.json).

Completed focused CLI checkpoint — 2026-10-18: source112 runs all230 exact
original executions at the original budgets: **208 Success,19 deadline Crash,
3 Bug**. Compared with the previous205/22/3 checkpoint, six cases now pass and
three prior passes time out; retention is204/210 and remains rejected. The
compiler stays on recovery. A separate historical parseInt replay retains all
four original strict/sloppy cases on the historical suite tree. These counts
are separate from the full pinned suite; full workspace, pinned-suite acceptance
and task cleanup remain open. [Exact IDs and lossless evidence](docs/rust-rewrite/naming-empty-cli-230-20261018.json).

Intl rebuild iteration — 2026-10-18: the opt-in
`LILA_TEST_KEEP_INTL_INCREMENTAL=1` retains only normal Intl library scratch
when the verification wrapper otherwise removes it. All 90 workspace-tooling
controls pass. In a private identity-edit experiment, ordinary builds took
98.5/127.4s; an incremental edit took11.5s and undo/reapply took13.4/13.6s.
Linked consumers verified both unchanged and changed provider identities.
[Exact scope, cache cost and lossless evidence](docs/rust-rewrite/intl-incremental-identity-20261018.json).
Full-build and conformance effects remain unmeasured; defaults are unchanged.

RegExp repair source checkpoint — 2026-10-18: the next candidate passes
all 603 tooling tests, retains every previous 600 ID, and passes all eight source
CI checks and the full module boundary. [Literal inventories and lossless logs](docs/rust-rewrite/regexp-kernel-tooling-603-20261018.json)
bind these checks to recovery source `dd8b807af`; the [earlier 600-test check](docs/rust-rewrite/regexp-kernel-tooling-600-20261018.json)
remains recorded. Native execution of the new RegExp Wasm factory, CLI retention
and full-suite acceptance remain pending.

Tooling verification checkpoint — 2026-10-18: the naming/empty-eval candidate
passes all 599 unfiltered tooling tests and all eight listed source CI checks.
Every previous 599 test ID is retained; all eight earlier Intl provenance
failures now pass. [Literal inventories and lossless logs](docs/rust-rewrite/naming-empty-tooling-599-20261018.json)
bind these results to recovery source `112a765c1`. The full default-feature
workspace/all-target type check also passes in19m05s from a cold dependency graph
([exact logs](docs/rust-rewrite/all-target-typecheck-20261018.json)). All 4,186 selected
native tests also pass, retaining every prior 4,180 ID with zero failures or
ignored tests ([literal inventories and transcripts](docs/rust-rewrite/native-core-4186-20261018.json)).
CLI retention, full workspace execution and pinned-suite acceptance remain open.

Intl refresh dependency repair — 2026-10-18: List identity now refreshes before
Duration, which binds List's generated identity and manifest. Six focused controls
pass. Restoring both stale outputs and running the actual13-recipe refresh passes
in10.06s; a repeat passes in9.59s while preserving bytes, modes, inodes and mtimes
for all131704 tracked regular files in the full checkout.
[Exact checks and lossless evidence](docs/rust-rewrite/intl-list-duration-refresh-order-20261018.json).
This is tooling verification; compiler retention and full-suite gates remain open.

Ordinary verification checkpoint — 2026-10-18: all 4,180 selected native tests
and all 581 tooling tests pass. The fresh release CLI completes the exact 230-case
replay at **205 Success, 22 deadline Crash, 3 Bug**: Unicode13 passes all13,
prior197 gives192/5, and remaining20 gives0/17/3. These results fail retention;
the compiler stays on recovery, and full workspace/pinned-suite/task cleanup
remain open. [Lossless replay evidence](docs/rust-rewrite/owned-native-throughput-cli-230-20261018.json)
records all original IDs, deadlines, caches and source identities.

A separate naming repair moves repeated UTF-16 name construction into one R
helper. The original thousand-method native control passes in48.92s. An isolated
cold-template A/B/B/A comparison measures81.3/89.3s before and16.0/15.8s after;
this measures native compilation of one template, not full CLI acceptance.
[Exact diagnostic scope](docs/rust-rewrite/function-name-kernel-diagnostic-20261018.json)
retains both passing controls and the earlier failed diagnostic attempts.
The generated pinned-suite status block is unchanged.

Native verification checkpoint — 2026-10-18: all 4,180 tests pass across
six complete selected Cargo targets: AOT 428, IR 1,599, IR integration 740, Engine 968,
native templates 17 and realm modules 428. Every one of the 4,161 earlier passing
IDs is retained; zero tests fail or are ignored. All 581 tooling tests also pass.
[Literal inventories and lossless evidence](docs/rust-rewrite/native-core-4180-20261018.json)
bind this result to recovery source `cb2904258`; compiler promotion awaits the fresh
230-case CLI replay. Full workspace, pinned suites and task closure remain open.

Batched source identity audit — 2026-10-18: the checker retains its Git
input set, GNU grep expressions, binary handling and historical-proof admission,
while batching content scans and importing provenance checks once. All 22 controls
pass, including binary files, unusual filenames, symlinks and batch boundaries.
On the same frozen full checkout, the candidate reproduced every original finding
in 35.7 seconds; the earlier audit took 1,116.4 seconds at lower priority under
different load. These are observed timings, not a controlled speedup ratio.
Two historical reports now admit only their exact frozen hashes for host-path
normalization. [Literal checks and evidence](docs/rust-rewrite/batched-identity-audit-20261018.json).

Opt-in Rustc code generation — 2026-10-18: the verification compiler wrapper
accepts `LILA_TEST_CODEGEN_JOBS=auto`. It keeps Cargo serial and supplies a private
jobserver bounded by the finite cloud budget, CPU quota and inherited affinity.
All 11 controls pass. Two fresh 1,024-function Cargo fixture builds return the
expected checksum; enabling or disabling the option reuses every existing output
with identical bytes, inodes and timestamps. Lila rebuild timing is still
unmeasured. [Exact fixture evidence](docs/rust-rewrite/bounded-rustc-codegen-20261018.json).

Intl provenance refresh — 2026-10-18: `intl_source_identities.py --write`
checks each producer first and regenerates only stale output, then rechecks all
13 recipes. This preserves unchanged timestamps that Cargo uses for freshness.
Five focused controls pass; an actual refresh preserves the bytes, modes, inodes
and modification times of all 5,858 checked-out tracked files. No Cargo build-time
saving is claimed. [Exact checks and lossless evidence](docs/rust-rewrite/intl-idempotent-refresh-20261018.json).

Verification checkpoint (2026-10-18): the ABI10/code-sharing candidate remains
rejected: the six native inventories contain 4,180 IDs, with 4,159 completed
passes, three failures and 18 unreported Engine IDs after a 900-second stall.
Of 4,161 earlier passing IDs, 4,141 were observed passing in this rejected run.
All 579 tooling tests pass. A separate repair passes 573 isolated tooling tests,
AOT/Engine type checks and exact emitted-byte controls; its native acceptance is
pending. A 256-source opt0 diagnostic reports 12.6% less emission/canonicalization time and
confirms the precompiled runtime loads into both real execution Engine modes.
[Literal results and lossless evidence](docs/rust-rewrite/rejected-owned-native-sharing-20261018.json)
retain the failed scopes. Compiler promotion, full suites and task cleanup stay
open; generated conformance status is unchanged.

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
