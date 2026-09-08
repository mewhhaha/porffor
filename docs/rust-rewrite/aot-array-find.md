# Observable Array find-family compilation

Owner: T16. Implementation baseline: `c5115bf03ba3fdec1a8eb9b3436a939431b2ca1f`.

The generic `Array.prototype.find`, `findIndex`, `findLast` and `findLastIndex`
entries use the existing shared observable length and indexed Get operations.
This is a direct JS-to-Wasm compiler change, not an interpreter fallback.

## Observable contract

`emit_array_like_length_snapshot` owns ToObject, one observable length Get and
ToLength before predicate validation. Borrowed TypedArray and Arguments
receivers therefore observe own and inherited length overrides, coercion and
abrupt completion instead of substituting private backing-store extent. The
global object is an ordinary valid receiver, not a nullish sentinel.

Each index uses the shared live indexed-read dispatcher. Find methods perform
Get without a preceding HasProperty, so holes are visited as undefined and
inherited values remain observable. The original length is retained across
callbacks, deletion, growth and resizable-buffer changes. The four closed kinds
retain forward/backward traversal and value/index projection; reverse traversal
stops at zero before subtracting.

IsCallable still accepts callable Proxies, including a revoked callable Proxy
on an empty receiver. Calls use the shared Proxy-aware invocation path with
thisArg and exactly `(value, index, boxedReceiver)`. Abrupt completion returns
before subsequent reads or calls, and a successful value result is the value
read before invoking the predicate, not a second read after mutation.

The separately owned strict `%TypedArray%.prototype` entries retain their brand
check and validated method-entry buffer witness. They do not read an overridden
public length. Generic Array borrowing does not acquire that strict contract.

## Emitted-loop lifetime and completion

A validated predicate is borrowed by the Call emitter and consumed only when
the loop owner releases its temporaries after both closing End instructions.
Releasing its locals immediately after emitting Call could allow later emitted
instructions to reuse the slots needed by the next runtime iteration. The
private, non-Copy witness and source guards make this lifetime explicit for all
eight Array/TypedArray find entrypoints.

No-match `undefined`/`-1` results are initialized after the loop, rather than
before observable helpers that can use completion-result scratch locals.
Successful matches and throws still return immediately from their own paths.

The source algorithms are ECMA-262's
[FindViaPredicate](https://tc39.es/ecma262/#sec-findviapredicate),
[Array.prototype.find](https://tc39.es/ecma262/#sec-array.prototype.find), and
[%TypedArray%.prototype.find](https://tc39.es/ecma262/#sec-%typedarray%.prototype.find).

## Reproduction and evidence boundaries

The `aot_array_find` engine target contains 24 regression programs, including
observable Arguments/TypedArray length, global and primitive receivers, Proxy
Get ordering without HasProperty, holes and inherited getters, no-match results
across all eight methods, callback revocation, abrupt identity, large reverse
indices, and resizable buffers. Each program explicitly selects WasmAot.

```sh
cargo fmt --all -- --check
cargo test --locked -p lila-aot-wasm \
  --test find_via_predicate_structure \
  --test array_find_algorithm_owner_structure \
  --test array_find_index_algorithm_owner_structure \
  --test array_find_last_algorithm_owner_structure \
  --test array_find_last_index_algorithm_owner_structure
python3 scripts/run_engine_regression_inventory.py aot_array_find \
  --output-dir /tmp/array-find-engine --timeout 600
```

The dedicated CI workflow runs the complete compiled engine inventory in fresh
processes using the existing fail-closed runner. It retains the inventory,
per-test logs and summary as a revision-specific artifact. Formatting, ownership
checks and every execution must pass; an empty selection is not success.

During preparation, all 24 test programs returned true in Node v22.16.0. That
validates reference expectations only, not Lila compilation or execution. Rust
and Wasmtime were unavailable in the editing environment; their verification
status must be read from the PR checks, not inferred from the reference result.

This batch does not change Test262 sources, harnesses, materializers, pin,
exclusions, expected failures or published conformance numbers. It does not
close T16 or establish complete ECMAScript conformance. The full current-pin
Wasm-AOT baseline, broader direct-call property dispatch and remaining Array
algorithms are separate work. No full-suite percentage is claimed here.
