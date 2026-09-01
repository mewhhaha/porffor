const BINARY_DATA_SOURCE: &str = include_str!("../src/builtins/binary_data.rs");
const BUILTINS_SOURCE: &str = include_str!("../src/builtins/mod.rs");
const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start boundary `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end boundary `{end}`"))
        .0
}

fn length_read_body() -> &'static str {
    bounded(
        OBJECTS_SOURCE,
        "                if matches!(key, PropertyKeyIr::ArrayLength)",
        "                let key_local = self.reserve_temp_local();\n                let key_tag_local = self.reserve_temp_local();",
    )
}

fn indexed_read_body() -> &'static str {
    bounded(
        OBJECTS_SOURCE,
        "    fn emit_typed_array_or_object_index_read_from_locals_inner(",
        "    pub(crate) fn emit_object_index_read_from_locals(",
    )
}

fn indexed_write_body() -> &'static str {
    bounded(
        OBJECTS_SOURCE,
        "    pub(crate) fn emit_typed_array_element_write_from_locals(",
        "    fn emit_arguments_length_delete(",
    )
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn unique_position(source: &str, needle: &str, label: &str) -> usize {
    assert_eq!(
        source.matches(needle).count(),
        1,
        "{label} must occur exactly once"
    );
    source
        .find(needle)
        .unwrap_or_else(|| panic!("missing {label}"))
}

fn position(source: &str, needle: &str, label: &str) -> usize {
    source
        .find(needle)
        .unwrap_or_else(|| panic!("missing {label}"))
}

fn assert_one_view_and_witness(body: &str, use_: &str, owner: &str) {
    for (needle, label) in [
        ("emit_load_typed_array_private_state(", "private-state load"),
        ("TypedArrayViewLocals::new(", "immutable view"),
        ("emit_typed_array_witness(", "buffer witness"),
        (use_, "closed witness use"),
    ] {
        assert_eq!(
            body.matches(needle).count(),
            1,
            "{owner} must contain exactly one {label}"
        );
    }

    for forbidden in [
        "emit_typed_array_current_byte_length(",
        "emit_validate_typed_array_current_byte_length(",
        "emit_load_array_buffer_byte_length(",
        "HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET",
        "HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET",
        "HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET",
    ] {
        assert!(
            !body.contains(forbidden),
            "{owner} must not bypass its witness through {forbidden}"
        );
    }
}

#[test]
fn objects_has_no_raw_typed_array_current_length_observer() {
    assert_eq!(
        OBJECTS_SOURCE
            .matches("emit_typed_array_current_byte_length(")
            .count(),
        0,
        "objects.rs must not reintroduce a raw TypedArray current-length observation"
    );

    for declaration in [
        "pub(crate) struct TypedArrayViewLocals",
        "pub(crate) enum TypedArrayAccessorKind",
        "pub(crate) enum TypedArrayWitnessUse",
        "pub(crate) fn emit_typed_array_witness(",
    ] {
        assert!(
            BINARY_DATA_SOURCE.contains(declaration),
            "the closed witness seam must remain crate-private: {declaration}"
        );
        assert!(!BINARY_DATA_SOURCE.contains(&declaration.replace("pub(crate)", "pub")));
    }
    for type_name in [
        "TypedArrayAccessorKind",
        "TypedArrayViewLocals",
        "TypedArrayWitnessUse",
    ] {
        assert_eq!(
            BUILTINS_SOURCE.matches(type_name).count(),
            1,
            "builtins must re-export {type_name} exactly once"
        );
    }
}

#[test]
fn indexed_write_signature_contains_only_its_five_domain_arguments() {
    let signature = bounded(
        OBJECTS_SOURCE,
        "    pub(crate) fn emit_typed_array_element_write_from_locals(\n",
        "    ) -> Result<(), EmitError> {",
    );

    assert_eq!(
        without_whitespace(signature),
        "&mutself,target_local:u32,index_local:u32,value_payload_local:u32,value_tag_local:u32,function:&mutFunction,"
    );
}

#[test]
fn length_read_projects_the_non_throwing_accessor_witness() {
    let body = length_read_body();
    assert_one_view_and_witness(
        body,
        "TypedArrayWitnessUse::Accessor",
        "TypedArray length read",
    );
    assert_eq!(body.matches("TypedArrayAccessorKind::Length").count(), 1);
    assert_eq!(body.matches("emit_load_array_buffer_data(").count(), 0);
    assert_eq!(
        body.matches("Instruction::LocalSet(typed_stored_byte_length_local)")
            .count(),
        0,
        "the accessor result must not overwrite the view's stored fixed extent"
    );
    assert!(!body.contains("Instruction::I64DivU"));
    assert!(!body.contains("TypedArrayWitnessUse::ValidatedMethodEntry"));

    let brand = unique_position(body, "emit_is_typed_array_i32(", "TypedArray brand check");
    let private_state = unique_position(
        body,
        "emit_load_typed_array_private_state(",
        "private-state load",
    );
    let view = unique_position(body, "TypedArrayViewLocals::new(", "immutable view");
    let witness = unique_position(body, "TypedArrayAccessorKind::Length", "length projection");
    let boxing = unique_position(
        body,
        "Instruction::LocalGet(typed_array_length_local)",
        "length Number boxing",
    );
    assert!(brand < private_state && private_state < view && view < witness && witness < boxing);
}

#[test]
fn indexed_read_validates_before_loading_the_backing_pointer() {
    let body = indexed_read_body();
    assert_one_view_and_witness(
        body,
        "TypedArrayWitnessUse::IntegerIndexedProperty",
        "shared indexed read",
    );
    assert_eq!(body.matches("emit_load_array_buffer_data(").count(), 1);
    assert_eq!(body.matches("emit_load_array_buffer_flags(").count(), 1);
    assert_eq!(
        body.matches("HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET").count(),
        1
    );
    assert!(!body.contains("Instruction::I64GeU"));

    let typed_array_branch = unique_position(
        body,
        "emit_is_typed_array_i32(target_local, target_tag_local, function)",
        "TypedArray branch",
    );
    let undefined = typed_array_branch
        + position(
            &body[typed_array_branch..],
            r#"
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
"#,
            "TypedArray undefined result initialization",
        );
    let private_state = unique_position(
        body,
        "emit_load_typed_array_private_state(",
        "private-state load",
    );
    let witness = unique_position(
        body,
        "TypedArrayWitnessUse::IntegerIndexedProperty",
        "integer-indexed witness",
    );
    let validity = unique_position(
        body,
        "Instruction::LocalGet(index_valid_local)",
        "validity branch",
    );
    let data = unique_position(body, "emit_load_array_buffer_data(", "backing-pointer load");
    let address = unique_position(
        body,
        "Instruction::LocalSet(address_local)",
        "element address",
    );
    let load = position(
        body,
        "emit_array_buffer_memory_load(",
        "element memory load",
    );
    assert!(
        undefined < private_state
            && private_state < witness
            && witness < validity
            && validity < data
            && data < address
            && address < load
    );
}

#[test]
fn indexed_write_coerces_before_its_fresh_witness_and_pointer() {
    let body = indexed_write_body();
    assert_one_view_and_witness(
        body,
        "TypedArrayWitnessUse::IntegerIndexedProperty",
        "shared indexed write",
    );
    assert_eq!(body.matches("emit_load_array_buffer_data(").count(), 1);
    assert_eq!(
        body.matches("HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET").count(),
        1
    );
    assert!(!body.contains("Instruction::I64GeU"));

    let element_kind = unique_position(
        body,
        "HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET",
        "element-kind load",
    );
    let coercion = unique_position(
        body,
        "emit_value_to_typed_array_element_payload(",
        "value coercion",
    );
    let throw = unique_position(
        body,
        "emit_return_current_completion_if_throw(function)",
        "coercion throw propagation",
    );
    let private_state = unique_position(
        body,
        "emit_load_typed_array_private_state(",
        "post-coercion private-state load",
    );
    let witness = unique_position(
        body,
        "TypedArrayWitnessUse::IntegerIndexedProperty",
        "post-coercion witness",
    );
    let validity = unique_position(
        body,
        "Instruction::LocalGet(index_valid_local)",
        "validity branch",
    );
    let data = unique_position(body, "emit_load_array_buffer_data(", "backing-pointer load");
    let address = unique_position(
        body,
        "Instruction::LocalSet(address_local)",
        "element address",
    );
    let store = unique_position(
        body,
        "emit_store_number_payload_to_typed_array_address_by_kind(",
        "element store",
    );
    assert!(
        element_kind < coercion
            && coercion < throw
            && throw < private_state
            && private_state < witness
            && witness < validity
            && validity < data
            && data < address
            && address < store
    );

    let releases = without_whitespace(
        r#"
        self.release_temp_local(index_valid_local);
        self.release_temp_local(address_local);
        self.release_temp_local(number_payload_local);
        self.release_temp_local(element_kind_local);
        self.release_temp_local(bytes_per_element_local);
        self.release_temp_local(stored_byte_length_local);
        self.release_temp_local(byte_offset_local);
        self.release_temp_local(data_ptr_local);
        self.release_temp_local(buffer_payload_local);
"#,
    );
    assert_eq!(without_whitespace(body).matches(&releases).count(), 1);
}
