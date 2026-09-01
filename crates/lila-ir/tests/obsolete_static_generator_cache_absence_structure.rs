const LOWERING_SOURCE: &str = include_str!("../src/lowering.rs");
const ASSIGNMENT_SOURCE: &str = include_str!("../src/lowering/assignment.rs");
const CALL_EXPRESSION_SOURCE: &str = include_str!("../src/lowering/call_expression.rs");
const FOR_OF_SOURCE: &str = include_str!("../src/lowering/for_of.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/obsolete-static-generator-cache-removal.md");
const TASK: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");

#[test]
fn write_never_static_generator_cache_surface_is_absent() {
    let product_sources = [
        LOWERING_SOURCE,
        ASSIGNMENT_SOURCE,
        CALL_EXPRESSION_SOURCE,
        FOR_OF_SOURCE,
    ];

    for name in [
        "static_generator_sum_values",
        "static_generator_element_values",
        "prepare_static_generator_declarations",
        "is_static_generator_declaration",
        "static_generator_call_values",
        "static_generator_call_values_owned",
        "static_generator_call_elements_owned",
        "static_generator_call_name",
        "static_generator_call_is_known",
        "array_iterator_from_static_generator_values",
        "array_iterator_from_lowered_elements",
    ] {
        assert!(
            product_sources.iter().all(|source| !source.contains(name)),
            "`{name}`"
        );
    }
}

#[test]
fn live_generator_and_iterator_authorities_remain() {
    for name in [
        "static_generator_call_overrides",
        "static_iterator_binding_values",
        "static_generator_declaration_values",
        "static_generator_declaration_values_by_name",
        "static_object_iterator_literal_values",
        "static_object_iterator_iife_values",
        "static_iterator_values_expr",
        "array_literal_from_static_generator_values",
    ] {
        assert!(LOWERING_SOURCE.contains(name), "`{name}`");
    }

    assert!(CALL_EXPRESSION_SOURCE.contains("self.static_generator_call_overrides.get(&name)"));
    assert!(ASSIGNMENT_SOURCE.contains("self.static_object_iterator_literal_values(rhs)"));
    assert!(ASSIGNMENT_SOURCE.contains("let value = self.lower_expression(rhs);"));
    assert!(FOR_OF_SOURCE.contains("let element_info = ValueInfo {"));
}

#[test]
fn removal_has_frozen_source_evidence() {
    for evidence in [CONTRACT, TASK] {
        for hash in [
            "8043d5ff10f4b61f90d5caea850ee1f648d81a7c5bfd413715fd1776194bd27c",
            "51ca4e5119307e3df723701e54632dc8f37cfe0f231ea0bd6401c10e7d1bd0d2",
            "1f6bb5a929cb2250a07ba4d1deb96379788633d9da460d1e95e43c9d61360c1e",
            "455ea8b701e57fe6497169d2cfcf94bae1f804a6f21e37c7f47e044ea3eba1bb",
            "8291e01437badcde47d9d2d412b4b68fb4fb6025daa5d26459ab55b70b6dae79",
        ] {
            assert!(evidence.contains(hash));
        }
        assert!(evidence.contains("no new JavaScript behavior"));
    }
}
