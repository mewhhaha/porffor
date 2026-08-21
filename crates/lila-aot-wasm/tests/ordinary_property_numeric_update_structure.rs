const REFERENCE_SOURCE: &str = include_str!("../../lila-ir/src/reference.rs");
const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const ORDINARY_PROPERTY_REFERENCE_LOWERING_SOURCE: &str =
    include_str!("../../lila-ir/src/lowering/ordinary_property_compound.rs");
const ORDINARY_PROPERTY_UPDATE_LOWERING_SOURCE: &str =
    include_str!("../../lila-ir/src/lowering/ordinary_property_update.rs");
const EARLY_ERRORS_SOURCE: &str = include_str!("../../lila-ir/src/early_errors.rs");
const EXPRESSIONS_SOURCE: &str = include_str!("../src/expressions.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const DATA_SOURCE: &str = include_str!("../src/data.rs");
const FIXTURE: &str = include_str!(
    "../../lila-cli/tests/fixtures/wasm_ordinary_property_numeric_update_reference.js"
);
const CLI_SOURCE: &str = include_str!("../../lila-cli/tests/cli/language_numerics.rs");
const RUNNER_SOURCE: &str = include_str!("../../lila-test262/src/lib.rs");
const KNOWN_FAILURES: &str = include_str!("../../lila-cli/tests/known-failures.tsv");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/ordinary-property-numeric-update-reference.md"
);
const README: &str = include_str!("../../../README.md");
const TASK: &str = include_str!("../../../tasks/08-environments-control-flow.md");

const EXACT_TEST262: &[(&str, &str)] = &[
    (
        "postfix-decrement/S11.3.2_A6_T1",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/postfix-decrement/S11.3.2_A6_T1.js"
        ),
    ),
    (
        "postfix-increment/S11.3.1_A6_T1",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/postfix-increment/S11.3.1_A6_T1.js"
        ),
    ),
    (
        "prefix-decrement/S11.4.5_A6_T1",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/prefix-decrement/S11.4.5_A6_T1.js"
        ),
    ),
    (
        "prefix-increment/S11.4.4_A6_T1",
        include_str!(
            "../../../test262/vendor/test262/test/language/expressions/prefix-increment/S11.4.4_A6_T1.js"
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
fn ir_owns_one_closed_ordinary_property_numeric_update_reference() {
    let carrier = bounded(
        REFERENCE_SOURCE,
        "pub struct OrdinaryPropertyNumericUpdateIr {",
        "/// A lowerer-owned ordinary property Reference",
    );
    for field in [
        "base_and_receiver: Box<TypedExpr>",
        "referenced_name: PropertyKeyIr",
        "strictness: Strictness",
        "op: NumericUpdateOp",
        "return_mode: UpdateReturnMode",
        "value_kind: ValueKind",
    ] {
        assert!(carrier.contains(field), "carrier lost {field}");
        assert!(!carrier.contains(&format!("pub {field}")));
    }
    assert!(carrier.contains("fn new("));
    assert!(!carrier.contains("pub fn new("));
    for accessor in [
        "base_and_receiver",
        "referenced_name",
        "strictness",
        "op",
        "return_mode",
        "value_kind",
    ] {
        assert!(carrier.contains(&format!("pub fn {accessor}(&self)")));
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
            "pub(crate) fn numeric_update(\n        self,",
            "op: NumericUpdateOp",
            "return_mode: UpdateReturnMode",
            "let value_kind = ValueKind::Dynamic;",
            "KindSet::from_kind(ValueKind::Number)\n                .union(KindSet::from_kind(ValueKind::BigInt))",
            "ExprIr::OrdinaryPropertyNumericUpdate(OrdinaryPropertyNumericUpdateIr::new(",
        ],
    );
    let numeric_signature = bounded(plan, "pub(crate) fn numeric_update(", ") -> TypedExpr {");
    assert!(!numeric_signature.contains("value_kind"));
    assert!(IR_SOURCE.contains("OrdinaryPropertyNumericUpdate(OrdinaryPropertyNumericUpdateIr),"));
    assert!(EARLY_ERRORS_SOURCE.contains("ExprIr::OrdinaryPropertyNumericUpdate(update) =>"));
    assert!(IR_SOURCE.contains("ExprIr::OrdinaryPropertyNumericUpdate(update) =>"));
}

#[test]
fn lowering_exhaustively_intercepts_simple_updates_before_decomposed_property_access() {
    let reference = bounded(
        ORDINARY_PROPERTY_REFERENCE_LOWERING_SOURCE,
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
            "(plan, referenced_name)",
        ],
    );

    let update = bounded(
        ORDINARY_PROPERTY_UPDATE_LOWERING_SOURCE,
        "pub(super) fn lower_ordinary_property_numeric_update(",
        "\n}\n\n#[cfg(test)]",
    );
    for mapping in [
        "UpdateOp::IncrementPost => (NumericUpdateOp::Increment, UpdateReturnMode::Postfix)",
        "UpdateOp::IncrementPre => (NumericUpdateOp::Increment, UpdateReturnMode::Prefix)",
        "UpdateOp::DecrementPost => (NumericUpdateOp::Decrement, UpdateReturnMode::Postfix)",
        "UpdateOp::DecrementPre => (NumericUpdateOp::Decrement, UpdateReturnMode::Prefix)",
    ] {
        assert!(update.contains(mapping), "update map lost {mapping}");
    }
    positions_in_order(
        update,
        &[
            "self.lower_ordinary_property_reference_plan(access)",
            "plan.numeric_update(op, return_mode)",
            "self.update_written_shape(",
        ],
    );
    assert!(ORDINARY_PROPERTY_UPDATE_LOWERING_SOURCE
        .contains("fn ordinary_property_numeric_update_owns_one_reference()"));

    let dispatch = bounded(
        LOWERING_SOURCE,
        "    fn lower_property_access_update(&mut self, op: UpdateOp, access: &PropertyAccess)",
        "    fn lower_update(&mut self, op: UpdateOp, target: &UpdateTarget)",
    );
    for marker in [
        "PropertyAccess::Simple(access) =>",
        "self.lower_ordinary_property_numeric_update(op, access)",
        "PropertyAccess::Super(access) => self.lower_super_property_numeric_update(op, access)",
        "PropertyAccess::Private(_) => self.unsupported_expr(\"private field update target\")",
    ] {
        assert!(dispatch.contains(marker), "dispatch lost {marker}");
    }
    assert!(!dispatch.contains("_ =>"));
    assert!(!IR_SOURCE.contains("\n    PropertyUpdate {"));
    assert!(!LOWERING_SOURCE.contains("ExprIr::PropertyUpdate"));
}

