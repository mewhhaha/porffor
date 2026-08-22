const REFERENCE_SOURCE: &str = include_str!("../../lila-ir/src/reference.rs");
const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const ORDINARY_PROPERTY_LOWERING_SOURCE: &str =
    include_str!("../../lila-ir/src/lowering/ordinary_property_compound.rs");
const EARLY_ERRORS_SOURCE: &str = include_str!("../../lila-ir/src/early_errors.rs");
const EXPRESSIONS_SOURCE: &str = include_str!("../src/expressions.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const DATA_SOURCE: &str = include_str!("../src/data.rs");
const FIXTURE: &str = include_str!(
    "../../lila-cli/tests/fixtures/wasm_ordinary_property_eager_compound_assignment.js"
);
const CLI_SOURCE: &str = include_str!("../../lila-cli/tests/cli/language_numerics.rs");
const RUNNER_SOURCE: &str = include_str!("../../lila-test262/src/lib.rs");
const KNOWN_FAILURES: &str = include_str!("../../lila-cli/tests/known-failures.tsv");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/ordinary-property-eager-compound-assignment-reference.md"
);
const README: &str = include_str!("../../../README.md");
const TASK: &str = include_str!("../../../tasks/08-environments-control-flow.md");

macro_rules! exact_test262 {
    ($name:literal) => {
        (
            $name,
            include_str!(concat!(
                "../../../test262/vendor/test262/test/language/expressions/compound-assignment/",
                $name,
                ".js"
            )),
        )
    };
}

