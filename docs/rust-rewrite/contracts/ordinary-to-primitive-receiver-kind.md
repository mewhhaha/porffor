# OrdinaryToPrimitive receiver-kind ownership

The private `OrdinaryToPrimitiveReceiverKind::{Object, Function}` domain is the
complete set of heap-record families admitted by the shared ordinary-object
ToPrimitive emitter. Its exhaustive projections own both decisions that differ
between those families: the runtime `ValueKind` tag and whether the record
reserves the boxed-primitive slot.

An arbitrary `ValueKind` entering the emitter and accidentally reading an
unrelated record offset or running the ordinary-object hook algorithm is now
unrepresentable. Adding a receiver family requires explicit tag and boxed-slot
decisions. The domain has no clone, copy, debug or equality capability, and
every producer moves one choice into the inner emitter.

The unused public Function-only wrapper and its private pending twin are gone.
They had no product caller; the live tagged ToPrimitive path already selects
the Function member before entering the same inner algorithm. The ordinary
Object wrapper remains the other live entry. Deleting the unreachable subgraph
also reduces pending-completion construction from four raw producers to three.

The recursive structure guard pins the exact domain, both exhaustive
projections, the two Object selections, the sole Function selection, the inner
emitter signature and absence of both deleted functions. The neighboring
pending-completion and conversion-Realm guards retain the live producer and
borrowed-source census.

This is source-equivalent Rust ownership hardening. It changes no conversion
operation, hook order, error Realm, completion route, emitted Wasm or ABI.

```sh
cargo test -p lila-aot-wasm --test ordinary_to_primitive_receiver_kind_structure
cargo test -p lila-aot-wasm --test pending_to_primitive_operation_identity_structure
cargo test -p lila-aot-wasm --test conversion_error_realm_source_structure
```

The receiver-kind target passes `4/4`; the neighboring pending-completion and
conversion-Realm targets pass `3/3` and `4/4`. The existing Wasm-backend
ToNumber and Error ToPrimitive CLI controls each pass `1/1`. The shared
`cargo xc`, workspace formatting, diff, module-boundary and task-plan checks
are green.
