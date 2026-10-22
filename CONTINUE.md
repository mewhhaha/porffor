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
**20 terminal decisions, seven partial/open outcomes, 133 proposed/unresolved**.
Rejected/superseded experiments are decisions, not shipped speedups.

The [latest checkpoint](docs/rust-rewrite/v8-baseline.md#replacement-assembly-checkpoint--published-2026-10-22)
keeps P107's eight-root collector at `1e2978361`: 3,569 scoped native passes,
six compiled families, exact overflow, matching product/test images and
406/410 affected Test262 successes with exactly the four tracked failures.
P004's fixed-overhead controls are complete. README ordinary scores are current;
slower observations, rejected variants and source-bound raw evidence remain.
A fresh full-suite/all-feature campaign is still separate work.

Next coherent batch:

1. P020: carry a prepared hash tied to the rooted key through Map/Set insertion
   and ordinary-property append. Recompute buckets after growth; retain equality,
   collision checks and callback/reentrant lookup boundaries.
2. Extend P078 to runtime-empty internal argument lists. Mutable JS arguments,
   rest and Array storage stay fresh; preserve iterator/length coercion effects.
3. Measure long fresh/hit/numeric collection/property cases and empty/two-argument
   call controls. Root-owned drafts pass metadata-only Rust and Node checks;
   product/native/Test262 acceptance is pending. This is not another P078 hill.
4. Investigate four sequential CLI chunks with only two concurrent processes.
   Preserve exact inventory/outcome unions, preflight lists, original stalls/cache
   limits and target-idle exclusivity. Long-lived CLI test PIDs retain several GiB.

Cloud drafts are in `/tmp/scriptw-performance-batch11`, with exact before/draft
hashes; apply only to matching input bytes. Use the
[baseline workflow](docs/rust-rewrite/v8-baseline.md), freeze the coherent batch
before product measurement, and push verified progress to main.
Native verification retains the two-GiB Cargo reserve and 256 MiB scratch per
process. Stage outputs with verified byte preservation; detach only actually
dirty rustc aliases, preserving fresh dependency artifacts. Product builds spend
about 174 of 276 seconds on two differently configured Rust emitter units;
each image already reuses 2,181 function stencils and rebuilds three. Do not
reimplement that cache or assume a crate/profile split will share those units.

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
