const DURATION: &str = include_str!("../src/builtins/temporal_duration.rs");
const FIELDS: &str = include_str!("../src/builtins/temporal_duration/fields.rs");
const METHODS: &str = include_str!("../src/builtins/temporal_duration_methods.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing {end}"))
        .0
}

#[test]
fn duration_fields_cannot_be_used_as_integer_scratch_arrays() {
    assert!(FIELDS.contains("pub(crate) struct TemporalDurationFields([u32; 10]);"));
    assert!(FIELDS.contains("fn number_bits(&self, unit: TemporalUnit) -> u32"));
    assert!(!FIELDS.contains("impl Index"));
    assert!(!FIELDS.contains("impl Deref"));
    for (source, function) in [
        (DURATION, "emit_temporal_duration_normalize_seconds"),
        (DURATION, "emit_temporal_duration_reject_invalid"),
        (DURATION, "emit_alloc_temporal_duration"),
        (DURATION, "emit_temporal_duration_load_record"),
        (METHODS, "emit_temporal_duration_partial_record"),
        (METHODS, "emit_to_temporal_duration"),
        (METHODS, "emit_temporal_duration_balance"),
        (METHODS, "emit_temporal_duration_parse_string"),
    ] {
        let signature = bounded(source, &format!("fn {function}("), ")");
        assert!(signature.contains("&TemporalDurationFields"), "{function}");
    }
    let allocator = bounded(
        DURATION,
        "fn emit_alloc_temporal_duration(",
        "fn emit_create_temporal_duration(",
    );
    assert!(!allocator.contains("F64ConvertI64"));
    assert!(!allocator.contains("I64Trunc"));
    assert!(!DURATION.contains("emit_temporal_duration_narrow_fields"));
}

#[test]
fn observable_conversion_precedes_range_validation() {
    let partial = bounded(
        METHODS,
        "fn emit_temporal_duration_partial_record(",
        "fn emit_to_temporal_duration(",
    );
    assert!(partial.contains("TEMPORAL_DURATION_ALPHABETICAL_FIELDS"));
    assert!(partial.contains("emit_temporal_duration_field_to_number("));
    assert!(!partial.contains("I64Trunc"));
    assert!(!partial.contains("F64Ge"));
    assert!(!partial.contains("emit_temporal_duration_reject_invalid("));
    let conversion = bounded(
        DURATION,
        "fn emit_temporal_duration_field_to_number(",
        "fn emit_temporal_duration_sign(",
    );
    assert!(conversion.contains("F64Trunc"));
    assert!(conversion.contains("emit_temporal_duration_canonicalize_zero("));
    let constructor = bounded(
        DURATION,
        "fn emit_temporal_duration_constructor(",
        "fn emit_temporal_duration_field(",
    );
    assert!(
        constructor
            .find("emit_temporal_duration_field_to_number(")
            .unwrap()
            < constructor
                .find("emit_temporal_duration_reject_invalid(")
                .unwrap()
    );
    let to_duration = bounded(
        METHODS,
        "fn emit_to_temporal_duration(",
        "fn emit_temporal_duration_from(",
    );
    assert!(
        to_duration
            .find("emit_temporal_duration_partial_record(")
            .unwrap()
            < to_duration
                .find("emit_temporal_duration_reject_invalid(")
                .unwrap()
    );
    assert!(to_duration.contains("emit_is_heap_object_like_tag_i32(value_tag_local"));
    let options = bounded(
        METHODS,
        "fn emit_temporal_duration_options_object(",
        "fn emit_temporal_duration_option_get(",
    );
    assert!(options.contains("emit_is_heap_object_like_tag_i32(options_tag_local"));
}

#[test]
fn wide_arithmetic_crosses_one_exact_number_boundary() {
    let normalize = bounded(
        DURATION,
        "fn emit_temporal_duration_normalize_seconds(",
        "fn emit_temporal_duration_reject_invalid(",
    );
    assert!(normalize.contains("emit_temporal_duration_number_divmod("));
    assert!(!normalize.contains("F64Div"));
    assert!(!normalize.contains("F64Mul"));
    let divmod = bounded(
        FIELDS,
        "fn emit_temporal_duration_number_divmod(",
        "fn emit_temporal_duration_scaled_time_number(",
    );
    assert!(!divmod.contains("I64Trunc"));
    assert!(!divmod.contains("F64Div"));
    assert!(divmod.contains("I64DivU"));
    assert!(divmod.contains("I64RemU"));
    let projection_signature =
        bounded(FIELDS, "fn emit_temporal_duration_scaled_time_number(", ")");
    assert!(projection_signature.contains("projection: TemporalDurationNumberProjection"));
    assert!(!projection_signature.contains("scale:"));
    assert!(!projection_signature.contains("divisor:"));
    assert!(FIELDS.contains("const _: () = {"));
    assert!(FIELDS.contains("unit.per_second() * unit.nanoseconds() == 1_000_000_000"));
    let balance = bounded(
        METHODS,
        "fn emit_temporal_duration_balance(",
        "fn emit_temporal_duration_add(",
    );
    assert!(balance.contains("emit_temporal_duration_scaled_time_number("));
    assert!(!balance.contains("9_223_372_036_854_775_807"));
    let total = bounded(
        METHODS,
        "fn emit_temporal_duration_total(",
        "fn emit_temporal_duration_to_string(",
    );
    assert!(total.contains("emit_temporal_duration_scaled_time_number("));
}

#[test]
fn calendar_consumers_use_bounded_integer_projections_and_shared_negation() {
    for source in [
        include_str!("../src/builtins/temporal_plain_date_methods.rs"),
        include_str!("../src/builtins/temporal_plain_date_time_methods.rs"),
        include_str!("../src/builtins/temporal_plain_year_month_methods.rs"),
    ] {
        assert!(source.contains("reserve_temporal_duration_date_field_locals(&duration_locals"));
        assert!(source.contains("emit_temporal_duration_negate_fields(&duration_locals"));
        assert!(!source.contains("duration_locals["));
        assert!(!source.contains("duration_locals.iter()"));
    }
    let time = include_str!("../src/builtins/temporal_plain_time_methods.rs");
    assert!(time.contains("emit_temporal_duration_negate_fields(&duration_locals"));
    let difference = include_str!("../src/builtins/temporal_difference.rs");
    assert!(difference.contains("emit_temporal_duration_set_integer_field(&duration_locals"));
    assert!(!difference.contains("duration_locals["));
}
