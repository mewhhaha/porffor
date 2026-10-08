use lila_ir::{
    find_spec_operation, AbruptCapability, BackendSpecOperation, CompletionAbruptKind,
    NormalResult, OperationDomain, OperationLoweringStatus, RowSource, SpecOperationFamily,
    TrackedGapReason, SPEC_OPERATION_CATALOG,
};

const BACKEND_JOIN_SOURCE: &str = include_str!("../src/backend_operation_evidence.rs");
const OPERATIONS_SOURCE: &str = include_str!("../../lila-ir/src/operations.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

#[test]
fn to_property_descriptor_has_backend_owned_descriptor_result_evidence() {
    let operation = BackendSpecOperation::ToPropertyDescriptor;
    let descriptor = operation.descriptor();
    assert_eq!(descriptor.name(), "ToPropertyDescriptor");
    assert_eq!(descriptor.family(), SpecOperationFamily::Object);
    assert_eq!(descriptor.domain(), OperationDomain::Value);
    assert_eq!(descriptor.normal_result(), NormalResult::PropertyDescriptor);
    assert_eq!(descriptor.abrupt(), AbruptCapability::MayThrow);

    let row = find_spec_operation("ToPropertyDescriptor")
        .expect("ToPropertyDescriptor must have a catalog row");
    assert_eq!(row.source(), RowSource::DerivedFromBackendOperation);
    assert_eq!(row.normal_result(), NormalResult::PropertyDescriptor);
    assert_eq!(row.abrupt(), &[CompletionAbruptKind::Throw]);
    assert!(matches!(
        row.lowering_status(),
        OperationLoweringStatus::SharedBackendEmitter(evidence)
            if evidence.operation() == operation
    ));

    let gaps = bounded(
        OPERATIONS_SOURCE,
        "pub(crate) const TRACKED_GAP_ROWS: &[TrackedGapRow] = &[",
        "pub const SPEC_OPERATION_ROW_COUNT",
    );
    assert!(!gaps.contains("name: \"ToPropertyDescriptor\""));

    let from_property_descriptor = find_spec_operation("FromPropertyDescriptor")
        .expect("FromPropertyDescriptor must retain its tracked gap row");
    assert_eq!(
        from_property_descriptor.source(),
        RowSource::TrackedGapTable
    );
    assert_eq!(
        from_property_descriptor.normal_result(),
        NormalResult::ObjectOrUndefined
    );
    assert!(matches!(
        from_property_descriptor.lowering_status(),
        OperationLoweringStatus::TrackedGap {
            reason: TrackedGapReason::NoImplementation,
            ..
        }
    ));
    assert!(gaps.contains("name: \"FromPropertyDescriptor\""));
}

#[test]
fn operation_catalog_census_includes_both_backend_operations() {
    let mut expression_emitters = 0;
    let mut backend_emitters = 0;
    let mut statement_emitters = 0;
    let mut tracked_gaps = 0;

    for row in &SPEC_OPERATION_CATALOG {
        match row.lowering_status() {
            OperationLoweringStatus::SharedWasmEmitter(_) => expression_emitters += 1,
            OperationLoweringStatus::SharedBackendEmitter(_) => backend_emitters += 1,
            OperationLoweringStatus::StatementEmission(_) => statement_emitters += 1,
            OperationLoweringStatus::TrackedGap { .. } => tracked_gaps += 1,
        }
    }

    assert_eq!(BackendSpecOperation::ALL.len(), 2);
    assert_eq!(
        (
            expression_emitters,
            backend_emitters,
            statement_emitters,
            tracked_gaps,
            SPEC_OPERATION_CATALOG.len(),
        ),
        (30, 2, 5, 10, 47)
    );
}

#[test]
fn backend_operation_join_is_exhaustive_for_both_real_emitters() {
    assert!(BACKEND_JOIN_SOURCE
        .contains("fn backend_spec_operations_are_backed(operation: BackendSpecOperation)"));
    assert!(BACKEND_JOIN_SOURCE.contains("match operation {"));
    for (operation, emitter) in [
        (
            "BackendSpecOperation::ArraySpeciesCreate",
            "FunctionBuilder::emit_array_species_create",
        ),
        (
            "BackendSpecOperation::ToPropertyDescriptor",
            "FunctionBuilder::emit_to_property_descriptor",
        ),
    ] {
        assert_eq!(BACKEND_JOIN_SOURCE.matches(operation).count(), 1);
        assert_eq!(BACKEND_JOIN_SOURCE.matches(emitter).count(), 1);
    }
    assert!(!BACKEND_JOIN_SOURCE.contains("_ =>"));
}
