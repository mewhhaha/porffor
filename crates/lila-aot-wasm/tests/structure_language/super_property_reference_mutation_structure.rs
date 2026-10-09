const REFERENCE_SOURCE: &str = include_str!("../../../lila-ir/src/reference.rs");
const IR_SOURCE: &str = include_str!("../../../lila-ir/src/ir.rs");
const LOWERING_SOURCE: &str = include_str!("../../../lila-ir/src/lowering.rs");
const SUPER_LOWERING_SOURCE: &str =
    include_str!("../../../lila-ir/src/lowering/super_property_mutation.rs");
const ASSIGNMENT_LOWERING_SOURCE: &str =
    include_str!("../../../lila-ir/src/lowering/assignment.rs");
const EARLY_ERRORS_SOURCE: &str = include_str!("../../../lila-ir/src/early_errors.rs");
const EXPRESSIONS_SOURCE: &str = include_str!("../../src/expressions.rs");
const SUPER_PROPERTY_MUTATION_SOURCE: &str =
    include_str!("../../src/expressions/super_property_mutation.rs");
const PLANNING_SOURCE: &str = include_str!("../../src/planning.rs");
const DATA_SOURCE: &str = include_str!("../../src/data.rs");
const MATERIALIZER_SOURCE: &str = include_str!("../../../lila-test262/src/lib.rs");
const FIXTURE_SOURCE: &str =
    include_str!("../../../lila-cli/tests/fixtures/wasm_super_property_reference_mutation.js");
const CONTRACT_SOURCE: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/super-property-reference-mutation.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start = source.find(start).expect("bounded source start");
    let tail = &source[start..];
    let end = tail.find(end).expect("bounded source end");
    &tail[..end]
}

fn ordered(source: &str, markers: &[&str]) {
    let mut cursor = 0;
    for marker in markers {
        let next = source[cursor..]
            .find(marker)
            .unwrap_or_else(|| panic!("missing ordered marker {marker}"));
        cursor += next + marker.len();
    }
}

#[test]
fn aot_mutation_lifecycle_has_one_private_file_owner_and_closed_callers() {
    assert_eq!(
        EXPRESSIONS_SOURCE
            .matches("\nmod super_property_mutation;\n")
            .count(),
        1
    );
    assert!(!EXPRESSIONS_SOURCE.contains("pub mod super_property_mutation"));
    for (state, count) in [
        ("EvaluatedRawSuperPropertyReferenceLocals", 7),
        ("CoercedSuperPropertyReferenceLocals", 6),
    ] {
        assert_eq!(SUPER_PROPERTY_MUTATION_SOURCE.matches(state).count(), count);
        assert!(!EXPRESSIONS_SOURCE.contains(state));
    }
    for (transition, count) in [
        ("evaluate_raw_super_property_reference(", 5),
        ("canonicalize_super_property_reference(", 4),
        ("emit_get_value_from_raw_super_property_reference(", 4),
        ("emit_put_value_from_coerced_super_property_reference(", 5),
    ] {
        assert_eq!(
            SUPER_PROPERTY_MUTATION_SOURCE.matches(transition).count(),
            count
        );
        assert!(!EXPRESSIONS_SOURCE.contains(transition));
    }
    for entry in ["read", "write", "mutation"] {
        assert_eq!(
            SUPER_PROPERTY_MUTATION_SOURCE
                .matches(&format!(
                    "pub(super) fn compile_super_property_{entry}_to_value("
                ))
                .count(),
            1
        );
        assert_eq!(
            EXPRESSIONS_SOURCE
                .matches(&format!(".compile_super_property_{entry}_to_value("))
                .count(),
            1
        );
    }
    assert_eq!(
        SUPER_PROPERTY_MUTATION_SOURCE
            .lines()
            .map(str::trim_start)
            .filter(|line| line.starts_with("struct "))
            .collect::<Vec<_>>(),
        [
            "struct EvaluatedRawSuperPropertyReferenceLocals {",
            "struct CoercedSuperPropertyReferenceLocals {"
        ]
    );
}

