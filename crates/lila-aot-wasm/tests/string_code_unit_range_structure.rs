const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const STRING_SOURCE: &str = include_str!("../src/builtins/string.rs");
const STRING_RANGE_SOURCE: &str = include_str!("../src/builtins/string/string_code_unit_range.rs");

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
fn string_code_unit_range_has_one_private_child_owner() {
    assert_eq!(
        STRING_SOURCE
            .matches("\nmod string_code_unit_range;\n")
            .count(),
        1
    );
    assert!(!STRING_SOURCE.contains("\npub mod string_code_unit_range;\n"));
    assert!(!STRING_SOURCE.contains("\npub(crate) mod string_code_unit_range;\n"));
    assert!(!STRING_SOURCE.contains("\nmod string_code_unit_range {\n"));
    assert!(STRING_RANGE_SOURCE.starts_with("use super::*;\n\n"));

    for declaration in [
        "struct UnitIndexLocal(u32);",
        "struct UnitLengthLocal(u32);",
        "struct RangeLengthLocal(u32);",
        "struct MaterializableRangeLocals {",
        "enum Method {",
    ] {
        assert_eq!(
            STRING_RANGE_SOURCE.matches(declaration).count(),
            1,
            "child must own exactly one `{declaration}`"
        );
    }
    for unique_declaration in [
        "struct RangeLengthLocal(u32);",
        "struct MaterializableRangeLocals {",
    ] {
        assert!(
            !STRING_SOURCE.contains(unique_declaration),
            "parent must not retain `{unique_declaration}`"
        );
    }
    assert_eq!(
        STRING_RANGE_SOURCE
            .lines()
            .map(str::trim_start)
            .filter(|line| line.starts_with("struct ") || line.starts_with("enum "))
            .count(),
        5
    );
    assert!(!STRING_RANGE_SOURCE.contains("pub struct "));
    assert!(!STRING_RANGE_SOURCE.contains("pub enum "));

    for definition in [
        "    fn emit_normalized_index(",
        "    fn emit_range(",
        "    fn emit_payload(",
        "pub(super) fn emit_slice(",
        "pub(super) fn emit_substring(",
        "fn emit(",
    ] {
        assert_eq!(
            STRING_RANGE_SOURCE.matches(definition).count(),
            1,
            "child must own exactly one `{definition}`"
        );
    }
    for unique_definition in [
        "    fn emit_normalized_index(",
        "    fn emit_range(",
        "    fn emit_payload(",
        "pub(super) fn emit_slice(",
        "pub(super) fn emit_substring(",
    ] {
        assert!(
            !STRING_SOURCE.contains(unique_definition),
            "parent must not retain `{unique_definition}`"
        );
    }
    assert_eq!(STRING_RANGE_SOURCE.matches("fn ").count(), 6);
    assert_eq!(
        STRING_RANGE_SOURCE
            .lines()
            .filter(|line| line.starts_with("pub(super) fn "))
            .count(),
        2
    );
    assert!(!STRING_RANGE_SOURCE.contains("pub(crate) fn "));
    assert!(!STRING_RANGE_SOURCE.contains("\npub fn "));

    for retained_parent_entry in [
        "    pub(crate) fn compile_string_slice_range_builtin(",
        "    pub(crate) fn compile_string_substring_range_builtin(",
    ] {
        assert_eq!(STRING_SOURCE.matches(retained_parent_entry).count(), 1);
        assert!(!STRING_RANGE_SOURCE.contains(retained_parent_entry));
    }
}

#[test]
fn string_ranges_have_one_typed_utf16_materializer() {
    let coordinator = STRING_RANGE_SOURCE;

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
    assert!(coordinator.contains("fn emit_payload(\n        self,"));

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
        "pub(crate) fn emit_string_char_code_at_from_locals(",
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
        "pub(crate) fn compile_array_prototype_join_builtin(",
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
fn created_realm_installs_both_self_backed_range_methods() {
    let method_metas = bounded(
        HOST_SOURCE,
        "        let string_prototype_method_metas = [",
        "        let boolean_prototype_method_metas = [",
    );
    for (name, builtin) in [
        ("slice", "StringPrototypeSlice"),
        ("substring", "StringPrototypeSubstring"),
    ] {
        let entry_start = format!("            (\n                \"{name}\",");
        assert_eq!(method_metas.matches(&entry_start).count(), 1);
        let entry = method_metas
            .split_once(&entry_start)
            .expect("created-realm String range method entry")
            .1
            .split_once("            ),")
            .expect("created-realm String range method entry end")
            .0;
        assert_eq!(
            entry
                .matches(&format!("StandardBuiltinId::{builtin}.function_id()"))
                .count(),
            1
        );
    }

    let installer = bounded(
        HOST_SOURCE,
        "        for (name, meta) in &string_prototype_method_metas {",
        "        for (name, meta) in &array_prototype_method_metas {",
    );
    assert_eq!(
        installer
            .matches("emit_function_value_payload_in_realm(")
            .count(),
        1
    );
    assert_eq!(
        installer
            .matches(
                "method_payload_local,\n                HEAP_FUNCTION_ENV_HANDLE_OFFSET,\n                method_payload_local,",
            )
            .count(),
        1
    );
    assert_eq!(
        installer
            .matches(
                "method_payload_local,\n                HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,\n                type_error_prototype_local,",
            )
            .count(),
        1
    );
    assert_eq!(
        installer
            .matches("string_prototype_local,\n                name,")
            .count(),
        1
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
        "    fn emit_state_in_inclusive_range_i32(",
    );

    for normalizer in [integer, slice] {
        assert_eq!(
            normalizer.matches("Instruction::I64TruncSatF64S").count(),
            1
        );
        assert!(!normalizer.contains("Instruction::I64TruncF64S"));
    }
}
