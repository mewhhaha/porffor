const REFERENCE_SOURCE: &str = include_str!("../../lila-ir/src/reference.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_global_object_environment_numeric_update.js");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/global-object-environment-numeric-update-reference.md"
);

macro_rules! witness {
    ($path:literal) => {
        (
            $path,
            include_str!(concat!("../../../test262/vendor/test262/test/", $path)),
        )
    };
}

const SELECTED_WITNESSES: [(&str, &str); 4] = [
    witness!(
        "language/expressions/prefix-increment/operator-prefix-increment-x-calls-putvalue-lhs-newvalue--1.js"
    ),
    witness!(
        "language/expressions/prefix-decrement/operator-prefix-decrement-x-calls-putvalue-lhs-newvalue--1.js"
    ),
    witness!(
        "language/expressions/postfix-increment/operator-x-postfix-increment-calls-putvalue-lhs-newvalue--1.js"
    ),
    witness!(
        "language/expressions/postfix-decrement/operator-x-postfix-decrement-calls-putvalue-lhs-newvalue--1.js"
    ),
];

const WITH_REGRESSION_WITNESSES: [(&str, &str); 4] = [
    witness!(
        "language/expressions/prefix-increment/operator-prefix-increment-x-calls-putvalue-lhs-newvalue-.js"
    ),
    witness!(
        "language/expressions/prefix-decrement/operator-prefix-decrement-x-calls-putvalue-lhs-newvalue-.js"
    ),
    witness!(
        "language/expressions/postfix-increment/operator-x-postfix-increment-calls-putvalue-lhs-newvalue-.js"
    ),
    witness!(
        "language/expressions/postfix-decrement/operator-x-postfix-decrement-calls-putvalue-lhs-newvalue-.js"
    ),
];

const GLOBAL_COMPOUND_REGRESSION_WITNESSES: [(&str, &str); 11] = [
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--1.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--3.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--5.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--7.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--9.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--11.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--13.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--15.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--17.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--19.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--21.js"
    ),
];

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier = source.find(earlier).expect("earlier operation");
    let later = source.find(later).expect("later operation");
    assert!(earlier < later, "`{earlier}` must precede `{later}`");
}

#[test]
fn one_private_fixed_role_carrier_drives_the_shared_numeric_lifecycle() {
    let carrier = bounded(
        REFERENCE_SOURCE,
        "#[derive(Debug)]\npub(crate) struct NumericUpdateBindings {",
        "/// Compiler-private bindings for one eager Object Environment compound",
    );
    for marker in ["old_value: String", "result: String", "write: String"] {
        assert!(carrier.contains(marker), "missing fixed role: {marker}");
    }
    assert!(!carrier.contains("pub(crate) old_value"));
    assert!(!carrier.contains("Clone"));
    assert!(!carrier.contains("Copy"));

    let allocator = bounded(
        REFERENCE_SOURCE,
        "impl NumericUpdateBindings {",
        "impl WithEnvironmentReferencePlan {",
    );
    assert_before(
        allocator,
        "allocate(\"object.environment.update.old.\")",
        "allocate(\"object.environment.update.result.\")",
    );
    assert_before(
        allocator,
        "allocate(\"object.environment.update.result.\")",
        "allocate(\"object.environment.update.write.\")",
    );

    let objects = bounded(
        REFERENCE_SOURCE,
        "impl ObjectEnvironmentBindingObject {",
        "/// Declarative-frame depth in the function currently being lowered.",
    );
    let lifecycle = bounded(
        objects,
        "    fn numeric_update(",
        "    /// GetValue, eager operation, same-base PutValue, then result.",
    );
    for marker in [
        "let NumericUpdateBindings {",
        "let old_value = self.clone().get_value(referenced_name, strictness);",
        "ExprIr::UpdateIdentifier {",
        "value_kind: NumericUpdateValueKind::Dynamic",
        "let write = self.put_value(referenced_name, strictness, updated_value);",
        "name: write_name.clone()",
        "name: result_name.clone()",
        "name: old_value_name.clone()",
    ] {
        assert!(
            lifecycle.contains(marker),
            "missing shared lifecycle marker: {marker}"
        );
    }
    assert_before(lifecycle, "let old_value =", "let update =");
    assert_before(lifecycle, "let update =", "let updated_value =");
    assert_before(lifecycle, "let updated_value =", "let write =");
    assert_before(lifecycle, "let write =", "let result =");
    assert_before(lifecycle, "let result =", "let after_write =");
    assert_before(lifecycle, "let after_write =", "let after_update =");
    assert!(!lifecycle.contains("ExprIr::GlobalPropertyUpdate"));
}

