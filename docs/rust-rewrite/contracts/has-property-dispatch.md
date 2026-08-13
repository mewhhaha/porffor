# `[[HasProperty]]` dispatch contract

This note defines the bounded T10 internal-method seam implemented by the Wasm
AOT backend. It is deliberately centered on one complete internal method. It
does not claim that the complete `[[Get]]`, `[[Set]]`, `[[Delete]]` or
`[[OwnPropertyKeys]]` dispatches share that closure. The direct-target
`[[GetOwnProperty]]` fact and `[[IsExtensible]]` pieces described below are
also consumed by bounded Proxy `[[Delete]]` post-trap validation. A richer
projection of that same direct descriptor authority is consumed by bounded
Proxy `[[Get]]` and `[[Set]]` post-trap validation. Proxy `[[Delete]]` also
consumes the typed live-slot reader and full handler `[[Get]]` seam described
below. Its absent-trap target recursion remains bounded, so this is not a claim
that the complete internal-method dispatch is closed.

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

`[[HasProperty]]`, `[[Delete]]`, `[[IsExtensible]]`, `[[GetPrototypeOf]]` and
the public `getOwnPropertyDescriptor` path consume the same loaded handler pair.
Their trap lookups go through the existing full object-read seam with the
handler itself as receiver, so Function, Array, arguments and nested-Proxy
handlers receive their real storage and prototype behavior. A lookup getter's
abrupt completion is routed before any absent/non-callable decision. A present
trap is called with the same tagged handler as `this`, and callable Proxy traps
are accepted by the shared call operation. `[[GetPrototypeOf]]` keeps its
existing object-or-null result check and exact-prototype check for
non-extensible targets; these bounded migrations change only how live slots and
handler methods are obtained.

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
variant, a `ProxyGet` variant containing fact/data-value/getter roles and a
`ProxySet` variant containing fact/data-value/setter roles. The accessor
endpoint itself is a closed `Getter`/`Setter` enum, so adding a projection or
representation without handling its exact endpoint is an exhaustive-match
compile error. Passing an incoming Proxy-Set value, a getter or a setter where
a descriptor data value is required is a Rust type error rather than a
positional-local transposition.

The richer projections read descriptor storage without invoking getters.
Array indices use the descriptor-kind and raw value/getter/setter readers, Array
`length` carries its numeric value, boxed-String virtual properties carry the
actual length or code-unit String, and arguments data indices use the existing
mapped-arguments read only after the descriptor is known to be data. Arguments
accessors and special `length`/`callee` properties read their endpoint/value
slots directly. Missing getters normalize both the historical raw-zero slot
and tagged ECMAScript `undefined`; missing setters normalize to tagged
`undefined` once. The
integer-indexed branch remains explicit even though its current descriptor is
configurable and therefore imposes no Proxy-Get/Set frozen-property restriction.
Ordinary entry storage is consulted before virtual fallbacks, so a Function
`prototype` entry retains an observed `writable: false` transition. The
DataView/intrinsic cases remain the next fallback, ahead of the generic
function-internal `prototype` slot.

Proxy `[[Get]]` now makes abrupt-result ordering a type boundary. A trap call
first produces `PendingProxyGetTrapResultLocals`. The only transition to
`NormalProxyGetTrapResultLocals` emits the current-completion throw return, and
the invariant accepts only that normal-only result together with typed target
and property-key roles. A plausible call site that passes the raw trap result,
drops the key tag or interchanges target/descriptor values therefore does not
compile. This also prevents a frozen-target TypeError from replacing the value
originally thrown by the trap.

For that normal result, Proxy `[[Get]]` accepts an absent or configurable
descriptor. A present non-configurable, non-writable data descriptor requires
`SameValue(trapResult, targetDesc.[[Value]])`; a present non-configurable
accessor whose `[[Get]]` is undefined requires the trap result itself to be
undefined. A callable Proxy getter is only observed as a stored endpoint and
is never invoked by validation. The former Object/Function-only raw entry scan
has been deleted from this consumer.

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
through the public builtin. The `has`, `deleteProperty`, `get` and `set`
invariant consumers therefore make no nested-Proxy-target closure claim. Proxy
`[[Delete]]` preserves the existing fixed-depth absent-trap target recursion,
while Proxy `[[Get]]` trap lookup, absent-trap fallback and broader nested
handler dispatch remain their existing bounded paths. Other Proxy internal
methods, module-namespace descriptor behavior and broader `[[Get]]` closure
remain T11 work.

## Verification boundary

The durable HasProperty regression combines ordinary inheritance, an Array
whose prototype is a Proxy, integer-indexed present and `-0` cases, an
absent-trap nested Proxy target and a callable-Proxy `has` trap. A second
regression covers the descriptor and extensibility consumer migration with
Function, Array, arguments and nested-Proxy handlers, exact handler `this`,
Object and Reflect entry points, and abrupt trap lookup. The `getPrototypeOf`
fixture applies the same contract to Function, Array, arguments and nested-Proxy
handlers, both public entry points, an inherited Proxy `get` trap and an abrupt
Proxy-handler lookup. The `deleteProperty` fixture applies it to the same
handler representations, including a Function-handler accessor that observes
exact tagged handler identity through `this`; its `prototype` read is an
additional receiver/identity assertion, not the tag-retention witness. It also
covers a callable Proxy trap, exact target/key/`this`, abrupt lookup and
trap-call sentinels, and an absent lookup that forwards to a Proxy target. The
focused compile-time and runtime checkpoint is:

```sh
cargo test -p lila-aot-wasm tests::operations_emits_has_property_spec_operation -- --exact
cargo test -p lila-aot-wasm tests::typedarray_has_property_module_validates -- --exact
cargo test -p lila-engine tests::wasm_backend_has_property_dispatches_every_live_exotic_branch -- --exact
cargo test -p lila-engine tests::wasm_backend_proxy_descriptor_and_extensibility_preserve_handler_tags -- --exact
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_supported_proxy_delete_property_fixture -- --exact
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_proxy_get_direct_descriptor_invariants -- --exact
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_proxy_set_direct_descriptor_invariants -- --exact
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_supported_proxy_get_prototype_of_fixture -- --exact
./target/debug/lila test262 run built-ins/Proxy/has --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Proxy/deleteProperty --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Proxy/get --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Proxy/set --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Proxy/getOwnPropertyDescriptor --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Proxy/getPrototypeOf --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Proxy/isExtensible --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Reflect/getPrototypeOf --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Reflect/deleteProperty --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Reflect/get --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Reflect/set --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/TypedArrayConstructors/internals/HasProperty --execution-backend wasm-aot --timeout-ms 120000 --threads 4
```

The direct-target Proxy Array invariants are covered by the focused delete,
descriptor, Get and Set regressions above. The Get fixture covers direct and
Reflect entry points, dense/sparse indices, named and Symbol keys, boxed String,
mapped and special arguments properties, undefined and callable-Proxy getters
without invocation, Function and DataView `prototype`, `SameValue` edge cases,
integer-indexed/configurable/absent false-positive guards and preservation of a
thrown trap before invariant validation. The Set fixture also covers
dense/sparse indices, symbols, boxed String, mapped arguments, arguments
accessors without getter invocation, callable-Proxy setters, `SameValue` edge
cases, a Function `prototype` frozen through its materialized descriptor entry
and both assignment/Reflect entry points. They are written but remain unrun
while the shared verification lane owns Cargo and Test262. Closure still requires the
complete pinned Proxy/Reflect, Object and TypedArray trees; these focused
filters are only the cheapest regression gates for this seam.
