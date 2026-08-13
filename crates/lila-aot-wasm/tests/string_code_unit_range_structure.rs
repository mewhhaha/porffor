const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const STRING_SOURCE: &str = include_str!("../src/builtins/string.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

#[test]
fn string_ranges_have_one_typed_utf16_materializer() {
    let coordinator = bounded(
        STRING_SOURCE,
        "mod string_code_unit_range {",
        "impl<'a> FunctionBuilder<'a> {",
    );

    for local in [
        "UnitIndexLocal",
        "UnitLengthLocal",
        "RangeLengthLocal",
        "MaterializableRangeLocals",
    ] {
        assert_eq!(
            coordinator.matches(&format!("struct {local}")).count(),
            1,
            "the range coordinator must own one private {local}"
        );
    }
    assert_eq!(coordinator.matches("#[must_use").count(), 4);
    assert!(!coordinator.contains("derive("));
    assert!(!coordinator.contains("impl Copy for"));
    assert!(coordinator.contains("fn emit_payload(\n            self,"));

    let method = bounded(coordinator, "enum Method {", "impl Method {");
    let variants = method
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && *line != "}")
        .collect::<Vec<_>>();
    assert_eq!(variants, ["Slice,", "Substring,"]);
    assert_eq!(coordinator.matches("Self::Slice =>").count(), 2);
    assert_eq!(coordinator.matches("Self::Substring =>").count(), 2);
    assert!(!coordinator.contains("_ =>"));

    assert_eq!(
        coordinator
            .matches("emit_utf16_code_unit_range_payload_from_locals(")
            .count(),
        1,
        "the consuming token must own the sole UTF-16 range materializer"
    );
    for forbidden in [
        "emit_utf16_code_unit_index_to_utf8_byte_offset_from_string_payload(",
        "emit_string_slice_payload_from_locals(",
        "emit_decode_utf8_scalar_at_index(",
    ] {
        assert!(
            !coordinator.contains(forbidden),
            "the visible range coordinator must not call `{forbidden}`"
        );
    }
    assert_eq!(
        coordinator
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        1
    );
    assert!(!coordinator.contains("emit_throw_runtime_error("));

    assert_before(
        coordinator,
        "emit_value_to_string_payload(",
        "emit_value_to_number_payload(start_tag_local",
    );
    assert_before(
        coordinator,
        "emit_value_to_number_payload(start_tag_local",
        "emit_value_to_number_payload(end_tag_local",
    );
    assert_before(
        coordinator,
        "method.emit_range(",
        ".emit_payload(builder, string_local, function)?",
    );
}

#[test]
fn standard_and_direct_entries_delegate_without_parallel_algorithms() {
    let compile_entries = bounded(
        STRING_SOURCE,
        "pub(crate) fn compile_string_slice_range_builtin(",
        "pub(crate) fn compile_string_concat_builtin(",
    );
    assert_eq!(
        compile_entries
            .matches("string_code_unit_range::emit_slice(")
            .count(),
        1
    );
    assert_eq!(
        compile_entries
            .matches("string_code_unit_range::emit_substring(")
            .count(),
        1
    );

    let standard_substring = bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::StringPrototypeSubstring => {",
        "StandardBuiltinId::StringPrototypeSlice => {",
    );
    let standard_slice = bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::StringPrototypeSlice => {",
        "StandardBuiltinId::StringPrototypeIndexOf => {",
    );
    assert_eq!(
        standard_substring
            .matches("compile_string_substring_range_builtin(function)?")
            .count(),
        1
    );
    assert_eq!(
        standard_slice
            .matches("compile_string_slice_range_builtin(function)?")
            .count(),
        1
    );
    for body in [standard_substring, standard_slice] {
        for forbidden in [
            "emit_value_to_string_payload(",
            "emit_value_to_number_payload(",
            "emit_utf16_code_unit_index_to_utf8_byte_offset_from_string_payload(",
            "emit_utf16_code_unit_range_payload_from_locals(",
            "emit_string_slice_payload_from_locals(",
        ] {
            assert!(
                !body.contains(forbidden),
                "standard String range entry must delegate instead of calling `{forbidden}`"
            );
        }
    }

    let direct_substring = bounded(
        STRING_SOURCE,
        "pub(crate) fn emit_string_substring_method_call(",
        "pub(crate) fn static_number_expr_value(",
    );
    assert_eq!(
        direct_substring
            .matches("emit_array_direct_builtin_method_call(")
            .count(),
        1
    );
    assert!(direct_substring.contains("StandardBuiltinId::StringPrototypeSubstring,"));
    for forbidden in [
        "active_throw_target(",
        "compile_expr_to_locals(",
        "emit_value_to_string_payload(",
        "emit_value_to_number_payload(",
        "emit_utf16_code_unit_range_payload_from_locals(",
        "emit_string_slice_payload_from_locals(",
    ] {
        assert!(
            !direct_substring.contains(forbidden),
            "direct substring must not retain parallel operation `{forbidden}`"
        );
    }

    let direct_builtin = bounded(
        ARRAY_SOURCE,
        "pub(crate) fn emit_array_direct_builtin_method_call(",
        "pub(crate) fn emit_array_push_method_call(",
    );
    assert_before(
        direct_builtin,
        "self.compile_expr_to_locals(",
        "self.emit_call_args_vector(args, function)",
    );
    assert_before(
        direct_builtin,
        "self.emit_call_args_vector(args, function)",
        "self.emit_direct_js_call_with_argv(",
    );
}

#[test]
fn annex_b_substr_retains_its_authoritative_utf16_range() {
    let substr = bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::StringPrototypeSubstr => {",
        "StandardBuiltinId::StringPrototypeSubstring => {",
    );
    assert_eq!(
        substr
            .matches("emit_utf16_code_unit_range_payload_from_locals(")
            .count(),
        1
    );
    assert!(!substr.contains("emit_utf16_code_unit_index_to_utf8_byte_offset_from_string_payload("));
    assert!(!substr.contains("emit_string_slice_payload_from_locals("));
}

#[test]
fn string_index_normalizers_saturate_before_clamping_to_length() {
    let integer = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_to_integer_clamped_to_string_len(",
        "    pub(crate) fn emit_to_slice_index_clamped_to_string_len(",
    );
    let slice = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_to_slice_index_clamped_to_string_len(",
        "    pub(crate) fn compile_for_of_array(",
    );

    for normalizer in [integer, slice] {
        assert_eq!(
            normalizer.matches("Instruction::I64TruncSatF64S").count(),
            1
        );
        assert!(!normalizer.contains("Instruction::I64TruncF64S"));
    }
}