#[test]
fn global_plan_selects_plain_has_property_then_consumes_the_same_object_record() {
    let objects = bounded(
        REFERENCE_SOURCE,
        "pub(crate) struct ObjectEnvironmentBindingObject {",
        "/// Declarative-frame depth in the function currently being lowered.",
    );
    for marker in [
        "enum ObjectEnvironmentBindingObjectSource {",
        "Materialized(String)",
        "GlobalObject",
        "pub(crate) fn materialized(",
        "fn global_object(info: ValueInfo) -> Self",
        "ObjectEnvironmentBindingObjectSource::Materialized(storage_name)",
        "ObjectEnvironmentBindingObjectSource::GlobalObject",
    ] {
        assert!(objects.contains(marker), "missing object domain: {marker}");
    }

    assert!(REFERENCE_SOURCE.contains(
        "#[derive(Debug)]\n#[must_use = \"a global Object Environment Reference must be consumed by logical assignment, numeric update, or eager compound assignment\"]\npub(crate) struct GlobalObjectEnvironmentReferencePlan {"
    ));
    let plan = bounded(
        REFERENCE_SOURCE,
        "#[must_use = \"a global Object Environment Reference must be consumed by logical assignment, numeric update, or eager compound assignment\"]",
        "/// Compiler-private bindings used by one Object Environment numeric update.",
    );
    assert!(plan.contains("pub(crate) struct GlobalObjectEnvironmentReferencePlan"));
    assert!(plan.contains("ObjectEnvironmentBindingObject::global_object(global_object_info)"));
    assert!(!plan.contains("binding_visible("));
    assert!(!plan.contains("unscopables_binding"));

    let numeric = bounded(plan, "    pub(crate) fn numeric_update(", "\n    }\n}");
    assert!(numeric.contains("bindings: NumericUpdateBindings"));
    assert!(numeric.contains("let present = binding_object.has_property(&referenced_name);"));
    assert!(numeric.contains("binding_object.numeric_update("));
    assert!(numeric.contains("name: NativeErrorKind::ReferenceError"));
    assert!(numeric.contains("condition: Box::new(present)"));
    assert!(numeric.contains("then_expr: Box::new(selected)"));
    assert!(numeric.contains("else_expr: Box::new(missing)"));
    assert_before(numeric, "let present =", "let selected =");
    assert_before(numeric, "let selected =", "let missing =");

    let object_impl = bounded(
        REFERENCE_SOURCE,
        "impl ObjectEnvironmentBindingObject {",
        "/// Declarative-frame depth in the function currently being lowered.",
    );
    let get = bounded(
        object_impl,
        "    fn get_value(self, referenced_name: &str, strictness: Strictness) -> TypedExpr {",
        "    /// SetMutableBinding on the Object Environment Record selected before RHS.",
    );
    let put = bounded(
        object_impl,
        "    fn put_value(",
        "    /// GetValue, ToNumeric/delta, same-base PutValue, then prefix/postfix",
    );
    assert!(get.contains("let recheck = self.has_property(referenced_name);"));
    assert!(get.contains("ExprIr::PropertyRead"));
    assert!(get.contains("Strictness::Sloppy => TypedExpr::undefined()"));
    assert!(get.contains("name: NativeErrorKind::ReferenceError"));
    assert!(put.contains("let recheck = self.has_property(referenced_name);"));
    assert!(put.contains("Strictness::Sloppy => TypedExpr::from_info("));
    assert!(put.contains("Strictness::Strict => TypedExpr::from_info("));
    assert!(put.contains("ExprIr::PropertyWrite"));
    assert!(put.contains("name: NativeErrorKind::ReferenceError"));
}

