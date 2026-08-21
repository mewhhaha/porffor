const REFERENCE_SOURCE: &str = include_str!("../../lila-ir/src/reference.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const COMPOUND_SOURCE: &str =
    include_str!("../../lila-ir/src/lowering/with_environment_compound.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_with_environment_compound_assignment.js");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/with-environment-eager-compound-assignment-reference.md"
);

macro_rules! witness {
    ($path:literal) => {
        (
            $path,
            include_str!(concat!("../../../test262/vendor/test262/test/", $path)),
        )
    };
}

const VENDORED_WITNESSES: [(&str, &str); 44] = [
    witness!("language/expressions/compound-assignment/S11.13.2_A5.1_T1.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.1_T2.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.1_T3.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.2_T1.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.2_T2.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.2_T3.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.3_T1.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.3_T2.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.3_T3.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.4_T1.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.4_T2.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.4_T3.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.5_T1.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.5_T2.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.5_T3.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.6_T1.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.6_T2.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.6_T3.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.7_T1.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.7_T2.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.7_T3.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.8_T1.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.8_T2.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.8_T3.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.9_T1.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.9_T2.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.9_T3.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.10_T1.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.10_T2.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.10_T3.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.11_T1.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.11_T2.js"),
    witness!("language/expressions/compound-assignment/S11.13.2_A5.11_T3.js"),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v-.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--2.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--4.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--6.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--8.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--10.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--12.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--14.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--16.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--18.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--20.js"
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
fn one_nonempty_noncopy_plan_owns_the_complete_compound_assignment() {
    assert!(REFERENCE_SOURCE.contains(
        "#[must_use = \"a with-environment Reference must be consumed by GetValue, PutValue, numeric update, or compound assignment\"]\npub(crate) struct WithEnvironmentReferencePlan {"
    ));
    let plan_type = bounded(
        REFERENCE_SOURCE,
        "#[must_use = \"a with-environment Reference must be consumed by GetValue, PutValue, numeric update, or compound assignment\"]",
        "/// One identifier Reference selected by the Global Environment Record's",
    );
    assert!(!plan_type.contains("Clone"));
    assert!(!plan_type.contains("Copy"));
    let plan_consumer = bounded(
        REFERENCE_SOURCE,
        "impl WithEnvironmentReferencePlan {",
        "/// `[[Strict]]` of a Reference Record (6.2.5).",
    );
    assert!(plan_consumer.contains("pub(crate) fn compound_assignment("));
    assert!(plan_consumer.contains("assignment: EagerCompoundAssignment"));
    assert!(plan_consumer.contains("for environment in outer"));
    assert!(plan_consumer.contains("innermost.compound_assignment_or_else("));

    let bindings = bounded(
        REFERENCE_SOURCE,
        "pub(crate) struct EagerCompoundAssignmentBindings {",
        "impl NumericUpdateBindings {",
    );
    for marker in [
        "old_value: String",
        "result: String",
        "write: String",
        "pub(crate) fn allocate(",
        "pub(crate) fn old_value(&self) -> TypedExpr",
        "pub(crate) fn seal(self, result: TypedExpr)",
        "pub(crate) struct EagerCompoundAssignment {",
        "bindings: EagerCompoundAssignmentBindings",
        "result: TypedExpr",
    ] {
        assert!(bindings.contains(marker), "missing sealed role: {marker}");
    }
    assert_before(
        bindings,
        "allocate(\"object.environment.compound.old.\")",
        "allocate(\"object.environment.compound.result.\")",
    );
    assert_before(
        bindings,
        "allocate(\"object.environment.compound.result.\")",
        "allocate(\"object.environment.compound.write.\")",
    );
    assert!(bindings.contains(
        "#[must_use = \"a sealed eager compound assignment must consume its Reference plan\"]"
    ));
}

#[test]
fn selected_branch_orders_get_apply_put_and_result_on_one_object() {
    let assignment = bounded(
        REFERENCE_SOURCE,
        "    fn compound_assignment_or_else(",
        "impl SelectedWithEnvironmentObjects {",
    );
    for marker in [
        "let binding_visible = binding_object.binding_visible(",
        "binding_object.eager_compound_assignment(referenced_name, strictness, assignment);",
        "let result_info = selected_assignment.value_info();",
        "condition: Box::new(binding_visible)",
        "then_expr: Box::new(selected_assignment)",
        "else_expr: Box::new(fallback)",
    ] {
        assert!(
            assignment.contains(marker),
            "missing assignment boundary: {marker}"
        );
    }
    assert_before(
        assignment,
        "let binding_visible =",
        "let selected_assignment =",
    );
    assert_before(assignment, "let selected_assignment =", "let result_info =");
    assert!(!assignment.contains("ExprIr::PropertyCompoundAssign"));
    assert!(!assignment.contains("ExprIr::PropertyUpdate"));
}

#[test]
fn lowering_exhausts_twelve_eager_ops_and_keeps_logical_assignment_out() {
    let op = bounded(
        COMPOUND_SOURCE,
        "enum EagerCompoundAssignmentOp {",
        "impl<'a> ScriptLowerer<'a> {",
    );
    assert!(op.contains("Arithmetic(ArithmeticOp)"));
    assert!(op.contains("Bitwise(BitwiseOp)"));
    assert!(!op.contains("Logical"));

    let arithmetic = bounded(
        LOWERING_SOURCE,
        "            AssignOp::Add\n            | AssignOp::Sub",
        "            AssignOp::BoolAnd | AssignOp::BoolOr | AssignOp::Coalesce => {",
    );
    for mapping in [
        "AssignOp::Add => ArithmeticOp::Add",
        "AssignOp::Sub => ArithmeticOp::Sub",
        "AssignOp::Mul => ArithmeticOp::Mul",
        "AssignOp::Div => ArithmeticOp::Div",
        "AssignOp::Mod => ArithmeticOp::Mod",
        "AssignOp::Exp => ArithmeticOp::Exp",
    ] {
        assert!(
            arithmetic.contains(mapping),
            "missing arithmetic map: {mapping}"
        );
    }
    assert!(arithmetic.contains("self.locate_identifier_reference(&name)"));
    assert!(arithmetic.contains(".select_preceding(reference.declarative_position())"));
    assert_before(
        arithmetic,
        "let reference =",
        "let value = self.lower_expression(rhs)",
    );
    assert!(arithmetic.contains("EagerCompoundAssignmentOp::Arithmetic(arithmetic)"));
    assert!(arithmetic.contains("self.lower_with_scoped_identifier_eager_compound_assignment("));
    let selected_arithmetic = bounded(
        arithmetic,
        "let name = self.interner.resolve_expect(identifier.sym()).to_string();\n                let arithmetic = match op {",
        "                let reference = self.locate_identifier_reference(&name);",
    );
    assert!(!selected_arithmetic.contains("_ =>"));

    let logical = bounded(
        LOWERING_SOURCE,
        "            AssignOp::BoolAnd | AssignOp::BoolOr | AssignOp::Coalesce => {",
        "            AssignOp::And\n            | AssignOp::Or",
    );
    assert!(!logical.contains("EagerCompoundAssignmentOp"));
    assert!(!logical.contains("lower_with_scoped_identifier_eager_compound_assignment"));

    let bitwise = bounded(
        LOWERING_SOURCE,
        "            AssignOp::And\n            | AssignOp::Or",
        "    fn lower_web_compat_call_assignment_target(&mut self, call: &Call) -> TypedExpr {",
    );
    for mapping in [
        "AssignOp::And => BitwiseOp::And",
        "AssignOp::Or => BitwiseOp::Or",
        "AssignOp::Xor => BitwiseOp::Xor",
        "AssignOp::Shl => BitwiseOp::Shl",
        "AssignOp::Shr => BitwiseOp::Shr",
        "AssignOp::Ushr => BitwiseOp::UShr",
    ] {
        assert!(bitwise.contains(mapping), "missing bitwise map: {mapping}");
    }
    assert!(bitwise.contains("self.locate_identifier_reference(&name)"));
    assert!(bitwise.contains(".select_preceding(reference.declarative_position())"));
    assert_before(
        bitwise,
        "let reference =",
        "let value = self.lower_expression(rhs)",
    );
    assert!(bitwise.contains("EagerCompoundAssignmentOp::Bitwise(bitwise)"));
    assert!(bitwise.contains("self.lower_with_scoped_identifier_eager_compound_assignment("));
    let selected_bitwise = bounded(
        bitwise,
        "let name = self.interner.resolve_expect(identifier.sym()).to_string();\n                let bitwise = match op {",
        "                let reference = self.locate_identifier_reference(&name);",
    );
    assert!(!selected_bitwise.contains("_ =>"));

    let apply = bounded(
        COMPOUND_SOURCE,
        "    fn apply_eager_compound_assignment(",
        "}\n\n#[cfg(test)]",
    );
    for marker in [
        "EagerCompoundAssignmentOp::Arithmetic(ArithmeticOp::Add)",
        "ArithmeticOp::Sub => ArithmeticBinaryOp::Sub",
        "ArithmeticOp::Mul => ArithmeticBinaryOp::Mul",
        "ArithmeticOp::Div => ArithmeticBinaryOp::Div",
        "ArithmeticOp::Mod => ArithmeticBinaryOp::Mod",
        "ArithmeticOp::Exp => ArithmeticBinaryOp::Exp",
        "BitwiseOp::And => BitwiseBinaryOp::And",
        "BitwiseOp::Or => BitwiseBinaryOp::Or",
        "BitwiseOp::Xor => BitwiseBinaryOp::Xor",
        "BitwiseOp::Shl => BitwiseBinaryOp::Shl",
        "BitwiseOp::Shr => BitwiseBinaryOp::Shr",
        "BitwiseOp::UShr => BitwiseBinaryOp::UShr",
    ] {
        assert!(apply.contains(marker), "missing closed operation: {marker}");
    }
    assert!(!apply.contains("_ =>"));
}

#[test]
fn fallback_is_dynamic_and_runtime_guarded_after_observable_selection() {
    let helper = bounded(
        COMPOUND_SOURCE,
        "    pub(super) fn lower_with_scoped_identifier_eager_compound_assignment(",
        "    /// The canonical dynamic operation shape used by both a selected Object",
    );
    for marker in [
        "let plan = self.with_environment_reference_plan(",
        "rhs.clone()",
        "EagerCompoundAssignmentBindings::allocate(",
        "let old_value = bindings.old_value();",
        "let applied = Self::apply_eager_compound_assignment(",
        "plan.compound_assignment(bindings.seal(applied), fallback)",
        "self.set_binding_value_info(&name, unknown_runtime_value_info());",
        "lower_global_object_environment_eager_compound_assignment(",
        "info.value_info = unknown_runtime_value_info();",
        "info.proven_present = false;",
        "GlobalObjectEnvironmentReferencePlan::new(self.global_this_info(), name, strictness)",
        ".compound_assignment(bindings.seal(applied))",
    ] {
        assert!(
            helper.contains(marker),
            "missing fallback boundary: {marker}"
        );
    }
    assert_before(helper, "let plan =", "let fallback =");
    assert_before(helper, "let fallback =", "let bindings =");
    assert_before(helper, "let bindings =", "let old_value =");
    assert_before(helper, "let old_value =", "let applied =");
    assert_before(helper, "info.value_info =", "info.proven_present = false;");
}

#[test]
fn consumer_and_exact_current_pin_inventory_cover_the_durable_contract() {
    for marker in [
        "addResult = addValue += 2",
        "subResult = subValue -= 2",
        "mulResult = mulValue *= 2",
        "divResult = divValue /= 3",
        "modResult = modValue %= 5",
        "expResult = expValue **= 3",
        "andResult = andValue &= 11",
        "orResult = orValue |= 3",
        "xorResult = xorValue ^= 10",
        "shlResult = shlValue <<= 2",
        "shrResult = shrValue >>= 2",
        "ushrResult = ushrValue >>>= 2",
        "trace === \"huhgdrhs\"",
        "target.tracedValue === 5",
        "functionFallbackValue === 100",
        "globalFallbackValue === 101",
        "outerFallbackScope.outerFallbackValue === 102",
        "strictCompoundCaught = error instanceof ReferenceError",
        "dynamicFallbackValue = \"4\"",
        "dynamicFallbackValue === \"41\"",
        "selectedFallbackValue = marker",
        "selectedFallbackValue === marker",
        "delete globalThis.deletedCompoundFallback",
        "deletedFallbackCaught = error instanceof ReferenceError",
        "!(\"deletedCompoundFallback\" in globalThis)",
        "globalThis.createdCompoundFallback = 4",
        "createdFallbackResult = createdCompoundFallback <<= 1",
        "globalThis.createdCompoundFallback === 8",
    ] {
        assert!(FIXTURE.contains(marker), "missing CLI witness: {marker}");
    }

    assert_eq!(VENDORED_WITNESSES.len(), 44);
    for (path, source) in VENDORED_WITNESSES {
        assert!(
            source.contains("flags: [noStrict]"),
            "wrong metadata: {path}"
        );
        assert!(source.contains("with ("), "missing Object ER use: {path}");
        assert!(source.contains("delete this.x"), "missing deletion: {path}");
    }
    for (_, source) in VENDORED_WITNESSES.into_iter().skip(33) {
        assert!(source.contains("assert.throws(ReferenceError"));
        assert!(source.contains("\"use strict\""));
    }
    for operator in [
        "x *= ", "x /= ", "x %= ", "x += ", "x -= ", "x <<= ", "x >>= ", "x >>>= ", "x &= ",
        "x ^= ", "x |= ",
    ] {
        assert!(
            VENDORED_WITNESSES
                .iter()
                .any(|(_, source)| source.contains(operator)),
            "missing evidenced operator: {operator}"
        );
    }
    assert!(VENDORED_WITNESSES
        .iter()
        .all(|(_, source)| !source.contains("x **= ")));

    assert!(CONTRACT.contains("44 `noStrict` files"));
    assert!(CONTRACT.contains("The producer's closed eager domain also includes `**=`"));
    assert!(CONTRACT.contains("not a forty-fifth Test262 claim"));
    assert!(CONTRACT.contains("Static post-expression metadata"));
    assert!(CONTRACT.contains("becomes all-runtime-tags Dynamic"));
    assert!(CONTRACT.contains("A configurable tracked global loses its static"));
    assert!(CONTRACT.contains("`proven_present` fact"));
}