#[test]
fn ir_owns_one_closed_super_reference_mutation() {
    let mutation = bounded(
        REFERENCE_SOURCE,
        "pub struct SuperPropertyMutationIr {",
        "/// A lowerer-owned Super Property Reference",
    );
    for field in [
        "receiver: Box<TypedExpr>",
        "referenced_name: PropertyKeyIr",
        "strictness: Strictness",
        "operation: SuperPropertyMutationOperationIr",
    ] {
        assert!(mutation.contains(field), "missing private field {field}");
        assert!(!mutation.contains(&format!("pub {field}")));
    }
    assert!(mutation.contains("fn new("));
    assert!(!mutation.contains("pub fn new("));
    for accessor in ["receiver", "referenced_name", "strictness", "operation"] {
        assert!(mutation.contains(&format!("pub fn {accessor}(&self)")));
    }

    let operations = bounded(
        REFERENCE_SOURCE,
        "pub enum SuperPropertyMutationOperationIr {",
        "impl SuperPropertyMutationIr {",
    );
    assert!(operations.contains("NumericUpdate {"));
    assert!(operations.contains("op: NumericUpdateOp"));
    assert!(operations.contains("return_mode: UpdateReturnMode"));
    assert!(operations.contains("value_kind: NumericUpdateValueKind"));
    assert!(operations.contains("EagerCompound {"));
    assert!(operations.contains("old_value_binding: String"));
    assert!(operations.contains("result: Box<TypedExpr>"));
    assert!(!operations.contains("LogicalBinaryOp"));

    assert!(REFERENCE_SOURCE.contains(
        "#[derive(Debug)]\n#[must_use = \"a Super Property Reference plan must be consumed by one mutation\"]\npub(crate) struct SuperPropertyReferencePlan"
    ));
    let plan = bounded(
        REFERENCE_SOURCE,
        "pub(crate) struct SuperPropertyReferencePlan {",
        "/// A Super Reference whose only consumer is the `delete` operator.",
    );
    assert!(!plan.contains("impl Clone for SuperPropertyReferencePlan"));
    assert!(!plan.contains("impl Copy for SuperPropertyReferencePlan"));
    assert!(plan.contains("pub(crate) fn numeric_update(\n        self,"));
    assert!(plan.contains("pub(crate) fn eager_compound_assignment(\n        self,"));
    assert!(plan.contains(
        "old_value_binding: String,\n        op: EagerCompoundAssignmentOp,\n        rhs: TypedExpr,"
    ));
    assert!(!plan.contains("FnOnce"));
    ordered(
        plan,
        &[
            "ExprIr::Identifier(old_value_binding.clone())",
            "let result = op.apply(old_value, rhs);",
            "SuperPropertyMutationOperationIr::EagerCompound {",
        ],
    );
    assert!(plan.contains("ExprIr::SuperPropertyMutation(SuperPropertyMutationIr::new("));
    assert!(IR_SOURCE.contains("SuperPropertyMutation(SuperPropertyMutationIr),"));
}

#[test]
fn lowering_intercepts_super_before_generic_update_and_keeps_rhs_in_the_fused_operation() {
    let update = bounded(
        SUPER_LOWERING_SOURCE,
        "pub(super) fn lower_super_property_numeric_update(",
        "pub(super) fn lower_super_property_eager_compound_assignment(",
    );
    for mapping in [
        "UpdateOp::IncrementPost => (NumericUpdateOp::Increment, UpdateReturnMode::Postfix)",
        "UpdateOp::IncrementPre => (NumericUpdateOp::Increment, UpdateReturnMode::Prefix)",
        "UpdateOp::DecrementPost => (NumericUpdateOp::Decrement, UpdateReturnMode::Postfix)",
        "UpdateOp::DecrementPre => (NumericUpdateOp::Decrement, UpdateReturnMode::Prefix)",
    ] {
        assert!(update.contains(mapping));
    }
    assert!(update.contains("plan.numeric_update(op, return_mode, value_kind)"));

    let compound = bounded(
        SUPER_LOWERING_SOURCE,
        "pub(super) fn lower_super_property_eager_compound_assignment(",
        "\n    }\n}",
    );
    ordered(
        compound,
        &[
            "self.lower_super_property_reference_plan(access)",
            "let rhs = self.lower_expression(rhs);",
            "plan.eager_compound_assignment(old_value_binding, op, rhs)",
        ],
    );

    let property_update = bounded(
        LOWERING_SOURCE,
        "fn lower_property_access_update(",
        "fn lower_update(",
    );
    ordered(
        property_update,
        &[
            "match access {",
            "PropertyAccess::Simple(access) =>",
            "self.lower_ordinary_property_numeric_update(op, access)",
            "PropertyAccess::Super(access) => self.lower_super_property_numeric_update(op, access)",
            "PropertyAccess::Private(access) => self.lower_private_numeric_update(op, access)",
        ],
    );
    assert!(!property_update.contains("_ =>"));
    assert!(
        ASSIGNMENT_LOWERING_SOURCE
            .matches(".lower_super_property_eager_compound_assignment(")
            .count()
            >= 2
    );
}