#[test]
fn aot_typestate_forces_get_tonumeric_delta_put_and_result_publication() {
    for prefix in [
        "#[derive(Debug)]\n#[must_use = \"a raw ordinary Property Reference must be consumed by GetValue\"]\nstruct EvaluatedRawOrdinaryPropertyReferenceLocals",
        "#[derive(Debug)]\n#[must_use = \"a numeric ordinary Property Reference must be advanced to its new value\"]\nstruct ReadOrdinaryPropertyNumericUpdateLocals",
        "#[derive(Debug)]\n#[must_use = \"a ready numeric ordinary Property Reference must be consumed by PutValue\"]\nstruct ReadyToWriteOrdinaryPropertyNumericUpdateLocals",
    ] {
        assert!(EXPRESSIONS_SOURCE.contains(prefix), "missing typestate {prefix}");
    }
    let roles = bounded(
        EXPRESSIONS_SOURCE,
        "struct EvaluatedRawOrdinaryPropertyReferenceLocals {",
        "/// The sealed input required by the shared ordinary Reference evaluator.",
    );
    assert!(!roles.contains("Clone"));
    assert!(!roles.contains("Copy"));

    let sealed = bounded(
        EXPRESSIONS_SOURCE,
        "trait OrdinaryPropertyReferenceSource {",
        "#[derive(Debug)]\n#[must_use = \"a raw Super Property Reference",
    );
    assert_eq!(
        sealed
            .matches("impl OrdinaryPropertyReferenceSource for ")
            .count(),
        2
    );
    assert!(sealed.contains(
        "impl OrdinaryPropertyReferenceSource for OrdinaryPropertyEagerCompoundAssignmentIr"
    ));
    assert!(
        sealed.contains("impl OrdinaryPropertyReferenceSource for OrdinaryPropertyNumericUpdateIr")
    );

    let get_numeric = bounded(
        EXPRESSIONS_SOURCE,
        "    fn emit_get_numeric_value_from_raw_ordinary_property_reference(",
        "    fn emit_numeric_update_from_read_ordinary_property_reference(",
    );
    positions_in_order(
        get_numeric,
        &[
            "self.emit_get_value_from_raw_ordinary_property_reference(",
            "let ReadOrdinaryPropertyReferenceLocals {",
            "match value_kind {",
            "ValueKind::Dynamic =>",
            "self.emit_value_to_numeric_locals(old_value_payload, old_value_tag, function)?",
            "ValueKind::Number =>",
            "ValueKind::BigInt => {}",
            "unreachable!(\"ordinary property numeric update requires Number, BigInt, or Dynamic\")",
            "Ok(ReadOrdinaryPropertyNumericUpdateLocals {",
        ],
    );
    assert!(!get_numeric.contains("_ =>"));

    let delta = bounded(
        EXPRESSIONS_SOURCE,
        "    fn emit_numeric_update_from_read_ordinary_property_reference(",
        "    fn emit_put_value_from_ready_ordinary_property_numeric_update(",
    );
    positions_in_order(
        delta,
        &[
            "let ReadOrdinaryPropertyNumericUpdateLocals {",
            "match update.op() {",
            "NumericUpdateOp::Increment =>",
            "NumericUpdateOp::Decrement =>",
            "Instruction::LocalSet(new_value_payload)",
            "Ok(ReadyToWriteOrdinaryPropertyNumericUpdateLocals {",
        ],
    );
    assert!(!delta.contains("_ =>"));

    let put = bounded(
        EXPRESSIONS_SOURCE,
        "    fn emit_put_value_from_ready_ordinary_property_numeric_update(",
        "    fn compile_ordinary_property_numeric_update_to_locals(",
    );
    positions_in_order(
        put,
        &[
            "let ReadyToWriteOrdinaryPropertyNumericUpdateLocals {",
            "self.emit_ordinary_set_result_via_helper(",
            "if update.strictness().throws_on_failed_set() {",
            "self.emit_throw_runtime_error_to_active_handler(",
            "\"Cannot assign to property\"",
            "match update.return_mode() {",
            "UpdateReturnMode::Prefix => (new_value_payload, new_value_tag)",
            "UpdateReturnMode::Postfix => (old_value_payload, old_value_tag)",
            "Instruction::LocalGet(published_payload)",
            "Instruction::LocalSet(payload_local)",
            "Instruction::LocalGet(published_tag)",
            "Instruction::LocalSet(tag_local)",
        ],
    );
    assert!(!put.contains("_ =>"));
    assert!(!put.contains("emit_value_to_property_key_locals("));

    let entry = bounded(
        EXPRESSIONS_SOURCE,
        "    fn compile_ordinary_property_numeric_update_to_locals(",
        "    fn evaluate_raw_super_property_reference(",
    );
    positions_in_order(
        entry,
        &[
            "self.evaluate_raw_ordinary_property_reference(update, function)?",
            "self.emit_get_numeric_value_from_raw_ordinary_property_reference(",
            "self.emit_numeric_update_from_read_ordinary_property_reference(",
            "self.emit_put_value_from_ready_ordinary_property_numeric_update(",
        ],
    );
    assert!(!EXPRESSIONS_SOURCE.contains("compile_property_update_to_locals"));
}