const EXACT_TEST262: &[(&str, &str)] = &[
    exact_test262!("S11.13.2_A7.1_T1"),
    exact_test262!("S11.13.2_A7.1_T2"),
    exact_test262!("S11.13.2_A7.1_T3"),
    exact_test262!("S11.13.2_A7.1_T4"),
    exact_test262!("S11.13.2_A7.2_T1"),
    exact_test262!("S11.13.2_A7.2_T2"),
    exact_test262!("S11.13.2_A7.2_T3"),
    exact_test262!("S11.13.2_A7.2_T4"),
    exact_test262!("S11.13.2_A7.3_T1"),
    exact_test262!("S11.13.2_A7.3_T2"),
    exact_test262!("S11.13.2_A7.3_T3"),
    exact_test262!("S11.13.2_A7.3_T4"),
    exact_test262!("S11.13.2_A7.4_T1"),
    exact_test262!("S11.13.2_A7.4_T2"),
    exact_test262!("S11.13.2_A7.4_T3"),
    exact_test262!("S11.13.2_A7.4_T4"),
    exact_test262!("S11.13.2_A7.5_T1"),
    exact_test262!("S11.13.2_A7.5_T2"),
    exact_test262!("S11.13.2_A7.5_T3"),
    exact_test262!("S11.13.2_A7.5_T4"),
    exact_test262!("S11.13.2_A7.6_T1"),
    exact_test262!("S11.13.2_A7.6_T2"),
    exact_test262!("S11.13.2_A7.6_T3"),
    exact_test262!("S11.13.2_A7.6_T4"),
    exact_test262!("S11.13.2_A7.7_T1"),
    exact_test262!("S11.13.2_A7.7_T2"),
    exact_test262!("S11.13.2_A7.7_T3"),
    exact_test262!("S11.13.2_A7.7_T4"),
    exact_test262!("S11.13.2_A7.8_T1"),
    exact_test262!("S11.13.2_A7.8_T2"),
    exact_test262!("S11.13.2_A7.8_T3"),
    exact_test262!("S11.13.2_A7.8_T4"),
    exact_test262!("S11.13.2_A7.9_T1"),
    exact_test262!("S11.13.2_A7.9_T2"),
    exact_test262!("S11.13.2_A7.9_T3"),
    exact_test262!("S11.13.2_A7.9_T4"),
    exact_test262!("S11.13.2_A7.10_T1"),
    exact_test262!("S11.13.2_A7.10_T2"),
    exact_test262!("S11.13.2_A7.10_T3"),
    exact_test262!("S11.13.2_A7.10_T4"),
    exact_test262!("S11.13.2_A7.11_T1"),
    exact_test262!("S11.13.2_A7.11_T2"),
    exact_test262!("S11.13.2_A7.11_T3"),
    exact_test262!("S11.13.2_A7.11_T4"),
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

fn positions_in_order(source: &str, markers: &[&str]) {
    let mut cursor = 0;
    for marker in markers {
        let offset = source[cursor..]
            .find(marker)
            .unwrap_or_else(|| panic!("missing marker after byte {cursor}: {marker}"));
        cursor += offset + marker.len();
    }
}

#[test]
fn ir_owns_one_closed_ordinary_property_eager_reference() {
    assert!(REFERENCE_SOURCE.contains(
        "#[derive(Debug)]\n#[must_use = \"an ordinary property Reference plan must be consumed by one mutation\"]\npub(crate) struct OrdinaryPropertyReferencePlan"
    ));
    let plan = bounded(
        REFERENCE_SOURCE,
        "pub(crate) struct OrdinaryPropertyReferencePlan {",
        "/// One fused mutation of a Super Property Reference.",
    );
    for field in [
        "base_and_receiver: Box<TypedExpr>",
        "referenced_name: PropertyKeyIr",
        "strictness: Strictness",
    ] {
        assert!(plan.contains(field));
        assert!(!plan.contains(&format!("pub {field}")));
    }
    assert!(!plan.contains("impl Clone for OrdinaryPropertyReferencePlan"));
    assert!(!plan.contains("impl Copy for OrdinaryPropertyReferencePlan"));
    positions_in_order(
        plan,
        &[
            "pub(crate) fn eager_compound_assignment(\n        self,",
            "old_value_binding: String",
            "op: EagerCompoundAssignmentOp",
            "rhs: TypedExpr",
            "possible_getters: PropertyHookTargets",
            "possible_setters: PropertyHookTargets",
            "ExprIr::Identifier(old_value_binding.clone())",
            "let result = op.apply(old_value, rhs);",
            "ExprIr::OrdinaryPropertyEagerCompoundAssignment(",
        ],
    );

    let carrier = bounded(
        REFERENCE_SOURCE,
        "pub struct OrdinaryPropertyEagerCompoundAssignmentIr {",
        "/// A lowerer-owned ordinary property Reference",
    );
    for field in [
        "base_and_receiver: Box<TypedExpr>",
        "referenced_name: PropertyKeyIr",
        "strictness: Strictness",
        "old_value_binding: String",
        "result: Box<TypedExpr>",
        "possible_getters: PropertyHookTargets",
        "possible_setters: PropertyHookTargets",
    ] {
        assert!(carrier.contains(field));
        assert!(!carrier.contains(&format!("pub {field}")));
    }
    assert!(carrier.contains("fn new("));
    assert!(!carrier.contains("pub fn new("));
    for accessor in [
        "base_and_receiver",
        "referenced_name",
        "strictness",
        "old_value_binding",
        "result",
        "possible_getters",
        "possible_setters",
    ] {
        assert!(carrier.contains(&format!("pub fn {accessor}(&self)")));
    }
    assert!(IR_SOURCE.contains(
        "OrdinaryPropertyEagerCompoundAssignment(OrdinaryPropertyEagerCompoundAssignmentIr),"
    ));
    assert!(EARLY_ERRORS_SOURCE
        .contains("ExprIr::OrdinaryPropertyEagerCompoundAssignment(assignment) =>"));
    assert!(
        LOWERING_SOURCE.contains("ExprIr::OrdinaryPropertyEagerCompoundAssignment(assignment) =>")
    );
}

#[test]
fn lowering_intercepts_all_eager_access_operators_before_generic_reference_decomposition() {
    let reference = bounded(
        ORDINARY_PROPERTY_LOWERING_SOURCE,
        "pub(super) fn lower_ordinary_property_reference_plan(",
        "    /// Lower one ordinary property Reference directly",
    );
    positions_in_order(
        reference,
        &[
            "let base_and_receiver = Box::new(self.lower_property_target(access.target()));",
            "let referenced_name = match access.field()",
            "self.lower_expression(expression)",
            "let plan = OrdinaryPropertyReferencePlan::new(",
            "self.reference_strictness()",
            "(plan, referenced_name, metadata)",
        ],
    );
    let producer = bounded(
        ORDINARY_PROPERTY_LOWERING_SOURCE,
        "pub(super) fn lower_ordinary_property_eager_compound_assignment(",
        "\n}\n\n#[cfg(test)]",
    );
    positions_in_order(
        producer,
        &[
            "self.lower_ordinary_property_reference_plan(access)",
            "self.record_ordinary_property_get(&metadata);",
            "let possible_getters = Self::possible_ordinary_property_getters(&metadata);",
            "let rhs = self.lower_expression(rhs);",
            "let possible_setters = self.possible_ordinary_property_setters(&metadata, true);",
            "let old_value_binding =",
            "plan.eager_compound_assignment(",
            "possible_getters",
            "possible_setters",
        ],
    );
    assert!(ORDINARY_PROPERTY_LOWERING_SOURCE
        .contains("fn ordinary_property_eager_compound_assignment_owns_one_reference()"));

    let assign = bounded(
        LOWERING_SOURCE,
        "    fn lower_assign(&mut self, op: AssignOp, lhs: &AssignTarget, rhs: &Expression)",
        "    fn lower_web_compat_call_assignment_target(",
    );
    for marker in [
        "EagerCompoundAssignmentOp::Arithmetic(arithmetic)",
        "EagerCompoundAssignmentOp::Bitwise(bitwise)",
    ] {
        assert!(
            assign.matches(marker).count() >= 2,
            "ordinary and super access arms must both select {marker}"
        );
    }
    assert_eq!(
        assign
            .matches(".lower_ordinary_property_eager_compound_assignment(")
            .count(),
        2,
        "arithmetic and bitwise access domains need the fused producer"
    );
    assert!(!IR_SOURCE.contains("\n    PropertyCompoundAssign {"));
    assert!(!LOWERING_SOURCE.contains("ExprIr::PropertyCompoundAssign"));
}

#[test]
fn aot_typestate_forces_raw_key_get_result_and_putvalue_transitions() {
    for prefix in [
        "#[derive(Debug)]\n#[must_use = \"a raw ordinary Property Reference must enter its operation-specific transition\"]\nstruct EvaluatedRawOrdinaryPropertyReferenceLocals",
        "#[derive(Debug)]\n#[must_use = \"a read ordinary Property Reference must be advanced to its applied result\"]\nstruct ReadOrdinaryPropertyReferenceLocals",
        "#[derive(Debug)]\n#[must_use = \"a ready ordinary Property Reference must be consumed by PutValue\"]\nstruct ReadyToWriteOrdinaryPropertyReferenceLocals",
    ] {
        assert!(EXPRESSIONS_SOURCE.contains(prefix), "missing typestate {prefix}");
    }
    let roles = bounded(
        EXPRESSIONS_SOURCE,
        "struct EvaluatedRawOrdinaryPropertyReferenceLocals {",
        "#[derive(Debug)]\n#[must_use = \"a raw Super Property Reference",
    );
    assert!(!roles.contains("Clone"));
    assert!(!roles.contains("Copy"));

    let evaluate = bounded(
        EXPRESSIONS_SOURCE,
        "    fn evaluate_raw_ordinary_property_reference(",
        "    fn emit_get_value_from_raw_ordinary_property_reference(",
    );
    positions_in_order(
        evaluate,
        &[
            "self.compile_expr_to_locals(\n            mutation.base_and_receiver()",
            "self.emit_propagate_throw_from_locals_if_needed(",
            "self.compile_raw_property_key_expression_to_locals(",
            "self.emit_propagate_throw_from_locals_if_needed(",
            "Ok(EvaluatedRawOrdinaryPropertyReferenceLocals {",
        ],
    );
    assert_eq!(
        evaluate
            .matches("compile_raw_property_key_expression_to_locals(")
            .count(),
        1
    );
    assert!(!evaluate.contains("emit_value_to_property_key_locals("));

    let get = bounded(
        EXPRESSIONS_SOURCE,
        "    fn emit_get_value_from_raw_ordinary_property_reference(",
        "    fn evaluate_rhs_for_raw_ordinary_property_assignment(",
    );
    positions_in_order(
        get,
        &[
            "let EvaluatedRawOrdinaryPropertyReferenceLocals {",
            "self.compile_nullish_tagged_i32(base_and_receiver_tag, function)?;",
            "self.emit_throw_runtime_error(",
            "self.emit_propagate_throw_from_locals_if_needed(",
            "self.emit_value_to_object_locals(",
            "self.emit_value_to_property_key_locals(",
            "self.emit_propagate_throw_from_locals_if_needed(",
            "self.emit_object_read_with_key_tag(",
            "target_object_payload,\n            target_object_tag,\n            base_and_receiver_payload,\n            base_and_receiver_tag,",
            "Ok(ReadOrdinaryPropertyReferenceLocals {",
        ],
    );
    assert_eq!(get.matches("emit_value_to_property_key_locals(").count(), 1);
    assert!(!get.contains("compile_raw_property_key_expression_to_locals("));

    let result = bounded(
        EXPRESSIONS_SOURCE,
        "    fn emit_result_from_read_ordinary_property_reference(",
        "    fn emit_put_value_from_ready_ordinary_property_reference(",
    );
    positions_in_order(
        result,
        &[
            "let ReadOrdinaryPropertyReferenceLocals {",
            ".insert(\n                mutation.old_value_binding().to_string(),",
            "self.compile_expr_to_locals(mutation.result(), result_payload, result_tag, function)",
            "self.emit_propagate_throw_from_locals_if_needed(result_payload, result_tag, function)?;",
            "Ok(ReadyToWriteOrdinaryPropertyReferenceLocals {",
        ],
    );

    let put = bounded(
        EXPRESSIONS_SOURCE,
        "    fn emit_put_value_from_ready_ordinary_property_reference(",
        "    fn compile_ordinary_property_eager_compound_assignment_to_locals(",
    );
    positions_in_order(
        put,
        &[
            "let ReadyToWriteOrdinaryPropertyReferenceLocals {",
            "self.emit_ordinary_set_result_via_helper(",
            "if strictness.throws_on_failed_set() {",
            "self.emit_throw_runtime_error_to_active_handler(",
            "\"Cannot assign to property\"",
            "Instruction::LocalGet(result_payload)",
            "Instruction::LocalSet(payload_local)",
            "Instruction::LocalGet(result_tag)",
            "Instruction::LocalSet(tag_local)",
        ],
    );
    assert!(!put.contains("emit_value_to_property_key_locals("));
    assert!(!put.contains("compile_raw_property_key_expression_to_locals("));

    let entry = bounded(
        EXPRESSIONS_SOURCE,
        "    fn compile_ordinary_property_eager_compound_assignment_to_locals(",
        "    fn evaluate_raw_super_property_reference(",
    );
    positions_in_order(
        entry,
        &[
            "self.evaluate_raw_ordinary_property_reference(mutation, function)?",
            "self.emit_get_value_from_raw_ordinary_property_reference(",
            "self.emit_result_from_read_ordinary_property_reference(",
            "self.emit_put_value_from_ready_ordinary_property_reference(",
        ],
    );
    assert!(!EXPRESSIONS_SOURCE.contains("compile_property_compound_assign_to_locals"));
}

#[test]
fn exhaustive_consumers_and_temp_budget_name_every_fused_phase() {
    for marker in [
        "const ORDINARY_PROPERTY_MUTATION_READ_PERSISTENT_TEMP_LOCALS: usize = 2 + 4 + 2;",
        "const ORDINARY_PROPERTY_MUTATION_WRITE_PERSISTENT_TEMP_LOCALS: usize = 2 + 4 + 2 + 3;",
        "const ORDINARY_PROPERTY_MUTATION_TO_OBJECT_TEMP_LOCALS: usize = 2 + 3 + 3;",
        "const ORDINARY_PROPERTY_MUTATION_TO_PROPERTY_KEY_TEMP_LOCALS: usize = 2;",
        "const ORDINARY_PROPERTY_MUTATION_GET_VALUE_TEMP_LOCALS: usize = 2;",
        "const ORDINARY_PROPERTY_MUTATION_SET_HELPER_TEMP_LOCALS: usize = 4 + 2;",
    ] {
        assert!(PLANNING_SOURCE.contains(marker), "planning lost {marker}");
    }
    let budget = bounded(
        PLANNING_SOURCE,
        "        ExprIr::OrdinaryPropertyEagerCompoundAssignment(mutation) => {",
        "        ExprIr::DeleteIdentifier { .. } => 0,",
    );
    positions_in_order(
        budget,
        &[
            "let read_phase = ORDINARY_PROPERTY_MUTATION_READ_PERSISTENT_TEMP_LOCALS",
            ".max(ORDINARY_PROPERTY_MUTATION_GET_VALUE_TEMP_LOCALS)",
            "let write_phase = ORDINARY_PROPERTY_MUTATION_WRITE_PERSISTENT_TEMP_LOCALS",
            ".max(ORDINARY_PROPERTY_MUTATION_SET_HELPER_TEMP_LOCALS)",
            "read_phase.max(write_phase)",
        ],
    );
    assert_eq!(
        EXPRESSIONS_SOURCE
            .matches("ExprIr::OrdinaryPropertyEagerCompoundAssignment(mutation) =>")
            .count(),
        2,
        "both expression emission entry points must consume the fused node"
    );
    assert_eq!(
        DATA_SOURCE
            .matches("ExprIr::OrdinaryPropertyEagerCompoundAssignment(mutation) =>")
            .count(),
        1,
        "data collection must traverse the fused node"
    );
    assert_eq!(
        PLANNING_SOURCE
            .matches("ExprIr::OrdinaryPropertyEagerCompoundAssignment(mutation) =>")
            .count(),
        7,
        "every planning traversal must name the fused node"
    );
}

#[test]
fn fixture_observes_all_eager_operators_and_the_reference_lifecycle() {
    for marker in [
        "] *= 3", "] /= 3", "] %= 5", "] += 3", "] -= 3", "] <<= 2", "] >>= 2", "] >>>= 2",
        "] &= 11", "] ^= 10", "] |= 3", "] **= 3",
    ] {
        assert!(FIXTURE.contains(marker), "fixture lost operator {marker}");
    }
    for oracle in [
        "abrupt base order",
        "abrupt raw key order",
        "nullishReference(null, \"null base\")",
        "nullishReference(undefined, \"undefined base\")",
        "ToPropertyKey abrupt order",
        "complete Reference lifecycle",
        "canonical key write",
        "mutated raw key not recoerced",
        "strict Set false nonpublication",
        "strict Set false no write",
        "sloppy Set false ignored write",
        "RHS abrupt nonpublication",
        "RHS abrupt skips Set",
    ] {
        assert!(FIXTURE.contains(oracle), "fixture lost oracle {oracle}");
    }
    assert!(FIXTURE.contains(
        "base,raw-key,to-key,proxy-get:p:true,getter:true,rhs,proxy-set:p:true:3,setter:true:3"
    ));
    assert!(FIXTURE.contains("\"use strict\";\n  return rejectingProxy[rejectingKey] +="));
    assert!(CLI_SOURCE
        .contains("fn run_wasm_backend_preserves_ordinary_property_eager_compound_reference()"));
    assert!(CLI_SOURCE.contains("wasm_ordinary_property_eager_compound_assignment.js"));
}

#[test]
fn exact_a7_inventory_is_raw_unmasked_and_keeps_all_t3_controls() {
    assert_eq!(EXACT_TEST262.len(), 44);
    for group in 1..=11 {
        for witness in 1..=4 {
            let name = format!("S11.13.2_A7.{group}_T{witness}");
            assert_eq!(
                EXACT_TEST262
                    .iter()
                    .filter(|(candidate, _)| *candidate == name)
                    .count(),
                1,
                "missing or duplicate exact witness {name}"
            );
        }
    }
    assert_eq!(
        EXACT_TEST262
            .iter()
            .filter(|(name, _)| name.ends_with("_T3"))
            .count(),
        11
    );
    for (name, source) in EXACT_TEST262 {
        assert!(
            !source.contains("flags:"),
            "{name} must execute in both modes"
        );
        assert!(
            !RUNNER_SOURCE.contains(name),
            "runner masks exact witness {name}"
        );
        assert!(
            !KNOWN_FAILURES.contains(name),
            "known-failure inventory masks exact witness {name}"
        );
    }
}

#[test]
fn status_records_the_exact_baseline_and_keeps_the_batch_bounded() {
    for source in [README, TASK] {
        for marker in [
            "ae1bd994b",
            "22/88",
            "66",
            "T1, T2 and T4",
            "T3 control",
            "Runtime/Bug",
            "post-batch",
            "88/88",
            "zero unsupported",
            "1/1",
            "75.42s",
        ] {
            assert!(
                source
                    .to_ascii_lowercase()
                    .contains(&marker.to_ascii_lowercase()),
                "status lost {marker}"
            );
        }
    }
    for marker in [
        "44 physical files",
        "for 88 matrix\nexecutions",
        "22/88",
        "66 failing executions",
        "Exponentiation has no file in this legacy matrix",
        "This batch does not change logical assignment",
        "prefix/postfix numeric update",
        "resumable/suspended property Reference",
    ] {
        assert!(CONTRACT.contains(marker), "contract lost boundary {marker}");
    }
}
