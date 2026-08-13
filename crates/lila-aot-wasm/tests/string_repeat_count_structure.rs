const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
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
fn repeat_count_has_one_total_to_integer_or_infinity_path() {
    let count = bounded(
        ARRAY_SOURCE,
        "pub(crate) fn emit_to_repeat_count_i64_from_value_locals(",
        "pub(crate) fn emit_array_grow_buffer(",
    );

    assert_eq!(
        count
            .matches("emit_to_integer_or_infinity_number_payload_from_number_payload(")
            .count(),
        1
    );
    assert_eq!(count.matches("Instruction::I64TruncSatF64U").count(), 1);
    assert!(!count.contains("Instruction::I64TruncF64U"));
    assert_eq!(
        count
            .matches("emit_throw_current_function_realm_range_error(")
            .count(),
        1
    );
    assert!(!count.contains("emit_throw_runtime_error("));

    assert_before(
        count,
        "emit_value_to_number_payload(",
        "emit_to_integer_or_infinity_number_payload_from_number_payload(",
    );
    assert_before(
        count,
        "emit_to_integer_or_infinity_number_payload_from_number_payload(",
        "Instruction::F64Lt",
    );
    assert_before(count, "Instruction::F64Lt", "Instruction::I64TruncSatF64U");
}

#[test]
fn repeat_empty_and_zero_fast_path_precedes_the_result_limit() {
    let repeat = bounded(
        STRING_SOURCE,
        "pub(crate) fn emit_repeat_string_payload_from_locals(",
        "pub(crate) fn emit_string_search_ascii_case_insensitive_literal_from_pattern_payload(",
    );
    let empty_result_offset = repeat
        .find("self.strings.payload(\"\")")
        .expect("empty repeat result");
    let fast_path = &repeat[..empty_result_offset];

    assert_eq!(fast_path.matches("Instruction::I64Eqz").count(), 2);
    assert_eq!(fast_path.matches("Instruction::I32Or").count(), 1);
    assert!(fast_path.contains("Instruction::LocalGet(src_len_local)"));
    assert!(fast_path.contains("Instruction::LocalGet(count_local)"));
    assert_before(
        repeat,
        "self.strings.payload(\"\")",
        "Instruction::I64Const(0xFFFF_FFFFu64 as i64)",
    );
    assert_eq!(
        repeat
            .matches("emit_throw_current_function_realm_range_error(")
            .count(),
        1
    );
    assert!(!repeat.contains("emit_throw_runtime_error("));
}

#[test]
fn repeat_builtin_normalizes_before_materializing() {
    let repeat = bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::StringPrototypeRepeat => {",
        "StandardBuiltinId::StringPrototypeNormalize => {",
    );

    assert_eq!(
        repeat
            .matches("emit_to_repeat_count_i64_from_value_locals(")
            .count(),
        1
    );
    assert_eq!(
        repeat
            .matches("emit_repeat_string_payload_from_locals(")
            .count(),
        1
    );
    assert_before(
        repeat,
        "emit_to_repeat_count_i64_from_value_locals(",
        "emit_repeat_string_payload_from_locals(",
    );
}
