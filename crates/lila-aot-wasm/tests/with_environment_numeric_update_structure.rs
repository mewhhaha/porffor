use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, EnvironmentIdentifierOperationIr, EnvironmentIdentifierResolutionStart, ExprIr,
    NumericUpdateOp, StatementIr, TypedExpr, UpdateReturnMode,
};

fn lower_control(source: &str) -> lila_ir::ProgramIr {
    let parsed = parse(source, ParseOptions::script()).expect("Reference control parses");
    let program = lower(&parsed);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    program
}

fn with_selection(statements: &[StatementIr]) -> Option<&TypedExpr> {
    statements.iter().find_map(|statement| match statement {
        StatementIr::Expression(expression)
            if matches!(&expression.expr, ExprIr::Conditional { .. }) =>
        {
            Some(expression)
        }
        StatementIr::Block(block) => with_selection(&block.statements),
        StatementIr::LexicalBlock(statements) => with_selection(statements),
        _ => None,
    })
}
const REFERENCE_SOURCE: &str = include_str!("../../lila-ir/src/reference.rs");
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
        "#[derive(Debug)]\n#[must_use = \"a with-environment Reference must be consumed by GetValue, PutValue, DeleteBinding, logical assignment, numeric update, or compound assignment\"]\npub(crate) struct WithEnvironmentReferencePlan {"
    ));
    let plan_type = bounded(
        REFERENCE_SOURCE,
        "#[must_use = \"a with-environment Reference must be consumed by GetValue, PutValue, DeleteBinding, logical assignment, numeric update, or compound assignment\"]",
        "\n}\n",
    );
    assert!(!plan_type.contains("Clone"));
    assert!(!plan_type.contains("Copy"));
    assert!(!REFERENCE_SOURCE.contains("impl Clone for WithEnvironmentReferencePlan"));
    assert!(!REFERENCE_SOURCE.contains("impl Copy for WithEnvironmentReferencePlan"));
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
fn with_updates_keep_object_selection_and_the_located_fallback_for_all_four_modes() {
    for (source, expected_operation, expected_return) in [
        (
            "value++",
            NumericUpdateOp::Increment,
            UpdateReturnMode::Postfix,
        ),
        (
            "++value",
            NumericUpdateOp::Increment,
            UpdateReturnMode::Prefix,
        ),
        (
            "value--",
            NumericUpdateOp::Decrement,
            UpdateReturnMode::Postfix,
        ),
        (
            "--value",
            NumericUpdateOp::Decrement,
            UpdateReturnMode::Prefix,
        ),
    ] {
        for (parameters, global_fallback) in [("scope", true), ("scope, value", false)] {
            let program = lower_control(&format!(
                "function mutate({parameters}) {{ with (scope) {{ {source}; }} }}"
            ));
            let function = program
                .script
                .as_ref()
                .expect("script IR")
                .functions
                .iter()
                .find(|function| function.name == "mutate")
                .expect("With owner");
            let ExprIr::Conditional {
                then_expr,
                else_expr,
                ..
            } = &with_selection(&function.body.statements)
                .expect("With HasBinding remains the outer selection")
                .expr
            else {
                unreachable!()
            };
            assert!(
                matches!(&then_expr.expr, ExprIr::MaterializeBinding { .. }),
                "selected object keeps its Get/Put lifecycle"
            );
            if global_fallback {
                let ExprIr::EnvironmentIdentifier(reference) = &else_expr.expr else {
                    panic!("global fallback must retain the Global Record: {else_expr:?}");
                };
                assert_eq!(reference.name, "value");
                assert_eq!(
                    reference.resolution_start(),
                    EnvironmentIdentifierResolutionStart::GlobalEnvironment
                );
                assert!(
                    matches!(&reference.operation, EnvironmentIdentifierOperationIr::Update { operation, return_mode }
                    if *operation == expected_operation && *return_mode == expected_return)
                );
            } else {
                assert!(
                    matches!(&else_expr.expr, ExprIr::UpdateIdentifier { name, op, return_mode, value_kind: lila_ir::NumericUpdateValueKind::Dynamic }
                    if name == "value" && *op == expected_operation && *return_mode == expected_return)
                );
            }
        }
    }
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
    let contract_words = CONTRACT.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(contract_words.contains("loses its static `proven_present` fact"));
}
