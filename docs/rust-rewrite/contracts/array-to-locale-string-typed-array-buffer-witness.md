# Generic Array `toLocaleString` TypedArray buffer witness

Status: normative theory, implementation, independent review and focused
Wasm-AOT verification complete for the generic
`Array.prototype.toLocaleString` TypedArray length-snapshot seam, 2026-08-24.

## Specification and compiler boundary

`Array.prototype.toLocaleString` first applies `ToObject` to its receiver and
then obtains one `LengthOfArrayLike` snapshot before beginning its ascending
element walk. When that operation resolves the standard TypedArray `length`
accessor, an out-of-bounds or detached view contributes zero length rather than
the TypeError required by the direct
`%TypedArray%.prototype.toLocaleString` method-entry validation. An available
fixed view contributes its stored whole-element extent, while an available
length-tracking view derives its whole-element length from the current backing
byte length. A trailing partial element is not visible.

The captured length remains the loop bound for the complete call. Growth during
an element invocation does not extend the walk, and shrinkage or detachment does
not shorten it. Element values are not captured with that length: every
iteration still performs a live integer-indexed read, so an index made
unavailable before its turn supplies the current indexed-read result to the
unchanged nullish and element-invocation algorithm.

The current Wasm-AOT compiler has a runtime TypedArray specialization inside
the generic Array entry. This migration changes only that specialization's
backing-store observation. It does not establish complete observable
`LengthOfArrayLike` behavior for own or inherited `length` mutations on a
TypedArray receiver.

## Closed projection and exact owner census

`compile_to_locale_string_builtin` remains the sole shared compiler. It has
exactly two public callers:

- `compile_array_prototype_to_locale_string_builtin` selects
  `ToLocaleStringReceiverKind::ArrayLike`; and
- `compile_typed_array_prototype_to_locale_string_builtin` selects
  `ToLocaleStringReceiverKind::TypedArray`.

The standard-builtin dispatcher maps each identifier to its matching wrapper
once. This lane changes only the TypedArray sub-arm of the `ArrayLike` entry.
After TypedArray detection and live-read routing are selected, that arm loads
one immutable private view with `emit_load_typed_array_private_state`, constructs
one `TypedArrayViewLocals`, and consumes one live witness through:

```rust
TypedArrayWitnessUse::ArrayLikeLengthSnapshot {
    length_local: len_local,
}
```

The witness is the sole owner of the arm's backing data and byte-length
observation, detached/out-of-bounds zeroing, fixed-versus-tracking extent and
whole-element division. The arm may not call either legacy current-byte-length
emitter, observe the backing store in parallel, reconstruct private TypedArray
slots directly, divide a byte length locally, overwrite `len_local`, or select
the throwing `ValidatedMethodEntry` projection.

The direct TypedArray entry remains a separate branch with one receiver-brand
guard and one `ValidatedMethodEntry` witness. Its initially detached or
out-of-bounds receiver still throws before the element algorithm begins. The
shared loop retains its one downstream call to
`emit_typed_array_or_object_index_read_from_locals`; that live indexed-read
owner is outside this migration.

## Observable ordering

The bounded compiler must retain this order:

1. generic nullish rejection and `ToObject`;
2. Array, arguments and object receiver classification;
3. TypedArray detection and selection of live indexed-read routing;
4. private-state load, immutable view construction and the non-throwing
   `ArrayLikeLengthSnapshot` witness;
5. the distinct ordinary-object observable `length` read and `ToLength`
   alternative;
6. the loop comparison against the captured length;
7. the live indexed read; and
8. element `toLocaleString` lookup, validation and Proxy-aware call.

No witness or raw length observation belongs in the loop. Consequently a
callback-triggered resize cannot alter the loop bound, while the later live
read still observes the resize.

## Durable structural regression

