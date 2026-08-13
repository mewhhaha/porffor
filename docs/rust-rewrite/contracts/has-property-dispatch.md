# `[[HasProperty]]` dispatch contract

This note defines the bounded T10 internal-method seam implemented by the Wasm
AOT backend. It is deliberately about one internal method. It does not claim
that `[[GetOwnProperty]]`, `[[Get]]`, `[[Set]]`, `[[Delete]]` or
`[[OwnPropertyKeys]]` already share the same representation.

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

`[[HasProperty]]` consumes both retained handler locals. Its trap lookup goes
through the existing full object-read seam with the handler itself as receiver,
so Function, Array, arguments and nested-Proxy handlers receive their real
storage and prototype behavior. A lookup getter's abrupt completion must leave
the HasProperty traversal before any absent/non-callable decision. A present
trap is called with the same tagged handler as `this`.

This contract intentionally migrates only the `has` consumer in the bounded
batch. Other Proxy internal methods that still reconstruct an Object handler
tag remain explicit T11 consumers of the same retained slot and shared
`[[Get]]`; the new representation makes those follow-up migrations possible
without another layout change.

The false-result invariant remains a separate seam. The existing
`Object.getOwnPropertyDescriptor` builtin does implement Array, arguments and
integer-indexed own descriptors, and `emit_object_is_extensible_i32` is already
typed. But invoking a full builtin descriptor allocation from inside the raw
HasProperty branch would couple completion routing and allocation into the
dispatcher, while the only reusable compact helper today is Array-specific.
This batch therefore does not duplicate that helper or call a builtin from the
raw branch. The durable next step is a value-free typed own-descriptor fact
emitter, consumed jointly by the public descriptor builtin and Proxy invariant
checks; its closed exotic cases are outside this handler-layout migration.

The last step is not closed by the HasProperty dispatch alone. The current
post-trap checker reads ordinary Object/Function property storage directly; it
does not yet consume a complete `[[GetOwnProperty]]` and `[[IsExtensible]]`
dispatch for Array, arguments, integer-indexed or module-namespace targets.
Consequently this batch must not claim all Proxy `has` invariants. In
particular, reporting an Array's non-configurable `length` as absent remains a
T11 blocker until that descriptor dispatch exists:
`Reflect.has(new Proxy([], { has() { return false; } }), "length")` must throw a
`TypeError`, while the current ordinary-only invariant checker can return
`false`. Adding an Array-only mirror of the invariant would duplicate the exact
abstraction T10 is meant to create.

## Verification boundary

The durable positive regression combines ordinary inheritance, an Array whose
prototype is a Proxy, integer-indexed present and `-0` cases, an absent-trap
nested Proxy target and a callable-Proxy `has` trap. The focused compile-time
and runtime checkpoint is:

```sh
cargo test -p lila-aot-wasm tests::operations_emits_has_property_spec_operation -- --exact
cargo test -p lila-aot-wasm tests::typedarray_has_property_module_validates -- --exact
cargo test -p lila-engine tests::wasm_backend_has_property_dispatches_every_live_exotic_branch -- --exact
./target/debug/lila test262 run built-ins/Proxy/has --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/TypedArrayConstructors/internals/HasProperty --execution-backend wasm-aot --timeout-ms 120000 --threads 4
```

The Proxy Array invariant above is intentionally documented rather than added
as a failing repository test. Closure ultimately requires the complete pinned
Proxy/Reflect, Object and TypedArray trees; these focused filters are only the
cheapest regression gates for this seam.