#[test]
fn exhaustive_consumers_and_temp_budget_name_each_numeric_update_phase() {
    for marker in [
        "const ORDINARY_PROPERTY_MUTATION_READ_PERSISTENT_TEMP_LOCALS: usize = 2 + 4;",
        "const ORDINARY_PROPERTY_MUTATION_WRITE_PERSISTENT_TEMP_LOCALS: usize = 2 + 4 + 3;",
        "const ORDINARY_PROPERTY_MUTATION_GET_VALUE_TEMP_LOCALS: usize = 2;",
        "const ORDINARY_PROPERTY_MUTATION_TO_NUMERIC_TEMP_LOCALS: usize = 4;",
        "const ORDINARY_PROPERTY_MUTATION_SET_HELPER_TEMP_LOCALS: usize = 4 + 2;",
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
        "        ExprIr::OrdinaryPropertyNumericUpdate(update) => {",
        "        ExprIr::OrdinaryPropertyEagerCompoundAssignment(mutation) => {",
    );
    positions_in_order(
        budget,
        &[
            "let to_numeric_temps = match update.value_kind()",
            "ValueKind::Dynamic => ORDINARY_PROPERTY_MUTATION_TO_NUMERIC_TEMP_LOCALS",
            "let read_phase = ORDINARY_PROPERTY_MUTATION_READ_PERSISTENT_TEMP_LOCALS",
            ".max(ORDINARY_PROPERTY_MUTATION_GET_VALUE_TEMP_LOCALS)",
            ".max(to_numeric_temps)",
            "let write_phase = ORDINARY_PROPERTY_MUTATION_WRITE_PERSISTENT_TEMP_LOCALS",
            "+ ORDINARY_PROPERTY_MUTATION_SET_HELPER_TEMP_LOCALS",
            "read_phase.max(write_phase)",
        ],
    );
    assert_eq!(
        EXPRESSIONS_SOURCE
            .matches("ExprIr::OrdinaryPropertyNumericUpdate(update) =>")
            .count(),
        2,
        "both expression emission entry points must consume the fused update"
    );
    assert_eq!(
        DATA_SOURCE
            .matches("ExprIr::OrdinaryPropertyNumericUpdate(update) =>")
            .count(),
        1,
        "data collection must traverse the fused update"
    );
    assert_eq!(
        PLANNING_SOURCE
            .matches("ExprIr::OrdinaryPropertyNumericUpdate(update) =>")
            .count(),
        7,
        "every planning traversal must name the fused update"
    );
}

