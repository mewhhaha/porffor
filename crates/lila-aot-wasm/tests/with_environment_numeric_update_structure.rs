const REFERENCE_SOURCE: &str = include_str!("../../lila-ir/src/reference.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_with_environment_numeric_update.js");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/with-environment-numeric-update-reference.md"
);

const VENDORED_WITNESSES: [(&str, &str); 16] = [
    (
        "language/expressions/postfix-increment/S11.3.1_A5_T1.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/postfix-increment/S11.3.1_A5_T1.js"
        ),
    ),
    (
        "language/expressions/postfix-increment/S11.3.1_A5_T2.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/postfix-increment/S11.3.1_A5_T2.js"
        ),
    ),
    (
        "language/expressions/postfix-increment/S11.3.1_A5_T3.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/postfix-increment/S11.3.1_A5_T3.js"
        ),
    ),
    (
        "language/expressions/postfix-decrement/S11.3.2_A5_T1.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/postfix-decrement/S11.3.2_A5_T1.js"
        ),
    ),
    (
        "language/expressions/postfix-decrement/S11.3.2_A5_T2.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/postfix-decrement/S11.3.2_A5_T2.js"
        ),
    ),
    (
        "language/expressions/postfix-decrement/S11.3.2_A5_T3.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/postfix-decrement/S11.3.2_A5_T3.js"
        ),
    ),
    (
        "language/expressions/prefix-increment/S11.4.4_A5_T1.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/prefix-increment/S11.4.4_A5_T1.js"
        ),
    ),
    (
        "language/expressions/prefix-increment/S11.4.4_A5_T2.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/prefix-increment/S11.4.4_A5_T2.js"
        ),
    ),
    (
        "language/expressions/prefix-increment/S11.4.4_A5_T3.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/prefix-increment/S11.4.4_A5_T3.js"
        ),
    ),
    (
        "language/expressions/prefix-decrement/S11.4.5_A5_T1.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/prefix-decrement/S11.4.5_A5_T1.js"
        ),
    ),
    (
        "language/expressions/prefix-decrement/S11.4.5_A5_T2.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/prefix-decrement/S11.4.5_A5_T2.js"
        ),
    ),
    (
        "language/expressions/prefix-decrement/S11.4.5_A5_T3.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/prefix-decrement/S11.4.5_A5_T3.js"
        ),
    ),
    (
        "language/expressions/postfix-increment/operator-x-postfix-increment-calls-putvalue-lhs-newvalue-.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/postfix-increment/operator-x-postfix-increment-calls-putvalue-lhs-newvalue-.js"
        ),
    ),
    (
        "language/expressions/postfix-decrement/operator-x-postfix-decrement-calls-putvalue-lhs-newvalue-.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/postfix-decrement/operator-x-postfix-decrement-calls-putvalue-lhs-newvalue-.js"
        ),
    ),
    (
        "language/expressions/prefix-increment/operator-prefix-increment-x-calls-putvalue-lhs-newvalue-.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/prefix-increment/operator-prefix-increment-x-calls-putvalue-lhs-newvalue-.js"
        ),
    ),
    (
        "language/expressions/prefix-decrement/operator-prefix-decrement-x-calls-putvalue-lhs-newvalue-.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/prefix-decrement/operator-prefix-decrement-x-calls-putvalue-lhs-newvalue-.js"
        ),
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
fn one_nonempty_noncopy_plan_owns_the_complete_numeric_update() {
    let selection = bounded(
        REFERENCE_SOURCE,
        "pub(crate) struct SelectedWithEnvironmentObjects {",
        "/// One dynamically queried Object Environment Record in ResolveBinding.",
    );
    assert!(selection.contains("innermost: ObjectEnvironmentBindingObject"));
    assert!(selection.contains("outer: Vec<ObjectEnvironmentBindingObject>"));

    assert!(REFERENCE_SOURCE.contains(
        "#[derive(Debug)]\n#[must_use = \"a with-environment Reference must be consumed by GetValue, PutValue, logical assignment, numeric update, or compound assignment\"]\npub(crate) struct WithEnvironmentReferencePlan {"
    ));
    let plan_type = bounded(
        REFERENCE_SOURCE,
        "#[must_use = \"a with-environment Reference must be consumed by GetValue, PutValue, logical assignment, numeric update, or compound assignment\"]",
        "/// One identifier Reference selected by the Global Environment Record's",
    );
    assert!(!plan_type.contains("Clone"));
    assert!(!plan_type.contains("Copy"));
    let plan_consumer = bounded(
        REFERENCE_SOURCE,
        "impl WithEnvironmentReferencePlan {",
        "/// `[[Strict]]` of a Reference Record (6.2.5).",
    );
    assert!(plan_consumer.contains("pub(crate) fn numeric_update("));
    assert!(plan_consumer.contains("op: NumericUpdateOp"));
    assert!(plan_consumer.contains("return_mode: UpdateReturnMode"));
    assert!(plan_consumer.contains("bindings: NumericUpdateBindings"));
    assert!(plan_consumer.contains("for environment in outer"));
    assert!(plan_consumer.contains("innermost.numeric_update_or_else("));
    assert!(!plan_consumer.contains("ExprIr::PropertyUpdate"));

    let bindings = bounded(
        REFERENCE_SOURCE,
        "pub(crate) struct NumericUpdateBindings {",
        "impl WithEnvironmentReferencePlan {",
    );
    assert!(bindings.contains("old_value: String"));
    assert!(bindings.contains("result: String"));
    assert!(bindings.contains("write: String"));
    assert!(bindings.contains("pub(crate) fn allocate("));
    assert_before(
        bindings,
        "allocate(\"object.environment.update.old.\")",
        "allocate(\"object.environment.update.result.\")",
    );
    assert_before(
        bindings,
        "allocate(\"object.environment.update.result.\")",
        "allocate(\"object.environment.update.write.\")",
    );
}

