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

The [performance backlog](docs/performance-hills.md) contains 160 hills:
**20 terminal decisions, eight partial/open outcomes, 132 proposed/unresolved**.
Rejected/superseded experiments are decisions, not shipped speedups.

Latest [Map checkpoint](docs/rust-rewrite/v8-baseline.md#map-insertion-checkpoint--published-2026-10-22):
source `e78c9e072`, **4,035 native passes / zero failed/ignored**, ten compiled
semantic families, unchanged source/linker and seven matching product/test images.
The exact disjoint one-plus-nine native union preserves disk-reserve deferrals;
no passed body was repeated. The **1,173 original Map/Set+four-failure replay**
returns **1,169 Success / 4 Bug / 0 Crash / 0 NotImplemented**, exactly the original
tracked failures. The wider preceding 15,899-case replay remains bound to
`2cb9fc53d`; neither replaces a full-suite campaign or changes the chart.
Value-only getOrInsert now shares its prepared hash. P020 stays partial for the
callback route and other internal collection paths. Worker prelude sources are
shared within 64 requests; JS state,
execution and snapshots remain independent. P154's Fast/SizeOptimized ABBA is
mixed, so default Fast stays. Ordinary current README scores and slower rows
are published separately from paired controls.

Next coherent work:

1. Verify the root-owned **P020/P081 prepared-key and numeric Math.imul draft**,
   outside the checkout
   under `/tmp/scriptw-performance-batch17/source`. Its metadata-only Rust check
   passes. Expanded stock-Node collection controls, numeric controls and actual
   distinct-realm Node oracles pass. The computed insertion proposal still uses
   repository assertions because stock Node lacks it. No product
   build/compiled fixture/measurement/native/Test262 acceptance is claimed yet.
   Exact two ordinary Number arguments and Continue-only calls can bypass argv
   after concrete FunctionObject, immutable StandardBuiltin/MathImul and actual
   current-Realm identity proof. Evaluate receiver/property/arguments once in
   their original order; wrappers, coercions, patched methods, cross-realm and
   other arities retain generic dispatch. Reuse existing modulo conversion.
   Prepared hashes also cover computed insertion, groupBy, cross-owner Set
   operations and lookup/unlink deletion. Keep current-table probing after user
   effects and the immutable key/identity hash invariant. These paths remain
   unaccepted until measurement and broad checks. Keep the draft separate until
   applied as a complete source/test/docs batch;
   freeze actual producers, run semantic controls and balanced performance,
   then native acceptance and original Math/call/proxy/realm Test262 scopes.
2. Keep the default CLI chunk count. The [four-chunk observation](docs/rust-rewrite/verification-cli-queue-observation-20261022.json)
   passes all 858 bodies but takes 783.161s versus the earlier two-chunk 757.216s.
   Sampled group peaks are 31.45 versus 32.00 GiB; individual native RSS maxima
   4.77 versus 5.26 GiB. Cache/file-page state differs, so this is not a causal
   memory/speed comparison. Original bodies, stalls, CPU pool and cache ceilings
   match. Opt-in flag stays available; default stays unchanged.
   The exact native CLI remains held at
   `target/performance-hills-20261021-attribution/candidate43-held-cli-native`.
3. Inspect the completed frozen43 repeated-key baseline in
   `target/performance-hills-20261021-attribution/candidate43-repeated-key-baseline`.
   Its 13 checked cases include nine stock-Node-compatible repeated lookups and
   four Wasmtime-only getOrInsert controls; stock Node lacks those proposal APIs.
   This is whole-invocation throughput, not isolated hash timing. Before P023/P112,
   one shared hash helper and one StringConstruction publication point make a
   raw cached u64 possible, but it costs eight payload bytes per String plus
   engine alignment and requires GC layout version 15→16/admission rebuilds.
   Compiler-only static facts avoid that ABI but need new provenance/dispatch.
4. Complete broader default/release acceptance and remaining performance hills. Use source fences and exact original IDs.

A frozen replay may continue while later source work proceeds; do not change
its compiler, pinned fixtures, workers, deadlines or saved results. Cloud tools
and original receipts remain under `/tmp/scriptw-performance-batch14`,
`/tmp/scriptw-performance-batch16` and
`target/performance-hills-20261021-attribution`. Push verified batches to main.
Native verification keeps two libtest processes, the two-GiB Cargo reserve,
256 MiB scratch per process and existing cache ceilings. Last group samples reach
32 GiB with max-pressure events but zero OOM; this is not JavaScript heap usage.
Stage exact outputs and archive only obsolete/unreferenced bytes after verification.
Detach only actually dirty rustc aliases; preserve fresh dependencies.
The latest product build takes 6m12s and again reports 1,165 NativeHost/914 Portable
stencil misses despite a small R byte change. Shared build/test function-cache
contention is a hypothesis, not a proven eviction cause. Inspect resolved cache
limits, exact keys and prune events before changing budgets. A crate/profile
split does not automatically share differently configured Rust units.

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
