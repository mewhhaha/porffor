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
fn receiver_operation_is_the_exact_non_copy_five_row_domain() {
    let declaration_offset = DTF_SOURCE
        .find("enum IntlDateTimeFormatReceiverOperation {")
        .expect("receiver operation declaration");
    assert_eq!(
        DTF_SOURCE[..declaration_offset]
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(str::trim),
        Some("}")
    );

    let declaration = bounded(
        DTF_SOURCE,
        "enum IntlDateTimeFormatReceiverOperation {",
        "impl IntlDateTimeFormatReceiverOperation {",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        [
            "ResolvedOptions,",
            "FormatGetter,",
            "FormatToParts,",
            "FormatRange,",
            "FormatRangeToParts,",
            "}",
        ]
    );
    assert!(!declaration.contains("derive("));
    for forbidden in [
        "impl Clone for IntlDateTimeFormatReceiverOperation",
        "impl Copy for IntlDateTimeFormatReceiverOperation",
        "impl Debug for IntlDateTimeFormatReceiverOperation",
        "impl Default for IntlDateTimeFormatReceiverOperation",
        "impl PartialEq for IntlDateTimeFormatReceiverOperation",
        "impl Eq for IntlDateTimeFormatReceiverOperation",
    ] {
        assert!(!DTF_SOURCE.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn receiver_operation_all_and_messages_preserve_the_existing_pool_order() {
    let implementation_source = bounded(
        DTF_SOURCE,
        "impl IntlDateTimeFormatReceiverOperation {",
        "/// Whether the parts this walk produces carry a `source` property",
    );
    let implementation = normalized(implementation_source);
    assert!(implementation.contains(concat!(
        "constALL:[Self;5]=[",
        "Self::ResolvedOptions,",
        "Self::FormatGetter,",
        "Self::FormatToParts,",
        "Self::FormatRange,",
        "Self::FormatRangeToParts,",
        "];",
    )));
    assert!(implementation.contains("constfnfull_message(&self)->&'staticstr{"));

    let projection = normalized(bounded(
        implementation_source,
        "match self {",
        "        }\n    }",
    ));
    for mapping in [
        "Self::ResolvedOptions=>{\"Intl.DateTimeFormat.prototype.resolvedOptionscalledonanon-Intl.DateTimeFormatobject\"}",
        "Self::FormatGetter=>{\"getIntl.DateTimeFormat.prototype.formatcalledonanon-Intl.DateTimeFormatobject\"}",
        "Self::FormatToParts=>{\"Intl.DateTimeFormat.prototype.formatToPartscalledonanon-Intl.DateTimeFormatobject\"}",
        "Self::FormatRange=>{\"Intl.DateTimeFormat.prototype.formatRangecalledonanon-Intl.DateTimeFormatobject\"}",
        "Self::FormatRangeToParts=>{\"Intl.DateTimeFormat.prototype.formatRangeToPartscalledonanon-Intl.DateTimeFormatobject\"}",
    ] {
        assert_eq!(projection.matches(mapping).count(), 1, "mapping `{mapping}`");
    }
    assert_eq!(projection.matches("=>").count(), 5);
    assert!(!projection.contains("_=>"));
    assert!(!projection.contains("unreachable!"));
    assert!(!projection.contains("default"));

    let pool = normalized(
        DTF_SOURCE
            .split_once("pub(crate) fn intl_date_time_format_pool_strings()")
            .expect("DateTimeFormat pool owner")
            .1,
    );
    assert_eq!(
        pool.matches("foroperationinIntlDateTimeFormatReceiverOperation::ALL{")
            .count(),
        1
    );
    assert_eq!(
        pool.matches("values.push(operation.full_message().to_string());")
            .count(),
        1
    );
    assert!(!pool.contains("formethodin["));
    assert!(!pool.contains("calledonanon-Intl.DateTimeFormatobject"));
}

#[test]
fn receiver_reader_accepts_only_the_named_operation_and_uses_its_full_message() {
    let signature = bounded(
        DTF_SOURCE,
        "fn emit_intl_dtf_record_from_receiver(",
        ") -> Result<(), EmitError> {",
    );
    assert!(signature.contains("operation: &IntlDateTimeFormatReceiverOperation,"));
    assert!(!signature.contains("method: &str"));

    let reader = bounded(
        DTF_SOURCE,
        "fn emit_intl_dtf_record_from_receiver(",
        "fn emit_intl_dtf_string_option(",
    );
    assert_eq!(
        reader
            .matches("let message = operation.full_message();")
            .count(),
        1
    );
    assert_eq!(
        reader
            .matches("emit_throw_current_function_realm_type_error(\n            message,")
            .count(),
        1
    );
    assert!(!reader.contains("format!("));
    assert!(!reader.contains("called on a non-Intl.DateTimeFormat object"));
}

#[test]
fn exactly_three_direct_producers_and_the_range_mode_mapping_cover_all_operations() {
    assert_eq!(
        DTF_SOURCE
            .matches("emit_intl_dtf_record_from_receiver(")
            .count(),
        5
    );

    let direct_producers = [
        (
            "pub(crate) fn emit_intl_date_time_format_resolved_options(",
            "pub(crate) fn emit_intl_date_time_format_format_getter(",
            "IntlDateTimeFormatReceiverOperation::ResolvedOptions",
        ),
        (
            "pub(crate) fn emit_intl_date_time_format_format_getter(",
            "pub(crate) fn emit_intl_date_time_format_bound_format(",
            "IntlDateTimeFormatReceiverOperation::FormatGetter",
        ),
        (
            "pub(crate) fn emit_intl_date_time_format_format_to_parts(",
            "fn emit_intl_dtf_range_argument_values(",
            "IntlDateTimeFormatReceiverOperation::FormatToParts",
        ),
    ];
    for (start, end, operation) in direct_producers {
        let producer = bounded(DTF_SOURCE, start, end);
        assert_eq!(
            producer.matches(operation).count(),
            1,
            "producer `{operation}`"
        );
        assert_eq!(
            producer
                .matches("emit_intl_dtf_record_from_receiver(")
                .count(),
            1,
            "producer `{operation}` reader call"
        );
    }

    let range = normalized(bounded(
        DTF_SOURCE,
        "fn emit_intl_dtf_format_range(",
        "pub(crate) fn emit_intl_date_time_format_format_range(",
    ));
    for mapping in [
        "DtfFormatMode::String=>IntlDateTimeFormatReceiverOperation::FormatRange",
        "DtfFormatMode::Parts=>IntlDateTimeFormatReceiverOperation::FormatRangeToParts",
    ] {
        assert_eq!(
            range.matches(mapping).count(),
            1,
            "range mapping `{mapping}`"
        );
    }
    assert_eq!(range.matches("letreceiver_operation=matchmode{").count(), 1);
    assert_eq!(
        range
            .matches(
                "emit_intl_dtf_record_from_receiver(record_local,&receiver_operation,function)"
            )
            .count(),
        1
    );
    assert!(!range.contains("_=>"));
    assert!(!range.contains("unreachable!"));
}
