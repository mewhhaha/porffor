use std::fs;
use std::path::Path;

use lila_ir::{
    find_spec_operation, AbruptCapability, BackendSpecOperation, CompletionAbruptKind,
    NormalResult, OperationDomain, OperationLoweringStatus, RowSource, SpecOperationFamily,
    TrackedGapReason, SPEC_OPERATION_CATALOG,
};

const BACKEND_JOIN_SOURCE: &str = include_str!("../src/backend_operation_evidence.rs");
const DEFINE_PROPERTY_SOURCE: &str = include_str!("../src/builtins/object/define_property.rs");
const OBJECT_BUILTIN_SOURCE: &str = include_str!("../src/builtins/object.rs");
const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const OPERATIONS_SOURCE: &str = include_str!("../../lila-ir/src/operations.rs");
const PROPERTY_DESCRIPTOR_SOURCE: &str = include_str!("../../lila-ir/src/property_descriptor.rs");
const REFLECT_SOURCE: &str = include_str!("../src/builtins/reflect.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn function_source<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature `{signature}`"));
    let tail = &source[start + signature.len()..];
    let end = ["\n    pub(crate) fn ", "\n    pub(super) fn ", "\n    fn "]
        .into_iter()
        .filter_map(|next| tail.find(next))
        .min()
        .unwrap_or(tail.len());
    &source[start..start + signature.len() + end]
}

fn count_in_rust_sources(directory: &Path, needle: &str) -> usize {
    fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

fn positions_in_order(source: &str, markers: &[&str]) {
    let mut cursor = 0;
    for marker in markers {
        let offset = source[cursor..]
            .find(marker)
            .unwrap_or_else(|| panic!("missing marker after byte {cursor}: `{marker}`"));
        cursor += offset + marker.len();
    }
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
        (29, 2, 5, 10, 46)
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

#[test]
fn conversion_returns_one_reserved_descriptor_only_after_step_nine() {
    let witness_declaration = "pub(crate) struct ReservedPropertyDescriptorLocals {";
    let witness_start = OBJECTS_SOURCE
        .find(witness_declaration)
        .expect("reserved descriptor witness declaration");
    let witness_attributes = OBJECTS_SOURCE[..witness_start]
        .rsplit("\n\n")
        .next()
        .unwrap_or_default()
        .trim();
    assert_eq!(
        witness_attributes,
        "#[must_use = \"a converted property descriptor must be materialized so its locals are released\"]"
    );
    let witness = bounded(
        OBJECTS_SOURCE,
        "#[must_use = \"a converted property descriptor must be materialized so its locals are released\"]",
        "/// The kind an entry is **stored** as.",
    );
    assert!(witness.contains(witness_declaration));
    assert!(witness.contains("descriptor: ValidatedDescriptor<ReservedPropertyDescriptorCarrier>,"));
    assert!(!witness.contains("pub descriptor:"));
    assert!(!witness.contains("#[derive"));
    for capability in ["Clone", "Copy"] {
        assert!(!OBJECTS_SOURCE.contains(&format!(
            "impl {capability} for ReservedPropertyDescriptorLocals"
        )));
    }

    assert_eq!(
        OBJECTS_SOURCE
            .matches("pub(crate) fn emit_to_property_descriptor(")
            .count(),
        1
    );
    let conversion = function_source(OBJECTS_SOURCE, "pub(crate) fn emit_to_property_descriptor(");
    assert!(conversion.contains(") -> Result<ReservedPropertyDescriptorLocals, EmitError> {"));
    assert_eq!(conversion.matches("self.reserve_temp_local();").count(), 20);
    assert_eq!(conversion.matches("self.release_temp_local(").count(), 2);
    assert_eq!(conversion.matches("Presence::Runtime {").count(), 6);
    assert_eq!(conversion.matches("TaggedLocals::new(").count(), 6);
    assert!(!conversion.contains("emit_alloc_plain_object_with_prototype"));
    assert!(!conversion.contains("emit_object_define_enumerable_data"));
    assert_eq!(
        conversion
            .matches("Ok(ReservedPropertyDescriptorLocals { descriptor })")
            .count(),
        1
    );
    positions_in_order(
        conversion,
        &[
            "\"Property descriptor cannot be both accessor and data\"",
            "self.emit_return_current_completion(function);",
            "self.release_temp_local(field_key_tag_local);",
            "self.release_temp_local(field_key_local);",
            "let descriptor = PartialDescriptor::<ReservedPropertyDescriptorCarrier>",
            ".from_runtime_checked();",
            "Ok(ReservedPropertyDescriptorLocals { descriptor })",
        ],
    );

    let runtime_checked_ledger = bounded(
        PROPERTY_DESCRIPTOR_SOURCE,
        "/// Declared call sites, and the lines that discharge the obligation:",
        "    pub fn from_runtime_checked(self) -> ValidatedDescriptor<C> {",
    );
    assert_eq!(
        runtime_checked_ledger
            .matches("FunctionBuilder::emit_to_property_descriptor")
            .count(),
        1
    );
    assert_eq!(
        runtime_checked_ledger
            .matches("Property descriptor cannot be both accessor and data")
            .count(),
        1
    );
    assert!(!runtime_checked_ledger.contains("emit_to_property_descriptor_object"));
}

#[test]
fn two_object_builtin_calls_convert_then_consume_the_descriptor() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "self.emit_to_property_descriptor("),
        2
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "self.emit_from_present_property_descriptor("),
        2
    );
    for source in [OBJECT_BUILTIN_SOURCE, DEFINE_PROPERTY_SOURCE] {
        assert_eq!(
            source.matches("self.emit_to_property_descriptor(").count(),
            1
        );
        assert_eq!(
            source
                .matches("self.emit_from_present_property_descriptor(")
                .count(),
            1
        );
        positions_in_order(
            source,
            &[
                "let descriptor = self.emit_to_property_descriptor(",
                "self.emit_from_present_property_descriptor(",
                "descriptor,",
            ],
        );
    }

    assert_eq!(
        count_in_rust_sources(&source_root, "emit_to_property_descriptor_object"),
        0
    );
}

