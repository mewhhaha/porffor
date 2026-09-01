const DTF_SOURCE: &str = include_str!("../src/builtins/intl_datetimeformat.rs");

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
fn range_method_identity_is_an_exhaustive_projection_of_format_mode() {
    let signature = bounded(
        DTF_SOURCE,
        "fn emit_intl_dtf_format_range(",
        ") -> Result<(), EmitError> {",
    );
    assert!(signature.contains("mode: DtfFormatMode,"));
    assert!(!signature.contains("method: &str"));

    let helper = bounded(
        DTF_SOURCE,
        "fn emit_intl_dtf_format_range(",
        "pub(crate) fn emit_intl_date_time_format_format_range(",
    );
    let receiver_operation = normalized(bounded(
        helper,
        "let receiver_operation = match mode {",
        "        };",
    ));
    for mapping in [
        "DtfFormatMode::String=>IntlDateTimeFormatReceiverOperation::FormatRange",
        "DtfFormatMode::Parts=>IntlDateTimeFormatReceiverOperation::FormatRangeToParts",
    ] {
        assert_eq!(
            receiver_operation.matches(mapping).count(),
            1,
            "mapping `{mapping}`"
        );
    }
    assert_eq!(receiver_operation.matches("=>").count(), 2);
    assert!(!receiver_operation.contains("_=>"));
    assert!(!receiver_operation.contains("unreachable!"));
    assert_eq!(
        helper
            .matches(
                "emit_intl_dtf_record_from_receiver(record_local, &receiver_operation, function)",
            )
            .count(),
        1
    );
}

#[test]
fn range_wrappers_pass_only_their_output_mode() {
    assert_eq!(
        DTF_SOURCE
            .matches("self.emit_intl_dtf_format_range(")
            .count(),
        2
    );

    let string_wrapper = normalized(bounded(
        DTF_SOURCE,
        "pub(crate) fn emit_intl_date_time_format_format_range(",
        "pub(crate) fn emit_intl_date_time_format_format_range_to_parts(",
    ));
    assert_eq!(
        string_wrapper
            .matches("self.emit_intl_dtf_format_range(DtfFormatMode::String,function)")
            .count(),
        1
    );
    assert!(!string_wrapper.contains("DtfFormatMode::Parts"));
    assert!(!string_wrapper.contains("\"Intl.DateTimeFormat.prototype.formatRange\""));

    let parts_wrapper = normalized(bounded(
        DTF_SOURCE,
        "pub(crate) fn emit_intl_date_time_format_format_range_to_parts(",
        "pub(crate) fn emit_intl_dtf_temporal_to_locale_string(",
    ));
    assert_eq!(
        parts_wrapper
            .matches("self.emit_intl_dtf_format_range(DtfFormatMode::Parts,function)")
            .count(),
        1
    );
    assert!(!parts_wrapper.contains("DtfFormatMode::String"));
    assert!(!parts_wrapper.contains("\"Intl.DateTimeFormat.prototype.formatRangeToParts\""));
}

#[test]
fn range_brand_check_precedes_arguments_and_mode_selects_the_result_tag() {
    let helper = bounded(
        DTF_SOURCE,
        "fn emit_intl_dtf_format_range(",
        "pub(crate) fn emit_intl_date_time_format_format_range(",
    );
    let normalized_helper = normalized(helper);
    let brand = normalized_helper
        .find(
            "self.emit_intl_dtf_record_from_receiver(record_local,&receiver_operation,function)?;",
        )
        .expect("missing range receiver brand check");
    let arguments = normalized_helper
        .find("self.emit_intl_dtf_range_argument_values(")
        .expect("missing range argument conversion");
    assert!(brand < arguments);

    let result_tag = normalized(bounded(
        helper,
        "let result_tag = match mode {",
        "        };",
    ));
    for mapping in [
        "DtfFormatMode::String=>ValueKind::String.tag()asi64",
        "DtfFormatMode::Parts=>ValueKind::Array.tag()asi64",
    ] {
        assert_eq!(
            result_tag.matches(mapping).count(),
            1,
            "result mapping `{mapping}`"
        );
    }
    assert_eq!(result_tag.matches("=>").count(), 2);
    assert!(!result_tag.contains("_=>"));
    assert!(!result_tag.contains("unreachable!"));
}
