# Continue Scriptw

Start with the [current inventory](docs/rust-rewrite/current-inventory.md) for
measured results, source identities and outstanding acceptance gates. Use the
[batch workflow](docs/rust-rewrite/batch-workflow.md) for commands and resource
policy. The [maintenance backlog](docs/maintenance-hills.md) tracks cleanup work.

## Current maintenance work

All 100 authorized maintenance hills are complete. The final
[compiler/runtime/Intl record](docs/rust-rewrite/maintenance-compiler-intl-wave-20261021.md)
closes the remaining 39 and links the exact verification evidence. The
[first-wave](docs/rust-rewrite/maintenance-first-wave-20261021.md),
[tooling-wave](docs/rust-rewrite/maintenance-tooling-wave-20261021.md),
[CLI-wave](docs/rust-rewrite/maintenance-cli-wave-20261021.md) and
[conformance-wave](docs/rust-rewrite/maintenance-conformance-wave-20261021.md)
records retain earlier checks and failed attempts.

All 109 all-feature native scopes are covered after two complete test-owner
replays. The final 685-test tooling checkpoint and complete standalone boundary
checks also pass. Default/release and broader acceptance remain open below. Continue using
Luna agents for read-only exploration/review and root-owned implementation. Push
each verified batch to main.

## Next performance work

<!-- scriptw-string-hash-progress:start -->
Current batch awaiting Test262 acceptance: **`97036e5a8`**, cached raw String
hashes and runtime layout 16. Native verification is complete: **4,083 passed /
zero failed / zero ignored** across twelve affected release scopes. The initial
eight stale fixture/trace expectations remain recorded; test-only repair
**`7efa13482`** passes the entire engine-lib and runtime-cache targets, joined
with the ten unchanged passing targets. Eighteen compiled semantic families pass.
The frozen product's **13,104 original Test262 executions are running** with four
workers and original budgets. P023/P112 stay pending and counts below remain the
last fully published checkpoint until the replay and portable evidence validate.

Active driver: `/tmp/scriptw-performance-batch24/repair-and-continue.py`.
Receipts and replay: `target/performance-hills-20261021-attribution/candidate53*`.
The complete new native result uses `candidate53b-native-checkpoint/receipt.json`
for the two repaired scopes and `candidate53-native-checkpoint/receipt.json` for
the other ten; never erase or reclassify the initial eight failures. Finish with
`publish-string-hash.py` and `update-docs.py` under the same batch directory,
validate generated status artifacts, then push the evidence/documentation update.
Product/benchmark/replay identity remains source53; native repair identity is
separate. Next attribution notes are in that directory's `next-batch-notes.md`.
<!-- scriptw-string-hash-progress:end -->

The 160-hill backlog has **22 terminal decisions, seven partial/open outcomes,
131 proposed/unresolved**. Continue Luna read-only exploration/review and root-owned
code. Push each verified batch to main; keep this file/tasks until all acceptance
criteria pass.

