# `%Function.prototype%[@@hasInstance]`

`%Function.prototype%[@@hasInstance]` is the built-in entry point for
`OrdinaryHasInstance`. The `instanceof` operator is the distinct entry point
that performs observable `@@hasInstance` dispatch before falling back to that
algorithm. Both paths share one closed request type so their operand order and
transition cannot be selected by loose booleans.

## Closed intrinsic identity

The shared builtin catalog owns one exact non-constructable
`StandardBuiltinId::FunctionPrototypeSymbolHasInstance`. Its function identity
is `$builtin.Function.prototype[Symbol.hasInstance]`; its `name` is
`"[Symbol.hasInstance]"`; and its `length` is 1. It has no own `prototype`
property and always returns a Boolean when it completes normally.

`%Function.prototype%` owns the symbol-keyed property with attributes:

```text
key:          WellKnownSymbol::HasInstance
value:        FunctionPrototypeSymbolHasInstance
writable:     false
enumerable:   false
configurable: false
```

The function object's own `name` and `length` properties retain the ordinary
built-in function attributes: non-writable, non-enumerable and configurable.
Bootstrap must use the well-known-symbol property-key encoding; the string
property `"Symbol.hasInstance"` is not the same property.

The Function constructor family roots this function body whenever it installs
`%Function.prototype%`. An installed property may therefore never point at a
stub or at an independently materialized duplicate function object.

## Closed backend request

Lowering and code generation preserve the two already-evaluated operands as:

```rust
HasInstanceRequestLocals::InstanceofOperator {
    object,
    constructor,
}
HasInstanceRequestLocals::OrdinaryHasInstance {
    constructor,
    object,
}
```

Each field names one payload/tag local pair. `InstanceofOperator` keeps source
evaluation order — left operand, then right operand — while the named fields
make the abstract-operation roles explicit. `OrdinaryHasInstance` is used by
the intrinsic body after its receiver and first argument have already been
loaded. Neither request recompiles an expression or reads an argument twice.

The request enum is exhaustive: adding a third dispatch mode becomes a compile
error at every emitter consumer. The different field order is intentional and
mirrors the specification signatures `InstanceofOperator(O, C)` and
`OrdinaryHasInstance(C, O)`.

## `InstanceofOperator`

For an `InstanceofOperator { object: O, constructor: C }` request:

1. If `C` is not an object, throw a `TypeError` before any property read.
2. Let `handler` be `GetMethod(C, @@hasInstance)`. This performs an observable
   symbol-keyed `Get`, accepts `undefined` or `null` as absent, throws if a
   present value is not callable, and propagates abrupt property access.
3. If `handler` is present, call it with `C` as `this` and the single argument
   `O`; return `ToBoolean` of the call result and propagate abrupt completion.
4. If the handler is absent and `C` is not callable, throw a `TypeError`.
5. Otherwise transition to
   `OrdinaryHasInstance { constructor: C, object: O }` without reloading or
   reevaluating either value.

The inherited `%Function.prototype%[@@hasInstance]` function is therefore the
ordinary handler for functions, while an own user-defined handler remains
observable for arbitrary objects and functions.

## `OrdinaryHasInstance`

Calling the intrinsic with receiver `C` and first argument `O` performs
`OrdinaryHasInstance(C, O)`:

1. If `C` is not callable, return `false`.
2. If `C` is a bound function exotic object, transition to
   `HasInstanceRequestLocals::InstanceofOperator` with `O` and
   `C.[[BoundTargetFunction]]`. This step occurs before reading `C.prototype`
   and preserves the target's observable `@@hasInstance` dispatch.
3. If `O` is not an object, return `false`.
4. Let `P` be `Get(C, "prototype")`. Propagate an abrupt completion from the
   property read.
5. If `P` is not an object, throw a `TypeError`.
6. Repeatedly apply `O.[[GetPrototypeOf]]()`, propagating abrupt completions.
   Return `false` on `null`, and return `true` when the result is `P`.

The ordinary path must perform the observable `Get`; reading a function's
cached prototype heap slot is not equivalent after `prototype` is redefined as
an accessor. The prototype walk must use the object's internal
`[[GetPrototypeOf]]`, including Proxy behavior.

Whether that accessor definition is permitted is decided by the stored own
property descriptor, not by a function-kind or heap-slot heuristic.
Constructable and generator functions retain their non-configurable default
`prototype`; a call-only function with no such property may create one, and a
configurable user-created property may change between data and accessor kinds.

The Ordinary request must not perform `GetMethod(C, @@hasInstance)` on its
receiver. Doing so would recursively redispatch to the inherited intrinsic;
only its bound-function transition re-enters the Operator request.

## Current evidence and boundary

The 2026-08-13 current-pin Wasm-AOT Function-prototype snapshot records three
physical failures in this exact family:

- `built-ins/Function/prototype/Symbol.hasInstance/length.js`;
- `built-ins/Function/prototype/Symbol.hasInstance/name.js`; and
- `built-ins/Function/prototype/Symbol.hasInstance/prop-desc.js`.

That artifact was pin-current selection evidence, not a current-HEAD execution
result. Its failure list is truncated after ten entries, so it does not prove
the historical outcome of the other eight tests in the directory. The source
before this batch independently proved the missing-property cause: Function
intrinsic bootstrap installed only `call`, `apply`, `bind` and `toString`.

The full focused intrinsic family also checks non-callable receivers, bound
targets, poisoned or non-object `prototype` values, primitive candidate values,
positive and negative prototype chains, and abrupt Proxy `[[GetPrototypeOf]]`.
Closing only the three descriptor witnesses while leaving a placeholder body
would not satisfy this contract.

The adjacent operator witnesses are:

- `language/expressions/instanceof/symbol-hasinstance-get-err.js`;
- `language/expressions/instanceof/symbol-hasinstance-invocation.js`;
- `language/expressions/instanceof/symbol-hasinstance-not-callable.js`; and
- `language/expressions/instanceof/symbol-hasinstance-to-boolean.js`.

On 2026-08-21, the complete eleven-file intrinsic leaf passed 22/22 strict and
sloppy Wasm-AOT executions. The adjacent four-file operator-hook prefix passed
8/8. `cargo xc`, all five bounded structure checks and the created-realm CLI
consumer were also green. These are focused current-HEAD checkpoints, not a
complete current-pin publication. Dynamic Function source generation remains a
non-claim of this lane.

## Focused verification

The coherent implementation batch was verified with:

```sh
cargo fmt --all -- --check
cargo xc
cargo test -p lila-aot-wasm --test function_prototype_symbol_has_instance_structure --quiet
cargo test -p lila-cli --test cli run_wasm_backend_supports_function_prototype_symbol_has_instance --quiet
./target/debug/lila --jobs 1 test262 run built-ins/Function/prototype/Symbol.hasInstance --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 180000
./target/debug/lila --jobs 1 test262 run language/expressions/instanceof/symbol-hasinstance --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 180000
```

The focused Test262 directory contains eleven files. Its result is a bounded
checkpoint, not a replacement for a complete current-pin Wasm-AOT publication.
