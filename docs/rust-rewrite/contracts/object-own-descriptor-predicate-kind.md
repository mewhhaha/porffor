# Object own-descriptor predicate kind

Status: capability hardening implemented and focused-verified in Batch X.

## Closed domain

`Object.hasOwn`, `Object.prototype.hasOwnProperty` and
`Object.prototype.propertyIsEnumerable` have one private
`builtins/object/own_descriptor_predicate.rs` owner. Its shared compiler accepts
only the capability-free `OwnDescriptorPredicateBuiltin` domain:

- `ObjectHasOwn`;
- `PrototypeHasOwnProperty`; and
- `PrototypePropertyIsEnumerable`.

Each builtins-visible semantic compiler wrapper constructs exactly its matching
kind. The raw domain and shared compiler are child-private, so the Object parent
cannot name, construct, import or project the policy. The shared compiler owns
that value and borrows it through three borrowed exhaustive decisions:
receiver/argument acquisition, coercion and nullish-error order, and Boolean
result projection. The type implements no clone, copy, debug, default,
comparison, ordering or hashing capability. A copied policy, equality shortcut
or independently chosen result projection therefore cannot silently split
those decisions.

## Preserved semantics

`Object.hasOwn` still acquires its receiver and key from arguments zero and one,
checks the receiver before object coercion, then converts the key. The two
prototype methods still acquire `this` and argument zero; `hasOwnProperty`
converts the key before its nullish receiver check, as required, and
`propertyIsEnumerable` retains the same order. All three still call the shared
own-property-descriptor builtin. The first two project descriptor presence;
only `propertyIsEnumerable` reads the descriptor's enumerable field.

The marker-bounded instruction-emitting compiler body, after replacing each
borrowed `match &builtin` token with its former `match builtin` token, retains
SHA-256
`320062e113be88c36172a2f864dae434f563a56fe9c4d663cc7a8571c719be02`.
The three unchanged marker-bounded wrapper selections retain SHA-256
`54c807ccd77513cc4b2e65e460f7df84d87efd2231d0d37e59593a794c88edba`.
The moved five-line domain and 191-line raw compiler retain SHA-256
`36ed9747dec1c589dd32f763a7bc907fc84d3070988bd0fef7641b08e6138098`
and `05f279033ab151a2c156cdb76ca0da20ad330d953c9ccae8ea055b9d9fbce4a1`.
After the required equivalent wrapper visibility spelling, the resulting
230-line child has SHA-256
`f4db50dd3eb3ba382999dec0dfd9fc578253de1328bbaf41c2a48a0b73b827ba`.

## Durable evidence

`object_own_descriptor_predicate_kind_structure.rs` recursively pins the
private child and parent exclusion, exact private three-variant domain, absence
of derived and manual incidental capabilities, one matching producer in each
builtins-visible wrapper, the three standard dispatcher mappings, and the same
owned authority crossing exactly three borrowed exhaustive decisions without
clone, equality, Boolean or catch-all escape.

The existing `wasm_object_own_descriptor_predicates.js` fixture distinguishes
all three policies across ordinary objects, arrays, strings, Symbols and Proxy
descriptor traps. Its exact CLI owner is
`object::run_wasm_backend_succeeds_for_object_own_descriptor_predicates`.

## Verification boundary and nonclaims

At the 2026-08-28 Batch X checkpoint, `cargo xc` is green, the four-test
structure target passes `4/4`, and the exact own-descriptor predicate CLI
fixture passes `1/1`. Formatting, diff, module-boundary and task-plan checks are
also green.

Batch AK moved the complete owner source-equivalently. Shared `cargo xc` is
green, the focused structure target passes `4/4`, and the exact CLI fixture
passes `1/1`. No Test262 cohort or semantic golden is required for the
unchanged instruction emitter.

This ownership-only hardening does not change emitted instructions, descriptor
lookup, coercion order, Realm selection, Proxy behavior or published
conformance counts. It does not claim the complete Object or descriptor
Test262 trees; semantic snapshot and broad conformance verification remain
outside this focused checkpoint.
