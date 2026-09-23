# JSON reviver property definitions

The static and dynamic JSON reviver paths share one post-call owner. A returned
value now goes through the intrinsic Boolean property-definition operation with
a complete writable, enumerable and configurable data descriptor. A false
result is ignored; an abrupt completion propagates unchanged. This is the
[InternalizeJSONProperty](https://tc39.es/ecma262/multipage/structured-data.html#sec-internalizejsonproperty)
and [CreateDataProperty](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-createdataproperty)
contract.

The canonical definition path owns sparse and dense descriptors, extensibility,
array length, accessor replacement and Proxy invariants. Array index definitions
reject growth past a non-writable length before mutating either dense or sparse
storage; defining existing indexes or filling holes below that length stays valid. The obsolete
array-only reviver mutation is removed. The private input descriptor has a null
prototype, while observable Proxy trap descriptor objects use the JSON builtin's
Realm through the shared intrinsic call. User replacement of public Object or
Reflect methods does not replace this internal operation.

Three standalone reproducers fail on both frozen main and checkpoint fifteen
revision one: a non-configurable sparse value is overwritten, a configurable
sparse accessor is not replaced correctly, and recreation on a non-extensible
array throws. The native target retains those cases and adds Proxy false/throw,
undefined rejection identity, locked length, prototype pollution and foreign
Realm controls. Candidate outcomes remain unverified until the next integrated
batch. The first integrated run passed six of eight native cases and exposed
a missing dependency root, inherited descriptor reads during private forwarding,
and missing locked-length validation. Those causes are repaired in the next
source batch; they are not counted as passing before verification.

```sh
cargo test --release --locked -p lila-engine --test aot_json_reviver_definitions -- --test-threads=2
cargo test --release --locked -p lila-aot-wasm --test json_reviver_frame_structure
cargo test --release --locked -p lila-engine --lib tests::wasm_backend_json_ -- --test-threads=2
```
