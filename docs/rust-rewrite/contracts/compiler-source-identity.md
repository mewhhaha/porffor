# Compiler source identity and retained evaluation

Source update — 2026-10-07; compilation and execution are deferred to the joined
verification checkpoint.

`ScriptLowerer::lower_identifier_name` treats an unbound `BPE` like any other
unbound name. The removed literal-name branch returned `1` without performing
ResolveBinding. Lexical bindings, global properties, getters, deletion and
ReferenceError now retain the ordinary environment path.

The former `lower_static_yield_star_generator_method_call` and its three marker
constants are removed. Ordinary string properties named `$LilaYieldStarGenerator`,
`$LilaYieldStarReturnNonObject` or `$LilaYieldStarThrowNonObject` cannot prove
generator identity. `next`, `return` and `throw` use their actual acquired callee,
receiver and arguments. Real generators continue through their existing typed
generator plans and runtime records.

`static_number_expr` accepts numeric literals, proven immutable global number
identifiers and recursively constant arithmetic. It does not accept any property
access. A receiver named `Number` or `Math` cannot supply a value or suppress a
Get. Exponentiation retains both operand evaluations and the existing runtime
numeric-operation path when a property is involved. This also removes the
incorrect compile-time `Number.MIN_VALUE = f64::MIN_POSITIVE` table entry; the
runtime intrinsic owner already uses `f64::from_bits(1)`.

The authored IR controls in `crates/lila-ir/src/tests/source_identity.rs` preserve
ordinary unbound-name reads, runtime exponentiation operands and actual method
calls on objects containing the former marker properties. The native controls
in `crates/lila-engine/tests/aot_source_identity.rs` cover Script and strict Script:

- Unbound, parameter, lexical, global-accessor and deleted `BPE` bindings.
- Actual method getters, receiver identity, argument side effects, returned
  primitive values and thrown object identity with the former markers present.
- Shadowed Number/Math getters, left/right evaluation before coercion, abrupt
  left operands, literal exponentiation and the smallest positive subnormal.

Run these after the combined workspace type checkpoint, through the existing
resource limiter:

```sh
python3 scripts/limited_verification.py --memory-mib 4096 -- cargo test --locked -p lila-ir --lib source_identity_
python3 scripts/limited_verification.py --memory-mib 4096 -- cargo test --locked -p lila-engine --test aot_source_identity
```

These edits do not establish full compiler source-identity coverage or change
generated conformance results. The accompanying call-fold and intrinsic-prototype
provenance work has separate owners in the same dry batch.

The descriptor and constructor consumers now use that batch's current-property
reader too. `define_property_call.rs` checks the current Object prototype before
claiming an inherited descriptor field is exact. A missing lookup means Unknown:
the shape representation does not distinguish proven absence from an unknown
prototype. Known own descriptor fields remain precise. Unknown inherited fields
retain the existing all-planned-hooks observation and caller-fact invalidation;
possible callable targets remain open for emission.

`call_candidate_analysis.rs` and both prototype observations in
`class_definition.rs` use the same current-property reader. A copied catalogue
prototype cannot justify a constructor or class instance shape after its live
property evidence changes.

The old unrelated-global wrapper test keeps its source and now rejects an exact
Realm-script admission inferred across unproven inherited descriptor fields.
`aot_live_prototype_consumers.rs` separately requires actual successful Realm
wrapper execution, inherited descriptor getter effects and own-field shadowing,
abrupt descriptor conversion, bound-constructor heritage, `super` methods and
`newTarget` prototype getter effects. These source controls remain unrun:

```sh
python3 scripts/limited_verification.py --memory-mib 4096 -- cargo test --locked -p lila-ir --lib descriptor_fields_without_current_inherited_proof_retain_possible_getter_effects
python3 scripts/limited_verification.py --memory-mib 4096 -- cargo test --locked -p lila-ir --lib define_property_with_unproven_inherited_fields_widens_a_wrapper_dependency
python3 scripts/limited_verification.py --memory-mib 4096 -- cargo test --locked -p lila-engine --test aot_live_prototype_consumers
```
