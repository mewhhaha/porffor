const REFERENCE_SOURCE: &str = include_str!("../../lila-ir/src/reference.rs");
const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const ORDINARY_PROPERTY_LOWERING_SOURCE: &str =
    include_str!("../../lila-ir/src/lowering/ordinary_property_compound.rs");
const EXPRESSIONS_SOURCE: &str = include_str!("../src/expressions.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const DATA_SOURCE: &str = include_str!("../src/data.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_ordinary_property_assignment_reference.js");
const CLI_SOURCE: &str = include_str!("../../lila-cli/tests/cli/language_numerics.rs");
const RUNNER_SOURCE: &str = include_str!("../../lila-test262/src/lib.rs");
const KNOWN_FAILURES: &str = include_str!("../../lila-cli/tests/known-failures.tsv");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/ordinary-property-plain-assignment-reference.md"
);
const README: &str = include_str!("../../../README.md");
const TASK: &str = include_str!("../../../tasks/08-environments-control-flow.md");

const EXACT_TEST262: &[(&str, &str)] = &[
    (
        "target-member-computed-reference-null.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/assignment/target-member-computed-reference-null.js"
        ),
    ),
    (
        "target-member-identifier-reference-null.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/assignment/target-member-identifier-reference-null.js"
        ),
    ),
    (
        "target-member-identifier-reference-undefined.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/assignment/target-member-identifier-reference-undefined.js"
        ),
    ),
];

