# `Object.prototype.toLocaleString` Invoke

## Semantic boundary

ECMA-262 defines
[`Object.prototype.toLocaleString`](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.prototype.tolocalestring)
as `Invoke(thisValue, "toString")`.
[`GetV`](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-getv)
uses `ToObject(thisValue)` only as the property-lookup target and passes the
exact original value as the receiver to `[[Get]]`. `Invoke` then applies
`IsCallable` and calls the result with that same original value as `this` and
an empty argument list.

The distinction is observable for primitives. A strict accessor on the
wrapper prototype receives the primitive, not the temporary wrapper, and so
does a strict method. A callable Proxy method reaches Proxy `[[Call]]`; its
`apply` trap receives the primitive as `thisArgument` and an empty argument
list.

`ToObject(nullish)` and a non-callable method both throw a TypeError in the
current Realm Record of the running built-in. Borrowing a created realm's
`Object.prototype.toLocaleString` must therefore produce that realm's
`%TypeError%`, not the entry script's `%TypeError%`.

## Closed compiler shape

`lila-aot-wasm/src/builtins/object/object_to_locale_string_invoke.rs` is the
sole owner of the complete invocation family. The Wasm-AOT emitter preserves
the lookup and receiver roles in a private, non-`Copy`
`ObjectToLocaleStringGetVLocals` value. Its `original_receiver` is never
overwritten. Its `boxed_lookup` is the current-function-Realm wrapper used only
as the `[[Get]]` target, and its `method` is the sole result slot. A single GetV
helper borrows that state and is the only boundary allowed to map it into
`emit_object_read`.

After GetV, one validator consumes the receiver roles and applies the general
`IsCallable` helper. Failure uses the current-function-Realm TypeError helper;
success returns a private, non-`Copy`
`ValidatedObjectToLocaleStringInvocationLocals` token pairing the callable
method with the exact original receiver.

The token's only consumer takes ownership and emits the Proxy-aware call with
no arguments. The builtin body cannot independently pass raw lookup, method or
receiver locals to GetV, validation or Call, so substituting the temporary box
or validating a different method is rejected by the Rust API shape.

The pre-GetV nullish branch also uses the current-function-Realm TypeError
helper. Observable order remains nullish validation, current-function-Realm
boxing, GetV and abrupt propagation, IsCallable, then Call and abrupt
propagation.

The compiler entry remains visible only within `crate::builtins`; the explicit
restricted visibility preserves the scope previously supplied by
`object.rs`'s `pub(super)` boundary. The standard builtin dispatcher remains
its sole external caller.

## Durable evidence

The source-structure regression fixes the private file module, exact type and
method inventory, sole standard-dispatch caller, both private type states, the
unique GetV mapping, general callability validation, current-function-Realm
failures, the ownership-consuming Proxy-aware call and the empty argument
list. It also rejects raw property-read, callability, call and entry-realm error
operations inside the builtin body.

The focused CLI fixture uses a strict Number-prototype getter returning a
callable Proxy. It observes the exact primitive at both the getter and Proxy
`apply` boundaries and observes zero call arguments. A strict Boolean method
covers the direct function path and the inherited Array call chain.
Created-realm nullish and non-callable cases fix both TypeError prototypes to
the borrowed method's Realm.

At Test262 pin `e9d582d6b8b13afc5ba9a676664741592b5c7f69`, the exact focused
inventory is four physical files:

- `built-ins/Object/prototype/toLocaleString/primitive_this_value.js`;
- `built-ins/Object/prototype/toLocaleString/primitive_this_value_getter.js`;
- `built-ins/Array/prototype/toLocaleString/primitive_this_value.js`; and
- `built-ins/Array/prototype/toLocaleString/primitive_this_value_getter.js`.

Each file declares `flags: [onlyStrict]`, so the inventory materializes as
exactly four executions rather than sloppy/strict pairs. On 2026-08-24, the
central verifier ran these focused gates:

```sh
cargo test -p lila-aot-wasm --test object_to_locale_string_invoke_structure -- --test-threads=1
cargo test -p lila-cli --test cli language_numerics::run_wasm_backend_succeeds_for_object_to_locale_string_invoke_fixture -- --exact --test-threads=1
./target/debug/lila --jobs 1 test262 run built-ins/Object/prototype/toLocaleString/primitive_this_value.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run built-ins/Object/prototype/toLocaleString/primitive_this_value_getter.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run built-ins/Array/prototype/toLocaleString/primitive_this_value.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run built-ins/Array/prototype/toLocaleString/primitive_this_value_getter.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
```

The batch-wide `cargo check` and `cargo xc` gates were green. The structure
target passed `3/3`, and the exact CLI fixture passed `1/1`. Each Test262 leaf
command discovered and passed its one strict execution, for `4/4` in total with
every failure bucket at zero.

## Baseline disclosure and nonclaims

The available `built-ins/Array/prototype` current-pin chunk predates this
source change. It reports 248/250 with exactly the two primitive
`toLocaleString` failures and observes `object,object` where Test262 expects
`boolean,boolean`. It is stale baseline evidence, not a current-SHA delta or a
green-subtree claim.

This seam does not introduce a reusable compiler-wide GetV or Invoke
abstraction, change wrapper-prototype Realm selection, implement ECMA-402
locale formatting, close Proxy or Object internal methods generally, remove a
Test262 materializer, refresh a snapshot or change a published conformance
count.