#[test]
fn aot_typestate_forces_one_key_coercion_get_and_putvalue() {
    assert!(SUPER_PROPERTY_MUTATION_SOURCE.contains(
        "#[must_use = \"raw Super Reference operands must enter GetValue or PutValue\"]"
    ));
    assert!(SUPER_PROPERTY_MUTATION_SOURCE
        .contains("#[must_use = \"a canonical Super Reference must be consumed by PutValue\"]"));
    assert!(!SUPER_PROPERTY_MUTATION_SOURCE.contains("#[derive"));
    let evaluate = bounded(
        SUPER_PROPERTY_MUTATION_SOURCE,
        "fn evaluate_raw_super_property_reference(",
        "fn canonicalize_super_property_reference(",
    );
    ordered(
        evaluate,
        &[
            "self.compile_expr_to_value(receiver,",
            "self.compile_raw_property_key_expression_to_value(",
            "self.emit_load_super_base(&base, function)?;",
            "Ok(EvaluatedRawSuperPropertyReferenceLocals {",
        ],
    );
    assert!(!evaluate.contains("emit_value_to_property_key_locals"));
    assert!(!evaluate.contains("emit_throw_if_null_super_base"));
    let canonicalize = bounded(
        SUPER_PROPERTY_MUTATION_SOURCE,
        "fn canonicalize_super_property_reference(",
        "fn emit_get_value_from_raw_super_property_reference(",
    );
    ordered(
        canonicalize,
        &[
            "let EvaluatedRawSuperPropertyReferenceLocals {",
            "self.emit_throw_if_null_super_base(&base, function)?;",
            "self.emit_value_to_property_key_locals(",
            "referenced_name.clear(function);",
            "Ok(CoercedSuperPropertyReferenceLocals {",
        ],
    );
    assert_eq!(
        canonicalize
            .matches("emit_value_to_property_key_locals(")
            .count(),
        1
    );
    let get = bounded(
        SUPER_PROPERTY_MUTATION_SOURCE,
        "fn emit_get_value_from_raw_super_property_reference(",
        "fn emit_put_value_from_coerced_super_property_reference(",
    );
    ordered(
        get,
        &[
            "self.canonicalize_super_property_reference(raw, function)?;",
            "crate::runtime_helpers::ObjectReadArguments::new(",
            "self.emit_propagate_current_throw_if_needed(function);",
            "old.copy_from(self.completion().value(), function);",
        ],
    );
    let put = bounded(
        SUPER_PROPERTY_MUTATION_SOURCE,
        "fn emit_put_value_from_coerced_super_property_reference(",
        "pub(super) fn compile_super_property_read_to_value(",
    );
    assert!(put.contains("reference: CoercedSuperPropertyReferenceLocals,"));
    ordered(
        put,
        &[
            "crate::runtime_helpers::OrdinarySetArguments::new(",
            "self.emit_propagate_current_throw_if_needed(function);",
            "if strictness.throws_on_failed_set()",
            "RuntimeErrorMessage::CANNOT_ASSIGN_TO_SUPER_PROPERTY,",
            "reference.clear(function);",
        ],
    );
    assert!(!put.contains("emit_value_to_property_key_locals"));
    assert!(!put.contains("emit_load_super_base"));
    let clear = bounded(
        SUPER_PROPERTY_MUTATION_SOURCE,
        "impl CoercedSuperPropertyReferenceLocals {",
        "impl FunctionBuilder<'_> {",
    );
    ordered(
        clear,
        &[
            "fn clear(self,",
            "self.property_key.clear(function);",
            "self.receiver.clear(function);",
            "self.base.clear(function);",
        ],
    );
}

