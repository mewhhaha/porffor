const REFERENCE_SOURCE: &str = include_str!("../../lila-ir/src/reference.rs");
const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const SUPER_LOWERING_SOURCE: &str =
    include_str!("../../lila-ir/src/lowering/super_property_mutation.rs");
const ASSIGNMENT_LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering/assignment.rs");
const EARLY_ERRORS_SOURCE: &str = include_str!("../../lila-ir/src/early_errors.rs");
const EXPRESSIONS_SOURCE: &str = include_str!("../src/expressions.rs");
const SUPER_PROPERTY_MUTATION_SOURCE: &str =
    include_str!("../src/expressions/super_property_mutation.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const DATA_SOURCE: &str = include_str!("../src/data.rs");
const MATERIALIZER_SOURCE: &str = include_str!("../../lila-test262/src/lib.rs");
const FIXTURE_SOURCE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_super_property_reference_mutation.js");
const CONTRACT_SOURCE: &str =
    include_str!("../../../docs/rust-rewrite/contracts/super-property-reference-mutation.md");

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
    assert!(!EXPRESSIONS_SOURCE.contains("\npub mod super_property_mutation;\n"));
    assert!(!EXPRESSIONS_SOURCE.contains("\nmod super_property_mutation {\n"));
    assert!(SUPER_PROPERTY_MUTATION_SOURCE.starts_with("use super::*;\n\n"));

    for state in [
        "EvaluatedRawSuperPropertyReferenceLocals",
        "CoercedSuperPropertyReferenceLocals",
    ] {
        assert_eq!(
            SUPER_PROPERTY_MUTATION_SOURCE.matches(state).count(),
            5,
            "closed carrier census for `{state}`"
        );
        assert!(
            !EXPRESSIONS_SOURCE.contains(state),
            "parent retained `{state}`"
        );
    }
    assert_eq!(
        SUPER_PROPERTY_MUTATION_SOURCE
            .lines()
            .map(str::trim_start)
            .filter(|line| {
                line.starts_with("struct ")
                    || line.starts_with("enum ")
                    || line.starts_with("pub struct ")
                    || line.starts_with("pub enum ")
                    || line.starts_with("pub(") && line.contains(" struct ")
                    || line.starts_with("pub(") && line.contains(" enum ")
            })
            .collect::<Vec<_>>(),
        [
            "struct EvaluatedRawSuperPropertyReferenceLocals {",
            "struct CoercedSuperPropertyReferenceLocals {",
        ]
    );

    for (transition, expected_count) in [
        ("evaluate_raw_super_property_reference(", 2),
        ("emit_get_value_from_raw_super_property_reference(", 2),
        ("emit_put_value_from_coerced_super_property_reference(", 3),
    ] {
        assert_eq!(
            SUPER_PROPERTY_MUTATION_SOURCE.matches(transition).count(),
            expected_count,
            "closed transition census for `{transition}`"
        );
        assert!(
            !EXPRESSIONS_SOURCE.contains(transition),
            "parent retained `{transition}`"
        );
    }
    assert_eq!(
        SUPER_PROPERTY_MUTATION_SOURCE
            .matches("pub(super) fn compile_super_property_mutation_to_locals(")
            .count(),
        1
    );
    assert!(!SUPER_PROPERTY_MUTATION_SOURCE
        .contains("pub(crate) fn compile_super_property_mutation_to_locals("));
    assert_eq!(
        EXPRESSIONS_SOURCE
            .matches(".compile_super_property_mutation_to_locals(")
            .count(),
        2
    );

    assert_eq!(
        SUPER_PROPERTY_MUTATION_SOURCE
            .lines()
            .map(str::trim_start)
            .filter(
                |line| line.starts_with("fn ") || line.starts_with("pub") && line.contains(" fn ")
            )
            .collect::<Vec<_>>(),
        [
            "fn evaluate_raw_super_property_reference(",
            "fn emit_get_value_from_raw_super_property_reference(",
            "fn emit_put_value_from_coerced_super_property_reference(",
            "pub(super) fn compile_super_property_mutation_to_locals(",
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
            "PropertyAccess::Private(_) => self.unsupported_expr(\"private field update target\")",
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
        "#[derive(Debug)]\n#[must_use = \"a raw Super Property Reference must be consumed by GetValue\"]\nstruct EvaluatedRawSuperPropertyReferenceLocals"
    ));
    assert!(SUPER_PROPERTY_MUTATION_SOURCE.contains(
        "#[derive(Debug)]\n#[must_use = \"a coerced Super Property Reference must be consumed by PutValue\"]\nstruct CoercedSuperPropertyReferenceLocals"
    ));

    let evaluate = bounded(
        SUPER_PROPERTY_MUTATION_SOURCE,
        "fn evaluate_raw_super_property_reference(",
        "fn emit_get_value_from_raw_super_property_reference(",
    );
    ordered(
        evaluate,
        &[
            "self.compile_expr_to_locals(receiver, receiver_payload, receiver_tag, function)?;",
            "self.compile_raw_property_key_expression_to_locals(",
            "self.emit_load_super_base(base_payload, base_tag, function)?;",
            "self.emit_throw_if_null_super_base(base_payload, base_tag, function)?;",
            "Ok(EvaluatedRawSuperPropertyReferenceLocals {",
        ],
    );
    assert!(!evaluate.contains("emit_value_to_property_key_locals"));

    let get_value = bounded(
        SUPER_PROPERTY_MUTATION_SOURCE,
        "fn emit_get_value_from_raw_super_property_reference(",
        "fn emit_put_value_from_coerced_super_property_reference(",
    );
    ordered(
        get_value,
        &[
            "let EvaluatedRawSuperPropertyReferenceLocals {",
            "self.emit_value_to_property_key_locals(",
            "self.emit_object_read_with_key_tag(",
            "Ok(CoercedSuperPropertyReferenceLocals {",
        ],
    );
    assert_eq!(
        get_value
            .matches("emit_value_to_property_key_locals(")
            .count(),
        1
    );
    assert!(!get_value.contains("emit_load_super_base"));

    let put_value = bounded(
        SUPER_PROPERTY_MUTATION_SOURCE,
        "fn emit_put_value_from_coerced_super_property_reference(",
        "fn compile_super_property_mutation_to_locals(",
    );
    ordered(
        put_value,
        &[
            "let CoercedSuperPropertyReferenceLocals {",
            "self.emit_ordinary_set_result_via_helper(",
            "self.with_reference_strictness(strictness, function",
            "emitter.emit_object_write_set_failure_else(\"Cannot assign to super property\"",
            "self.release_temp_local(property_key_tag);",
            "self.release_temp_local(property_key_payload);",
            "self.release_temp_local(receiver_tag);",
            "self.release_temp_local(receiver_payload);",
            "self.release_temp_local(base_tag);",
            "self.release_temp_local(base_payload);",
        ],
    );
    assert!(!put_value.contains("emit_value_to_property_key_locals"));
    assert!(!put_value.contains("emit_load_super_base"));
}