Latest accepted product: **`5c63c6597`**. Prepared ordinary definitions close
P020 and pass **3,893 native tests / zero failures / zero ignored** across nine
affected targets and seventeen compiled semantic families. The **7,174 original
affected Test262 executions** give **7,170 Success / four known Bug / zero Crash /
zero NotImplemented**. No new failures appear. Runtime payload identities match.
[Measurements and raw evidence](docs/rust-rewrite/v8-baseline.md#prepared-property-definition-checkpoint--published-2026-10-23)
retain slower screens and the cold-start/image/RSS tradeoffs: roughly 7.1% faster
long-key insertion, 26.5-ms slower cold startup and 0.85-MiB larger CLI.
The README V8 table remains the separate source50 ordinary comparison.

**The fresh full Test262 campaign is complete and published (2026-10-23):**
**102,990 Success / 4 known Bug / 0 Crash / 0 NotImplemented**, all 748 nodes
and 102,994 original executions. The four failures are exactly the retained
upstream disagreements; no new failures appeared. The canonical generation and
README graph bind source50 (`b9ecfe64b`) directly. The graph partitions 429 native-host
successes into the unchanged standalone-Wasm out-of-scope category, without
hiding failures. [Full evidence and reproduction](docs/rust-rewrite/test262-current.md).

The isolated `/workspace/scriptw-full-suite-20261022` checkout stayed at
`4aac260e7`; its compiled product is source `b9ecfe64b`. The same campaign
resumed at a read-only barrier with 16 sequential nodes per invocation, keeping
four case workers and all original budgets. Final publication exit is zero;
sampled whole-cgroup peak is 34,264,653,824 bytes, with no OOM events. This includes
tmpfs/file pages, not just process or JS memory. Original pause/exit130 evidence
is retained beside the successful continuation. All aggregate/leaf snapshots
and completion records are committed portably; no live watcher needs restarting.

Next coherent work:

The additive `scripts/check-source-guards.py` covers both structural targets:
277 source-only checks in about ten seconds, with nine tooling controls. Eight
typed modules remain in full Cargo; all 166 builtin and 151 language tests pass.
Tracked Cargo config is restored. Do not use `/dev/shm` for persistent history:
twenty relocated historical archives are absent in this environment. Current
products and raw observations remain verified; missing old binaries are not
restorable from the recorded paths. The availability audit is retained in the
[hot-read checkpoint](docs/rust-rewrite/performance-hot-reads-20261022.json).

Fresh graph generation now uses the portable full-run evidence. Verify with
`python3 -B scripts/generate-test-graph.py --check`. Only refresh evidence when a
new complete canonical publication exists; never mix an old failure replay
into fresh full counts. The original reconciliation and native baseline remain
immutable historical evidence. Keep this file/tasks and unresolved hill counts.

The NativeHost compressed-storage experiment is **rejected**, with
[raw evidence](docs/rust-rewrite/performance-nativehost-storage-rejected-20261022.json).
Source `f9f4efa9e` built in 1m01s and reduced the CLI by 59,296,072
bytes and runner by 59,296,136 bytes, but the 56-process startup ABBA gives cold 1,059.862→1,129.665ms and cached
775.355→830.447ms, with child peak RSS about 9–16 MiB higher. All 21 Portable
programs, runtime keys and outputs match accepted48; all eight runtime files and
both exact decoded native digests are verified. A cached-dependency bulk-decoder
screen overlaps the streaming costs. No native/Test262 acceptance ran for this
rejected source. Its code is restored to accepted48; existing Portable storage
compression remains. Do not revive the draft without a different measured
hypothesis. Source52 is the next direct baseline; source48 raw products had been
archived and that shared-memory archive is now unavailable. Its raw measurements
and identities remain in the committed evidence.

1. Evaluate P023/P112 cached raw string hashes using the existing repeated-key
   baseline. A lazy u64 costs eight payload bytes plus alignment and requires
   layout15→16 with host constructors/admission/tests. Measure unhashed small-string
   retention as well as repeated long-key wins; source inspection is not closure.
2. Promote the standalone emitter only with exact cached-dependency identity
   checks. Linking 0.500s and fixture validation 4.436s exclude library compilation
   and have no Engine/product execution authority. Full R cache hits keep the
   product rebuild at 3m42s; retain current dependencies and actual cache keys.
3. Finish broader default/release, retention, original full-suite and remaining
   performance acceptance. One fresh full family is now published; the broader
   release criteria remain open. Use exact original IDs and source/linker fences.

Keep the default CLI chunk count: the four-chunk 858-body observation passes but
does not improve the measured two-chunk run. Two live libtests, original worker
budgets, 256MiB scratch per process, two-GiB Cargo reserve and retained-module
cache ceilings stay. Group memory includes tmpfs/file pages, not live JS heap.
Use actual filesystem free bytes; preserve and archive before releasing staging.
Detach only actually dirty rustc aliases. Current cloud tools are under
`/tmp/scriptw-performance-batch23`; raw results under
`target/performance-hills-20261021-attribution`. Historical artifacts are exact
restore-only archives with inventories. A frozen replay may continue while later
source work proceeds, but its compiler, fixtures, budgets and results stay fixed.

## Decisions to retain

- Keep the four original Test262 disagreements as ordinary failures with their
  README explanations and [upstream tracking](test262/errata/proposals/upstream-status-20261021.md).
  Hold local fixture/specification proposals; no upstream posting is authorized.
- Wasm is the only product target. C/native output and a REPL are excluded.
- Canonical Test262 publication stays at its complete measured baseline until
  a new full original-suite campaign authorizes replacement.
- Preserve original failed runs, compiler identities and immutable evidence.
  A passing diagnostic retry does not replace a failed receipt.
- Keep this file and tasks until every acceptance criterion passes.

## Broader continuation

1. Complete current-source default and optimized release acceptance. The earlier
   maintenance all-feature checkpoint is complete; retain the stopped pre-rename
   campaign and its original-budget timeout as historical evidence.
2. Finish retention, real shard/resume, debug/optimized differential and
   robustness, host reproducibility and calibrated performance gates. The
   formerly timed-out native control now passes at its original budget; its
   historical failure has not been erased or assigned an unproven cause.
3. Complete the required original Test262 families and native publication. Keep
   unresolved upstream failures visible; do not claim literal green acceptance.

Exact outstanding gates and the missing historical replay input are recorded in
[the inventory](docs/rust-rewrite/current-inventory.md) and
[acceptance criteria](docs/rust-rewrite/conformance-acceptance.md).
