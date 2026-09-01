const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const ERRORS_SOURCE: &str = include_str!("../src/builtins/errors.rs");

fn declaration_source<'a>(source: &'a str, signature: &str, next: &str) -> &'a str {
    let start = source.find(signature).expect("declaration signature");
    let tail = &source[start + signature.len()..];
    let end = tail.find(next).expect("next declaration");
    &source[start..start + signature.len() + end]
}

fn function_source<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source.find(signature).expect("function signature");
    let tail = &source[start + signature.len()..];
    let end = ["\n    pub(crate) fn ", "\n    pub(super) fn ", "\n    fn "]
        .into_iter()
        .filter_map(|next| tail.find(next))
        .min()
        .unwrap_or(tail.len());
    &source[start..start + signature.len() + end]
}

#[test]
fn compatibility_predicate_exhaustively_maps_all_presence_states() {
    let implementation = declaration_source(
        OBJECTS_SOURCE,
        "impl DescriptorCompatibilityPredicate {",
        "\n}\n\n/// A [`ValidatedDescriptor`] over Wasm locals.",
    );
    let mapping = function_source(implementation, "fn from_presence<T>(");

    assert!(mapping.contains("Presence::Absent => Self::Never"));
    assert!(mapping.contains("Presence::Present(_) => Self::Always"));
    assert!(mapping.contains("Presence::Runtime { present, .. } => Self::AtRuntime"));
    assert!(!mapping.contains("_ =>"));
}

#[test]
fn compatibility_predicate_exhaustively_emits_all_predicate_states() {
    let emitter = function_source(OBJECTS_SOURCE, "fn emit_descriptor_compatibility_check(");

    assert!(emitter.contains("DescriptorCompatibilityPredicate::Never => return Ok(())"));
    assert!(
        emitter.contains("DescriptorCompatibilityPredicate::Always => emit_check(self, function)?")
    );
    assert!(emitter.contains("DescriptorCompatibilityPredicate::AtRuntime { first, second }"));
    assert!(!emitter.contains("_ =>"));
}

#[test]
fn compatibility_predicate_has_no_incidental_value_capabilities() {
    let declaration = declaration_source(
        OBJECTS_SOURCE,
        "enum DescriptorCompatibilityPredicate {",
        "\n}\n\nimpl DescriptorCompatibilityPredicate",
    );
    let attributes = &OBJECTS_SOURCE[..OBJECTS_SOURCE
        .find("enum DescriptorCompatibilityPredicate {")
        .expect("predicate declaration")];
    let nearest_attributes = attributes.rsplit("\n\n").next().unwrap_or_default();

    assert!(declaration.contains("Never,"));
    assert!(declaration.contains("Always,"));
    assert!(declaration.contains("AtRuntime { first: u32, second: Option<u32> }"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(
            !nearest_attributes.contains(capability),
            "unexpected {capability}"
        );
    }
}

#[test]
fn ordinary_and_stored_validators_share_the_closed_predicate_emitter() {
    let stored = function_source(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_validate_stored_descriptor(",
    );
    let ordinary = function_source(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_object_define_entry_validated(",
    );

    for (body, fields) in [
        (
            stored,
            &[
                "descriptor.configurable",
                "descriptor.enumerable",
                "descriptor.writable",
                "descriptor.value",
            ][..],
        ),
        (
            ordinary,
            &[
                "descriptor.configurable",
                "descriptor.enumerable",
                "descriptor.writable",
                "descriptor.value",
                "descriptor.get",
                "descriptor.set",
            ][..],
        ),
    ] {
        for field in fields {
            assert!(
                body.contains(&format!(
                    "DescriptorCompatibilityPredicate::from_presence(&{field})"
                )),
                "missing compatibility predicate for {field}"
            );
        }
        assert!(!body.contains("Presence::Absent | Presence::Present"));
    }
    assert!(stored.contains("(descriptor.get, stored.getter)"));
    assert!(stored.contains("(descriptor.set, stored.setter)"));
    assert!(stored.contains("DescriptorCompatibilityPredicate::from_presence(&field)"));
}

#[test]
fn static_kind_changes_use_the_same_always_predicate() {
    let ordinary = function_source(OBJECTS_SOURCE, "fn emit_descriptor_kind_change_throw(");
    let stored = function_source(
        OBJECTS_SOURCE,
        "fn emit_array_descriptor_kind_change_rejection(",
    );

    for body in [ordinary, stored] {
        assert!(body.contains("DescriptorCompatibilityPredicate::from_kind_terms(terms)"));
        assert!(!body.contains("(true, false) => return"));
        assert!(!body.contains("runtime_flags().is_empty()"));
    }
}

#[test]
fn fresh_runtime_errors_append_properties_without_recursive_validation() {
    for signature in [
        "pub(crate) fn emit_runtime_error_object(",
        "fn emit_throw_runtime_error_with_prototype_local_kind(",
    ] {
        let body = function_source(ERRORS_SOURCE, signature);
        assert_eq!(
            body.matches("emit_object_append_data_property_with_flags(")
                .count(),
            2
        );
        assert!(!body.contains("emit_object_define_data("));
    }
}
