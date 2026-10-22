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

Latest [key/argument checkpoint](docs/rust-rewrite/v8-baseline.md#key-and-argument-checkpoint--published-2026-10-22):
source `2cb9fc53d`, **3,572 native passes / zero failed/ignored**, nine compiled
semantic families, unchanged source/linker and seven matching product/test images.
Actual ordinary README scores, slower observations and separate balanced controls
are published. The [final 15,899-case Test262 replay](docs/rust-rewrite/performance-key-arguments-replay-20261022.json)
returns **15,895 Success / 4 Bug / 0 Crash / 0 NotImplemented**, with exactly
the four original failures, using the frozen `2cb9fc53d` compiler after native
drain. Full-suite chart counts remain unchanged. P020 stays partial; P078's
runtime-empty extension is not another hill.

Next coherent work:

1. Finish value-only Map.getOrInsert hash sharing. Outer lookup still hashes
   before its inner put; callback getOrInsertComputed must retain fresh lookup.
   Root-owned `/tmp/scriptw-performance-batch16` has hash-bound emitter,
   harness-reuse, semantic and throughput drafts. Both emitter and Test262
   metadata-only checks pass; product/native/performance acceptance is not done.
   Its worker draft shares only immutable prelude bytes within each 64-request
   session, with fresh Engine/Realm/Store and exact snapshots per request.
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
and all original receipts remain under `/tmp/scriptw-performance-batch14` and
`target/performance-hills-20261021-attribution`. Push verified batches to main.
Native verification keeps two libtest processes, the two-GiB Cargo reserve,
256 MiB scratch per process and existing cache ceilings. Last group samples reach
32 GiB with max-pressure events but zero OOM; this is not JavaScript heap usage.
Stage exact outputs and archive only obsolete/unreferenced bytes after verification.
Detach only actually dirty rustc aliases; preserve fresh dependencies.
The 6m54s product build recompiled 1,165 NativeHost and 914 Portable stencils:
this shared argv change affects many bodies. Do not mistake that for an absent
cache or assume a crate/profile split shares differently configured Rust units.

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
