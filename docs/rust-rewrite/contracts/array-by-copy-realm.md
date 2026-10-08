# Array copy-method result Realm

The four Array methods `toReversed`, `toSorted`, `toSpliced` and `with` create
ordinary Arrays through `ArrayCreate`, using the executing builtin's intrinsic
Array prototype. Receiver Realm, receiver constructor/species and public
`Array` replacement do not choose that prototype. See the current
[Array method algorithms](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.toreversed)
and [ArrayCreate](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-arraycreate).

All four actual product emitters consume
`emit_alloc_array_payload_with_length_in_current_function_realm`. Its existing
private non-Copy prototype local carries the defining-Realm intrinsic to
installation and is released once. A self-backed builtin obtains its saved
intrinsic table; an explicit zero-environment entry uses the entry prototype.
The three new joins change only allocation calls. Length/index/argument
conversion, method-specific Get order, skipping replaced/deleted positions,
read-through holes, own data publication and native error ordering retain
their existing owners. Native bounds errors already use the called Realm.

Two paired strict/sloppy Engine cohorts cover borrowed methods in both
directions, saved prototypes/public constructor poisoning, Array/generic/Proxy
receivers, no Has/species observations, once-only conversions/Gets, dense hole
publication and original foreign abrupt identity with finally and prior effects.
The global allocator inventory follows the three actual added consumers.

The complete source, controls and documentation passed the ref93 combined
all-target Rust type checkpoint; emitted-Wasm/runtime execution remains pending. This batch
does not establish complete Array semantics or a new conformance count.