const CONTROL_TEST262: &[(&str, &str)] = &[
    (
        "target-member-computed-reference-undefined.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/assignment/target-member-computed-reference-undefined.js"
        ),
    ),
    (
        "target-member-computed-reference.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/assignment/target-member-computed-reference.js"
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
fn ir_owns_one_closed_plain_assignment_reference() {
    let carrier = bounded(
        REFERENCE_SOURCE,
        "pub struct OrdinaryPropertyAssignmentIr {",
        "/// One fused numeric update of an ordinary property Reference.",
    );
    for field in [
        "base_and_receiver: Box<TypedExpr>",
        "referenced_name: PropertyKeyIr",
        "rhs: Box<TypedExpr>",
        "strictness: Strictness",
    ] {
        assert!(carrier.contains(field), "carrier lost {field}");
        assert!(!carrier.contains(&format!("pub {field}")));
    }
    assert!(carrier.contains("fn new("));
    assert!(!carrier.contains("pub fn new("));
    for accessor in ["base_and_receiver", "referenced_name", "rhs", "strictness"] {
        assert!(
            carrier.contains(&format!("pub fn {accessor}(&self)")),
            "carrier lost {accessor} accessor"
        );
    }

    assert!(REFERENCE_SOURCE.contains(
        "#[derive(Debug)]\n#[must_use = \"an ordinary property Reference plan must be consumed by one mutation\"]\npub(crate) struct OrdinaryPropertyReferencePlan"
    ));
    let plan = bounded(
        REFERENCE_SOURCE,
        "pub(crate) struct OrdinaryPropertyReferencePlan {",
        "/// One fused mutation of a Super Property Reference.",
    );
    assert!(!plan.contains("impl Clone for OrdinaryPropertyReferencePlan"));
    assert!(!plan.contains("impl Copy for OrdinaryPropertyReferencePlan"));
    positions_in_order(
        plan,
        &[
            "pub(crate) fn plain_assignment(self, rhs: TypedExpr)",
            "rhs.value_info()",
            "ExprIr::OrdinaryPropertyAssignment(OrdinaryPropertyAssignmentIr::new(",
            "self.base_and_receiver",
            "self.referenced_name",
            "Box::new(rhs)",
            "self.strictness",
        ],
    );
    assert!(IR_SOURCE.contains("OrdinaryPropertyAssignment(OrdinaryPropertyAssignmentIr)"));
}

#[test]
fn lowering_builds_the_reference_before_rhs_and_intercepts_the_closed_ast_arm() {
    let helper = bounded(
        ORDINARY_PROPERTY_LOWERING_SOURCE,
        "    pub(super) fn lower_ordinary_property_plain_assignment(",
        "    /// Lower one ordinary property Reference directly into its fused eager",
    );
    positions_in_order(
        helper,
        &[
            "self.lower_ordinary_property_reference_plan(access)",
            "let rhs_value = self.lower_expression(rhs);",
            "plan.plain_assignment(rhs_value)",
        ],
    );
    assert!(!helper.contains("ExprIr::PropertyWrite"));

    let assign_target = bounded(
        LOWERING_SOURCE,
        "                AssignTarget::Access(access) => match access {",
        "                AssignTarget::Pattern(pattern)",
    );
    positions_in_order(
        assign_target,
        &[
            "PropertyAccess::Simple(access) => {",
            "self.lower_ordinary_property_plain_assignment(access, rhs)",
            "PropertyAccess::Private(_) | PropertyAccess::Super(_) => {",
            "self.lower_property_assign(access, rhs)",
        ],
    );
    assert!(ORDINARY_PROPERTY_LOWERING_SOURCE
        .contains("fn ordinary_property_plain_assignment_retains_base_key_rhs_and_strictness()"));
}

#[test]
fn backend_typestate_orders_rhs_before_to_object_key_coercion_and_set() {
    for declaration in [
        "#[derive(Debug)]\n#[must_use = \"a raw ordinary Property Reference must enter its operation-specific transition\"]\nstruct EvaluatedRawOrdinaryPropertyReferenceLocals",
        "#[derive(Debug)]\n#[must_use = \"an evaluated ordinary property assignment must enter PutValue\"]\nstruct EvaluatedRawOrdinaryPropertyAssignmentLocals",
        "#[derive(Debug)]\n#[must_use = \"a canonical ordinary property assignment must be consumed by PutValue\"]\nstruct ReadyToWriteOrdinaryPropertyAssignmentLocals",
    ] {
        assert!(EXPRESSIONS_SOURCE.contains(declaration));
    }
    let assignment_states = bounded(
        EXPRESSIONS_SOURCE,
        "struct EvaluatedRawOrdinaryPropertyAssignmentLocals {",
        "struct ReadOrdinaryPropertyReferenceLocals {",
    );
    assert!(!assignment_states.contains("Clone"));
    assert!(!assignment_states.contains("Copy"));
    for field in [
        "base_and_receiver_payload",
        "base_and_receiver_tag",
        "target_object_payload",
        "target_object_tag",
        "property_key_payload",
        "property_key_tag",
        "rhs_payload",
        "rhs_tag",
        "set_result",
    ] {
        assert!(assignment_states.contains(field), "typestate lost {field}");
    }

    let sealed_sources = bounded(
        EXPRESSIONS_SOURCE,
        "trait OrdinaryPropertyReferenceSource {",
        "#[derive(Debug)]\n#[must_use = \"a raw Super Property Reference must be consumed by GetValue\"]",
    );
    assert_eq!(
        sealed_sources
            .matches("impl OrdinaryPropertyReferenceSource for")
            .count(),
        3
    );
    for source in [
        "OrdinaryPropertyAssignmentIr",
        "OrdinaryPropertyEagerCompoundAssignmentIr",
        "OrdinaryPropertyNumericUpdateIr",
    ] {
        assert!(sealed_sources.contains(&format!(
            "impl OrdinaryPropertyReferenceSource for {source}"
        )));
    }

    let raw = bounded(
        EXPRESSIONS_SOURCE,
        "    fn evaluate_raw_ordinary_property_reference(",
        "    fn emit_get_value_from_raw_ordinary_property_reference(",
    );
    positions_in_order(
        raw,
        &[
            "self.compile_expr_to_locals(\n            mutation.base_and_receiver()",
            "self.emit_propagate_throw_from_locals_if_needed(",
            "self.compile_raw_property_key_expression_to_locals(\n            mutation.referenced_name()",
            "self.emit_propagate_throw_from_locals_if_needed(",
        ],
    );

    let rhs = bounded(
        EXPRESSIONS_SOURCE,
        "    fn evaluate_rhs_for_raw_ordinary_property_assignment(",
        "    fn canonicalize_raw_ordinary_property_assignment(",
    );
    positions_in_order(
        rhs,
        &[
            "let EvaluatedRawOrdinaryPropertyReferenceLocals",
            "self.compile_expr_to_locals(assignment.rhs(), rhs_payload, rhs_tag, function)?",
            "self.emit_propagate_throw_from_locals_if_needed(rhs_payload, rhs_tag, function)?",
            "Ok(EvaluatedRawOrdinaryPropertyAssignmentLocals",
        ],
    );

    let canonicalize = bounded(
        EXPRESSIONS_SOURCE,
        "    fn canonicalize_raw_ordinary_property_assignment(",
        "    fn emit_put_value_from_ready_ordinary_property_assignment(",
    );
    positions_in_order(
        canonicalize,
        &[
            "let EvaluatedRawOrdinaryPropertyAssignmentLocals",
            "self.compile_nullish_tagged_i32(base_and_receiver_tag, function)?",
            "function.instruction(&Instruction::If(BlockType::Empty));",
            "self.emit_throw_runtime_error(",
            "TYPE_ERROR_NAME",
            "\"Cannot convert undefined or null to object\"",
            "rhs_payload",
            "rhs_tag",
            "self.emit_propagate_throw_from_locals_if_needed(rhs_payload, rhs_tag, function)?",
            "function.instruction(&Instruction::End);",
            "let target_object_payload = self.reserve_temp_local();",
            "let target_object_tag = self.reserve_temp_local();",
            "self.emit_value_to_object_locals(",
            "base_and_receiver_payload",
            "base_and_receiver_tag",
            "target_object_payload",
            "target_object_tag",
            "self.emit_value_to_property_key_locals(property_key_payload, property_key_tag, function)?",
            "let set_result = self.reserve_temp_local();",
            "Ok(ReadyToWriteOrdinaryPropertyAssignmentLocals",
        ],
    );

    let put = bounded(
        EXPRESSIONS_SOURCE,
        "    fn emit_put_value_from_ready_ordinary_property_assignment(",
        "    fn compile_ordinary_property_assignment_to_locals(",
    );
    positions_in_order(
        put,
        &[
            "self.emit_ordinary_set_result_via_helper(",
            "target_object_payload",
            "target_object_tag",
            "base_and_receiver_payload",
            "base_and_receiver_tag",
            "property_key_payload",
            "property_key_tag",
            "rhs_payload",
            "rhs_tag",
            "set_result",
            "if strictness.throws_on_failed_set()",
            "self.emit_throw_runtime_error_to_active_handler(",
            "function.instruction(&Instruction::LocalGet(rhs_payload));",
            "function.instruction(&Instruction::LocalSet(payload_local));",
            "function.instruction(&Instruction::LocalGet(rhs_tag));",
            "function.instruction(&Instruction::LocalSet(tag_local));",
            "self.release_temp_local(set_result);",
            "self.release_temp_local(target_object_tag);",
            "self.release_temp_local(target_object_payload);",
            "self.release_temp_local(rhs_tag);",
            "self.release_temp_local(rhs_payload);",
            "self.release_temp_local(property_key_tag);",
            "self.release_temp_local(property_key_payload);",
            "self.release_temp_local(base_and_receiver_tag);",
            "self.release_temp_local(base_and_receiver_payload);",
        ],
    );

    let entry = bounded(
        EXPRESSIONS_SOURCE,
        "    fn compile_ordinary_property_assignment_to_locals(",
        "    fn emit_result_from_read_ordinary_property_reference(",
    );
    positions_in_order(
        entry,
        &[
            "self.evaluate_raw_ordinary_property_reference(assignment, function)?",
            "self.evaluate_rhs_for_raw_ordinary_property_assignment(",
            "self.canonicalize_raw_ordinary_property_assignment(evaluated_assignment, function)?",
            "self.emit_put_value_from_ready_ordinary_property_assignment(",
        ],
    );
}

#[test]
fn exhaustive_consumers_and_budget_name_every_plain_assignment_phase() {
    for marker in [
        "const ORDINARY_PROPERTY_ASSIGNMENT_RAW_TEMP_LOCALS: usize = 4;",
        "const ORDINARY_PROPERTY_ASSIGNMENT_EVALUATED_TEMP_LOCALS: usize = 4 + 2;",
        "const ORDINARY_PROPERTY_ASSIGNMENT_CANONICAL_TEMP_LOCALS: usize = 4 + 2 + 2;",
        "const ORDINARY_PROPERTY_ASSIGNMENT_READY_TEMP_LOCALS: usize = 4 + 2 + 2 + 1;",
        "const ORDINARY_PROPERTY_ASSIGNMENT_TO_OBJECT_TEMP_LOCALS: usize = 2 + 3 + 3;",
        "const ORDINARY_PROPERTY_ASSIGNMENT_TO_PROPERTY_KEY_TEMP_LOCALS: usize = 2;",
        "const ORDINARY_PROPERTY_ASSIGNMENT_SET_HELPER_TEMP_LOCALS: usize = 4 + 2;",
    ] {
        assert!(PLANNING_SOURCE.contains(marker), "planning lost {marker}");
    }
    let expr_budgets = bounded(
        PLANNING_SOURCE,
        "pub(crate) fn count_expr_temp_locals(expr: &TypedExpr) -> usize {",
        "pub(crate) fn collect_hoisted_vars_block_root(",
    );
    let budget = bounded(
        expr_budgets,
        "        ExprIr::OrdinaryPropertyAssignment(assignment) => {",
        "        ExprIr::OrdinaryPropertyNumericUpdate(update) => {",
    );
    positions_in_order(
        budget,
        &[
            "let raw_phase = ORDINARY_PROPERTY_ASSIGNMENT_RAW_TEMP_LOCALS",
            "count_expr_temp_locals(assignment.base_and_receiver()).max(key_child)",
            "let evaluated_phase = ORDINARY_PROPERTY_ASSIGNMENT_EVALUATED_TEMP_LOCALS",
            "count_expr_temp_locals(assignment.rhs())",
            "let canonical_phase = ORDINARY_PROPERTY_ASSIGNMENT_CANONICAL_TEMP_LOCALS",
            "ORDINARY_PROPERTY_ASSIGNMENT_TO_OBJECT_TEMP_LOCALS",
            ".max(ORDINARY_PROPERTY_ASSIGNMENT_TO_PROPERTY_KEY_TEMP_LOCALS)",
            "let write_phase = ORDINARY_PROPERTY_ASSIGNMENT_READY_TEMP_LOCALS",
            "+ ORDINARY_PROPERTY_ASSIGNMENT_SET_HELPER_TEMP_LOCALS",
            "raw_phase\n                .max(evaluated_phase)\n                .max(canonical_phase)\n                .max(write_phase)",
        ],
    );
    assert_eq!(
        EXPRESSIONS_SOURCE
            .matches("ExprIr::OrdinaryPropertyAssignment(assignment) =>")
            .count(),
        2
    );
    assert_eq!(
        PLANNING_SOURCE
            .matches("ExprIr::OrdinaryPropertyAssignment(assignment) =>")
            .count(),
        7
    );
    assert_eq!(
        DATA_SOURCE
            .matches("ExprIr::OrdinaryPropertyAssignment(assignment) =>")
            .count(),
        1
    );
}

#[test]
fn fixture_observes_the_plain_put_value_lifecycle() {
    for marker in [
        "complete plain property Reference lifecycle",
        "base,raw-key,rhs,to-key,proxy-set:p:true:7,setter:true:7",
        "sole ToPropertyKey",
        "abrupt base order",
        "abrupt raw key order",
        "nullishReference(null, \"null base\")",
        "nullishReference(undefined, \"undefined base\")",
        "staticNullishReference(null, \"null base\")",
        "staticNullishReference(undefined, \"undefined base\")",
        "RHS throw identity",
        "RHS mutation precedes ToPropertyKey",
        "mutated raw key sole ToPropertyKey",
        "abrupt Set nonpublication",
        "sloppy false Set result",
        "strict false Set nonpublication",
        "sloppy primitive assignment result",
        "strict primitive Set nonpublication",
    ] {
        assert!(FIXTURE.contains(marker), "fixture lost oracle {marker}");
    }
    assert!(FIXTURE.contains("\"use strict\";\n  return falseSetProxy.p = 13;"));
    assert!(FIXTURE.contains("\"use strict\";\n  return (1).p = 15;"));
    assert!(CLI_SOURCE
        .contains("fn run_wasm_backend_preserves_ordinary_property_plain_assignment_reference()"));
    assert!(CLI_SOURCE.contains("wasm_ordinary_property_assignment_reference.js"));
}

#[test]
fn exact_inventory_is_raw_unmasked_and_controls_remain_separate() {
    assert_eq!(EXACT_TEST262.len(), 3);
    assert_eq!(CONTROL_TEST262.len(), 2);
    for (path, source) in EXACT_TEST262.iter().chain(CONTROL_TEST262) {
        assert!(
            !source.contains("flags:"),
            "{path} must execute in sloppy and strict modes"
        );
        assert!(!RUNNER_SOURCE.contains(path), "runner masks {path}");
        assert!(!KNOWN_FAILURES.contains(path), "known failures mask {path}");
        assert!(source.contains("sec-assignment-operators"));
    }
    assert!(EXACT_TEST262[0].1.contains("property key evaluated"));
    assert!(EXACT_TEST262[0]
        .1
        .contains("right-hand side expression evaluated"));
    for (_, source) in &EXACT_TEST262[1..] {
        assert!(source.contains("count += 1"));
        assert!(source.contains("assert.sameValue(count, 1)"));
    }
}

#[test]
fn verified_status_records_the_exact_baseline_results_and_nonclaims() {
    for source in [README, TASK] {
        for marker in [
            "eb32c63a",
            "target-member-computed-reference-null.js",
            "target-member-identifier-reference-null.js",
            "target-member-identifier-reference-undefined.js",
            "1/6",
            "target-member-computed-reference-undefined.js",
            "target-member-computed-reference.js",
            "each `2/2`",
            "known-failure entry owns",
            "workspace/all-target check",
            "15.18 seconds",
            "`cargo xc`",
            "focused IR invariant `1/1`",
            "structure executable `7/7`",
            "retained eager-compound and numeric",
            "exact Wasm CLI fixture",
            "66.90 seconds",
            "all `6/6`",
            "zero unsupported, not-implemented, crash or bug outcomes",
            "controls remain `4/4`",
            "`(1).p`",
            "property-read assertion",
            "broader assignment leaf",
        ] {
            assert!(source.contains(marker), "status lost {marker}");
        }
    }
    for marker in [
        "three physical files produce six",
        "The selected current-head baseline is 1/6",
        "evaluate `rhs`",
        "apply `ToObject`",
        "apply `ToPropertyKey` exactly once",
        "perform `[[Set]]`",
        "publish `rhs` only after PutValue completes normally",
        "This batch does not change compound or logical assignment",
        "resumable property",
    ] {
        assert!(CONTRACT.contains(marker), "contract lost {marker}");
    }
}