#[test]
fn selected_branch_orders_get_numeric_delta_put_and_result() {
    let selection = bounded(
        REFERENCE_SOURCE,
        "    fn numeric_update_or_else(",
        "impl SelectedWithEnvironmentObjects {",
    );

    for marker in [
        "let binding_visible = binding_object.binding_visible(",
        "binding_object.numeric_update(",
        "condition: Box::new(binding_visible)",
        "then_expr: Box::new(selected_update)",
        "else_expr: Box::new(fallback)",
    ] {
        assert!(
            selection.contains(marker),
            "missing selection boundary: {marker}"
        );
    }
    assert_before(selection, "let binding_visible =", "let selected_update =");

    let objects = bounded(
        REFERENCE_SOURCE,
        "impl ObjectEnvironmentBindingObject {",
        "/// Declarative-frame depth in the function currently being lowered.",
    );
    let update = bounded(
        objects,
        "    fn numeric_update(",
        "    /// GetValue, eager operation, same-base PutValue, then result.",
    );
    for marker in [
        "let NumericUpdateBindings {",
        "old_value: old_value_name,\n            result: result_name,\n            write: write_name,",
        "let old_value = self.clone().get_value(referenced_name, strictness);",
        "ExprIr::UpdateIdentifier {",
        "let updated_value = TypedExpr::from_info(",
        "let write = self.put_value(referenced_name, strictness, updated_value);",
        "let result = TypedExpr::from_info(",
        "name: write_name.clone()",
        "name: result_name.clone()",
        "name: old_value_name.clone()",
    ] {
        assert!(update.contains(marker), "missing update boundary: {marker}");
    }
    assert_before(update, "let old_value =", "let update =");
    assert_before(update, "let update =", "let updated_value =");
    assert_before(update, "let updated_value =", "let write = self.put_value");
    assert_before(
        update,
        "let write = self.put_value",
        "let result = TypedExpr::from_info",
    );
    assert_before(
        update,
        "let result = TypedExpr::from_info",
        "let after_write =",
    );
    assert_before(update, "let after_write =", "let after_update =");
    assert!(!update.contains("ExprIr::PropertyUpdate"));

    let get = bounded(
        REFERENCE_SOURCE,
        "    fn get_value(self, referenced_name: &str, strictness: Strictness) -> TypedExpr {",
        "    /// SetMutableBinding on the Object Environment Record selected before RHS.",
    );
    assert!(get.contains("let recheck = self.has_property(referenced_name);"));
    assert_before(get, "let recheck", "ExprIr::PropertyRead");

    let put = bounded(
        REFERENCE_SOURCE,
        "    fn put_value(",
        "/// Declarative-frame depth in the function currently being lowered.",
    );
    assert!(put.contains("let recheck = self.has_property(referenced_name);"));
    assert!(put.contains("Strictness::Strict"));
    assert!(put.contains("name: NativeErrorKind::ReferenceError"));
    assert_before(
        put,
        "name: OBJECT_ENVIRONMENT_VALUE_BINDING",
        "body: Box::new(after_recheck)",
    );
}

