# OrdinaryToPrimitive receiver-kind ownership

The private `OrdinaryToPrimitiveReceiverKind::{Object, Function}` domain is the
complete set of heap-record families admitted by the shared ordinary-object
ToPrimitive emitter. Its exhaustive projection owns the runtime `ValueKind`
tag used by property reads and calls. Every primitive wrapper follows the same
observable `@@toPrimitive`, `toString`, and `valueOf` lookup protocol; the emitter
must not bypass hooks by reading a wrapper's internal primitive payload.

An arbitrary `ValueKind` entering the emitter and accidentally reading an
unrelated record offset or running the ordinary-object hook algorithm is now
unrepresentable. Adding a receiver family requires an explicit runtime tag. The domain has no clone, copy, debug or equality capability, and
every producer moves one choice into the inner emitter.

The unused public Function-only wrapper and its private pending twin are gone.
They had no product caller; the live tagged ToPrimitive path already selects
the Function member before entering the same inner algorithm. The ordinary
Object wrapper remains the other live entry. Deleting the unreachable subgraph
also reduces pending-completion construction from four raw producers to three.

The recursive structure guard pins the exact domain, its exhaustive
projection, the two Object selections, the sole Function selection, the inner
emitter signature and absence of both deleted functions. The neighboring
pending-completion and conversion-Realm guards retain the live producer and
borrowed-source census.

The observed-failure repair removed a direct-payload shortcut for Number,
String, and Boolean wrappers. Redefining their conversion hooks now affects
ordinary conversions and Function-constructor source coercion, including getter
side effects and abrupt completions. The completion ABI and realm routing remain
the shared conversion protocol.

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