`crates/lila-aot-wasm/tests/typed_array_to_locale_string_witness_structure.rs`
bounds `compile_to_locale_string_builtin` through the next compiler function
and isolates its direct and generic arms. It requires two statically distinct
private-state loads, immutable views and witnesses in the shared owner: one
`ValidatedMethodEntry` in the direct arm and one `ArrayLikeLengthSnapshot` in
the generic arm. The generic arm rejects raw current-length emitters, direct
private or backing-store loads, local unsigned division, direct `len_local`
writes and every other witness projection.

The guard also pins both wrappers, both dispatcher edges, the complete
`len_local` writer inventory, captured loop bound, downstream live indexed
read, element invocation order and reverse temporary-local release. Its focused
fixture control bounds the exact CLI test registration and requires the
tracking, fixed out-of-bounds, Uint16 odd-byte, detached and final zero-failure
markers. These are source-structure mutation guards, not runtime or Test262
pass evidence.

## Focused runtime evidence

`crates/lila-cli/tests/fixtures/wasm_array_to_locale_string_core.js`, registered
by
`array::run_wasm_backend_succeeds_for_supported_array_to_locale_string_fixture`,
uses distinct failure bits and publishes `failures === 0`. Its TypedArray
matrix covers:

- a length-tracking Uint8 view reflecting shrinkage at the next generic call;
- an out-of-bounds fixed view producing the empty string through the generic
  Array method while the direct TypedArray method throws;
- a length-tracking Uint16 view flooring an odd five-byte backing length to two
  elements; and
- a detached view producing the empty string through the generic Array method.

The exact pinned Test262 cohort is three source leaves and their six ordinary
sloppy/strict variants:

- `built-ins/Array/prototype/toLocaleString/resizable-buffer.js`;
- `built-ins/Array/prototype/toLocaleString/user-provided-tolocalestring-grow.js`;
  and
- `built-ins/Array/prototype/toLocaleString/user-provided-tolocalestring-shrink.js`.

All three currently pass through the test-specific
`rewrite_array_to_locale_string_resizable_case` materializer. That materializer
reduces the constructor matrix to `Uint8Array` and is inventoried as a T18
semantic shortcut. Any focused result is therefore adapted harness evidence,
not raw all-constructor or BigInt Test262 truth. This lane does not retire the
materializer or change a published conformance count.

The focused checkpoint is green. `cargo check -p lila-aot-wasm` and `cargo xc`
passed; the witness structure target passed `4/4`, the companion invocation
target passed `4/4`, and the exact CLI fixture passed `1/1`. The three adapted
Test262 leaves above passed all `6/6` ordinary sloppy/strict Wasm-AOT variants,
with every parser, early-error, lowering, runtime, Wasm-backend, host-harness,
unsupported, crash and bug bucket at zero. The commands were:

```text
cargo test -p lila-aot-wasm --test typed_array_to_locale_string_witness_structure
cargo test -p lila-aot-wasm --test to_locale_string_invocation_structure
cargo test -p lila-cli --test cli array::run_wasm_backend_succeeds_for_supported_array_to_locale_string_fixture -- --exact
./target/debug/lila --jobs 1 test262 run built-ins/Array/prototype/toLocaleString/resizable-buffer.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run built-ins/Array/prototype/toLocaleString/user-provided-tolocalestring-grow.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run built-ins/Array/prototype/toLocaleString/user-provided-tolocalestring-shrink.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
```

## Explicit nonclaims

This lane does not change the direct TypedArray method-entry policy, the shared
indexed-read helper, integer-indexed exotic semantics, ordinary-object
`length` lookup and coercion, own or inherited TypedArray `length` shadowing,
separator selection, locale formatting, nullish element handling, `GetV`,
Proxy dispatch, callback Realm behavior, SharedArrayBuffer synchronization or
Atomics ordering. It does not migrate either `flatMap` raw observation or any
raw consumer in `objects.rs`; those two Array sites and three object
property/index read/write owners are the five remaining non-throwing raw
consumers after this lane.

It proves no all-constructor, BigInt, complete `toLocaleString`, TypedArray,
binary-data, Test262 or T17 closure. It changes no Test262 materializer,
unsupported-feature policy, baseline snapshot, README status or published
count.
