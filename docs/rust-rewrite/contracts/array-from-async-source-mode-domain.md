# Array.fromAsync source-mode domain

Status: implemented and verified through the Batch AL focused checkpoint.

## Closed wire authority

`ArrayFromAsyncSourceMode` is the private, capability-free Rust authority for
the three source routes admitted by `Array.fromAsync`: `ArrayLike`,
`AsyncIterator` and `SyncIterator`. Its one borrowed exhaustive `code`
projection maps those variants to the existing `0`, `1` and `2` runtime heap
wire values. The state offset, Wasm local and all load/store instruction order
remain unchanged.

The source has exactly three semantic producers: the array-like state
initializer and the async-iterator and sync-iterator assignments to the shared
iterator-mode local. Seven callback owners contain exactly eight comparisons:
four against ArrayLike, three against AsyncIterator and one against
SyncIterator. No raw Array.fromAsync mode constant remains, so a Rust emitter
site cannot compile with an arbitrary numeric mode or transpose two named
routes by editing their independent values.

The runtime state cell necessarily remains an integer wire value. This
invariant closes the Rust producer and comparison vocabulary; it does not add a
runtime corruption check or change Array.fromAsync iteration, awaiting,
closing, rejection or publication semantics.

## Structural evidence

`array_from_async_source_mode_structure.rs` pins the exact private domain,
absence of clone, copy, debug, equality, default, hash and ordering
capabilities, 13 type mentions, 11 borrowed code projections, the three
semantic producers, eight comparisons, and the unchanged iterator-local state
handoff. Replacing only the typed projections with their former constant names
recovers the frozen whitespace-normalized producer/consumer bodies: 51,969
bytes with FNV-1a `0x18bf07d71957d97f`.

The implementation-time source gates are recorded in T16. At the Batch AL
checkpoint, `cargo xc` is green, the structure target passes `4/4`, the two
exact CLI controls pass `2/2`, and the three pinned Test262 leaves pass all
`6/6` Wasm-AOT variants with every failure bucket at zero:

```sh
cargo test -p lila-aot-wasm --test array_from_async_source_mode_structure --quiet
cargo test -p lila-cli --test cli array::run_wasm_backend_awaits_array_from_async_array_like_values_and_mapper_results -- --exact --test-threads=1
cargo test -p lila-cli --test cli array::run_wasm_backend_closes_array_from_async_iterators_and_preserves_original_errors -- --exact --test-threads=1
./target/debug/lila test262 run built-ins/Array/fromAsync/non-iterable-input.js --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila test262 run built-ins/Array/fromAsync/sync-iterable-input.js --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila test262 run built-ins/Array/fromAsync/async-iterable-input.js --execution-backend wasm-aot --timeout-ms 180000 --threads 1
```

No semantic golden, broad Array.fromAsync tree, rewrite retirement or
published-status change is claimed by Batch AL.
