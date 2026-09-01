use std::fs;
use std::path::Path;

const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const DATA_SOURCE: &str = include_str!("../src/data.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const MAPPING_OWNER_SOURCE: &str = include_str!("../src/functions/arguments_index_mapping.rs");
const OBJECT_SOURCE: &str = include_str!("../src/builtins/object.rs");
const DEFINE_PROPERTY_SOURCE: &str = include_str!("../src/builtins/object/define_property.rs");
const GET_OWN_PROPERTY_DESCRIPTOR_SOURCE: &str =
    include_str!("../src/builtins/object/get_own_property_descriptor.rs");
const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_arguments_mapped_descriptors.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/arguments-index-descriptor-exotic.md");

fn function_source<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source.find(signature).expect("function signature");
    let after_signature = start + signature.len();
    let tail = &source[after_signature..];
    let end = ["\n    pub(crate) fn ", "\n    pub(super) fn ", "\n    fn "]
        .into_iter()
        .filter_map(|next| tail.find(next))
        .min()
        .unwrap_or(tail.len());
    &source[start..after_signature + end]
}

fn recursive_rust_source_count(root: &Path, needle: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return recursive_rust_source_count(&path, needle);
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

#[test]
fn arguments_index_define_has_one_typed_validated_boundary() {
    assert!(CONTRACT.contains("ECMA-262 10.4.4.1-5"));
    assert!(!DEFINE_PROPERTY_SOURCE.contains("fn emit_arguments_define_data_index("));
    assert!(!DEFINE_PROPERTY_SOURCE.contains("fn emit_arguments_define_accessor_index("));
    assert_eq!(
        DEFINE_PROPERTY_SOURCE
            .matches("self.emit_arguments_define_index_descriptor(")
            .count(),
        2
    );

    let body = function_source(
        DEFINE_PROPERTY_SOURCE,
        "fn emit_arguments_define_index_descriptor(",
    );
    assert!(body.contains("descriptor: WasmDescriptor"));
    for projection in [
        "StoredDescriptorDataLocals::new(existing_value)",
        "StoredDescriptorGetterLocals::new(existing_value)",
        "StoredDescriptorSetterLocals::new(existing_setter)",
    ] {
        assert!(body.contains(projection));
    }
    let mapping = body
        .find("self.emit_arguments_index_mapping_from_descriptor_word(")
        .expect("pre-mutation mapping capture");
    let validation = body
        .find("self.emit_validate_stored_descriptor(")
        .expect("shared descriptor compatibility validation");
    let extensibility = body
        .find("HEAP_ARGUMENTS_NON_EXTENSIBLE_OFFSET")
        .expect("absent-index extensibility check");
    assert!(mapping < validation);
    assert!(extensibility < validation);
    for mutation in [
        "self.emit_arguments_store_index_entry(",
        "self.emit_arguments_parameter_map_write(",
        "self.emit_arguments_mapping_restore_on_data_descriptor(",
    ] {
        let positions = body.match_indices(mutation).collect::<Vec<_>>();
        assert!(!positions.is_empty(), "missing mutation route {mutation}");
        assert!(
            positions.iter().all(|(position, _)| validation < *position),
            "{mutation} must follow descriptor validation"
        );
    }
    assert!(!body.contains("ARGUMENTS_DESCRIPTOR_MAPPED"));
}

#[test]
fn mapped_slot_is_one_private_typed_role_across_descriptor_mutation() {
    assert_eq!(
        FUNCTIONS_SOURCE
            .matches("\nmod arguments_index_mapping;\n")
            .count(),
        1
    );
    assert!(!FUNCTIONS_SOURCE.contains("\npub mod arguments_index_mapping;\n"));
    assert!(!FUNCTIONS_SOURCE.contains("arguments_index_mapping::"));

    let mapping_type = MAPPING_OWNER_SOURCE
        .find("pub(crate) struct ArgumentsIndexMappingLocals")
        .expect("typed mapping carrier");
    let declaration_start = MAPPING_OWNER_SOURCE[..mapping_type]
        .rfind("#[must_use")
        .expect("mapping carrier must-use attribute");
    let mapping_declaration = &MAPPING_OWNER_SOURCE[declaration_start..mapping_type];
    assert!(!mapping_declaration.contains("derive(Clone, Copy"));
    assert!(!FUNCTIONS_SOURCE.contains("struct ArgumentsIndexMappingLocals"));

    for owner_method in [
        "emit_arguments_index_mapping_from_descriptor_word",
        "emit_arguments_parameter_map_read",
        "emit_arguments_parameter_map_write",
        "emit_arguments_mapping_restore_on_data_descriptor",
        "release_arguments_index_mapping",
    ] {
        let definition = format!("pub(crate) fn {owner_method}(");
        assert_eq!(MAPPING_OWNER_SOURCE.matches(&definition).count(), 1);
        assert!(!FUNCTIONS_SOURCE.contains(&definition));
    }
    assert_eq!(
        MAPPING_OWNER_SOURCE
            .matches("ArgumentsIndexMappingLocals { mapped, slot }")
            .count(),
        1
    );
    assert!(!FUNCTIONS_SOURCE.contains("ArgumentsIndexMappingLocals {"));
    assert!(!OBJECT_SOURCE.contains("ArgumentsIndexMappingLocals {"));
    assert_eq!(MAPPING_OWNER_SOURCE.matches("mapping.mapped").count(), 4);
    assert_eq!(MAPPING_OWNER_SOURCE.matches("mapping.slot").count(), 6);

    let capture = function_source(
        MAPPING_OWNER_SOURCE,
        "pub(crate) fn emit_arguments_index_mapping_from_descriptor_word(",
    );
    assert!(capture.contains("ARGUMENTS_DESCRIPTOR_MAPPED"));
    assert!(capture.contains("MappedSlot::SHIFT"));

    let read = function_source(FUNCTIONS_SOURCE, "fn emit_arguments_data_read(");
    assert!(read.contains("emit_arguments_index_mapping_from_descriptor_word("));
    assert!(read.contains("emit_arguments_parameter_map_read("));
    assert!(!read.contains("ARGUMENTS_DESCRIPTOR_MAPPED"));
    assert!(!read.contains("MappedSlot::SHIFT"));

    let write = function_source(
        MAPPING_OWNER_SOURCE,
        "pub(crate) fn emit_arguments_parameter_map_write(",
    );
    assert!(write.contains("mapping: &ArgumentsIndexMappingLocals"));
    assert!(write.contains("mapping.slot"));
    assert!(!write.contains("emit_arguments_descriptor_kind_for_index("));
    assert!(!write.contains("index_local"));

    let indexed_set = function_source(FUNCTIONS_SOURCE, "pub(crate) fn emit_arguments_write(");
    assert!(
        indexed_set.contains("self.emit_ordinary_set_result_without_receiver_fallback_via_helper(")
    );
    assert!(!indexed_set.contains("ARRAY_DESCRIPTOR_NORMAL_DATA"));

    let receiver_create = function_source(
        FUNCTIONS_SOURCE,
        "pub(crate) fn emit_arguments_receiver_index_set_result(",
    );
    assert!(receiver_create.contains("HEAP_ARGUMENTS_NON_EXTENSIBLE_OFFSET"));
    assert!(receiver_create.contains("self.emit_arguments_store_index_entry("));
    let absent_creation = &receiver_create[receiver_create
        .find("HEAP_ARGUMENTS_NON_EXTENSIBLE_OFFSET")
        .unwrap()..];
    assert!(absent_creation.contains("self.emit_arguments_store_index_entry("));

    let restore = function_source(
        MAPPING_OWNER_SOURCE,
        "pub(crate) fn emit_arguments_mapping_restore_on_data_descriptor(",
    );
    assert!(restore.contains("ARGUMENTS_DESCRIPTOR_MAPPED"));
    assert!(restore.contains("mapping.slot"));
    assert!(restore.contains("MappedSlot::SHIFT"));
    assert!(restore.contains("Instruction::I64Shl"));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (call, expected) in [
        ("self.emit_arguments_index_mapping_from_descriptor_word(", 5),
        ("self.emit_arguments_parameter_map_read(", 3),
        ("self.emit_arguments_parameter_map_write(", 4),
        ("self.emit_arguments_mapping_restore_on_data_descriptor(", 1),
        ("self.release_arguments_index_mapping(", 5),
    ] {
        assert_eq!(
            recursive_rust_source_count(&source_root, call),
            expected,
            "unexpected recursive caller census for {call}"
        );
    }
}

#[test]
fn arguments_index_accessor_descriptor_materialization_is_storage_only() {
    let body = function_source(
        GET_OWN_PROPERTY_DESCRIPTOR_SOURCE,
        "pub(in crate::builtins) fn compile_object_get_own_property_descriptor_builtin(",
    );
    let indexed = body
        .find("self.emit_array_descriptor_kind_for_index(")
        .expect("indexed descriptor lookup");
    let materialization = body[indexed..]
        .find("self.emit_alloc_accessor_descriptor_from_locals_with_flag_local(")
        .map(|offset| indexed + offset)
        .expect("indexed accessor materialization");
    let indexed_path = &body[indexed..materialization];
    assert!(indexed_path.contains("self.emit_array_read("));
    assert!(indexed_path.contains("self.emit_arguments_parameter_map_read("));
    assert!(indexed_path.contains("OBJECT_DESCRIPTOR_ACCESSOR"));
    assert!(!indexed_path.contains("self.emit_arguments_read("));
    assert!(
        indexed_path.find("self.emit_array_read(").unwrap()
            < indexed_path.find("OBJECT_DESCRIPTOR_ACCESSOR").unwrap()
    );
}

#[test]
fn dynamic_arguments_named_writes_never_enter_ordinary_object_storage() {
    let write = function_source(OBJECTS_SOURCE, "pub(crate) fn emit_object_write(");
    let arguments_dispatch = write
        .find("self.emit_arguments_property_write(")
        .expect("Arguments representation dispatch");
    let ordinary_storage = write
        .find("HEAP_OBJECT_ENTRY_SIZE")
        .expect("ordinary object storage scan");
    assert!(arguments_dispatch < ordinary_storage);

    let arguments_write = function_source(
        FUNCTIONS_SOURCE,
        "pub(crate) fn emit_arguments_property_write(",
    );
    assert!(arguments_write.contains("self.emit_arguments_write("));
    assert!(arguments_write.contains("self.emit_arguments_named_property_write("));
    assert!(arguments_write.contains("self.emit_arguments_length_write("));
    assert!(arguments_write.contains("self.emit_arguments_callee_write("));
    assert!(!arguments_write.contains("HEAP_OBJECT_ENTRY_SIZE"));

    let named_write = function_source(FUNCTIONS_SOURCE, "fn emit_arguments_named_property_write(");
    assert!(
        named_write.contains("self.emit_ordinary_set_result_without_receiver_fallback_via_helper(")
    );
    assert!(named_write.contains("self.emit_object_write_set_failure_else("));
    assert!(!named_write.contains("emit_array_define_named_data_property("));

    let receiver_write = function_source(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_ordinary_set_data_on_receiver_result_with_depth(",
    );
    assert!(receiver_write.contains("emit_is_array_named_entry_backed_tag_i32("));
    assert!(receiver_write.contains("HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET"));

    let ordinary_set = function_source(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_ordinary_set_result_with_receiver_fallback(",
    );
    assert!(ordinary_set.contains("ValueKind::Arguments.tag() as i64"));
    assert!(ordinary_set.contains("self.strings.payload(\"length\")"));
    assert!(ordinary_set.contains("self.strings.payload(\"callee\")"));
    assert!(ordinary_set.contains("self.emit_arguments_property_write("));
    assert!(ordinary_set.contains("emit_is_array_named_entry_backed_tag_i32("));
    assert!(ordinary_set.contains("HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET"));
    let indexed_source = ordinary_set
        .find("Dense and sparse Array/Arguments indices")
        .expect("Array/Arguments indexed source-side scan");
    let indexed_source = &ordinary_set[indexed_source..];
    let indexed_source = &indexed_source[..indexed_source
        .find("self.emit_is_object_entry_backed_tag_i32(current_tag_local, function);")
        .expect("named source-side scan")];
    assert!(indexed_source.contains("emit_is_array_named_entry_backed_tag_i32("));
    assert!(indexed_source.contains("emit_arguments_descriptor_kind_for_index("));
    assert!(indexed_source.contains("emit_array_descriptor_kind_for_index("));
    assert!(indexed_source.contains("emit_array_accessor_setter_for_index("));

    let get_prototype = function_source(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_ordinary_get_prototype_of(",
    );
    assert!(get_prototype
        .contains("self.emit_is_array_named_entry_backed_tag_i32(object_tag_local, function);"));
    assert!(get_prototype.contains("HEAP_ARRAY_PROTOTYPE_TAG_OFFSET"));

    let set_prototype = function_source(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_ordinary_set_prototype_of_i32(",
    );
    assert_eq!(
        set_prototype
            .matches("self.emit_is_array_named_entry_backed_tag_i32(object_tag_local, function);")
            .count(),
        2,
        "null and non-null prototype writes must preserve Array/Arguments tags"
    );
    assert_eq!(
        set_prototype
            .matches("HEAP_ARRAY_PROTOTYPE_TAG_OFFSET")
            .count(),
        2
    );

    for witness in [
        "honorsOwnNamedSetter",
        "honorsInheritedNamedSetter",
        "honorsNonWritableNamedProperty",
        "rejectsAbsentIndexOnNonExtensibleArguments",
        "honorsInheritedIndexSetterAfterDelete",
        "rejectsAbsentIndexAssignmentOnNonExtensibleArguments",
        "honorsArgumentsPrototypeIndexedDescriptors",
    ] {
        assert!(FIXTURE.contains(witness));
    }

    assert!(receiver_write.contains("self.emit_arguments_receiver_index_set_result("));

    assert!(ARRAY_SOURCE.contains("HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET"));
    for message in [
        "Cannot assign to arguments property",
        "Cannot define arguments index on a non-extensible object",
    ] {
        assert!(DATA_SOURCE.contains(message));
    }
}