#[test]
fn fused_consumer_publishes_results_only_after_putvalue() {
    let body = bounded(
        SUPER_PROPERTY_MUTATION_SOURCE,
        "pub(super) fn compile_super_property_mutation_to_locals(",
        "\n    }\n}\n",
    );
    ordered(
        body,
        &[
            "let raw_reference = self.evaluate_raw_super_property_reference(",
            "let coerced_reference = self.emit_get_value_from_raw_super_property_reference(",
            "match mutation.operation()",
        ],
    );

    let numeric_start = body
        .find("SuperPropertyMutationOperationIr::NumericUpdate {")
        .expect("numeric operation arm");
    let eager_start = body
        .find("SuperPropertyMutationOperationIr::EagerCompound {")
        .expect("eager operation arm");
    let numeric = &body[numeric_start..eager_start];
    ordered(
        numeric,
        &[
            "self.emit_put_value_from_coerced_super_property_reference(",
            "function.instruction(&Instruction::LocalSet(payload_local));",
            "function.instruction(&Instruction::LocalSet(tag_local));",
        ],
    );
    let eager = &body[eager_start..];
    ordered(
        eager,
        &[
            "self.compile_expr_to_locals(",
            "self.emit_put_value_from_coerced_super_property_reference(",
            "function.instruction(&Instruction::LocalSet(payload_local));",
            "function.instruction(&Instruction::LocalSet(tag_local));",
        ],
    );

    let budget_constants = bounded(
        PLANNING_SOURCE,
        "const SUPER_PROPERTY_MUTATION_PERSISTENT_TEMP_LOCALS",
        "fn count_sync_disposable_resources_temp_locals(",
    );
    assert!(
        budget_constants.contains("SUPER_PROPERTY_MUTATION_PERSISTENT_TEMP_LOCALS: usize = 5 + 6;")
    );
    assert!(budget_constants.contains("SUPER_PROPERTY_MUTATION_GET_VALUE_TEMP_LOCALS: usize = 2;"));
    assert!(budget_constants.contains("SUPER_PROPERTY_MUTATION_TO_NUMERIC_TEMP_LOCALS: usize = 4;"));
    assert!(
        budget_constants.contains("SUPER_PROPERTY_MUTATION_SET_HELPER_TEMP_LOCALS: usize = 4 + 2;")
    );
    let budget = bounded(
        PLANNING_SOURCE,
        "pub(crate) fn count_expr_temp_locals(",
        "pub(crate) fn collect_hoisted_vars_block_root(",
    );
    let mutation_budget = bounded(
        budget,
        "ExprIr::SuperPropertyMutation(mutation) => {",
        "ExprIr::PrivateRead { target, .. }",
    );
    ordered(
        mutation_budget,
        &[
            "let key_child = match mutation.referenced_name()",
            "let operation_child = match mutation.operation()",
            "SUPER_PROPERTY_MUTATION_PERSISTENT_TEMP_LOCALS",
            "+ count_expr_temp_locals(mutation.receiver())",
            ".max(key_child)",
            ".max(operation_child)",
            ".max(SUPER_PROPERTY_MUTATION_GET_VALUE_TEMP_LOCALS)",
            ".max(SUPER_PROPERTY_MUTATION_TO_NUMERIC_TEMP_LOCALS)",
            ".max(SUPER_PROPERTY_MUTATION_SET_HELPER_TEMP_LOCALS)",
            ".max(REFERENCE_STRICTNESS_FLAG_LOCALS)",
        ],
    );
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
