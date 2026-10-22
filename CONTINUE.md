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

Latest [numeric/collection checkpoint](docs/rust-rewrite/v8-baseline.md#numeric-and-collection-checkpoint--published-2026-10-22):
acceptance `37ab9fa44`, **3,575 native passed/zero failed or ignored**, twelve compiled
semantic families, dynamic-call controls and seven identical product/test images.
The exact 3,235 original Test262 replay gives **3,231 Success / 4 known Bug / 0 Crash / 0 NotImplemented**.
Timing source stays `bf681edcb`; source 48 only removes a Rust annotation. All 21 P
modules match exactly; runtime keys/native bytes match, while package and manifest
source provenance is validated explicitly. Original failed attempts remain.
P081's numeric Math.imul extension is verified, with generic fallbacks. P020 spans
callbacks, grouping, Set kernels and deletion but stays partial. The README uses
the measured source 47 ordinary report; initial slower paired controls and their targeted
rechecks remain. Wider full-suite/default/all-feature acceptance stays open.

Next coherent work:

The hot-read batch at **`b9ecfe64b`** has completed affected native verification;
**fresh full Test262 acceptance remains pending**.
It replaces the Array-index classifier's floating-point parse/string roundtrip
with bounded ASCII decimal recognition (exact ArrayIndex domain), classifies
concrete Array indices before named-header probes, builds charAt/at's already-read
unit directly while preserving full-span reuse, and copies a string iterator's
validated one/two UTF-16 units without a per-unit loop. Typed-array numeric-key
semantics remain separate. New semantic matrices cover canonical/noncanonical
keys, String exotic descriptors, coercion/throws, UTF-16 and iterator freshness;
seven checked Node-compatible workloads include named/nonindex regression controls.
Seven runtime targets pass 3,576 tests; the complete structural target passes
166 after a stale trim-policy visibility assertion is repaired, with product
inputs unchanged. Seventeen compiled semantic families and independent runtime
identities pass. Paired indexed/character/ASCII-iterator medians improve, startup
is roughly flat, and slower initial cohorts plus integer/Unicode rechecks remain.
[Results and portable evidence](docs/rust-rewrite/performance-hot-reads-20261022.json).
The README now uses source50's separate ordinary V8 comparison. Hill counts
remain 21 terminal / eight partial / 131 unresolved until broader acceptance.

A canonical full102994-execution campaign is running using
`scripts/publish-real-status-low-ram.sh`, four case workers and original budgets.
Use the frozen source50 product from an isolated checkout at
`/workspace/scriptw-full-suite-20261022`; record its identity separately from
that later test/doc/tool-only checkout. Keep that checkout's HEAD and source
inputs unchanged for the campaign so implementation can continue independently
in the development worktree. The same family resumed at a read-only progress
barrier with MAX_MATRIX_NODES=16: nodes remain sequential and individually
checkpointed, with the same worker and memory limits. Original and resumed
logs/memory samples are separate `candidate50-full-suite-*` target artifacts;
`/tmp/scriptw-performance-batch22/resume-full-suite16.py` owns the live watcher.
After it finishes, bind the graph's 429-case scope projection directly
to that aggregate and its leaves, without replaying the suite. Preserve all four
upstream Bugs as ordinary failures. Commands/watchers are in
`/tmp/scriptw-performance-batch22`; raw results under the attribution target.

The additive `scripts/check-source-guards.py` now passes 143 source-only checks
in about five seconds, with six tooling controls; full Cargo166 remains required.
Tracked Cargo config is restored. Do not use `/dev/shm` for persistent history:
twenty relocated historical archives are absent in this environment. Current
products and raw observations remain verified; missing old binaries are not
restorable from the recorded paths. Availability audit is in the new bundle.

Fresh graph support is implemented and tooling-verified: after importing the
completed canonical publication from the frozen checkout, run
`python3 -B scripts/generate-test-graph.py --refresh-evidence --full-evidence`,
then `--check`. It retains the native aggregate and all leaves in a portable
record and checks their exact census; no second Test262 run or old failure replay
is mixed into fresh counts. Twenty-two graph evidence/renderer controls pass.
Do not use the full-evidence option against the old publication while the new
campaign is still running. Keep this file/tasks and unresolved hill counts.

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
   retain the fresh-empty no-hash shortcut for rejected definitions. Also inspect
   P025/P047 concrete Array index reads before unnecessary named-header probes,
   P097 charAt/at single-unit extraction and P098 iterator output copy. Preserve
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
   performance acceptance. Use exact original IDs and source/linker fences.

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