#[test]
fn lowering_routes_all_four_closed_modes_only_for_unproven_unresolvable_globals() {
    let update = bounded(
        LOWERING_SOURCE,
        "    fn lower_update(&mut self, op: UpdateOp, target: &UpdateTarget) -> TypedExpr {",
        "    fn lower_located_identifier_numeric_update(",
    );
    for mapping in [
        "UpdateOp::IncrementPost => (NumericUpdateOp::Increment, UpdateReturnMode::Postfix)",
        "UpdateOp::IncrementPre => (NumericUpdateOp::Increment, UpdateReturnMode::Prefix)",
        "UpdateOp::DecrementPost => (NumericUpdateOp::Decrement, UpdateReturnMode::Postfix)",
        "UpdateOp::DecrementPre => (NumericUpdateOp::Decrement, UpdateReturnMode::Prefix)",
    ] {
        assert!(update.contains(mapping), "missing closed mode: {mapping}");
    }
    assert!(update.contains("if matches!(&reference, LocatedIdentifierReference::Unresolvable)"));
    assert!(update.contains("&& !self.global_property_is_proven_present(&name)"));
    assert!(update.contains("return self.lower_global_object_environment_numeric_update("));
    assert!(!update.contains("_ =>"));

    let helper = bounded(
        LOWERING_SOURCE,
        "    fn lower_global_object_environment_numeric_update(",
        "    fn lower_located_identifier_numeric_update(",
    );
    for marker in [
        "info.value_info.widen_for_possible_replacement();",
        "info.proven_present = false;",
        "NumericUpdateBindings::allocate(",
        "let strictness = self.reference_strictness();",
        "GlobalObjectEnvironmentReferencePlan::new(self.global_this_info(), name, strictness)",
        ".numeric_update(op, return_mode, bindings)",
    ] {
        assert!(helper.contains(marker), "missing lowering seam: {marker}");
    }
    assert_before(
        helper,
        "info.value_info.widen_for_possible_replacement();",
        "info.proven_present =",
    );
    assert_before(helper, "let bindings =", "let strictness =");
    assert_before(
        helper,
        "let strictness =",
        "GlobalObjectEnvironmentReferencePlan::new(",
    );
    assert!(!helper.contains("ExprIr::GlobalPropertyUpdate"));
}

#[test]
fn durable_consumer_and_exact_current_pin_inventories_bound_the_claim() {
    for (path, source) in SELECTED_WITNESSES {
        assert!(source.contains("flags: [noStrict]"), "{path}");
        assert!(
            source.contains("Object.defineProperty(this, \"x\""),
            "{path}"
        );
        assert!(source.contains("delete this.x;"), "{path}");
        assert!(source.contains("\"use strict\";"), "{path}");
        assert!(source.contains("assert.throws(ReferenceError"), "{path}");
        assert!(CONTRACT.contains(path), "contract omits {path}");
    }
    for (path, source) in WITH_REGRESSION_WITNESSES {
        assert!(source.contains("flags: [noStrict]"), "{path}");
        assert!(source.contains("with (scope)"), "{path}");
    }
    for (path, source) in GLOBAL_COMPOUND_REGRESSION_WITNESSES {
        assert!(source.contains("flags: [noStrict]"), "{path}");
        assert!(CONTRACT.contains("eleven global eager-compound files"));
    }

    for marker in [
        "++globalPrefixNumber",
        "globalPostfixNumber++",
        "--globalPrefixBigInt",
        "globalPostfixBigInt--",
        "++strictPrefixIncrement",
        "--strictPrefixDecrement",
        "strictPostfixIncrement++",
        "strictPostfixDecrement--",
        "sloppyPostfixNumber++",
        "--sloppyPrefixBigInt",
        "++initiallyMissingGlobalUpdate",
        "lifecycleTrace === \"h\"",
        "lifecycleTrace === \"hhgdnhs\"",
        "strictResult === \"not written\"",
    ] {
        assert!(FIXTURE.contains(marker), "fixture omits {marker}");
    }
    assert!(CONTRACT.contains("The plain assignment witness is not part of this cohort"));
    assert!(CONTRACT.contains("Logical assignments have a separate short-circuit lifecycle"));
    assert!(CONTRACT.contains("Broad language and pinned-matrix publication remain"));
}
