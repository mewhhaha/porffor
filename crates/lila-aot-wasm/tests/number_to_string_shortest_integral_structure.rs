const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const NUMBER_TO_STRING_SOURCE: &str = include_str!("../src/operations/number_to_string.rs");
const RYU_SOURCE: &str = include_str!("../src/operations/number_to_string/ryu.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const LILA_IR_MANIFEST: &str = include_str!("../../lila-ir/Cargo.toml");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}`"))
        .0
}

#[test]
fn number_to_string_has_one_private_ryu_owner() {
    assert_eq!(
        OPERATIONS_SOURCE.matches("mod number_to_string;").count(),
        1
    );
    assert!(!OPERATIONS_SOURCE.contains("pub mod number_to_string;"));
    assert_eq!(NUMBER_TO_STRING_SOURCE.matches("mod ryu;").count(), 1);
    assert!(!NUMBER_TO_STRING_SOURCE.contains("pub mod ryu;"));
    assert_eq!(
        NUMBER_TO_STRING_SOURCE
            .matches("pub(crate) fn emit_number_to_string_payload(")
            .count(),
        1
    );
    assert_eq!(
        RYU_SOURCE
            .matches("pub(super) fn emit_ryu_number_to_string_payload(")
            .count(),
        1
    );
}

#[test]
fn ryu_power_domain_is_closed_and_traps_if_its_proof_is_violated() {
    let domain = bounded(
        RYU_SOURCE,
        "enum RyuPowerTable {",
        "\n}\n\nimpl RyuPowerTable",
    );
    assert_eq!(domain.matches("\n    InversePow5,").count(), 1);
    assert_eq!(domain.matches("\n    Pow5,").count(), 1);
    let projections = bounded(
        RYU_SOURCE,
        "impl RyuPowerTable {",
        "\n}\n\nconst fn pow5_bits",
    );
    for arm in [
        "Self::InversePow5 => 291",
        "Self::Pow5 => 325",
        "Self::InversePow5 => inverse_pow5(index)",
        "Self::Pow5 => pow5(index)",
    ] {
        assert!(projections.contains(arm), "missing exhaustive arm `{arm}`");
    }
    assert!(!projections.contains("_ =>"));

    let lookup = bounded(
        RYU_SOURCE,
        "fn emit_ryu_power_lookup(",
        "\n    fn emit_unsigned_multiply_128(",
    );
    assert!(lookup.contains("for index in 0..=table.last_index()"));
    assert!(lookup.contains("Instruction::LocalGet(high_local)"));
    assert!(lookup.contains("Instruction::I64Eqz"));
    assert!(lookup.contains("Instruction::Unreachable"));
}

#[test]
fn dynamic_formatter_uses_shortest_decimal_and_ecmascript_spelling_projections() {
    for operation in [
        "fn emit_unsigned_multiply_128(",
        "fn emit_ryu_mul_shift(",
        "fn emit_ryu_shortest_decimal(",
        "fn emit_ryu_common_digit_removal(",
        "fn emit_ryu_trailing_zero_digit_removal(",
        "fn emit_ecmascript_decimal_payload(",
        "fn emit_scientific_decimal_payload(",
    ] {
        assert!(RYU_SOURCE.contains(operation), "missing `{operation}`");
    }
    for threshold in ["I64Const(21)", "I64Const(-6)"] {
        assert!(RYU_SOURCE.contains(threshold), "missing `{threshold}`");
    }
    for removed_heuristic in [
        "1_000_000.0",
        "frac_scaled_local",
        "emit_fraction_width_local",
        "shortest_int_u_local",
        "self.strings.payload(\"1e+21\")",
    ] {
        assert!(
            !NUMBER_TO_STRING_SOURCE.contains(removed_heuristic)
                && !RYU_SOURCE.contains(removed_heuristic)
                && !OPERATIONS_SOURCE.contains(removed_heuristic),
            "obsolete formatter path remains: `{removed_heuristic}`"
        );
    }
}

#[test]
fn static_number_spelling_uses_the_pinned_ecmascript_authority() {
    assert!(LILA_IR_MANIFEST.contains("ryu-js = \"=1.0.2\""));
    let formatter = bounded(
        LOWERING_SOURCE,
        "fn js_number_to_string(value: f64) -> String {",
        "\n    }\n\n    fn parse_float_string",
    );
    assert!(formatter.contains("ryu_js::Buffer::new().format(value).to_string()"));
    assert!(!formatter.contains("value.to_string()"));

    let property_key = bounded(
        LOWERING_SOURCE,
        "fn static_number_property_key_from_value(value: f64) -> String {",
        "\n    }\n\n    fn static_number_index_expr",
    );
    assert!(property_key.contains("Self::js_number_to_string(value)"));
    assert!(!property_key.contains("format!("));
}
