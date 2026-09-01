const LOWERING_SOURCE: &str = include_str!("../src/lowering.rs");
const LOWERING_HELPERS_SOURCE: &str = include_str!("../src/lowering_helpers.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const REGEXP_SOURCE: &str = include_str!("../src/regexp.rs");
const CLASS_DEFINITION_SOURCE: &str = include_str!("../src/lowering/class_definition.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/obsolete-lowering-specialization-removal.md"
);
const TASK: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");

fn generated_function_output_source() -> &'static str {
    let start = LOWERING_SOURCE
        .find("pub(crate) struct GeneratedFunctionOutput {")
        .expect("generated-function output declaration");
    let end = LOWERING_SOURCE[start..]
        .find("\n}")
        .map(|offset| start + offset + 2)
        .expect("generated-function output end");
    &LOWERING_SOURCE[start..end]
}

#[test]
fn disconnected_lowering_specializations_are_absent() {
    for name in [
        "target_has_private_brand",
        "lower_generated_iterator_function_expression",
        "lower_generated_iterator_function",
        "lower_this_range_generator_function_body",
        "single_lexical_number_binding",
        "single_lexical_expression_binding",
        "expression_is_this_unsigned_right_shift_zero",
        "while_body_yields_and_increments",
        "alloc_generated_iterator_values_name",
        "lower_generator_body_as_array_iterator",
        "lower_yield_star_generator_iife",
        "delegate_method_returns_non_object",
        "static_generator_declaration_elements",
        "static_generator_statement_list_elements",
        "static_generator_yield_string_element",
        "static_generator_for_loop_string_elements",
        "static_generator_string_for_loop_initializer",
        "static_generator_string_for_loop_body",
        "static_string_from_char_code_yield_name",
        "static_generator_yield_identifier_name",
        "static_string_from_char_code_arg_is_named",
        "static_string_from_char_code_arg_name",
        "static_negated_string_match_regex",
        "static_string_from_char_code_value",
        "static_generator_declaration_elements_by_name",
        "merge_operand_shapes",
    ] {
        assert!(!LOWERING_SOURCE.contains(name), "`{name}`");
    }
    assert!(!LOWERING_HELPERS_SOURCE.contains("StaticStringGeneratorLoopBody"));
    assert!(!LIB_SOURCE.contains("use regress::Regex;"));
}

#[test]
fn live_lowering_authorities_and_output_fields_remain() {
    for name in [
        "lower_generator_expression",
        "lower_static_yield_star_generator_method_call",
        "static_generator_declaration_values",
        "static_generator_for_loop_condition",
        "static_string_typed_expr",
        "merge_heap_shapes",
        "lower_generated_ast_function",
    ] {
        assert!(LOWERING_SOURCE.contains(name), "`{name}`");
    }

    let fields = generated_function_output_source()
        .lines()
        .filter(|line| line.trim_start().starts_with("pub(crate) "))
        .collect::<Vec<_>>();
    assert_eq!(
        fields,
        [
            "pub(crate) struct GeneratedFunctionOutput {",
            "    pub(crate) return_info: ValueInfo,",
            "    pub(crate) construct_this_info: Option<ValueInfo>,",
        ]
    );
    assert!(CLASS_DEFINITION_SOURCE.contains(".return_info"));
    assert!(CLASS_DEFINITION_SOURCE.contains("output.construct_this_info"));
    assert!(REGEXP_SOURCE.contains("use regress::{"));
}

#[test]
fn removal_has_frozen_source_evidence() {
    for evidence in [CONTRACT, TASK] {
        for hash in [
            "5fa129a28e54d16a8d17a6d160906b0c4e018205424be6173ed5571d2fadf9b2",
            "8ee9816ca0c120d3d1513ac8b831c3a0783f39b7db85b431c12ee89502a1c5a9",
            "02dbdf1e8f7aa05681dffa2ef505eade66622569ac53cde246e368a25ae737ff",
            "1320eddae0b215dfd5cc7f4f36bdaae2b85aa0738dea59bdbd6a4835a6faf9d8",
            "092a89c3965593b028c642d690c41a3c5bce5089396c747c49cdbf30c3a7d518",
            "55261d8d96ceb75dbbece9833835c68d4a56c695ad8f3bbce3288145cb6efeba",
            "92e5b6db98afaf7bb5c97c1db79246f5b3d5ea40408b15b0f48d82d65c5958e3",
            "326e77a61a4c63276a206c7eb836621ba4b8bfb3f1e3bb44ce7ca914904abef6",
            "02a744bb3487bffa56d2fc11df81f51d862a0ee57da8aa56c88af443e5465530",
            "c78546c57688c1d6cbb796baf74584b5c4bc61c448ff6541212aed7efa88d974",
        ] {
            assert!(evidence.contains(hash));
        }
        assert!(evidence.contains("no new JavaScript behavior"));
    }
}
