const PLAIN_TIME_SOURCE: &str = include_str!("../src/builtins/temporal_plain_time.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/temporal-plain-time-field-authority.md");
const TASK: &str = include_str!("../../../tasks/22-date-temporal.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn temporal_time_unit_exhaustively_owns_plain_time_layout_and_range() {
    let projections = bounded(
        PLAIN_TIME_SOURCE,
        "impl TemporalTimeUnit {",
        "/// `ToTemporalTimeRecord` reads the property bag",
    );
    let expected = r#"
        const fn plain_time_field_index(self) -> usize {
            match self {
                Self::Hour => 0,
                Self::Minute => 1,
                Self::Second => 2,
                Self::Millisecond => 3,
                Self::Microsecond => 4,
                Self::Nanosecond => 5,
            }
        }

        const fn plain_time_record_offset(self) -> u64 {
            match self {
                Self::Hour => HEAP_TEMPORAL_PLAIN_TIME_HOUR_OFFSET,
                Self::Minute => HEAP_TEMPORAL_PLAIN_TIME_MINUTE_OFFSET,
                Self::Second => HEAP_TEMPORAL_PLAIN_TIME_SECOND_OFFSET,
                Self::Millisecond => HEAP_TEMPORAL_PLAIN_TIME_MILLISECOND_OFFSET,
                Self::Microsecond => HEAP_TEMPORAL_PLAIN_TIME_MICROSECOND_OFFSET,
                Self::Nanosecond => HEAP_TEMPORAL_PLAIN_TIME_NANOSECOND_OFFSET,
            }
        }

        const fn plain_time_field_maximum(self) -> i64 {
            match self {
                Self::Hour => 23,
                Self::Minute | Self::Second => 59,
                Self::Millisecond | Self::Microsecond | Self::Nanosecond => 999,
            }
        }
    }
"#;
    assert_eq!(normalized(projections), normalized(expected));
    assert_eq!(projections.matches("match self {").count(), 3);
    for variant in [
        "Hour",
        "Minute",
        "Second",
        "Millisecond",
        "Microsecond",
        "Nanosecond",
    ] {
        assert_eq!(
            projections.matches(&format!("Self::{variant}")).count(),
            3,
            "field `{variant}` must select index, record offset and range"
        );
    }
    for forbidden in ["_ =>", "unreachable!", "default", "==", "!="] {
        assert!(!projections.contains(forbidden), "found `{forbidden}`");
    }
    for removed_parallel_authority in [
        "TEMPORAL_PLAIN_TIME_FIELD_OFFSETS",
        "TEMPORAL_PLAIN_TIME_FIELD_MAXIMA",
        "TEMPORAL_PLAIN_TIME_FIELD_NAMES",
    ] {
        assert!(
            !PLAIN_TIME_SOURCE.contains(removed_parallel_authority),
            "found `{removed_parallel_authority}`"
        );
    }
}

#[test]
fn record_range_and_scalar_consumers_select_locals_through_the_unit_authority() {
    assert_eq!(
        PLAIN_TIME_SOURCE
            .matches("for unit in TemporalTimeUnit::ALL {")
            .count(),
        5,
        "allocation, loading, rejection, constraint and scalar conversion share one domain"
    );
    assert_eq!(
        PLAIN_TIME_SOURCE
            .matches("field_locals[unit.plain_time_field_index()]")
            .count(),
        6,
        "all five ordered consumers and the accessor select through the field index"
    );
    assert!(!PLAIN_TIME_SOURCE.contains("TemporalTimeUnit::ALL.iter().zip(field_locals.iter())"));
    assert_eq!(
        PLAIN_TIME_SOURCE
            .matches("unit.plain_time_record_offset()")
            .count(),
        2,
        "record writes and reads share the offset projection"
    );
    assert_eq!(
        PLAIN_TIME_SOURCE
            .matches("unit.plain_time_field_maximum()")
            .count(),
        3,
        "reject and both constrain bounds share the maximum projection"
    );

    let total = bounded(
        PLAIN_TIME_SOURCE,
        "    pub(crate) fn emit_temporal_plain_time_total_nanoseconds(",
        "    pub(crate) fn emit_temporal_plain_time_from_nanoseconds(",
    );
    assert!(total.contains("unit.nanoseconds()"));
    assert!(!total.contains("field_locals[index]"));
}

#[test]
fn standard_dispatch_produces_only_named_plain_time_units() {
    let dispatch = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::TemporalPlainTimeCompare => {",
        "            StandardBuiltinId::TemporalPlainTimePrototypeWith => {",
    );
    for (getter, unit) in [
        ("Hour", "Hour"),
        ("Minute", "Minute"),
        ("Second", "Second"),
        ("Millisecond", "Millisecond"),
        ("Microsecond", "Microsecond"),
        ("Nanosecond", "Nanosecond"),
    ] {
        let producer = format!(
            "StandardBuiltinId::TemporalPlainTimePrototype{getter}Getter => {{\n                self.emit_temporal_plain_time_field(TemporalTimeUnit::{unit}, function)?;"
        );
        assert_eq!(dispatch.matches(&producer).count(), 1, "getter `{getter}`");
    }
    assert_eq!(
        dispatch
            .matches("self.emit_temporal_plain_time_field(")
            .count(),
        6
    );
    assert!(!dispatch.contains("| StandardBuiltinId"));
    assert!(!dispatch.contains("builtin, function"));

    let emitter = bounded(
        PLAIN_TIME_SOURCE,
        "    pub(crate) fn emit_temporal_plain_time_field(",
        "    pub(crate) fn emit_temporal_plain_time_value_of(",
    );
    assert!(emitter.contains("unit: TemporalTimeUnit,"));
    assert!(emitter.contains("field_locals[unit.plain_time_field_index()]"));
    assert!(!emitter.contains("StandardBuiltinId"));
    assert!(!emitter.contains("unreachable!"));
}

#[test]
fn contract_and_task_record_the_invariant_and_non_claim() {
    let normalized_contract = normalized(CONTRACT);
    let normalized_task = normalized(TASK);
    for evidence in [
        "TemporalTimeUnit",
        "record offset",
        "valid maximum",
        "property-bag read order",
        "does not change emitted Wasm",
    ] {
        let normalized_evidence = normalized(evidence);
        assert!(
            normalized_contract.contains(&normalized_evidence),
            "contract evidence `{evidence}`"
        );
        assert!(
            normalized_task.contains(&normalized_evidence),
            "task evidence `{evidence}`"
        );
    }
}
