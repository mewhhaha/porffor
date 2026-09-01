# Array.fromAsync iterator-result property domain

Status: implemented, independently reviewed and focused-verified for the four
iterator-result continuations, 2026-08-27.

## Closed domain

`ArrayFromAsyncIteratorResultProperty` has exactly two inhabitants: `Done` and
`Value`. Its sole projection exhaustively maps them to the observable property
keys `"done"` and `"value"`. The domain derives no capabilities and cannot be
duplicated, collapsed through equality or inequality, or replaced by a Boolean
default.

The shared reader accepts only this domain rather than a string. Four
continuations each read `Done` before `Value`, producing the exact census of
eight typed reads and 11 total type mentions. Each selection is consumed by
the shared reader's sole key projection. Adding another iterator-result field
therefore requires an explicit key mapping, and no caller can compile with an
unrelated or misspelled property name.

## Evidence

`array_from_async_iterator_result_property_domain_structure.rs` pins the exact
domain, exhaustive mapping, typed reader signature, four continuation owners,
eight-call census, full-body fingerprints and read order. The focused structure
target passes `4/4`.
Existing async-value and iterator-closing CLI fixtures exercise both properties
without a dedicated new fixture; their two exact witnesses pass `2/2`.
The coordinated workspace checkpoint passes `cargo fmt --all -- --check`,
`cargo xc`, `git diff --check`, the module-boundary check and the task-plan
check; the compile retains the repository's existing warnings.

```sh
cargo test -p lila-aot-wasm --test array_from_async_iterator_result_property_domain_structure --quiet
cargo test -p lila-cli --test cli array::run_wasm_backend_awaits_array_from_async_array_like_values_and_mapper_results -- --exact --test-threads=1
cargo test -p lila-cli --test cli array::run_wasm_backend_closes_array_from_async_iterators_and_preserves_original_errors -- --exact --test-threads=1
```

This source-only capability closure changes no property access order, abrupt
route, iterator closing, Promise settlement, Realm authority or published
conformance count. The coordinated 679-dump semantic golden passes `2/2` in
800.46 seconds; no retained structural change is attributed to this domain.
Broad Test262 verification remains deferred.
