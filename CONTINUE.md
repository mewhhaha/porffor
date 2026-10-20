# Continue Scriptw — current inventory, 2026-10-20

Start with [the current project inventory](docs/rust-rewrite/current-inventory.md)
and its [source-bound evidence](docs/rust-rewrite/current-inventory-20261020.json).
The durable [acceptance criteria](docs/rust-rewrite/conformance-acceptance.md)
and [ownership registry](docs/rust-rewrite/conformance-ownership.md) remain authoritative.

## Latest scriptw release checkpoint

- Rename source `c5053c9d4` builds the ordinary opt3/default-feature CLI and
  Wasmtime benchmark runner; the product dependency graph has no oracle.
- All 634 tooling tests pass with no skips; 19 focused native codec/cache
  tests and three Intl outputs pass. The same three React SSR outputs match
  Node in this ordinary release build.
- [Local cold/hot/execution versus V8 measurements](docs/rust-rewrite/v8-baseline.md)
  are complete with [source-bound raw evidence](docs/rust-rewrite/v8-baseline-20261020.json).
  This is not calibrated T25 or full-suite acceptance.
- The [fresh full Test262 baseline](docs/rust-rewrite/test262-current.md) measured all 102,994 current-pin executions; the README chart uses its exact native publication.
  Native workspace and release closure remain open; keep this file/tasks.

## Pre-rename verified and published

- Compiler source: `cbf7cc71f`; checkpoint/scheduling batch published at `5fa2e8432`.
- 14 source audits, two shell negative controls and all 624 tooling tests pass.
- All 12 ordinary library scopes pass 3,865 tests. Engine Intl299 also passes.
- The current debug CLI runs six fresh examples through Wasm-AOT and emits a
  real program/runtime Wasm pair. Wasm is the sole output target; C/native
  output and a REPL are excluded goals, not unfinished work or release gates.
- A separate [React SSR probe](docs/rust-rewrite/react-ssr.md) passes three
  exact Node/HTML comparisons through Wasm-AOT using React 19.3.1's unmodified
  synchronous renderer; the public browser entry fails. Its emitted Wasm pair
  is hash-bound in the evidence. This adds no broad-suite or task closure.
- Previous source `78593daf2` passed full AF9654 plus 11 docs. Those outcomes
  belong to that source and are not current-source completion.

## Current stop and failure

The saved AF run completed 49/113 scopes: **7,560 pass, one fail, zero ignored**.
There are 64 uncompleted scopes, including the remaining 2,094 expected native
bodies and documentation tests. Default, final Cargo checks, 230 retention,
real mechanics and full campaigns remain unstarted at this frozen source.
The separate new ordinary release build above does not close those gates.

Package names here use their current spelling; literal original execution IDs
remain in the dated evidence.
Failure: `scriptw-test262:differential_generated_campaign` /
`builtin_scenarios_use_actual_selected_workers_for_all_six_stateful_families`.
The `builtin-stateful-v1` seed-0 array-buffer Wasm worker timed out at the
original 5,000 ms deadline. The oracle completed. Underlying cause is unresolved;
preserve the failed executable, transcript, aggregate and original request.
Do not replace it with a passing diagnostic replay.

The previous machine stopped before cohort011 because of its disk reserve.
After cloud restoration its old coordinator processes are absent, while the
current disk observation has ample headroom. The old frozen-source verification
coordinators remain stopped. Existing incomplete output directories are evidence,
not restart targets.

## Continue in this order

1. Reconcile the original receipts and exact inventories at frozen compiler
   source `cbf7cc71f`. Re-admit only the 64 uncompleted AF scopes in a fresh
   continuation namespace; carry completed bodies through source/artifact proof.
2. Diagnose and repair the retained original-budget worker timeout with a
   general fix. Batch source changes before compilation, preserve all prior IDs,
   then verify affected scopes and the required full checkpoint.
3. Complete current-source AF/default and Cargo checks; ordinary opt3/non-oracle
   release acceptance beyond the completed build, plus original230/prior219
   retention; real serial/shard/resume;
   complete debug and optimized T25 corpus/campaigns and idle measurements.
4. Finish two fresh complete current-pin Wasm-AOT families, real semantic
   failure burn-down, backlog generation twice, canonical publisher output,
   historical5365 replay, host reproducibility and calibrated performance.
5. Delete this file and `tasks/` only when every acceptance criterion passes.

Use the repository's cloud resource wrapper and original worker/deadline/cache
policies. Recorded task status remains T00/T27/T28/T29 complete, T01–T25 in
progress, T26 blocked. The detailed inventory distinguishes implemented
capabilities from verification and release acceptance.

Earlier continuation notes remain in Git history. The [full prior AF proof](docs/rust-rewrite/full-af-checkpoint-20261020.json)
and [focused current batch proof](docs/rust-rewrite/checkpoint-batch-focused-20261020.json)
remain unchanged.
