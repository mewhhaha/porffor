const DATE_SOURCE: &str = include_str!("../src/builtins/temporal_plain_date_methods.rs");
const DATE_TIME_SOURCE: &str = include_str!("../src/builtins/temporal_plain_date_time_methods.rs");
const TIME_SOURCE: &str = include_str!("../src/builtins/temporal_plain_time_methods.rs");
const YEAR_MONTH_SOURCE: &str =
    include_str!("../src/builtins/temporal_plain_year_month_methods.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

#[test]
fn temporal_plain_difference_operation_is_a_private_two_variant_domain() {
    let declaration = bounded(
        DATE_TIME_SOURCE,
        "pub(super) enum TemporalPlainDifferenceOperation {",
        "\n}\n\n/// The three compile-time consumers of the shared DateTime difference-settings",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    assert_eq!(variants, ["Until,", "Since,"]);
    assert!(!DATE_TIME_SOURCE.contains("pub enum TemporalPlainDifferenceOperation"));
    assert!(!DATE_TIME_SOURCE.contains("pub(crate) enum TemporalPlainDifferenceOperation"));
    assert!(!declaration.contains("Default"));
}

#[test]
fn all_four_plain_difference_emitters_choose_rounding_and_result_exhaustively() {
    let emitters = [
        (
            DATE_SOURCE,
            "    pub(super) fn emit_temporal_plain_date_until_or_since(",
            "    pub(crate) fn emit_temporal_plain_date_to_plain_date_time(",
        ),
        (
            YEAR_MONTH_SOURCE,
            "    pub(super) fn emit_temporal_plain_year_month_until_or_since(",
            "    pub(crate) fn emit_temporal_plain_year_month_to_locale_string(",
        ),
        (
            TIME_SOURCE,
            "    pub(super) fn emit_temporal_plain_time_until_or_since(",
            "    pub(crate) fn emit_temporal_plain_time_record_to_string(",
        ),
        (
            DATE_TIME_SOURCE,
            "    pub(super) fn emit_temporal_plain_date_time_until_or_since(",
            "    pub(crate) fn emit_temporal_plain_date_time_to_locale_string(",
        ),
    ];

    for (source, start, end) in emitters {
        let emitter = bounded(source, start, end);
        assert_eq!(
            emitter
                .matches("operation: TemporalPlainDifferenceOperation,")
                .count(),
            1,
            "typed operation missing from `{start}`"
        );
        assert_eq!(
            emitter.matches("match operation {").count(),
            2,
            "rounding and result choices must both be exhaustive in `{start}`"
        );
        assert_eq!(
            emitter
                .matches("TemporalPlainDifferenceOperation::Until =>")
                .count(),
            2
        );
        assert_eq!(
            emitter
                .matches("TemporalPlainDifferenceOperation::Since =>")
                .count(),
            2
        );
        assert!(!emitter.contains("since: bool"));
        assert!(!emitter.contains("if since"));
        assert!(!emitter.contains("negates_result"));
        assert!(!emitter.contains("matches!(operation"));
        assert!(!emitter.contains("_ =>"));
        assert!(!emitter.contains("unreachable!"));
    }
}

#[test]
fn exactly_eight_standard_producers_name_their_plain_difference_operation() {
    assert_eq!(
        STANDARD_SOURCE
            .matches("TemporalPlainDifferenceOperation::Until,")
            .count(),
        4
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches("TemporalPlainDifferenceOperation::Since,")
            .count(),
        4
    );
    for emitter in [
        "emit_temporal_plain_date_until_or_since(",
        "emit_temporal_plain_year_month_until_or_since(",
        "emit_temporal_plain_time_until_or_since(",
        "emit_temporal_plain_date_time_until_or_since(",
    ] {
        assert_eq!(STANDARD_SOURCE.matches(emitter).count(), 2, "`{emitter}`");
        assert!(!STANDARD_SOURCE.contains(&format!("{emitter}false")));
        assert!(!STANDARD_SOURCE.contains(&format!("{emitter}true")));
    }
    assert!(!STANDARD_SOURCE.contains("PlainDateTimeDifference"));
}