#[test]
fn present_descriptor_materializer_consumes_and_releases_every_field() {
    assert_eq!(
        OBJECTS_SOURCE
            .matches("pub(crate) fn emit_from_present_property_descriptor(")
            .count(),
        1
    );
    let materializer = function_source(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_from_present_property_descriptor(",
    );
    assert!(materializer.contains("reserved_descriptor: ReservedPropertyDescriptorLocals,"));
    assert!(!materializer.contains("reserved_descriptor: &ReservedPropertyDescriptorLocals,"));
    assert_eq!(
        materializer
            .matches("reserved_descriptor.descriptor.into_partial()")
            .count(),
        1
    );
    positions_in_order(
        materializer,
        &[
            "let PartialDescriptor {",
            "value,",
            "writable,",
            "get,",
            "set,",
            "enumerable,",
            "configurable,",
            "} = reserved_descriptor.descriptor.into_partial();",
        ],
    );
    assert!(materializer
        .contains("for field in [set, get, configurable, enumerable, writable, value] {"));
    for arm in [
        "Presence::Absent => {}",
        "Presence::Present(value) => {",
        "Presence::Runtime { present, value } => {",
    ] {
        assert!(materializer.contains(arm), "missing release arm `{arm}`");
    }
    assert!(!materializer.contains("_ =>"));
    assert_eq!(
        materializer
            .matches("self.release_temp_local(value.tag);")
            .count(),
        2
    );
    assert_eq!(
        materializer
            .matches("self.release_temp_local(value.payload);")
            .count(),
        2
    );
    assert_eq!(
        materializer
            .matches("self.release_temp_local(present);")
            .count(),
        1
    );
    assert_eq!(
        materializer
            .matches("self.release_temp_local(field_key_local);")
            .count(),
        1
    );
}

#[test]
fn reflect_define_property_remains_an_open_coded_nonclaim() {
    let reflect = function_source(
        REFLECT_SOURCE,
        "pub(crate) fn compile_reflect_define_property_builtin(",
    );
    assert!(
        reflect.contains("// ToPropertyDescriptor observes these fields in specification order.")
    );
    for field in [
        "\"enumerable\"",
        "\"configurable\"",
        "\"value\"",
        "\"writable\"",
        "\"get\"",
        "\"set\"",
    ] {
        assert!(
            reflect.contains(field),
            "missing open-coded field `{field}`"
        );
    }
    assert!(reflect.contains("\"Property descriptor cannot be both accessor and data\""));
    assert!(!reflect.contains("self.emit_to_property_descriptor("));
    assert!(!reflect.contains("self.emit_from_present_property_descriptor("));
}
