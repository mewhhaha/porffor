const OPTIONS_SOURCE: &str = include_str!("../../src/builtins/temporal_options.rs");
const DATE_SOURCE: &str = include_str!("../../src/builtins/temporal_plain_date_methods.rs");
const DATE_CONVERSION: &str =
    include_str!("../../src/builtins/temporal_plain_date_methods/convert.rs");
const DATE_TIME_SOURCE: &str =
    include_str!("../../src/builtins/temporal_plain_date_time_methods.rs");
const MONTH_DAY_SOURCE: &str = include_str!("../../src/builtins/temporal_plain_month_day.rs");
const TIME_SOURCE: &str = include_str!("../../src/builtins/temporal_plain_time_methods.rs");
const YEAR_MONTH_SOURCE: &str =
    include_str!("../../src/builtins/temporal_plain_year_month_methods.rs");
const HELPER_SOURCE: &str = include_str!("../../src/runtime_helpers.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end: {end}"))
        .0
}

fn compact(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn temporal_conversion_overflow_options_is_private_and_data_bearing() {
    let domain = bounded(
        OPTIONS_SOURCE,
        "/// Whether a `ToTemporal*` conversion owns an observable `overflow` options",
        "/// `GetTemporalOverflowOption`.",
    );
    assert!(compact(domain).contains(
        "pub(super)enumTemporalConversionOverflowOptions<'a>{Read(&'aValueLocals),Omit,}"
    ));
    for forbidden in [
        "pub enum TemporalConversionOverflowOptions",
        "pub(crate) enum TemporalConversionOverflowOptions",
        "Default",
        "PartialEq",
        "payload_local",
        "tag_local",
    ] {
        assert!(
            !domain.contains(forbidden),
            "unlawful options domain: {forbidden}"
        );
    }
    let optional = bounded(
        HELPER_SOURCE,
        "pub(crate) struct RuntimeHelperOptionalValueParameter {",
        "pub(crate) struct HelperCompletion",
    );
    assert!(!optional.contains("pub presence:"));
    assert!(!optional.contains("pub value:"));
    assert!(optional.contains("fn emit_if_present("));
    assert!(optional.contains("Some(value)"));
    assert!(optional.contains("None =>"));
    assert!(optional.contains("WasmRuntimeValueTag::Undefined"));
}

#[test]
fn all_five_converters_preserve_the_closed_observable_overflow_choice() {
    for (source, start, end) in [
        (
            YEAR_MONTH_SOURCE,
            "fn emit_temporal_to_temporal_year_month(",
            "fn emit_temporal_parse_year_month_string(",
        ),
        (
            TIME_SOURCE,
            "fn emit_to_temporal_time(",
            "fn emit_temporal_plain_time_from(",
        ),
        (
            DATE_TIME_SOURCE,
            "fn emit_to_temporal_date_time(",
            "fn emit_temporal_plain_date_time_from(",
        ),
        (
            MONTH_DAY_SOURCE,
            "fn emit_temporal_to_temporal_month_day(",
            "fn emit_temporal_plain_month_day_from(",
        ),
    ] {
        let body = bounded(source, start, end);
        assert!(body.contains("overflow_options: TemporalConversionOverflowOptions<'_>"));
        assert!(body.contains("TemporalConversionOverflowOptions::Read(options)"));
        assert!(!body.contains("read_options: bool"));
        assert!(!body.contains("options_payload_local"));
    }
    let facade = bounded(
        DATE_CONVERSION,
        "fn emit_temporal_to_temporal_date(",
        "fn compile_temporal_plain_date_convert_helper(",
    );
    let normalized = compact(facade);
    assert!(normalized.contains("TemporalConversionOverflowOptions::Read(options)=>Some(options)"));
    assert!(normalized.contains("TemporalConversionOverflowOptions::Omit=>None"));
    assert!(!facade.contains("set_undefined"));
    let kernel = DATE_CONVERSION
        .split_once("fn emit_temporal_to_temporal_date_kernel(")
        .unwrap()
        .1;
    assert!(kernel.contains("overflow_options: &RuntimeHelperOptionalValueParameter"));
    assert!(kernel.contains("overflow_options.emit_if_present("));
    assert!(!kernel.contains("TemporalConversionOverflowOptions::"));
    assert!(!kernel.contains("read_options: bool"));
}

#[test]
fn conversion_producers_choose_read_or_omit_without_dummy_locals() {
    for source in [
        DATE_SOURCE,
        YEAR_MONTH_SOURCE,
        TIME_SOURCE,
        DATE_TIME_SOURCE,
        MONTH_DAY_SOURCE,
    ] {
        assert!(source.contains("TemporalConversionOverflowOptions::Read(&options)"));
        assert!(source.contains("TemporalConversionOverflowOptions::Omit"));
        for forbidden in [
            "read_options: bool",
            "undefined_payload_local",
            "undefined_tag_local",
        ] {
            assert!(!source.contains(forbidden));
        }
    }
}

#[test]
fn plain_conversion_projects_only_normal_records_and_retains_caller_throw_routing() {
    let facade = bounded(
        DATE_CONVERSION,
        "fn emit_temporal_to_temporal_date(",
        "fn compile_temporal_plain_date_convert_helper(",
    );
    let guard = facade.find("CompletionKind::Normal").unwrap();
    let projection = facade
        .find("self.emit_temporal_plain_date_load_record(")
        .unwrap();
    let abrupt = facade.find("self.completion().copy_from(&pending").unwrap();
    assert!(guard < projection && projection < abrupt);
    assert!(facade.contains("self.emit_propagate_current_throw_if_needed(function)"));
    assert!(facade.contains("record.clear(function)"));
    assert!(facade.contains("pending.clear(function)"));
    assert!(!facade.contains("load_current_realm"));
    let compiler = bounded(
        DATE_CONVERSION,
        "fn compile_temporal_plain_date_convert_helper(",
        "fn emit_temporal_to_temporal_date_kernel(",
    );
    assert!(compiler.contains("self.emit_temporal_to_temporal_date_kernel("));
    assert!(
        compiler
            .find("self.emit_temporal_to_temporal_date_kernel(")
            .unwrap()
            < compiler
                .find("self.emit_alloc_temporal_plain_date(")
                .unwrap()
    );
    assert!(compiler.contains("TemporalPrototypeSource::Intrinsic"));
    assert!(
        compiler.find("self.completion().emit(").unwrap()
            < compiler
                .find("self.clear_helper_function_context(")
                .unwrap()
    );
    assert!(!DATE_SOURCE.contains("fn emit_temporal_to_temporal_date_kernel("));
    assert!(!DATE_CONVERSION.contains("pub(crate) fn emit_temporal_to_temporal_date_kernel("));
}
