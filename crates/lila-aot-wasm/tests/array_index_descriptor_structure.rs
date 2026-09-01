const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const DEFINE_PROPERTY_SOURCE: &str = include_str!("../src/builtins/object/define_property.rs");
const GET_OWN_PROPERTY_DESCRIPTOR_SOURCE: &str =
    include_str!("../src/builtins/object/get_own_property_descriptor.rs");
const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");

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

#[test]
fn array_index_define_uses_one_validated_descriptor_before_mutation() {
    assert!(!ARRAY_SOURCE.contains("fn emit_array_define_data_index("));
    assert!(!ARRAY_SOURCE.contains("fn emit_array_define_accessor_index("));
    assert!(
        DEFINE_PROPERTY_SOURCE
            .matches("WasmPartialDescriptor {")
            .count()
            >= 2
    );
    assert_eq!(
        DEFINE_PROPERTY_SOURCE
            .matches("self.emit_array_define_index_descriptor(")
            .count(),
        2
    );

    let body = function_source(
        ARRAY_SOURCE,
        "pub(crate) fn emit_array_define_index_descriptor(",
    );
    assert!(body.contains("descriptor: WasmDescriptor"));

    let validation = body
        .find("self.emit_validate_stored_descriptor(")
        .expect("shared descriptor compatibility validation");
    for mutation in [
        "self.emit_array_write(",
        "self.emit_store_array_accessor_setter_for_index(",
        "self.emit_store_array_descriptor_for_index(",
    ] {
        let positions = body.match_indices(mutation).collect::<Vec<_>>();
        assert!(!positions.is_empty(), "missing mutation route {mutation}");
        assert!(
            positions.iter().all(|(position, _)| validation < *position),
            "{mutation} must follow validation"
        );
    }

    let call_positions = DEFINE_PROPERTY_SOURCE
        .match_indices("self.emit_array_define_index_descriptor(")
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    assert_eq!(call_positions.len(), 2);
    let descriptors = call_positions
        .into_iter()
        .map(|call| {
            let start = DEFINE_PROPERTY_SOURCE[..call]
                .rfind("let descriptor = WasmPartialDescriptor {")
                .expect("typed descriptor before Array call");
            &DEFINE_PROPERTY_SOURCE[start..call]
        })
        .collect::<Vec<_>>();
    for (field, local) in [
        ("get", "getter_present_local"),
        ("set", "setter_present_local"),
        ("enumerable", "enumerable_present_local"),
        ("configurable", "configurable_present_local"),
    ] {
        assert!(descriptors[0].contains(&format!(
            "{field}: Presence::Runtime {{\n                present: {local},"
        )));
    }
    for (field, local) in [
        ("value", "value_present_local"),
        ("writable", "writable_present_local"),
        ("enumerable", "enumerable_present_local"),
        ("configurable", "configurable_present_local"),
    ] {
        assert!(descriptors[1].contains(&format!(
            "{field}: Presence::Runtime {{\n                present: {local},"
        )));
    }
}

#[test]
fn array_index_compatibility_reuses_the_typed_stored_descriptor_validator() {
    assert!(OBJECTS_SOURCE.contains("pub(crate) struct StoredDescriptorLocals"));
    let validation_body = function_source(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_validate_stored_descriptor(",
    );
    assert!(validation_body.contains("let classification = classify(descriptor);"));
    assert!(validation_body.contains("builder.emit_tagged_payload_same_value_i32("));
    for projection in [
        "StoredDescriptorDataLocals::new(existing_value)",
        "StoredDescriptorGetterLocals::new(existing_value)",
        "StoredDescriptorSetterLocals::new(existing_setter)",
    ] {
        assert!(ARRAY_SOURCE.contains(projection));
    }
    assert!(ARRAY_SOURCE.contains("let descriptor = descriptor.into_partial();"));

    // Arguments now consumes the same typed validator through its own exotic
    // post-application mapping protocol.
    assert!(DEFINE_PROPERTY_SOURCE.contains("fn emit_arguments_define_index_descriptor("));
    assert!(!DEFINE_PROPERTY_SOURCE.contains("fn emit_arguments_define_data_index("));
    assert!(!DEFINE_PROPERTY_SOURCE.contains("fn emit_arguments_define_accessor_index("));
}

#[test]
fn array_index_accessor_descriptors_materialize_both_accessor_fields() {
    let body = function_source(
        GET_OWN_PROPERTY_DESCRIPTOR_SOURCE,
        "pub(in crate::builtins) fn compile_object_get_own_property_descriptor_builtin(",
    );
    let materialization = body
        .find("self.emit_array_accessor_setter_for_index(")
        .expect("array-index descriptor materialization must load its setter");
    let descriptor = body[materialization..]
        .find("self.emit_alloc_accessor_descriptor_from_locals_with_flag_local(")
        .expect("array-index descriptor materialization must preserve accessor kind");
    assert!(descriptor > 0);
}
