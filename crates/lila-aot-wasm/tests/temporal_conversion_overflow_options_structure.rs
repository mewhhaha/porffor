const OPTIONS_SOURCE: &str = include_str!("../src/builtins/temporal_options.rs");
const DATE_SOURCE: &str = include_str!("../src/builtins/temporal_plain_date_methods.rs");
const DATE_TIME_SOURCE: &str = include_str!("../src/builtins/temporal_plain_date_time_methods.rs");
const MONTH_DAY_SOURCE: &str = include_str!("../src/builtins/temporal_plain_month_day.rs");
const TIME_SOURCE: &str = include_str!("../src/builtins/temporal_plain_time_methods.rs");
const YEAR_MONTH_SOURCE: &str =
    include_str!("../src/builtins/temporal_plain_year_month_methods.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn without_bounded(source: &str, start: &str, end: &str) -> String {
    let (before, tail) = source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"));
    let (_, after) = tail
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"));
    format!("{before}{after}")
}

#[test]
fn temporal_conversion_overflow_options_is_private_and_data_bearing() {
    let domain = bounded(
        OPTIONS_SOURCE,
        "/// Whether a `ToTemporal*` conversion owns an observable `overflow` options",
        "\n\n/// `GetTemporalOverflowOption`.",
    );
    let declaration = bounded(
        OPTIONS_SOURCE,
        "pub(super) enum TemporalConversionOverflowOptions {",
        "\n}\n\n/// `GetTemporalOverflowOption`.",
    );
    let normalized = declaration.split_whitespace().collect::<String>();

    assert_eq!(normalized, "Read{payload_local:u32,tag_local:u32},Omit,");
    assert!(!OPTIONS_SOURCE.contains("pub enum TemporalConversionOverflowOptions"));
    assert!(!OPTIONS_SOURCE.contains("pub(crate) enum TemporalConversionOverflowOptions"));
    assert!(!domain.contains("Default"));
    assert!(!domain.contains("PartialEq"));
    assert!(!domain.contains("Eq"));
    assert!(!OPTIONS_SOURCE.contains("impl TemporalConversionOverflowOptions"));
}

#[test]
fn all_five_converters_match_every_observable_overflow_read_exhaustively() {
    let consumers = [
        (
            DATE_SOURCE,
            "    pub(super) fn emit_temporal_to_temporal_date(",
            "    pub(crate) fn emit_temporal_plain_date_from(",
            3,
        ),
        (
            YEAR_MONTH_SOURCE,
            "    pub(super) fn emit_temporal_to_temporal_year_month(",
            "    pub(crate) fn emit_temporal_parse_year_month_string(",
            3,
        ),
        (
            TIME_SOURCE,
            "    pub(super) fn emit_to_temporal_time(",
            "    pub(crate) fn emit_temporal_plain_time_from(",
            3,
        ),
        (
            DATE_TIME_SOURCE,
            "    pub(super) fn emit_to_temporal_date_time(",
            "    pub(crate) fn emit_temporal_plain_date_time_from(",
            4,
        ),
        (
            MONTH_DAY_SOURCE,
            "    pub(super) fn emit_temporal_to_temporal_month_day(",
            "    pub(crate) fn emit_temporal_plain_month_day_from(",
            3,
        ),
    ];

    for (source, start, end, decisions) in consumers {
        let consumer = bounded(source, start, end);
        assert_eq!(
            consumer
                .matches("overflow_options: TemporalConversionOverflowOptions,")
                .count(),
            1,
            "typed options missing from `{start}`"
        );
        assert_eq!(
            consumer.matches("match overflow_options {").count(),
            decisions
        );
        assert_eq!(
            consumer
                .matches("TemporalConversionOverflowOptions::Read {")
                .count(),
            decisions
        );
        assert_eq!(
            consumer
                .matches("TemporalConversionOverflowOptions::Omit => {}")
                .count(),
            decisions
        );
        assert!(!consumer.contains("read_options"));
        assert!(!consumer.contains("matches!(overflow_options"));
        assert!(!consumer.contains("if let"));
        assert!(!consumer.contains("_ =>"));
        assert!(!consumer.contains("unreachable!"));
    }
}

#[test]
fn exactly_twenty_producers_choose_read_or_omit_without_dummy_locals() {
    let producers = [
        without_bounded(
            DATE_SOURCE,
            "    pub(super) fn emit_temporal_to_temporal_date(",
            "    pub(crate) fn emit_temporal_plain_date_from(",
        ),
        without_bounded(
            YEAR_MONTH_SOURCE,
            "    pub(super) fn emit_temporal_to_temporal_year_month(",
            "    pub(crate) fn emit_temporal_parse_year_month_string(",
        ),
        without_bounded(
            TIME_SOURCE,
            "    pub(super) fn emit_to_temporal_time(",
            "    pub(crate) fn emit_temporal_plain_time_from(",
        ),
        without_bounded(
            DATE_TIME_SOURCE,
            "    pub(super) fn emit_to_temporal_date_time(",
            "    pub(crate) fn emit_temporal_plain_date_time_from(",
        ),
        without_bounded(
            MONTH_DAY_SOURCE,
            "    pub(super) fn emit_temporal_to_temporal_month_day(",
            "    pub(crate) fn emit_temporal_plain_month_day_from(",
        ),
    ]
    .concat();

    assert_eq!(
        producers
            .matches("TemporalConversionOverflowOptions::Read {")
            .count(),
        5
    );
    assert_eq!(
        producers
            .matches("TemporalConversionOverflowOptions::Omit,")
            .count(),
        15
    );
    assert!(!producers.contains("read_options"));
    assert!(!producers.contains("undefined_payload_local"));
    assert!(!producers.contains("undefined_tag_local"));
    assert!(!producers.contains("undefined_local"));
}
