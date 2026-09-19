const EPOCH: &str = include_str!("../src/builtins/temporal.rs");
const METHODS: &str = include_str!("../src/builtins/temporal_instant/methods.rs");
const ROUND: &str = include_str!("../src/builtins/temporal_instant/round.rs");
const ROUND_OPTIONS: &str = include_str!("../src/builtins/temporal_instant/round/options.rs");
const SETTINGS: &str = include_str!("../src/builtins/temporal_plain_date_time_methods.rs");
const CATALOG: &str = include_str!("../../lila-ir/src/builtins/catalog.rs");
const PLAN: &str = include_str!("../src/planning.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .expect(start)
        .1
        .split_once(end)
        .expect(end)
        .0
}

fn before(source: &str, first: &str, second: &str) {
    assert!(source.find(first).expect(first) < source.find(second).expect(second));
}

#[test]
fn exact_epoch_splitting_has_one_closed_layout_authority() {
    let authority = bounded(
        EPOCH,
        "impl TemporalEpochNanosecondsRecord {",
        "pub(super) enum TemporalZonedDateTimePlainTarget",
    );
    assert!(!authority.contains("_ =>"));
    for record in ["INSTANT", "ZONED_DATE_TIME"] {
        for field in ["PAYLOAD", "TAG"] {
            assert_eq!(
                authority
                    .matches(&format!(
                        "HEAP_TEMPORAL_{record}_EPOCH_NANOSECONDS_{field}_OFFSET"
                    ))
                    .count(),
                1
            );
        }
    }
    let splitter = bounded(
        EPOCH,
        "pub(super) fn emit_temporal_epoch_nanoseconds_pair(",
        "pub(crate) fn emit_temporal_epoch_nanoseconds_record_to_milliseconds(",
    );
    assert!(splitter.contains("record: TemporalEpochNanosecondsRecord"));
    assert!(splitter.contains("record.offsets()"));
    assert!(splitter.contains("emit_temporal_heap_bigint_millisecond_quotient("));
    assert!(!splitter.contains("F64"));
    assert!(!EPOCH.contains("fn emit_temporal_zoned_date_time_epoch_pair("));
}

#[test]
fn arithmetic_and_rounding_consume_the_private_validated_epoch_boundary() {
    for source in [METHODS, ROUND] {
        before(
            source,
            "emit_temporal_instant_record_from_receiver(",
            "emit_temporal_epoch_nanoseconds_pair(",
        );
        before(
            source,
            "emit_temporal_epoch_nanoseconds_bigint(",
            "emit_temporal_instant_validated_epoch(",
        );
        before(
            source,
            "emit_temporal_instant_validated_epoch(",
            "emit_alloc_validated_temporal_instant(",
        );
        assert!(!source.contains("self.emit_alloc_temporal_instant("));
        assert!(!source.contains("EpochNanoseconds("));
    }
    before(METHODS, "self.emit_to_temporal_duration(", "for unit in [");
    before(
        METHODS,
        "emit_temporal_duration_renormalize(",
        "emit_temporal_normalize_seconds_and_subseconds(",
    );
    assert!(METHODS.contains("StandardBuiltinId::TemporalInstantFrom.function_id()"));
    assert!(METHODS.contains("emit_direct_js_call("));
    assert!(METHODS.contains("emit_temporal_round_difference_time("));
}

#[test]
fn instant_rounding_keeps_its_full_day_and_as_if_positive_contract() {
    assert!(ROUND_OPTIONS.contains("NANOSECONDS_PER_TEMPORAL_DAY / unit.nanoseconds()"));
    assert!(ROUND_OPTIONS.contains("Instruction::I64GtU"));
    assert!(!ROUND_OPTIONS.contains("emit_temporal_plain_time_validate_increment("));
    before(
        ROUND_OPTIONS,
        "emit_temporal_duration_rounding_increment_option(",
        "emit_temporal_duration_rounding_mode_option(",
    );
    before(
        ROUND_OPTIONS,
        "emit_temporal_duration_rounding_mode_option(",
        "emit_temporal_duration_unit_option(",
    );
    assert!(ROUND.contains("RoundNumberToIncrementAsIfPositive"));
    before(
        ROUND,
        "emit_temporal_normalize_seconds_and_subseconds(",
        "Instruction::LocalSet(day)",
    );
    assert!(ROUND.contains("Instruction::LocalSet(parity)"));
    assert!(ROUND.contains("Instruction::LocalSet(positive)"));
    assert!(!ROUND.contains("F64"));
}

#[test]
fn instant_category_validation_follows_all_difference_option_reads() {
    let reader = bounded(
        SETTINGS,
        "pub(super) fn emit_temporal_date_time_difference_settings(",
        "/// Temporal proposal 5.3.x `until`",
    );
    for (first, second) in [
        (
            "TemporalUnitOptionProperty::LargestUnit",
            "emit_temporal_duration_rounding_increment_option(",
        ),
        (
            "emit_temporal_duration_rounding_increment_option(",
            "emit_temporal_duration_rounding_mode_option(",
        ),
        (
            "emit_temporal_duration_rounding_mode_option(",
            "TemporalUnitOptionProperty::SmallestUnit",
        ),
        (
            "TemporalUnitOptionProperty::SmallestUnit",
            "emit_temporal_require_unit_range(",
        ),
    ] {
        before(reader, first, second);
    }
    assert!(METHODS.contains("TemporalDateTimeDifferenceSettingsPlan::InstantSince"));
    assert!(METHODS.contains("matches!(operation, InstantDifference::Since)"));
    for name in ["Add", "Subtract", "Round", "Until", "Since"] {
        let row = bounded(
            CATALOG,
            &format!("    TemporalInstantPrototype{name} {{"),
            "\n    }",
        );
        assert!(row.contains("flags: [SYNCHRONOUS_USER_CODE]"));
        assert!(PLAN.contains(&format!(
            "StandardBuiltinId::TemporalInstantPrototype{name},"
        )));
    }
    let family = bounded(
        PLAN,
        "// The whole `Temporal.Instant` family",
        "StandardBuiltinId::TemporalZonedDateTimeConstructor",
    );
    assert!(family
        .contains("self.require_standard_builtin(StandardBuiltinId::TemporalDurationConstructor)"));
}

#[test]
fn instant_conversion_preserves_primitive_types_and_realm_owned_abrupts() {
    let conversion = bounded(
        EPOCH,
        "pub(crate) fn emit_temporal_instant_from(",
        "pub(crate) fn emit_temporal_zoned_date_time_compare(",
    );
    before(
        conversion,
        "OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME",
        "emit_tagged_to_primitive_locals_in_current_function_realm(",
    );
    before(
        conversion,
        "emit_tagged_to_primitive_locals_in_current_function_realm(",
        "emit_current_function_realm_primitive_to_tagged_locals(",
    );
    before(
        conversion,
        "emit_current_function_realm_primitive_to_tagged_locals(",
        "emit_throw_current_function_realm_type_error(",
    );
    before(
        conversion,
        "emit_throw_current_function_realm_type_error(",
        "emit_temporal_parse_iso_string(",
    );
    assert!(conversion.contains("ToPrimitiveHint::String"));
    assert!(!conversion.contains("emit_value_to_string_payload("));
    assert!(!conversion.contains("ValueKind::Array"));
    assert!(!conversion.contains("ValueKind::Function"));
}
