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

The 160-hill backlog has **21 terminal decisions, eight partial/open outcomes,
131 proposed/unresolved**. Continue Luna read-only exploration/review and root-owned
code. Push each verified batch to main; keep this file/tasks until all acceptance
criteria pass.

Latest accepted product: **`b9ecfe64b`**. Its hot-read changes pass **3,742
native tests / zero failures / zero ignored** across eight affected targets and
seventeen compiled semantic families. Seven independent product/test runtime
identities match. [Performance measurements](docs/rust-rewrite/v8-baseline.md#hot-read-checkpoint--published-2026-10-22)
retain the slower initial controls and overlapping rechecks. The README uses
this product's separate ordinary Wasmtime/V8 comparison.

**The fresh full Test262 campaign is complete and published (2026-10-23):**
**102,990 Success / 4 known Bug / 0 Crash / 0 NotImplemented**, all 748 nodes
and 102,994 original executions. The four failures are exactly the retained
upstream disagreements; no new failures appeared. The canonical generation and
README graph now bind this product directly. The graph partitions 429 native-host
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

The additive `scripts/check-source-guards.py` now passes 143 source-only checks
in about five seconds, with six tooling controls; full Cargo166 remains required.
Tracked Cargo config is restored. Do not use `/dev/shm` for persistent history:
twenty relocated historical archives are absent in this environment. Current
products and raw observations remain verified; missing old binaries are not
restorable from the recorded paths. Availability audit is in the new bundle.

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
hypothesis. Source50 is the next direct baseline; source48 raw products had been
archived and that shared-memory archive is now unavailable. Its raw measurements
and identities remain in the committed evidence.

1. Close remaining P020 ordinary define-own duplicate lookup/append hashing;
   retain the fresh-empty no-hash shortcut for rejected definitions. P025/P047
   index classification and P097/P098 character/iterator changes already ship
   in the fully rerun source50 checkpoint; pursue remaining work only with a
   new measured hypothesis. Preserve
   proxy/accessor/prototype/hole/UTF16/effect behavior, and measure focused controls
   plus ordinary object/retained-row regressions before retaining candidates.
2. Evaluate P023/P112 cached raw string hashes using the existing repeated-key
   baseline. A lazy u64 costs eight payload bytes plus alignment and requires
   layout15→16 with host constructors/admission/tests. Measure unhashed small-string
   retention as well as repeated long-key wins; source inspection is not closure.
3. Promote the standalone emitter only with exact cached-dependency identity
   checks. Linking 0.500s and fixture validation 4.436s exclude library compilation
   and have no Engine/product execution authority. Full R cache hits keep the
   product rebuild at 3m42s; retain current dependencies and actual cache keys.
4. Finish broader default/release, retention, original full-suite and remaining
   performance acceptance. One fresh full family is now published; the broader
   release criteria remain open. Use exact original IDs and source/linker fences.

Keep the default CLI chunk count: the four-chunk 858-body observation passes but
does not improve the measured two-chunk run. Two live libtests, original worker
budgets, 256MiB scratch per process, two-GiB Cargo reserve and retained-module
cache ceilings stay. Group memory includes tmpfs/file pages, not live JS heap.
Use actual filesystem free bytes; preserve and archive before releasing staging.
Detach only actually dirty rustc aliases. Current cloud tools are under
`/tmp/scriptw-performance-batch20`; raw results under
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

1. Complete current-source default and optimized release acceptance. The new
   all-feature native checkpoint is complete; retain the stopped pre-rename
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

Candidate47 compiled all12semantic families,21-case screen/ABBA and ordinary
V8 observations at bf681edcb; genuine direct/alias and integer gains are retained
with slower controls and targeted ABBAABBA rechecks. Native preflight ran no
test bodies: its optional must_use annotation changed emit.rs, a pinned Intl
provenance input. Candidate48 removes only that annotation; all15original
identity checks pass without regeneration. Product48 correctness and21-case
P/R byte-equivalence checks will bridge the separately labeled47timings to
current-source48native/Test262 acceptance. No47timing is relabeled48. Tools
for this continuation are under /tmp/scriptw-performance-batch20.
