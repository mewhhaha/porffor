const RUNTIME_SOURCE: &str = include_str!("../src/lib.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

#[test]
fn intrinsic_property_descriptor_has_only_four_semantic_shapes() {
    assert!(!RUNTIME_SOURCE.contains("pub struct IntrinsicPropertyDescriptor"));

    let descriptor = bounded(
        RUNTIME_SOURCE,
        "pub enum IntrinsicPropertyDescriptor {",
        "impl IntrinsicPropertyDescriptor {",
    );
    for shape in [
        "FunctionName {",
        "FunctionLength {",
        "ConstructorPrototype {",
        "PrototypeConstructor {",
    ] {
        assert_eq!(
            descriptor.matches(shape).count(),
            1,
            "{shape} must be declared exactly once"
        );
    }
    for field in ["key:", "value:", "attributes:"] {
        assert!(!descriptor.contains(field));
    }
}

#[test]
fn property_shape_projects_every_observable_field_exhaustively() {
    let projections = bounded(
        RUNTIME_SOURCE,
        "impl IntrinsicPropertyDescriptor {",
        "macro_rules! intrinsic_registry",
    );
    for method in ["owner", "key", "value", "attributes"] {
        assert!(
            projections.contains(&format!("pub const fn {method}")),
            "missing {method} projection"
        );
    }
    assert_eq!(projections.matches("match self {").count(), 4);
    assert!(!projections.contains("_ =>"));
}

#[test]
fn registry_builder_constructs_only_closed_property_shapes() {
    for removed_builder in [
        "function_name_descriptor",
        "function_length_descriptor",
        "constructor_prototype_descriptor",
        "prototype_constructor_descriptor",
    ] {
        assert!(!RUNTIME_SOURCE.contains(removed_builder));
    }

    let builder = bounded(
        RUNTIME_SOURCE,
        "const INTRINSIC_PROPERTY_PLACEHOLDER:",
        "const fn role_is_constructor_side(",
    );
    for shape in [
        "IntrinsicPropertyDescriptor::FunctionName",
        "IntrinsicPropertyDescriptor::FunctionLength",
        "IntrinsicPropertyDescriptor::ConstructorPrototype",
        "IntrinsicPropertyDescriptor::PrototypeConstructor",
    ] {
        assert!(builder.contains(shape), "builder must construct {shape}");
    }
}
