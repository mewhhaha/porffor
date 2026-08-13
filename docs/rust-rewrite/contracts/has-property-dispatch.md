# `[[HasProperty]]` dispatch contract

This note defines the bounded T10 internal-method seam implemented by the Wasm
AOT backend. It is deliberately centered on one complete internal method. It
does not claim that `[[Get]]`, `[[Set]]`, `[[Delete]]` or
`[[OwnPropertyKeys]]` share the same full dispatch. The direct-target
`[[GetOwnProperty]]` fact and `[[IsExtensible]]` pieces described below are
also consumed by bounded Proxy `[[Delete]]` post-trap validation. A richer
projection of that same direct descriptor authority is consumed by bounded
Proxy `[[Set]]` post-trap validation. Neither migration changes trap lookup,
fallback or the complete internal-method dispatch.

## One product entry

Every cross-module `[[HasProperty]]` consumer calls
`emit_object_has_property_i32` or its property-key-tag-preserving sibling.
The representation-specific emitters are private to `objects.rs`. In
particular, an Array builtin cannot select an emitter named "ordinary" and
thereby make a Proxy, integer-indexed object or later exotic invisible.

The full entry walks the prototype chain itself. At each object it consumes the
same closed six-branch declaration in this order:

1. Proxy;
2. integer-indexed TypedArray;
3. Array;
4. arguments object;
5. boxed String;
6. ordinary property storage, including the Function `prototype` internal
   slot.

The declaration generates both the Rust enum and the order consumed by code
generation. The emitter matches it exhaustively. A new representation cannot
be added to the declaration without also adding its emitted branch, and an
existing representation cannot be dropped from the consumed order by editing
a second hand-maintained list.

Array and arguments misses load the actual prototype and restart the same
dispatch. A boxed String virtual-property miss continues into ordinary storage
on that same object; an ordinary miss advances to its prototype. A Proxy whose
`has` trap is absent replaces the current object with `[[ProxyTarget]]` and also
restarts the same dispatch. Neither path calls a weaker "ordinary" recursion
helper.

## Proxy ordering

Proxy `[[HasProperty]]` performs these observable steps in order:

1. reject a revoked Proxy;
2. load `[[ProxyTarget]]`;
3. perform `GetMethod(handler, "has")` through the full object-read seam;
4. when absent, restart full dispatch at the target;
5. otherwise call any callable value, including a callable Proxy, with the
   handler as `this` and `(target, key)` as arguments;
6. apply `ToBoolean` to the result;
7. validate a false result against the target.

## Handler slot invariant

A Proxy record must retain `[[ProxyTarget]]` and `[[ProxyHandler]]` as two
complete tagged values. The bounded representation seam is one
`ProxySlotLocals` value containing distinct `ProxyTargetLocals` and
`ProxyHandlerLocals` newtypes over `TaggedLocals`, plus one allocator that
requires that complete record before it can mark an object as a Proxy. Both
`Proxy` and `Proxy.revocable` call that allocator; the underlying slot writer
is private. A constructor that supplies a handler payload without a handler
tag, or swaps target and handler, must therefore fail to compile.

The handler payload remains in the existing proxy-marker word, so revocation
continues to replace only that word with `PROXY_HANDLER_PAYLOAD_MIN`. Readers
must test the marker before loading the retained handler tag. The tag occupies
a proxy-only word in the generic 256-byte object allocation and is registered
as a non-pointer layout slot; it does not move the target words or alter any
other object representation.

One typed reader is the only authority that maps the four Proxy heap words back
to `ProxySlotLocals`. Its input record still has distinct target and handler
newtypes, so omitting a tag or swapping the two roles is a compile error at both
the writer and the reader. The reader also owns the revoked-handler check;
callers choose one of the closed completion routes but cannot load a supposedly
live slot set without emitting that check first. Raw consumers may inspect the
handler-payload marker only to decide whether an Object is a Proxy; they may not
reconstruct the retained fields themselves.

`[[HasProperty]]`, `[[IsExtensible]]`, `[[GetPrototypeOf]]` and the public
`getOwnPropertyDescriptor` path consume the same loaded handler pair. Their trap
lookups go through the existing full object-read seam with the handler itself as
receiver, so Function, Array, arguments and nested-Proxy handlers receive their
real storage and prototype behavior. A lookup getter's abrupt completion is
routed before any absent/non-callable decision. A present trap is called with
the same tagged handler as `this`. `[[GetPrototypeOf]]` keeps its existing
object-or-null result check and exact-prototype check for non-extensible targets;
this bounded migration changes only how its live slots and handler method are
obtained.

The Proxy `has` false-result and `deleteProperty` true-result invariants use a
separate value-free direct-own-descriptor fact over the same closed
representation order. Array, arguments, integer-indexed, boxed-String,
Function-special and ordinary own properties are therefore checked without
allocating a public descriptor object. Both consumers first accept an absent
property, then reject a present non-configurable descriptor, and only then send
a present configurable descriptor through the shared `[[IsExtensible]]`
operation.

