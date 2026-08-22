const REFERENCE_SOURCE: &str = include_str!("../../lila-ir/src/reference.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const COMPOUND_SOURCE: &str =
    include_str!("../../lila-ir/src/lowering/with_environment_compound.rs");
const FIXTURE: &str = include_str!(
    "../../lila-cli/tests/fixtures/wasm_global_object_environment_compound_assignment.js"
);
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/global-object-environment-eager-compound-assignment-reference.md"
);

macro_rules! witness {
    ($path:literal) => {
        (
            $path,
            include_str!(concat!("../../../test262/vendor/test262/test/", $path)),
        )
    };
}

const SELECTED_WITNESSES: [(&str, &str); 11] = [
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

const ADJACENT_PREFIX_WITNESSES: [(&str, &str); 22] = [
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v-.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--1.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--2.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--3.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--4.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--5.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--6.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--7.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--8.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--9.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--10.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--11.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--12.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--13.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--14.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--15.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--16.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--17.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--18.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--19.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--20.js"
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
fn distinct_noncopy_global_plan_selects_plain_has_property_without_unscopables() {
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
        "fn get_value(self, referenced_name: &str, strictness: Strictness)",
        "fn put_value(",
        "fn eager_compound_assignment(",
    ] {
        assert!(
            objects.contains(marker),
            "missing object boundary: {marker}"
        );
    }

    let plan = bounded(
        REFERENCE_SOURCE,
        "#[must_use = \"a global Object Environment Reference must be consumed by logical assignment, numeric update, or eager compound assignment\"]",
        "/// Compiler-private bindings used by one Object Environment numeric update.",
    );
    assert!(plan.contains("pub(crate) struct GlobalObjectEnvironmentReferencePlan"));
    assert!(!plan.contains("Clone"));
    assert!(!plan.contains("Copy"));
    assert!(plan.contains("ObjectEnvironmentBindingObject::global_object(global_object_info)"));
    assert!(plan.contains("pub(crate) fn compound_assignment(self,"));
    assert!(plan.contains("let present = binding_object.has_property(&referenced_name);"));
    assert!(plan.contains("binding_object.eager_compound_assignment("));
    assert!(plan.contains("name: NativeErrorKind::ReferenceError"));
    assert!(plan.contains("condition: Box::new(present)"));
    assert!(plan.contains("then_expr: Box::new(selected)"));
    assert!(plan.contains("else_expr: Box::new(missing)"));
    assert!(!plan.contains("binding_visible"));
    assert!(!plan.contains("unscopables_binding"));
    assert_before(plan, "let present =", "let selected =");
    assert_before(plan, "let selected =", "let missing =");
}

#[test]
fn shared_sealed_lifecycle_rechecks_get_and_put_before_exposing_result() {
    let objects = bounded(
        REFERENCE_SOURCE,
        "impl ObjectEnvironmentBindingObject {",
        "/// Declarative-frame depth in the function currently being lowered.",
    );
    let get = bounded(
        objects,
        "    fn get_value(self, referenced_name: &str, strictness: Strictness) -> TypedExpr {",
        "    /// SetMutableBinding on the Object Environment Record selected before RHS.",
    );
    assert!(get.contains("let recheck = self.has_property(referenced_name);"));
    assert!(get.contains("ExprIr::PropertyRead"));
    assert!(get.contains("Strictness::Sloppy => TypedExpr::undefined()"));
    assert!(get.contains("Strictness::Strict => TypedExpr::from_info("));
    assert!(get.contains("name: NativeErrorKind::ReferenceError"));
    assert_before(get, "let recheck =", "let read =");

    let put = bounded(
        objects,
        "    fn put_value(",
        "    /// GetValue, eager operation, same-base PutValue, then result.",
    );
    assert!(put.contains("let recheck = self.has_property(referenced_name);"));
    assert!(put.contains("ExprIr::PropertyWrite"));
    assert!(put.contains("Strictness::Sloppy => TypedExpr::from_info("));
    assert!(put.contains("value: Box::new(recheck)"));
    assert!(put.contains("Strictness::Strict => TypedExpr::from_info("));
    assert!(put.contains("condition: Box::new(recheck)"));
    assert!(put.contains("name: NativeErrorKind::ReferenceError"));
    assert_before(put, "let recheck =", "let write =");

    let lifecycle = bounded(
        REFERENCE_SOURCE,
        "    fn eager_compound_assignment(",
        "/// Declarative-frame depth in the function currently being lowered.",
    );
    for marker in [
        "EagerCompoundAssignmentBindings {",
        "old_value: old_value_name",
        "result: result_name",
        "write: write_name",
        "let old_value = self.clone().get_value(referenced_name, strictness);",
        "let write = self.put_value(referenced_name, strictness, result.clone());",
        "name: write_name.clone()",
        "name: result_name.clone()",
        "name: old_value_name.clone()",
    ] {
        assert!(
            lifecycle.contains(marker),
            "missing lifecycle boundary: {marker}"
        );
    }
    assert_before(lifecycle, "let old_value =", "let result_info =");
    assert_before(lifecycle, "let result_info =", "let write =");
    assert_before(lifecycle, "let write =", "let after_write =");
    assert_before(lifecycle, "let after_write =", "let after_apply =");
    assert!(!lifecycle.contains("ExprIr::GlobalPropertyCompoundAssign"));

    let carrier = bounded(
        REFERENCE_SOURCE,
        "pub(crate) struct EagerCompoundAssignmentBindings {",
        "impl NumericUpdateBindings {",
    );
    for marker in [
        "old_value: String",
        "result: String",
        "write: String",
        "pub(crate) fn allocate(",
        "allocate(\"object.environment.compound.old.\")",
        "allocate(\"object.environment.compound.result.\")",
        "allocate(\"object.environment.compound.write.\")",
        "pub(crate) fn old_value(&self) -> TypedExpr",
        "pub(crate) fn seal(self, result: TypedExpr) -> EagerCompoundAssignment",
        "pub(crate) struct EagerCompoundAssignment {",
    ] {
        assert!(
            carrier.contains(marker),
            "missing carrier boundary: {marker}"
        );
    }
    assert!(carrier.contains(
        "#[must_use = \"a sealed eager compound assignment must consume its Reference plan\"]"
    ));
}

#[test]
fn lowering_routes_the_closed_eager_domain_through_the_global_plan() {
    let helper = bounded(
        COMPOUND_SOURCE,
        "    pub(super) fn lower_global_object_environment_eager_compound_assignment(",
        "\n}\n\n#[cfg(test)]",
    );
    for marker in [
        "info.value_info = unknown_runtime_value_info();",
        "if info.configurable",
        "info.proven_present = false;",
        "EagerCompoundAssignmentBindings::allocate(",
        "op.apply(bindings.old_value(), rhs)",
        "let strictness = self.reference_strictness();",
        "GlobalObjectEnvironmentReferencePlan::new(self.global_this_info(), name, strictness)",
        ".compound_assignment(bindings.seal(applied))",
    ] {
        assert!(
            helper.contains(marker),
            "missing global lowering seam: {marker}"
        );
    }
    assert_before(helper, "let bindings =", "let applied =");
    assert_before(helper, "let applied =", "let strictness =");
    assert_before(
        helper,
        "let strictness =",
        "GlobalObjectEnvironmentReferencePlan::new(",
    );
    assert!(!helper.contains("GlobalPropertyCompoundAssign"));

    let operation = bounded(
        REFERENCE_SOURCE,
        "pub(crate) enum EagerCompoundAssignmentOp {",
        "impl EagerCompoundAssignmentOp {",
    );
    assert!(operation.contains("Arithmetic(ArithmeticOp)"));
    assert!(operation.contains("Bitwise(BitwiseOp)"));
    assert!(!operation.contains("Logical"));

    let apply = bounded(
        REFERENCE_SOURCE,
        "impl EagerCompoundAssignmentOp {",
        "/// One fused mutation of a Super Property Reference.",
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
        assert!(
            apply.contains(marker),
            "missing exhaustive operation: {marker}"
        );
    }
    assert!(!apply.contains("_ =>"));

    let arithmetic = bounded(
        LOWERING_SOURCE,
        "            AssignOp::Add\n            | AssignOp::Sub",
        "            AssignOp::BoolAnd | AssignOp::BoolOr | AssignOp::Coalesce => {",
    );
    let bitwise = bounded(
        LOWERING_SOURCE,
        "            AssignOp::And\n            | AssignOp::Or",
        "    fn lower_web_compat_call_assignment_target(&mut self, call: &Call) -> TypedExpr {",
    );
    for arm in [arithmetic, bitwise] {
        assert!(arm.contains("self.locate_identifier_reference(&name)"));
        assert!(arm.contains("lower_with_scoped_identifier_eager_compound_assignment("));
        assert!(arm.contains("lower_global_object_environment_eager_compound_assignment("));
    }
}

#[test]
fn consumer_and_exact_current_pin_inventory_cover_the_durable_contract() {
    for marker in [
        "globalXorValue ^= compoundRhs()",
        "globalOrValue |= compoundRhs()",
        "globalMulValue *= compoundRhs()",
        "globalDivValue /= compoundRhs()",
        "globalModValue %= compoundRhs()",
        "globalAddValue += compoundRhs()",
        "globalSubValue -= compoundRhs()",
        "globalShlValue <<= compoundRhs()",
        "globalShrValue >>= compoundRhs()",
        "globalUshrValue >>>= compoundRhs()",
        "globalAndValue &= compoundRhs()",
        "strictResult === \"not written\"",
        "initiallyAbsentGlobal += absentRhs()",
        "absentRhsCount === 0",
        "let sloppyResult = sloppyGlobalValue += sloppyRhs()",
        "sloppyTrace === \"gr\"",
        "Object.prototype.hasOwnProperty.call(globalThis, \"sloppyGlobalValue\")",
        "let inheritedResult = inheritedGlobalValue -= 2",
        "Object.prototype.hasOwnProperty.call(globalThis, \"inheritedGlobalValue\")",
    ] {
        assert!(FIXTURE.contains(marker), "missing CLI witness: {marker}");
    }

    assert_eq!(SELECTED_WITNESSES.len(), 11);
    for (path, source) in SELECTED_WITNESSES {
        assert!(path.ends_with(".js"));
        assert!(source.contains("flags: [noStrict]"), "missing flag: {path}");
        assert!(
            source.contains("Object.defineProperty(this, \"x\""),
            "{path}"
        );
        assert!(source.contains("\"use strict\""), "{path}");
        assert!(source.contains("assert.throws(ReferenceError"), "{path}");
        assert!(
            !source.contains("with (scope)"),
            "global witness used with: {path}"
        );
        assert!(
            CONTRACT.contains(path.rsplit('/').next().expect("file name")),
            "{path}"
        );
    }

    assert_eq!(ADJACENT_PREFIX_WITNESSES.len(), 22);
    for (index, (path, source)) in ADJACENT_PREFIX_WITNESSES.into_iter().enumerate() {
        assert!(source.contains("flags: [noStrict]"), "missing flag: {path}");
        if index == 0 || index % 2 == 0 {
            assert!(
                source.contains("with (scope)"),
                "with regression witness: {path}"
            );
        } else {
            assert!(
                source.contains("Object.defineProperty(this, \"x\""),
                "{path}"
            );
        }
    }

    assert!(CONTRACT.contains("The producer's closed eager domain also includes `**=`"));
    assert!(CONTRACT.contains("Logical assignments, property References, declarative bindings"));
    assert!(CONTRACT.contains("subtree and pinned matrix remain later verification checkpoints."));
}
