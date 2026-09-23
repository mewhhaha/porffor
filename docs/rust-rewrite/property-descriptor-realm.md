# Property descriptor Realm ownership

`Object.defineProperty` and `Object.defineProperties` materialize present
Property Descriptor fields through `emit_from_present_property_descriptor`.
The fresh ordinary object's prototype is the executing builtin's intrinsic
`%Object.prototype%`. Neither the target object's Realm nor the Proxy trap's
Realm selects that prototype. The materializer preserves field presence,
field order and the existing consuming local-ownership contract.

The builtin ABI represents an entry-Realm direct call with a zero environment.
A nonzero environment identifies a function context with a defining Realm and
an initialized intrinsic table. Descriptor allocation preserves the zero case
and traps on missing Realm, intrinsic-table or Object-prototype entries in the
nonzero case, matching the existing Reflect descriptor allocator. Public
`Object` and `TypeError` bindings do not participate in intrinsic selection.

Descriptor rejection paths use the existing current-function Realm TypeError
emitter. This includes Proxy false results, Arguments index and `callee`
validation, boxed-string indexes, Array length validation and the shared Array
index descriptor validator. Proxy invariant checks and ArraySetLength's
RangeError path already use the executing builtin's Realm. Those paths remain
on their existing implementations.

Public class fields call the canonical definition dispatcher from the class's
execution context. Both the descriptor observed by a returned Proxy receiver
and a failed definition's TypeError therefore belong to the class Realm.
Thrown values from traps propagate unchanged, and failed definitions prevent
later field initializers from running.

## Evidence and verification

Two direct ordinary-source reproducers failed on frozen checkpoint 11,
compiler SHA-256
`aa07baab681ae8c92f754b2603fecdc1a7f0b52f1d06ec71058df84efc792971`:
foreign `Object.defineProperty` exposed an entry-Realm descriptor prototype,
and a false Proxy trap produced an entry-Realm TypeError. Both exited with
code 1 without timing out. Evidence is retained under
`target/failure-review/completed-baseline-20260914/batch11-independent-review-probes`.

Six regressions in `aot_define_property_realm` cover method-Realm descriptors,
partial descriptors and ordered reads, exotic rejection errors, mutable public
constructor bindings, public-field Realm ownership in both directions, and
abrupt initialization. The shared Reflect descriptor test remains a control.
Compilation and execution of the new regressions are pending integration;
these changes do not publish a Test262 count.

Focused refresh commands:

```sh
cargo test -p lila-engine --test aot_define_property_realm -- --test-threads=1
cargo test -p lila-engine --test aot_public_class_fields -- --test-threads=1
cargo test -p lila-engine --test reflect_descriptor_object_realm -- --test-threads=1
cargo test -p lila-aot-wasm --test to_property_descriptor_operation_evidence_structure --test reflect_descriptor_object_realm_structure
```