There is still only one direct-own-descriptor representation loop. Its closed
`DirectOwnDescriptorProjectionLocals` result domain has a value-free `Fact`
variant and a `ProxySet` variant containing the complete
`ProxySetDescriptorLocals`: the fact, a typed descriptor data value and a typed
accessor setter. Adding a representation or projection without handling it is
an exhaustive-match compile error. Passing an incoming Proxy-Set value where a
descriptor data value or setter is required is a Rust type error rather than a
positional-local transposition.

The richer projection reads descriptor storage without invoking getters.
Array indices use the descriptor-kind and raw value/setter readers, Array
`length` carries its numeric value, boxed-String virtual properties carry the
actual length or code-unit String, and arguments data indices use the existing
mapped-arguments read only after the descriptor is known to be data. Arguments
accessors and special `length`/`callee` properties read their setter/value slots
directly. Missing setters are normalized to tagged `undefined` once. The
integer-indexed branch remains explicit even though its current descriptor is
configurable and therefore imposes no Proxy-Set frozen-property restriction.
Ordinary entry storage is consulted before virtual fallbacks, so a Function
`prototype` entry retains an observed `writable: false` transition. The
DataView/intrinsic cases remain the next fallback, ahead of the generic
function-internal `prototype` slot.

Proxy `[[Set]]` validates a truthy trap result in ECMA-262 order: accept an
absent descriptor; for a present non-configurable, non-writable data descriptor
require `SameValue(V, targetDesc.[[Value]])`; for a present non-configurable
accessor reject only when `targetDesc.[[Set]]` is `undefined`; otherwise accept.
A callable Proxy setter is therefore accepted even though its runtime tag is
Object. The old Object/Function/arguments `HEAP_OBJECT_ENTRY_SIZE` scan is not a
descriptor authority and has been removed from this consumer.

This remains a bounded consumer migration. The direct fact is not the recursive
Proxy descriptor-record protocol: when `[[ProxyTarget]]` is itself a Proxy, the
eventual implementation must perform that target's `GetMethod`, call,
descriptor conversion and complete compatibility validation without allocating
through the public builtin. The `has`, `deleteProperty` and `set` invariant
consumers therefore make no nested-Proxy-target closure claim. Proxy `[[Get]]`
still has its older value-bearing scan. Other Proxy internal methods,
module-namespace descriptor behavior, and broader nested exotic-handler
`[[Get]]` closure remain T11 work.

## Verification boundary

The durable HasProperty regression combines ordinary inheritance, an Array
whose prototype is a Proxy, integer-indexed present and `-0` cases, an
absent-trap nested Proxy target and a callable-Proxy `has` trap. A second
regression covers the descriptor and extensibility consumer migration with
Function, Array, arguments and nested-Proxy handlers, exact handler `this`,
Object and Reflect entry points, and abrupt trap lookup. The `getPrototypeOf`
fixture applies the same contract to Function, Array, arguments and nested-Proxy
handlers, both public entry points, an inherited Proxy `get` trap and an abrupt
Proxy-handler lookup. The focused compile-time and runtime checkpoint is:

```sh
cargo test -p lila-aot-wasm tests::operations_emits_has_property_spec_operation -- --exact
cargo test -p lila-aot-wasm tests::typedarray_has_property_module_validates -- --exact
cargo test -p lila-engine tests::wasm_backend_has_property_dispatches_every_live_exotic_branch -- --exact
cargo test -p lila-engine tests::wasm_backend_proxy_descriptor_and_extensibility_preserve_handler_tags -- --exact
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_supported_proxy_delete_property_fixture -- --exact
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_proxy_set_direct_descriptor_invariants -- --exact
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_supported_proxy_get_prototype_of_fixture -- --exact
./target/debug/lila test262 run built-ins/Proxy/has --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Proxy/deleteProperty --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Proxy/set --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Proxy/getOwnPropertyDescriptor --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Proxy/getPrototypeOf --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Proxy/isExtensible --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Reflect/getPrototypeOf --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Reflect/deleteProperty --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Reflect/set --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/TypedArrayConstructors/internals/HasProperty --execution-backend wasm-aot --timeout-ms 120000 --threads 4
```

The direct-target Proxy Array invariants are covered by the focused delete,
descriptor and Set regressions above. The Set fixture also covers dense/sparse
indices, symbols, boxed String, mapped arguments, arguments accessors without
getter invocation, callable-Proxy setters, `SameValue` edge cases, a Function
`prototype` frozen through its materialized descriptor entry and both
assignment/Reflect entry points. It is written but remains unrun while the
shared verification lane owns Cargo and Test262. Closure still requires the
complete pinned Proxy/Reflect, Object and TypedArray trees; these focused
filters are only the cheapest regression gates for this seam.
