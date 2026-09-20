# Per-Realm `%ThrowTypeError%` ownership

The entry Realm and every created Realm each allocate one anonymous,
nonconstructable `%ThrowTypeError%` function through the canonical function
allocator. The existing catalog retains zero length, immutable `length` and
`name` properties, native source text and the non-extensible object shape.
Created-Realm allocation supplies its defining Realm and callable
`%Function.prototype%` together, then publishes the function in the existing
traced Realm-intrinsic slot. It does not replace the entry global.

The same function is the getter and setter of the Realm's restricted
`Function.prototype.caller` and `arguments` properties. Those descriptors are
non-enumerable and configurable. Unmapped Arguments use the executing ordinary
function's defining Realm, with the same getter/setter identity, and
non-enumerable, non-configurable `callee` attributes. Strict and non-simple
parameter lists select this protocol; sloppy simple lists retain the existing
writable data `callee`. An arrow reads its enclosing owner's Arguments object.

A callable's environment carries its own identity before publication. The
shared thrower body allocates a fresh TypeError from the called function's
Realm, including after extraction, binding, proxy forwarding, accessor
invocation, global constructor replacement or deletion of the prototype's
restricted properties. Neither `this` nor arguments are coerced. Ordinary
argument evaluation still occurs before invocation.

Realm, intrinsics and thrower addresses must be populated before an Arguments
descriptor receives a Function tag. Missing bootstrap state is an internal
invariant failure, not a request to use the entry Realm or table index zero.
The emitter holds the loaded thrower in one owned local until both accessor
payloads have been stored.

## Evidence and boundary

Frozen batch20r6 reproduced a Wasmtime unreachable trap for
`sloppy-script:built-ins/ThrowTypeError/distinct-cross-realm.js`. The missing
foreign intrinsic slot was loaded as payload zero and published with a
Function tag; its indirect call reached table index zero, which the harness
named `gc`. Separate ordinary-source probes confirmed missing foreign
prototype accessors and the foreign Arguments trap, while the entry singleton
survived creation of another Realm. This repair changes the ownership and
publication cause. The explicit GC capability policy and host-GC body remain
unchanged; collection is still rejected when the collector is unavailable.

The source algorithms are ECMA-262
[AddRestrictedFunctionProperties and %ThrowTypeError%](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-addrestrictedfunctionproperties),
[CreateUnmappedArgumentsObject](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-createunmappedargumentsobject),
and the Realm transition for a built-in function's call. The implementation
uses existing finite prepared-source admission for the foreign Function
fixtures; no runtime source generation or fixture substitution is added.

## Verification

The focused targets are:

```sh
cargo test -p lila-engine --test aot_throw_type_error_realm -- --test-threads=1
cargo test -p lila-ir --test throw_type_error_realm
cargo test -p lila-aot-wasm --test throw_type_error_realm_structure
cargo test -p lila-aot-wasm --lib functions::realm_function_materialization_tests::created_realm_function_sites_require_the_coupled_context -- --exact
cargo test -p lila-aot-wasm --lib arguments_protocol::tests
cargo test -p lila-aot-wasm --test callable_function_prototype_structure
cargo test -p lila-aot-wasm --test heap_collector_policy_structure
cargo test -p lila-cli --test cli language_errors::run_wasm_backend_reports_gc_requires_real_collector -- --exact --test-threads=1
```

The native cases cover separate Arguments instances in two foreign Realms,
all extracted/forwarded/property call routes, intrinsic metadata,
mutable-global independence, strict/non-simple/lexical Arguments and suspended
owners. The IR controls preserve function/parameter/lexical ownership facts
and runtime ownership of an extracted callable. The source guards cover
publication, callable self-environment, both descriptor stores and traced
roots. Paired replay must include all applicable modes of the 14 pinned
ThrowTypeError files and the selected Function/Arguments descriptor controls.
This bounded change does not publish a full-suite conformance result.
