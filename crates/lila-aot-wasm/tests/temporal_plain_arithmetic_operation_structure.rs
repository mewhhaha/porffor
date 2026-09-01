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
fn temporal_plain_arithmetic_operation_is_a_private_two_variant_domain() {
    let declaration = bounded(
        DATE_TIME_SOURCE,
        "pub(super) enum TemporalPlainArithmeticOperation {",
        "\n}\n\n/// Which `until` or `since` operation a plain Temporal builtin emits.",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    assert_eq!(variants, ["Add,", "Subtract,"]);
    assert!(!DATE_TIME_SOURCE.contains("pub enum TemporalPlainArithmeticOperation"));
    assert!(!DATE_TIME_SOURCE.contains("pub(crate) enum TemporalPlainArithmeticOperation"));
    assert!(!declaration.contains("Default"));
}

#[test]
fn all_four_plain_arithmetic_emitters_consume_the_operation_exhaustively() {
    let emitters = [
        (
            DATE_SOURCE,
            "    pub(super) fn emit_temporal_plain_date_add_or_subtract(",
            "    pub(super) fn emit_temporal_plain_date_until_or_since(",
        ),
        (
            YEAR_MONTH_SOURCE,
            "    pub(super) fn emit_temporal_plain_year_month_add_or_subtract(",
            "    pub(super) fn emit_temporal_plain_year_month_until_or_since(",
        ),
        (
            TIME_SOURCE,
            "    pub(super) fn emit_temporal_plain_time_add_or_subtract(",
            "    pub(crate) fn emit_temporal_plain_time_validate_increment(",
        ),
        (
            DATE_TIME_SOURCE,
            "    pub(super) fn emit_temporal_plain_date_time_add_or_subtract(",
            "    pub(crate) fn emit_temporal_plain_date_time_round(",
        ),
    ];

    for (source, start, end) in emitters {
        let emitter = bounded(source, start, end);
        assert_eq!(
            emitter
                .matches("operation: TemporalPlainArithmeticOperation,")
                .count(),
            1,
            "typed operation missing from `{start}`"
        );
        assert_eq!(emitter.matches("match operation {").count(), 1);
        assert_eq!(
            emitter
                .matches("TemporalPlainArithmeticOperation::Add =>")
                .count(),
            1
        );
        assert_eq!(
            emitter
                .matches("TemporalPlainArithmeticOperation::Subtract =>")
                .count(),
            1
        );
        assert!(!emitter.contains("subtract: bool"));
        assert!(!emitter.contains("if subtract"));
        assert!(!emitter.contains("matches!(operation"));
        assert!(!emitter.contains("_ =>"));
        assert!(!emitter.contains("unreachable!"));
    }
}

#[test]
fn exactly_eight_standard_producers_name_their_plain_arithmetic_operation() {
    assert_eq!(
        STANDARD_SOURCE
            .matches("TemporalPlainArithmeticOperation::Add,")
            .count(),
        4
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches("TemporalPlainArithmeticOperation::Subtract,")
            .count(),
        4
    );
    for emitter in [
        "emit_temporal_plain_date_add_or_subtract(",
        "emit_temporal_plain_year_month_add_or_subtract(",
        "emit_temporal_plain_time_add_or_subtract(",
        "emit_temporal_plain_date_time_add_or_subtract(",
    ] {
        assert_eq!(STANDARD_SOURCE.matches(emitter).count(), 2, "`{emitter}`");
        assert!(!STANDARD_SOURCE.contains(&format!("{emitter}false")));
        assert!(!STANDARD_SOURCE.contains(&format!("{emitter}true")));
    }
}