#[test]
fn lowering_spends_the_plan_for_all_four_closed_update_forms() {
    let reachability = bounded(
        LOWERING_SOURCE,
        "enum IdentifierUpdateReachability {",
        "impl LocatedIdentifierReference {",
    );
    assert!(reachability.contains("Definite"));
    assert!(reachability.contains("WithEnvironmentFallback"));
    assert!(!reachability.contains("_ =>"));

    let update = bounded(
        LOWERING_SOURCE,
        "    fn lower_update(&mut self, op: UpdateOp, target: &UpdateTarget) -> TypedExpr {",
        "    fn lower_unary(&mut self, op: UnaryOp, target: &Expression) -> TypedExpr {",
    );

    for mapping in [
        "UpdateOp::IncrementPost => (NumericUpdateOp::Increment, UpdateReturnMode::Postfix)",
        "UpdateOp::IncrementPre => (NumericUpdateOp::Increment, UpdateReturnMode::Prefix)",
        "UpdateOp::DecrementPost => (NumericUpdateOp::Decrement, UpdateReturnMode::Postfix)",
        "UpdateOp::DecrementPre => (NumericUpdateOp::Decrement, UpdateReturnMode::Prefix)",
    ] {
        assert!(
            update.contains(mapping),
            "missing closed update mapping: {mapping}"
        );
    }
    assert!(update.contains(".with_environment_chain"));
    assert!(update.contains(".select_preceding("));
    assert!(update.contains("self.with_environment_reference_plan("));
    assert!(update.contains("NumericUpdateBindings::allocate("));
    assert!(update.contains("self.alloc_temp_binding_name(prefix)"));
    assert!(update.contains("plan.numeric_update("));
    assert!(update.contains("IdentifierUpdateReachability::WithEnvironmentFallback"));
    assert!(update.contains("IdentifierUpdateReachability::Definite"));
    assert!(!update.contains("_ =>"));

    let fallback = bounded(
        LOWERING_SOURCE,
        "    fn lower_located_identifier_numeric_update(",
        "    fn lower_unary(&mut self, op: UnaryOp, target: &Expression) -> TypedExpr {",
    );
    assert_eq!(
        fallback
            .matches("IdentifierUpdateReachability::WithEnvironmentFallback =>")
            .count(),
        8,
    );
    assert_eq!(
        fallback.matches("NumericUpdateValueKind::Dynamic").count(),
        7,
    );
    assert_eq!(
        fallback.matches("widen_for_possible_replacement()").count(),
        3,
    );
    assert!(!fallback.contains("self.merge_value_infos("));
    assert!(fallback
        .contains("if reachability == IdentifierUpdateReachability::WithEnvironmentFallback"));
    assert!(fallback.contains("if let Some(info) = self.global_properties.get_mut(&name)"));
    assert!(fallback.contains("info.value_info.widen_for_possible_replacement();"));
    assert!(fallback.contains("if info.configurable"));
    assert!(fallback.contains("info.proven_present = false;"));
    assert_before(
        fallback,
        "info.value_info.widen_for_possible_replacement();",
        "info.proven_present = false;",
    );
    assert_eq!(fallback.matches("ExprIr::RuntimeThrow {").count(), 1);
    assert_eq!(
        fallback
            .matches("name: NativeErrorKind::ReferenceError")
            .count(),
        1,
    );
    assert_eq!(
        fallback
            .matches("message: \"unbound identifier in with scope\"")
            .count(),
        1,
    );
    let proven_global = bounded(
        fallback,
        "        } else if self.global_property_is_proven_present(&name) {",
        "        } else {\n            match reachability {",
    );
    assert!(proven_global.contains("(None, NumericUpdateValueKind::Dynamic)"));
    assert!(!proven_global.contains("RuntimeThrow"));

    let guarded_global = bounded(
        LOWERING_SOURCE,
        "        let strictness = self.reference_strictness();\n        if let Some(storage_name) = binding_storage_name {",
        "    fn lower_unary(&mut self, op: UnaryOp, target: &Expression) -> TypedExpr {",
    );
    assert!(guarded_global.contains("ExprIr::GlobalPropertyUpdate"));
    assert!(guarded_global.contains("IdentifierUpdateReachability::Definite => update"));
    assert!(guarded_global.contains("IdentifierUpdateReachability::WithEnvironmentFallback => {"));
    assert!(guarded_global.contains("let present = TypedExpr::spec_has_property("));
    assert!(guarded_global.contains("ExprIr::Identifier(GLOBAL_THIS_NAME.to_string())"));
    assert!(guarded_global.contains("ExprIr::RuntimeThrow {"));
    assert_before(
        guarded_global,
        "let present = TypedExpr::spec_has_property(",
        "ExprIr::Conditional {",
    );
    assert_before(
        guarded_global,
        "condition: Box::new(present)",
        "then_expr: Box::new(update)",
    );
    assert_before(
        guarded_global,
        "then_expr: Box::new(update)",
        "else_expr: Box::new(missing)",
    );
    assert!(!fallback.contains("_ =>"));
}

