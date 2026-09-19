# Legacy accessor definers

`Object.prototype.__defineGetter__` and `__defineSetter__` are ordinary native
methods with length 2 and no constructor protocol. Both return `undefined`.
Their catalog identities, default Object prototype installation, and created
Realm installation use the same builtins.

Each method performs `ToObject(this)`, checks the accessor with `IsCallable`,
and then converts the key once with the string hint. Nullish receivers and
non-callable accessors therefore fail before observable key conversion. Callable
Proxies retain their normal callability semantics, including a revoked callable
Proxy whose eventual invocation would throw.

A private null-prototype descriptor supplies only the selected accessor and true
`enumerable`/`configurable` fields to the canonical Object property-definition
path. The opposite accessor remains absent, preserving an existing accessor on
replacement. Inherited descriptor properties and mutation of the public
`Object.defineProperty` binding cannot affect this internal operation. Symbol
keys are decoded from the backend PropertyKey representation before crossing the
JavaScript builtin argument boundary.

The shared dispatcher owns ordinary and exotic rejection, Proxy invariants and
trap descriptor construction. Primitive boxing, TypeErrors, and Proxy descriptor
objects use the executing definer's Realm, including when a foreign method is
borrowed after that Realm's public constructor bindings change.

The immutable-main replay snapshot at 2026-09-18 20:23:20 UTC contains 18 failing
executions for these two absent methods, nine each. The staged replay list adds
three observed passing Object descriptor controls. These are partial-cohort
observations, not a full-suite status. Root-owned candidate compilation and
execution are required before claiming those failures repaired.

Focused validation targets are `lila-engine --test aot_legacy_accessor_definers`,
the builtin catalog tests in `lila-ir`, and the accessor dependency test in
`lila-aot-wasm` planning. Existing `aot_define_property_realm`,
`reflect_descriptor_object_realm` and `object_builtin_policy_domains_structure`
retain the shared descriptor and lookup behavior.