#[test]
fn fixture_observes_all_modes_numeric_domains_and_reference_phases() {
    for marker in [
        "++values[updateKey(\"numberPrefixIncrement\")]",
        "values[updateKey(\"numberPostfixIncrement\")]++",
        "--values[updateKey(\"numberPrefixDecrement\")]",
        "values[updateKey(\"numberPostfixDecrement\")]--",
        "++values[updateKey(\"bigintPrefixIncrement\")]",
        "values[updateKey(\"bigintPostfixIncrement\")]++",
        "--values[updateKey(\"bigintPrefixDecrement\")]",
        "values[updateKey(\"bigintPostfixDecrement\")]--",
    ] {
        assert!(FIXTURE.contains(marker), "fixture lost mode {marker}");
    }
    for oracle in [
        "one key coercion per update",
        "abrupt base order",
        "abrupt raw key order",
        "nullishReference(null, \"null base\")",
        "nullishReference(undefined, \"undefined base\")",
        "ToPropertyKey abrupt order",
        "complete numeric update Reference lifecycle",
        "mutated raw key not recoerced",
        "ToNumeric abrupt nonpublication",
        "ToNumeric abrupt skips Set",
        "strict Set false nonpublication",
        "strict Set false no write",
        "sloppy Set false postfix result",
        "sloppy Set false ignored write",
    ] {
        assert!(FIXTURE.contains(oracle), "fixture lost oracle {oracle}");
    }
    assert!(FIXTURE.contains(
        "base,raw-key,to-key:p,proxy-get:p:true,getter:true,to-numeric,proxy-set:p:true:2,setter:true:2"
    ));
    assert!(FIXTURE.contains("\"use strict\";\n  return rejectingProxy[rejectingKey]++;"));
    assert!(CLI_SOURCE
        .contains("fn run_wasm_backend_preserves_ordinary_property_numeric_update_reference()"));
    assert!(CLI_SOURCE.contains("wasm_ordinary_property_numeric_update_reference.js"));
}

#[test]
fn exact_a6_inventory_is_raw_unmasked_and_runs_in_both_modes() {
    assert_eq!(EXACT_TEST262.len(), 4);
    for (path, source) in EXACT_TEST262 {
        assert!(
            !source.contains("flags:"),
            "{path} must execute in both modes"
        );
        let basename = path.rsplit('/').next().expect("exact basename");
        assert!(!RUNNER_SOURCE.contains(basename), "runner masks {path}");
        assert!(
            !KNOWN_FAILURES.contains(basename),
            "known failures mask {path}"
        );
        assert!(source.contains("base = null"));
        assert!(source.contains("throw new DummyError()"));
        assert!(source.contains("property key evaluated"));
    }
}

#[test]
fn dry_status_records_the_exact_current_baseline_and_nonclaims() {
    for source in [README, TASK] {
        for marker in [
            "0f004c0c6",
            "0/8",
            "Runtime/Bug",
            "nullish-base `TypeError`",
            "Post-batch verification is green",
            "8/8",
            "60.43s",
            "zero unsupported",
        ] {
            assert!(source.contains(marker), "status lost {marker}");
        }
    }
    for marker in [
        "four physical files and eight executions",
        "NumericUpdateOp::{Increment, Decrement}",
        "UpdateReturnMode::{Prefix, Postfix}",
        "ToPropertyKey` exactly once",
        "apply `ToNumeric` exactly once",
        "This batch does not change eager or logical compound assignment",
        "resumable/suspended property",
    ] {
        assert!(CONTRACT.contains(marker), "contract lost {marker}");
    }
}