#[test]
fn consumer_and_exact_current_pin_inventory_cover_the_durable_contract() {
    for marker in [
        "result = ++functionValue",
        "globalPostfixResult = globalPostfixValue++",
        "nestedPrefixResult = --nestedPrefixValue",
        "result = functionValue--",
        "trace === \"huhgdhs\"",
        "mutatedFallbackValue = 2n",
        "mutatedFallbackResult === 2n",
        "mutatedFallbackValue === 3n",
        "selectedFallbackValue = \"mutated\"",
        "selectedFallbackValue === \"mutated\"",
        "throwingFallbackValue = throwingReplacement",
        "throwingFallbackValue === throwingReplacement",
        "delete globalThis.deletedFallbackValue",
        "deletedFallbackCaught = error instanceof ReferenceError",
        "!(\"deletedFallbackValue\" in globalThis)",
        "deletedFallbackType === \"undefined\"",
        "globalThis.createdFallbackValue = 4",
        "createdFallbackResult = createdFallbackValue++",
        "createdFallbackResult === 4",
        "globalThis.createdFallbackValue === 5",
        "strictCaught === 4",
    ] {
        assert!(FIXTURE.contains(marker), "missing CLI witness: {marker}");
    }

    assert_eq!(VENDORED_WITNESSES.len(), 16);
    for (path, source) in VENDORED_WITNESSES {
        assert!(
            source.contains("flags: [noStrict]"),
            "wrong metadata: {path}"
        );
        assert!(source.contains("with ("), "missing Object ER use: {path}");
        assert!(source.contains("delete this.x"), "missing deletion: {path}");
        assert!(
            CONTRACT.contains(path),
            "missing contract inventory: {path}"
        );
    }
    for (_, source) in VENDORED_WITNESSES.into_iter().skip(12) {
        assert!(source.contains("assert.throws(ReferenceError"));
        assert!(source.contains("\"use strict\""));
    }
    assert!(CONTRACT.contains("16 `noStrict` files"));
    assert!(CONTRACT.contains("Strict *references created by a"));
    assert!(CONTRACT.contains("nested function* are in scope"));
    assert!(CONTRACT.contains("unscopables-inc-dec.js"));
    assert!(CONTRACT.contains("Property-reference updates, compound assignment"));
    assert!(CONTRACT.contains("Static post-expression metadata becomes fully Dynamic"));
    assert!(CONTRACT.contains("loses its static `proven_present` fact"));
}
