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
fn intrinsic_kind_exhaustively_owns_function_property_attributes() {
    assert!(!RUNTIME_SOURCE.contains("length_name_configurable"));

    let projection = bounded(
        RUNTIME_SOURCE,
        "const fn function_length_name_attributes(",
        "const fn intrinsic_property_descriptor_count(",
    );
    assert!(projection.contains("owner: IntrinsicKind"));
    assert!(projection.contains("match owner {"));
    assert!(!projection.contains("_ =>"));
    assert!(!projection.contains("if "));

    let variants = [
        "ObjectConstructor",
        "ObjectPrototype",
        "FunctionConstructor",
        "FunctionPrototype",
        "ArrayConstructor",
        "ArrayPrototype",
        "BigIntConstructor",
        "BigIntPrototype",
        "DateConstructor",
        "DatePrototype",
        "ProxyConstructor",
        "ArrayBufferConstructor",
        "ArrayBufferPrototype",
        "DataViewConstructor",
        "DataViewPrototype",
        "TypedArrayConstructor",
        "TypedArrayPrototype",
        "Uint8ArrayConstructor",
        "Uint8ArrayPrototype",
        "TypeErrorConstructor",
        "TypeErrorPrototype",
        "IteratorPrototype",
        "ThrowTypeError",
    ];
    for variant in variants {
        assert_eq!(
            projection
                .matches(&format!("IntrinsicKind::{variant}"))
                .count(),
            1,
            "{variant} must have exactly one property-attribute projection"
        );
    }
    assert_eq!(
        projection
            .matches("IntrinsicPropertyAttributes::BUILTIN_FUNCTION_LENGTH_NAME_FIXED")
            .count(),
        1
    );
    assert_eq!(
        projection
            .matches("IntrinsicPropertyAttributes::BUILTIN_FUNCTION_LENGTH_NAME_CONFIGURABLE")
            .count(),
        1
    );
}

#[test]
fn callable_rows_and_property_shapes_use_the_owner_projection() {
    let registry = bounded(
        RUNTIME_SOURCE,
        "intrinsic_registry! {",
        "const fn function_length_name_attributes(",
    );
    assert_eq!(
        registry
            .matches("shape: IntrinsicDescriptorShape::Constructor(")
            .count()
            + registry
                .matches("shape: IntrinsicDescriptorShape::Function(")
                .count()
            + registry
                .matches("shape: IntrinsicDescriptorShape::CallablePrototype(")
                .count(),
        13,
        "all callable shapes must own function metadata"
    );
    let descriptor = bounded(
        RUNTIME_SOURCE,
        "impl IntrinsicPropertyDescriptor {",
        "macro_rules! intrinsic_registry",
    );
    assert!(descriptor.contains("Self::FunctionName { owner, .. }"));
    assert!(descriptor.contains("Self::FunctionLength { owner, .. }"));
    assert_eq!(
        descriptor
            .matches("function_length_name_attributes(owner)")
            .count(),
        1
    );
}
