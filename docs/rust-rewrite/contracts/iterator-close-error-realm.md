# IteratorClose algorithm-created TypeError Realm

Status: focused verification passed on 2026-08-29.

## Invariant

The shared `emit_iterator_close` owner creates exactly two algorithm errors:

- `IteratorClose return method must be callable` when a present `return`
  property is not callable; and
- `IteratorClose return result must be object` when calling `return` completes
  normally with a primitive result.

Both TypeErrors always use the current function Realm. IteratorClose does not
derive either error's prototype from the iterator object. Synchronous
destructuring, ArrayAccumulation, `for-of`, and `Math.sumPrecise` protocol
errors now follow the same Realm rule through their separate consumer domain.

## Entry routes and completion precedence

The shared owner has 67 external entry routes. The census excludes the calls
that connect the two preserving wrappers to each other and to
`emit_iterator_close`:

- 16 routes call `emit_iterator_close` directly;
- 48 routes call `emit_iterator_close_preserving_current_throw`; and
- 3 routes call `emit_iterator_close_preserving_saved_throw` directly.

The preserving routes keep their existing completion rule. They save an
incoming Throw, perform IteratorClose, and restore that original Throw even if
close creates one of the two TypeErrors above. Direct routes can expose the
close-generated error. The Realm change does not alter property-read, call,
object-result, or completion-precedence order for either class of caller.

## Entry-Realm fallback

`emit_throw_current_function_realm_type_error` reads the active function
environment when `current_env_local` is nonzero. A zero `current_env_local`
uses the main Realm fallback. Entry code therefore retains its main-Realm
behavior, while borrowed created-Realm iterator helpers construct these
TypeErrors from the executing helper's Realm.

## Nonclaim

This contract owns only the two errors created by IteratorClose. At this
checkpoint, direct `for-of` `GetIterator` and `IteratorStep` errors were a
separate change. This checkpoint therefore made no complete synchronous
iterator-protocol error-Realm claim. That separate change is now present for
all three direct synchronous `for-of` owners and is documented in
[`direct-synchronous-for-of-protocol-error-realm.md`](./direct-synchronous-for-of-protocol-error-realm.md).
The later four-consumer boundary also covers Array destructuring,
ArrayAccumulation, and `Math.sumPrecise`; see
[`sync-iterator-consumer-capability.md`](./sync-iterator-consumer-capability.md).
Neither change expands this contract's ownership beyond IteratorClose.

## Focused verification

`cargo check -p lila-aot-wasm` passes. The source-structure target passes
`4/4`; the exact created-Realm CLI test passes `1/1`; and the affected
`iterator_close` CLI sweep passes `6/6`. The fixture passes `node --check`.
The two pinned direct `for-of` leaves each pass `2/2`, for `4/4` Wasm-AOT
executions with every failure and non-success bucket at zero.

The focused commands were:

```sh
cargo test -p lila-aot-wasm --test iterator_close_error_realm_structure -- --test-threads=1
cargo test -p lila-cli --test cli iterator::run_wasm_backend_uses_borrowed_iterator_helper_realm_for_iterator_close_errors -- --exact
./target/debug/lila --jobs 1 test262 run language/statements/for-of/iterator-close-non-throw-get-method-non-callable.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run language/statements/for-of/iterator-close-non-object.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
```

The CLI owner runs
`crates/lila-cli/tests/fixtures/wasm_iterator_close_generated_error_realm.js`.
It borrows `Iterator.prototype.some` from a created Realm and observes both
generated errors plus a valid object-return control.

No semantic golden, published-status refresh, complete Test262 leaf, or broad
workspace suite was run for this checkpoint.
