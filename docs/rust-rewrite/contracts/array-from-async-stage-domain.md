# Array.fromAsync continuation-stage domain

Status: implemented and verified through the Batch AM focused checkpoint.

## Closed stage authority

`ArrayFromAsyncStage` is the private, capability-free Rust authority for the
six continuation states admitted by `Array.fromAsync`: `InputValue`,
`MappedValue`, `AsyncIteratorResult`, `SyncIteratorDoneValue`,
`AsyncCloseResult` and `SyncCloseValue`. One exhaustive borrowed `code`
projection maps them to the existing `0` through `5` runtime heap wire values.
The stage offset, stage local and all load, store and comparison instruction
order remain unchanged.

The six algorithm owners contain exactly thirteen stage producers and nine
comparisons. Twelve producers write the state cell and the thirteenth updates
the paired stage local before the same `InputValue` state write. Every
comparison reads that shared local. The former six raw stage constants are
gone, so a Rust emitter site cannot compile with an arbitrary numeric stage or
transpose independently maintained names and values.

The runtime state cell remains an integer wire value. This invariant closes
the Rust producer and comparison vocabulary; it does not add runtime
corruption recovery or change Array.fromAsync iteration, awaiting, iterator
closing, rejection or result publication.

## Structural evidence

`array_from_async_stage_domain_structure.rs` recursively pins the exact private
six-variant domain, exhaustive borrowed mapping, absence of clone, copy,
debug, equality, default, hash and ordering capabilities, 24 product type
mentions, 22 typed projections, the thirteen stage producers, nine
comparisons, and all 15 unchanged stage-offset mentions. Erasing only the
typed stage vocabulary recovers the frozen whitespace-normalized six-owner
source: 41,030 bytes with FNV-1a `0xd722936e349517a9`.

The neighboring Batch AL guard additionally erases the stage vocabulary before
checking its existing 51,969-byte source-mode fingerprint. That maintenance
does not change the AL producer, comparison or runtime evidence.

At the Batch AM checkpoint, `cargo xc` is green, the new stage and neighboring
source-mode structure targets each pass `4/4`, the three exact CLI controls
pass `3/3`, and the pinned Test262 leaves pass all `6/6` Wasm-AOT variants with
every failure bucket at zero:

```sh
cargo test -p lila-aot-wasm --test array_from_async_stage_domain_structure --quiet
cargo test -p lila-cli --test cli array::run_wasm_backend_awaits_array_from_async_array_like_values_and_mapper_results -- --exact --test-threads=1
cargo test -p lila-cli --test cli array::run_wasm_backend_preserves_async_iterator_values_and_awaits_mapper_results_once -- --exact --test-threads=1
cargo test -p lila-cli --test cli array::run_wasm_backend_closes_array_from_async_iterators_and_preserves_original_errors -- --exact --test-threads=1
./target/debug/lila test262 run built-ins/Array/fromAsync/mapfn-result-awaited-once-per-iteration.js --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila test262 run built-ins/Array/fromAsync/mapfn-async-throws-close-async-iterator.js --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila test262 run built-ins/Array/fromAsync/mapfn-async-throws-close-sync-iterator.js --execution-backend wasm-aot --timeout-ms 180000 --threads 1
```

No semantic golden, broad Array.fromAsync tree, Test262 rewrite retirement or
published-status change is claimed by Batch AM.
