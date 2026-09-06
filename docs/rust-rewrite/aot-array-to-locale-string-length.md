# Observable Array toLocaleString length

Owner: T16. Baseline: `abdcf56f1bd88d5debbb1d8c291f2e7213f77371`.

## Implementation boundary

The generic `Array.prototype.toLocaleString` entry now delegates its receiver
conversion and length to `emit_array_like_length_snapshot`, the same operation
used by flatMap and shared map/filter/every/some iteration. The old
callback-oriented helper name is replaced at all callers and ownership guards;
its implementation is otherwise unchanged. No parallel ToLength policy is added.

The operation performs current-function-Realm ToObject, one ordinary `Get` of
`length`, abrupt-completion propagation, then ToLength. Its normalized integer
bound survives the entire element loop. In particular, Array and arguments
storage extent and a TypedArray's private buffer witness cannot bypass an own
or inherited `length` property. Accessor receiver identity, numeric conversion
side effects, thrown values, fractional lengths and nullish rejection remain
observable through their existing shared operations.

After successful length conversion, the generic entry classifies a TypedArray
only to select live indexed reads. Length acquisition may already have resized
or detached the buffer. The returned length still controls the loop, while each
later unavailable index contributes the existing undefined/nullish result.

The direct `%TypedArray%.prototype.toLocaleString` entry is intentionally
unchanged: receiver-brand validation and one `ValidatedMethodEntry` witness
supply private length and reject initially detached or out-of-bounds views.
An own throwing `length` getter is ignored by that direct method, not by the
borrowed Array method. No method-entry validation is inserted into the loop.

Normative references: [Array.prototype.toLocaleString](https://tc39.es/ecma262/#sec-array.prototype.tolocalestring),
[LengthOfArrayLike](https://tc39.es/ecma262/#sec-lengthofarraylike),
[ToLength](https://tc39.es/ecma262/#sec-tolength), and
[%TypedArray%.prototype.toLocaleString](https://tc39.es/ecma262/#sec-%typedarray%.prototype.tolocalestring).

## Regression and CI contract

`crates/lila-engine/tests/aot_array_to_locale_string_length.rs` contains 19
explicit `ExecutionBackend::WasmAot` regression programs. They cover Number and
BigInt TypedArray overrides, inherited and own accessors, arguments redefinition
and deletion, exact length/coercion/index/call ordering, abrupt propagation,
ToLength inputs, large bounds through `2^32 - 1`, Proxy Get without HasProperty,
length snapshots with live later elements, resize/detach during acquisition,
out-of-bounds views, unchanged direct-method behavior, primitive boxing, and
Function receivers. The large-bound test throws at index zero; it never attempts
a multi-billion-element allocation or iteration. Existing callback regressions
retain the shared ToLength owner's larger-number coverage.

The updated TypedArray ownership target retains its direct validation, wrapper,
dispatch, live-read, temporary-lifetime and CLI-fixture guards. Its generic arm
now requires observable length and rejects the retired private witness/storage
paths. A separate test pins the shared helper's exact operation order.

Array conformance CI runs every new engine regression through the existing
complete-inventory runner, rejects empty/ignored/missing/failing/timed-out
executions, runs the updated and adjacent ownership targets, retains the CLI
fixture, and executes the complete pinned real Array toLocaleString subtree.
No Test262 source, prelude, materializer, suite pin, expected failure, exclusion,
or generated aggregate is changed.

```sh
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked -p lila-aot-wasm --test typed_array_to_locale_string_witness_structure --test to_locale_string_invocation_structure --test array_callback_iteration_structure --test array_flat_map_algorithm_owner_structure --test array_flat_map_typed_array_witness_structure --test array_map_algorithm_owner_structure --test array_filter_algorithm_owner_structure --test array_every_algorithm_owner_structure --test array_some_algorithm_owner_structure
python3 scripts/run_engine_regression_inventory.py aot_array_to_locale_string_length --output-dir /tmp/locale-length-engine
cargo test --locked -p lila-cli --test cli -- array::run_wasm_backend_succeeds_for_supported_array_to_locale_string_fixture --exact --test-threads=1
cargo build --locked -p lila-cli
./target/debug/lila test262 run built-ins/Array/prototype/toLocaleString/ --execution-backend wasm --threads 2 --jobs 2 --timeout-ms 60000 --snapshot-dir /tmp/locale-length-test262 --snapshot-name locale-length
```

## Evidence and remaining work

Node reference execution checks JavaScript expectations only, not this Rust
compiler or its emitted Wasm. Modified-compiler results must be attached to the
exact tested source SHA in the PR before marking this capability verified.
The 2026-08-24 private-witness checkpoint remains historical evidence only.

This does not repair the separate element-Invoke dispatch gap for nested Array
and arguments values, implement ECMA-402 locale formatting, change the shared
integer-indexed object model, or retire the existing T18 constructor-matrix
materializer. A green materialized subtree would not establish unchanged-source,
all-constructor Test262 closure. T16 and the complete current-pin Wasm-AOT T26
release gate remain open; no full-suite percentage is inferred from this batch.
