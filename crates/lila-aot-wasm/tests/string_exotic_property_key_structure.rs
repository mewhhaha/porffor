const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");

fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after: {start}"))
        .0
}

#[test]
fn computed_string_keys_have_one_closed_lowering_classification() {
    let declaration = between(
        LOWERING_SOURCE,
        "enum StringExoticComputedKey {",
        "}\n\nimpl StringExoticComputedKey",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| line.ends_with(','))
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        [
            "CanonicalIndex(Box<TypedExpr>),",
            "OrdinaryPropertyKey(PropertyKeyIr),",
        ]
    );

    let conversion = between(
        LOWERING_SOURCE,
        "impl StringExoticComputedKey {",
        "\n}\n\n#[derive(Debug, Clone, PartialEq, Eq)]",
    );
    assert!(conversion.contains("match self {"));
    assert!(conversion.contains("Self::CanonicalIndex(index) => PropertyKeyIr::ArrayIndex(index),"));
    assert!(conversion.contains("Self::OrdinaryPropertyKey(key) => key,"));
    assert!(!conversion.contains("_ =>"));
    assert!(!conversion.contains("unreachable!"));
}

#[test]
fn failure_to_prove_a_string_index_preserves_the_property_key() {
    let lowering = between(
        LOWERING_SOURCE,
        "fn lower_string_index_key(",
        "fn lower_arguments_index_key(",
    );
    assert_eq!(
        lowering
            .matches(".classify_string_exotic_computed_key(expr)")
            .count(),
        1
    );
    assert!(!lowering.contains("string index must be number"));

    let classifier = between(
        lowering,
        "fn classify_string_exotic_computed_key(",
        "fn static_string_exotic_index(",
    );
    assert!(classifier.contains("self.static_array_numeric_property_key(expr)"));
    assert!(classifier.contains("self.lower_static_property_key(expr)"));
    assert!(classifier.contains(
        "StringExoticComputedKey::OrdinaryPropertyKey(PropertyKeyIr::StringExpr(Box::new(key)))"
    ));
    assert!(!classifier.contains("unsupported_expr"));
}

#[test]
fn backend_classifies_dynamic_keys_and_preserves_prototype_fallback() {
    let string_read = between(
        OBJECTS_SOURCE,
        "ValueKind::String => match key {",
        "ValueKind::Arguments => match key {",
    );
    let dynamic_key = between(
        string_read,
        "PropertyKeyIr::StringExpr(_) => {",
        "\n                _ => {",
    );
    assert!(dynamic_key.contains("compile_object_key_to_locals("));
    assert!(dynamic_key.contains("emit_canonical_numeric_index_string("));
    assert!(dynamic_key.contains("emit_string_index_read("));
    assert!(dynamic_key.contains("STRING_PROTOTYPE_GLOBAL_INDEX"));
    assert!(dynamic_key.contains("emit_object_read_with_key_tag("));
    assert!(!dynamic_key.contains("emit_string_index_0_to_4_or_minus_one("));

    let proven_index = between(
        string_read,
        "PropertyKeyIr::ArrayIndex(_) => {",
        "PropertyKeyIr::StringExpr(_) => {",
    );
    assert!(proven_index.contains("Instruction::I64LtU"));
    assert!(proven_index.contains("Instruction::Else"));
    assert!(proven_index.contains("STRING_PROTOTYPE_GLOBAL_INDEX"));
    assert!(proven_index.contains("emit_object_read("));
}
