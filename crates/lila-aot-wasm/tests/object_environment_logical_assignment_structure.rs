const REFERENCE_SOURCE: &str = include_str!("../../lila-ir/src/reference.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const LOGICAL_SOURCE: &str =
    include_str!("../../lila-ir/src/lowering/object_environment_logical.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_object_environment_logical_assignment.js");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/object-environment-logical-assignment-reference.md"
);

macro_rules! witness {
    ($path:literal) => {
        (
            $path,
            include_str!(concat!("../../../test262/vendor/test262/test/", $path)),
        )
    };
}

const SELECTED_WITNESSES: [(&str, &str); 3] = [
    witness!(
        "language/expressions/logical-assignment/lgcl-and-assignment-operator-unresolved-lhs.js"
    ),
    witness!(
        "language/expressions/logical-assignment/lgcl-or-assignment-operator-unresolved-lhs.js"
    ),
    witness!(
        "language/expressions/logical-assignment/lgcl-nullish-assignment-operator-unresolved-lhs.js"
    ),
];

const ADJACENT_RHS_WITNESSES: [(&str, &str); 6] = [
    witness!(
        "language/expressions/logical-assignment/lgcl-and-assignment-operator-unresolved-rhs.js"
    ),
    witness!(
        "language/expressions/logical-assignment/lgcl-and-assignment-operator-unresolved-rhs-put.js"
    ),
    witness!(
        "language/expressions/logical-assignment/lgcl-or-assignment-operator-unresolved-rhs.js"
    ),
    witness!(
        "language/expressions/logical-assignment/lgcl-or-assignment-operator-unresolved-rhs-put.js"
    ),
    witness!(
        "language/expressions/logical-assignment/lgcl-nullish-assignment-operator-unresolved-rhs.js"
    ),
    witness!(
        "language/expressions/logical-assignment/lgcl-nullish-assignment-operator-unresolved-rhs-put.js"
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
fn shared_private_lifecycle_keeps_putvalue_inside_the_taken_branch() {
    let objects = bounded(
        REFERENCE_SOURCE,
        "impl ObjectEnvironmentBindingObject {",
        "/// Declarative-frame depth in the function currently being lowered.",
    );
    let logical = bounded(
        objects,
        "    fn logical_assignment(",
        "    /// GetValue, eager operation, same-base PutValue, then result.",
    );
    for marker in [
        "op: LogicalBinaryOp",
        "let lhs = self.clone().get_value(referenced_name, strictness);",
        "let write = self.put_value(referenced_name, strictness, rhs);",
        "ExprIr::LogicalShortCircuit {",
        "lhs: Box::new(lhs)",
        "rhs: Box::new(write)",
    ] {
        assert!(
            logical.contains(marker),
            "missing shared lifecycle: {marker}"
        );
    }
    assert_before(logical, "let lhs =", "let write =");
    assert_before(logical, "let write =", "ExprIr::LogicalShortCircuit");
    assert!(!logical.contains("ExprIr::GlobalPropertyWrite"));

    let get = bounded(
        objects,
        "    fn get_value(self, referenced_name: &str, strictness: Strictness) -> TypedExpr {",
        "    /// SetMutableBinding on the Object Environment Record selected before RHS.",
    );
    let put = bounded(
        objects,
        "    fn put_value(",
        "    /// GetValue, ToNumeric/delta, same-base PutValue, then prefix/postfix",
    );
    assert!(get.contains("let recheck = self.has_property(referenced_name);"));
    assert!(get.contains("Strictness::Sloppy => TypedExpr::undefined()"));
    assert!(get.contains("name: NativeErrorKind::ReferenceError"));
    assert!(put.contains("let recheck = self.has_property(referenced_name);"));
    assert!(put.contains("Strictness::Sloppy => TypedExpr::from_info("));
    assert!(put.contains("Strictness::Strict => TypedExpr::from_info("));
    assert!(put.contains("ExprIr::PropertyWrite"));
    assert!(put.contains("name: NativeErrorKind::ReferenceError"));
}

#[test]
fn noncopy_with_and_global_plans_consume_one_selected_reference() {
    assert!(REFERENCE_SOURCE.contains(
        "#[derive(Debug)]\n#[must_use = \"a with-environment Reference must be consumed by GetValue, PutValue, logical assignment, numeric update, or compound assignment\"]\npub(crate) struct WithEnvironmentReferencePlan {"
    ));
    let with_impl = bounded(
        REFERENCE_SOURCE,
        "impl WithEnvironmentReferencePlan {",
        "/// `[[Strict]]` of a Reference Record (6.2.5).",
    );
    let with_logical = bounded(
        with_impl,
        "    pub(crate) fn logical_assignment(",
        "    /// Consume one ResolveBinding result for an eager compound assignment.",
    );
    for marker in [
        "self,",
        "op: LogicalBinaryOp",
        "rhs: TypedExpr",
        "fallback: TypedExpr",
        "for environment in outer",
        "environment.logical_assignment_or_else(",
        "rhs.clone()",
        "innermost.logical_assignment_or_else(",
    ] {
        assert!(with_logical.contains(marker), "missing with plan: {marker}");
    }

    let resolution = bounded(
        REFERENCE_SOURCE,
        "    fn logical_assignment_or_else(",
        "    /// Compose one selected Object Environment Record's GetBindingValue,",
    );
    assert!(resolution.contains("binding_object.binding_visible("));
    assert!(resolution.contains("binding_object.logical_assignment("));
    assert!(resolution.contains("condition: Box::new(binding_visible)"));
    assert!(resolution.contains("then_expr: Box::new(selected)"));
    assert!(resolution.contains("else_expr: Box::new(fallback)"));
    assert_before(resolution, "let binding_visible =", "let selected =");

    assert!(REFERENCE_SOURCE.contains(
        "#[derive(Debug)]\n#[must_use = \"a global Object Environment Reference must be consumed by logical assignment, numeric update, or eager compound assignment\"]\npub(crate) struct GlobalObjectEnvironmentReferencePlan {"
    ));
    let global_impl = bounded(
        REFERENCE_SOURCE,
        "impl GlobalObjectEnvironmentReferencePlan {",
        "/// Compiler-private bindings used by one Object Environment numeric update.",
    );
    assert!(
        global_impl.contains("ObjectEnvironmentBindingObject::global_object(global_object_info)")
    );
    assert!(!global_impl.contains("binding_visible("));
    assert!(!global_impl.contains("unscopables_binding"));
    let global_logical = bounded(
        global_impl,
        "    pub(crate) fn logical_assignment(",
        "\n    }\n}",
    );
    for marker in [
        "let present = binding_object.has_property(&referenced_name);",
        "binding_object.logical_assignment(&referenced_name, strictness, op, rhs)",
        "name: NativeErrorKind::ReferenceError",
        "condition: Box::new(present)",
        "then_expr: Box::new(selected)",
        "else_expr: Box::new(missing)",
    ] {
        assert!(
            global_logical.contains(marker),
            "missing global plan: {marker}"
        );
    }
    assert_before(global_logical, "let present =", "let selected =");
    assert_before(global_logical, "let selected =", "let missing =");
}

#[test]
fn pre_rhs_location_snapshot_and_closed_mapper_make_ordering_explicit() {
    let located = bounded(
        LOGICAL_SOURCE,
        "/// One identifier logical-assignment Reference located before RHS lowering.",
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
    );
    assert!(located.contains("pub(super) struct LocatedIdentifierLogicalAssignment {"));
    assert!(located.contains(
        "#[must_use = \"a pre-RHS logical-assignment Reference must be consumed after RHS lowering\"]\npub(super) struct LocatedIdentifierLogicalAssignment {"
    ));
    assert!(located.contains("reference: LocatedIdentifierReference"));
    assert!(located.contains("proven_global_value: Option<ValueInfo>"));
    assert!(located.contains("pub(super) fn reject_definite_tdz(self)"));
    assert!(!located.contains("#[derive"));
    assert!(!located.contains("Clone"));
    assert!(!located.contains("Copy"));
    assert!(!located.contains("pub(super) reference:"));
    assert!(!located.contains("pub(super) proven_global_value:"));

    let producer = bounded(
        LOGICAL_SOURCE,
        "    pub(super) fn locate_identifier_logical_assignment(",
        "    pub(super) fn lower_global_object_environment_logical_assignment(",
    );
    assert!(producer.contains("let reference = self.locate_identifier_reference(name);"));
    assert!(producer.contains(".filter(|info| info.proven_present)"));
    assert!(producer.contains(".map(|info| info.value_info.clone())"));
    assert!(producer.contains("LocatedIdentifierLogicalAssignment {"));
    assert_eq!(
        LOGICAL_SOURCE
            .matches("fn locate_identifier_logical_assignment(")
            .count(),
        1
    );
    assert_eq!(
        LOWERING_SOURCE
            .matches("self.locate_identifier_logical_assignment(&name)")
            .count(),
        1,
    );

    let reachability = bounded(
        LOGICAL_SOURCE,
        "pub(super) enum LogicalAssignmentReachability {",
        "impl<'a> ScriptLowerer<'a> {",
    );
    assert!(reachability.contains("Definite"));
    assert!(reachability.contains("WithEnvironmentFallback"));

    let arm = bounded(
        LOWERING_SOURCE,
        "            AssignOp::BoolAnd | AssignOp::BoolOr | AssignOp::Coalesce => {",
        "            AssignOp::And\n            | AssignOp::Or",
    );
    for mapping in [
        "AssignOp::BoolAnd => LogicalBinaryOp::And",
        "AssignOp::BoolOr => LogicalBinaryOp::Or",
        "AssignOp::Coalesce => LogicalBinaryOp::Coalesce",
    ] {
        assert!(arm.contains(mapping), "missing closed mapping: {mapping}");
    }
    assert_before(
        arm,
        "let reference = self.locate_identifier_logical_assignment(&name);",
        "let rhs_value = self.lower_expression(rhs);",
    );
    assert!(arm.contains("reference.reject_definite_tdz()"));
    assert!(arm.contains("reference.is_unproven_global()"));
    assert!(arm.contains("plan.logical_assignment(logical_op, rhs_value, fallback)"));
    assert!(arm.contains("LogicalAssignmentReachability::WithEnvironmentFallback"));
    assert!(arm.contains("LogicalAssignmentReachability::Definite"));

    let consumer = bounded(
        LOGICAL_SOURCE,
        "    pub(super) fn lower_located_identifier_logical_assignment(",
        "\n    }\n}",
    );
    assert!(consumer.contains("located: LocatedIdentifierLogicalAssignment"));
    assert!(consumer.contains("let LocatedIdentifierLogicalAssignment {"));
    assert!(consumer.contains(".or(proven_global_value)"));
    assert!(consumer.contains("ExprIr::LogicalShortCircuit {"));
    assert!(consumer.contains("rhs: Box::new(write)"));
    assert_eq!(
        consumer
            .matches("LogicalAssignmentReachability::WithEnvironmentFallback =>")
            .count(),
        3,
    );
    assert!(consumer.contains("unknown_runtime_value_info()"));

    let global = bounded(
        LOGICAL_SOURCE,
        "    pub(super) fn lower_global_object_environment_logical_assignment(",
        "    pub(super) fn lower_located_identifier_logical_assignment(",
    );
    assert!(global.contains("info.value_info = unknown_runtime_value_info();"));
    assert!(global.contains("info.proven_present = false;"));
    assert!(global.contains("GlobalObjectEnvironmentReferencePlan::new("));
    assert!(global.contains(".logical_assignment(op, rhs)"));
}

#[test]
fn durable_fixture_and_exact_current_pin_inventory_bound_the_claim() {
    for (path, source) in SELECTED_WITNESSES {
        assert!(source.contains("flags: [onlyStrict]"), "{path}");
        assert!(
            source.contains("features: [logical-assignment-operators]"),
            "{path}"
        );
        assert!(source.contains("assert.throws(ReferenceError"), "{path}");
        assert!(source.contains("unresolved"), "{path}");
        assert!(CONTRACT.contains(path), "contract omits {path}");
    }
    for (path, source) in ADJACENT_RHS_WITNESSES {
        assert!(
            source.contains("features: [logical-assignment-operators]"),
            "{path}"
        );
        assert!(source.contains("unresolved"), "{path}");
    }

    for marker in [
        "missingLogicalAnd &&= missingRhs()",
        "missingLogicalOr ||= missingRhs()",
        "missingLogicalNullish ??= missingRhs()",
        "shortLogicalAnd &&= shortRhs()",
        "shortLogicalOr ||= shortRhs()",
        "shortLogicalNullish ??= shortRhs()",
        "snapshotLogicalValue ||= (snapshotLogicalValue = \"rhs\")",
        "strictLogicalAnd &&= strictRhs(\"and\")",
        "strictLogicalOr ||= strictRhs(\"or\")",
        "strictLogicalNullish ??= strictRhs(\"nullish\")",
        "sloppyLogicalAnd &&= \"and\"",
        "sloppyLogicalOr ||= \"or\"",
        "sloppyLogicalNullish ??= \"nullish\"",
        "[Symbol.unscopables]: { nestedLogicalValue: true }",
        "lifecycleTrace === \"huhgdrhs\"",
        "strictResult === \"not written\"",
    ] {
        assert!(FIXTURE.contains(marker), "fixture omits {marker}");
    }
    assert!(CONTRACT.contains("No vendored Test262 logical-assignment file contains a `with`"));
    assert!(CONTRACT.contains("Property and private References are separate lowering domains"));
    assert!(CONTRACT
        .contains("assignment, eager arithmetic/bitwise compound assignment, numeric update,"));
}