#[test]
fn fused_consumer_publishes_results_only_after_putvalue() {
    let body = bounded(
        SUPER_PROPERTY_MUTATION_SOURCE,
        "pub(super) fn compile_super_property_mutation_to_value(",
        "fn compile_super_property_capture(",
    );
    ordered(
        body,
        &[
            "let raw = self.evaluate_raw_super_property_reference(",
            "self.emit_get_value_from_raw_super_property_reference(raw, &old, function)?;",
            "match mutation.operation()",
        ],
    );
    let numeric = bounded(body, "            SuperPropertyMutationOperationIr::NumericUpdate {\n                op,",
        "            SuperPropertyMutationOperationIr::EagerCompound {\n                old_value_binding,");
    ordered(
        numeric,
        &[
            "self.emit_numeric_reference_old_value(",
            "self.emit_numeric_update_to_locals(",
            "self.emit_put_value_from_coerced_super_property_reference(",
            "output.copy_from(",
        ],
    );
    assert!(numeric.contains("UpdateReturnMode::Prefix => &new"));
    assert!(numeric.contains("UpdateReturnMode::Postfix => &old"));
    let eager = &body[body
        .find(
            "SuperPropertyMutationOperationIr::EagerCompound {\n                old_value_binding,",
        )
        .unwrap()..];
    ordered(
        eager,
        &[
            "self.retain_expression_operand(",
            "self.compile_expr_to_value(result, &new, function)",
            "compiled?;",
            "self.emit_put_value_from_coerced_super_property_reference(",
            "output.copy_from(&new, function);",
        ],
    );
    let write = bounded(
        SUPER_PROPERTY_MUTATION_SOURCE,
        "pub(super) fn compile_super_property_write_to_value(",
        "pub(super) fn compile_super_property_mutation_to_value(",
    );
    ordered(
        write,
        &[
            "self.evaluate_raw_super_property_reference(",
            "self.compile_expr_to_value(value, &rhs, function)?;",
            "self.canonicalize_super_property_reference(raw, function)?;",
            "self.emit_put_value_from_coerced_super_property_reference(",
            "output.copy_from(&rhs, function);",
        ],
    );
    for operation in ["Capture", "PutCaptured"] {
        assert!(body.contains(&format!("SuperPropertyMutationOperationIr::{operation}")));
        assert!(PLANNING_SOURCE.contains(&format!("SuperPropertyMutationOperationIr::{operation}")));
    }
    assert!(body.contains("new.clear(function);"));
    assert!(body.contains("old.clear(function);"));
}

#[test]
fn exhaustive_consumers_and_exact_evidence_inventory_remain_visible() {
    for source in [
        IR_SOURCE,
        SUPER_LOWERING_SOURCE,
        REFERENCE_SOURCE,
        EARLY_ERRORS_SOURCE,
        EXPRESSIONS_SOURCE,
        SUPER_PROPERTY_MUTATION_SOURCE,
        PLANNING_SOURCE,
        DATA_SOURCE,
    ] {
        assert!(source.contains("SuperPropertyMutation"));
    }
    assert!(!LOWERING_SOURCE.contains("SuperPropertyMutation"));
    for source in [
        REFERENCE_SOURCE,
        EARLY_ERRORS_SOURCE,
        SUPER_PROPERTY_MUTATION_SOURCE,
        PLANNING_SOURCE,
        DATA_SOURCE,
    ] {
        assert!(source.contains("SuperPropertyMutationOperationIr::NumericUpdate"));
        assert!(source.contains("SuperPropertyMutationOperationIr::EagerCompound"));
    }

    let cases = [
        "language/expressions/super/prop-expr-getsuperbase-before-topropertykey-putvalue-increment.js",
        "language/expressions/super/prop-expr-uninitialized-this-putvalue-increment.js",
        "language/expressions/super/prop-expr-uninitialized-this-putvalue-compound-assign.js",
        "language/expressions/super/prop-expr-getsuperbase-before-topropertykey-putvalue-compound-assign.js",
    ];
    for case in cases {
        assert!(CONTRACT_SOURCE.contains(case));
        assert!(!MATERIALIZER_SOURCE.contains(case));
    }
    assert!(CONTRACT_SOURCE.contains("reported `2/8`"));
    assert!(CONTRACT_SOURCE.contains("near-HEAD measurements"));
    assert!(CONTRACT_SOURCE.contains("logical assignment through a Super Reference"));

    for oracle in [
        "compoundTrace === \"key,getA,rhs,setA:3:true\"",
        "prefixTrace === \"key,getA,setA:2:true\"",
        "coercions === 2",
        "strictFailureResult === \"not published\"",
        "uninitializedUpdateTrace === \"\"",
        "uninitializedTrace === \"\"",
    ] {
        assert!(FIXTURE_SOURCE.contains(oracle));
    }
    let strict_failure = bounded(
        FIXTURE_SOURCE,
        "var strictFailureMethod = {",
        "Object.setPrototypeOf(strictFailureMethod, lockedBase);",
    );
    ordered(
        strict_failure,
        &["update() {", "\"use strict\";", "return super.locked++;"],
    );
    for mode in [
        "numberPostIncrement",
        "numberPrefixIncrement",
        "numberPostDecrement",
        "numberPrefixDecrement",
        "bigintPostIncrement",
        "bigintPrefixIncrement",
        "bigintPostDecrement",
        "bigintPrefixDecrement",
    ] {
        assert!(FIXTURE_SOURCE.contains(mode));
    }
}
