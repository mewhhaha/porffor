const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/spec-operation-object-target-kind.md");
const TASK: &str = include_str!("../../../tasks/04-spec-operations-and-completion-abi.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after {start}: {end}"))
        .0
}

#[test]
fn object_target_projection_exhaustively_classifies_every_value_kind() {
    let projection = bounded(
        OPERATIONS_SOURCE,
        "fn spec_operation_object_target_kind(",
        "/// Whether numeric conversion may interpret",
    );
    for kind in [
        "Undefined",
        "Null",
        "Boolean",
        "Number",
        "BigInt",
        "Symbol",
        "String",
        "Object",
        "Array",
        "Arguments",
        "Function",
        "Dynamic",
    ] {
        assert_eq!(projection.matches(&format!("ValueKind::{kind}")).count(), 1);
    }
    assert!(!projection.contains("_ =>"));
    assert!(!projection.contains("unreachable!"));
}

#[test]
fn all_six_object_only_operations_consume_the_shared_classification() {
    let body = bounded(
        OPERATIONS_SOURCE,
        "pub(crate) fn compile_spec_operation_to_locals(",
        "pub(crate) fn emit_primitive_to_numeric_locals_without_throw_return(",
    );
    assert_eq!(
        body.matches("spec_operation_object_target_kind(target.kind)")
            .count(),
        6
    );
    assert_eq!(body.matches("match &object_target_kind").count(), 6);
    assert_eq!(
        body.matches("if let SpecOperationObjectTargetKind::RuntimeDynamic = object_target_kind")
            .count(),
        6
    );
    for operation in [
        "Get",
        "HasProperty",
        "HasOwnProperty",
        "DeletePropertyOrThrow",
        "Set",
        "CreateDataPropertyOrThrow",
    ] {
        assert!(body.contains(&format!("SpecOperationIr::{operation} =>")));
    }
    assert!(!body.contains("match target.kind"));
    assert!(!body.contains("if target.kind == ValueKind::Dynamic"));
}

#[test]
fn object_target_kind_has_no_incidental_capabilities() {
    let declaration = bounded(
        OPERATIONS_SOURCE,
        "enum SpecOperationObjectTargetKind {",
        "fn spec_operation_object_target_kind(",
    );
    assert!(!declaration.contains("derive"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(!OPERATIONS_SOURCE.contains(&format!(
            "impl {capability} for SpecOperationObjectTargetKind"
        )));
    }
}

#[test]
fn contract_and_task_record_shared_object_target_ownership() {
    for source in [CONTRACT, TASK] {
        assert!(source.contains("SpecOperationObjectTargetKind"));
        assert!(source.contains("StaticallyObjectLike"));
        assert!(source.contains("RuntimeDynamic"));
        assert!(source.contains("StaticallyPrimitive"));
        assert!(source.contains("six") || source.contains("Six"));
    }
}
