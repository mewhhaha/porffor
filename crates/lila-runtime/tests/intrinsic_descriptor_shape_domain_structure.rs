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
fn descriptor_shape_is_the_only_callable_role_state() {
    let metadata = bounded(
        RUNTIME_SOURCE,
        "pub struct IntrinsicFunctionMetadata {",
        "pub enum IntrinsicDescriptorShape {",
    );
    assert!(metadata.contains("pub name: &'static str,"));
    assert!(metadata.contains("pub length: u32,"));
    assert!(!metadata.contains("constructable"));

    let descriptor = bounded(
        RUNTIME_SOURCE,
        "pub struct IntrinsicDescriptor {",
        "impl IntrinsicDescriptor {",
    );
    assert!(descriptor.contains("pub shape: IntrinsicDescriptorShape,"));
    assert!(!descriptor.contains("pub role:"));
    assert!(!descriptor.contains("pub function:"));

    let shape = bounded(
        RUNTIME_SOURCE,
        "pub enum IntrinsicDescriptorShape {",
        "impl IntrinsicDescriptorShape {",
    );
    assert!(shape.contains("Constructor(IntrinsicFunctionMetadata),"));
    assert!(shape.contains("Function(IntrinsicFunctionMetadata),"));
    assert!(shape.contains("CallablePrototype(IntrinsicFunctionMetadata),"));
    assert!(shape.contains("Prototype,"));
}

#[test]
fn descriptor_shape_projects_role_callability_and_constructability_exhaustively() {
    let projections = bounded(
        RUNTIME_SOURCE,
        "impl IntrinsicDescriptorShape {",
        "pub struct IntrinsicDescriptor {",
    );
    for method in ["role", "function", "is_callable", "is_constructable"] {
        assert!(
            projections.contains(&format!("pub const fn {method}")),
            "missing {method} projection"
        );
    }
    assert_eq!(projections.matches("match self {").count(), 4);
    assert!(!projections.contains("_ =>"));
}

#[test]
fn registry_uses_each_closed_descriptor_shape_with_the_expected_census() {
    let registry = bounded(
        RUNTIME_SOURCE,
        "intrinsic_registry! {",
        "const fn function_length_name_attributes(",
    );
    let expected = [
        ("Constructor", 10),
        ("Function", 2),
        ("CallablePrototype", 1),
        ("Prototype", 10),
    ];
    for (shape, count) in expected {
        assert_eq!(
            registry
                .matches(&format!("shape: IntrinsicDescriptorShape::{shape}"))
                .count(),
            count,
            "unexpected {shape} row census"
        );
    }
    assert!(!registry.contains("role: IntrinsicRole::"));
    assert!(!registry.contains("function: Some("));
    assert!(!registry.contains("function: None"));
    assert!(!registry.contains("constructable:"));
}
