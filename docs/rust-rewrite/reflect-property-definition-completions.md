# Property definition completions

`Reflect.defineProperty` converts the caller's attributes before target
dispatch. A missing field in that converted descriptor must remain missing
during private forwarding. An actual Proxy trap receives a fresh ordinary
descriptor object with the executing method Realm's `Object.prototype`.

The Reflect emitter materializes the converted fields once for possible trap
exposure. `emit_proxy_define_property_trap_result` marks a callable trap handled
regardless of whether its result is truthy, false or undefined. Only the
unhandled branch can forward internally; no user code has received its
descriptor object. That branch clears both prototype slots before recursive
Reflect dispatch or the ordinary Object definition fallback. The original
attributes object and every descriptor retained by an actual trap remain
unchanged. This avoids inherited `get`, `set`, `value` or `writable` observations
during the private conversion without suppressing any such observation on the
actual input attributes.

`Object.defineProperty` applies the same noescape rule before recursive
forwarding through another Proxy. Its ordinary direct path reads the converted
object's own fields, but recursive dispatch would otherwise invoke
ToPropertyDescriptor again with an inherited-field source. Callable Object
traps keep their fresh method-Realm object, including traps whose false or
undefined return subsequently causes Object.defineProperty to throw.

TypedArray numeric-index definitions use the shared
`binary_data/define_property.rs` operation. It accepts a `WasmDescriptor` whose
runtime validation obligation is recorded in the IR descriptor ledger. Both
public method emitters derive that descriptor from their already converted
fields, preserving presence and normalized Boolean attributes.

The operation first uses the canonical integer-index witness, then rejects
incompatible attributes normally with false. A present value reaches the
existing Number/BigInt element writer, which propagates the original throw and
refreshes the buffer witness after conversion. Normal completion publishes
true even if conversion detached the buffer or made the view out of bounds,
in which case the writer skips storage. `Object.defineProperty` turns only
normal false into its own TypeError. Reflect returns the Boolean directly,
before the generic ordinary-object fallback can translate failures.

These outcomes follow the [TypedArray definition operation and element
writer](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-typedarray-defineownproperty-p-desc).
The distinction between private forwarding and actual trap exposure follows
the [Proxy definition operation](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-proxy-object-internal-methods-and-internal-slots-defineownproperty-p-desc).

Focused verification targets are:

```sh
cargo test --release --locked -p lila-engine --test aot_reflect_completed_descriptors --test reflect_descriptor_object_realm --test aot_define_property_realm -- --test-threads=2
cargo test --release --locked -p lila-aot-wasm --test reflect_define_property_completion_structure --test reflect_descriptor_object_realm_structure --test object_define_property_descriptor_roles_structure --test to_property_descriptor_operation_evidence_structure --test typed_array_integer_index_witness_structure --test typed_array_property_index_witness_structure
```

The native controls cover ordinary data/accessor descriptors under prototype
pollution, nested nullish Proxy traps, retained main/foreign Realm trap objects,
Object definitions through nested nullish Proxy layers, exact input and
target-hook throws including undefined, Number/BigInt element
coercion, ordinary false rejection, and post-coercion detachment/resize. The
source guards pin both callers to the same validated Boolean operation and
keep throw propagation ahead of either public result policy. This change does
not replace the remaining ordinary definition fallback or claim complete
Proxy/descriptor conformance.
