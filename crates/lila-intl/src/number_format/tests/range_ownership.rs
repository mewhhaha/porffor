use super::*;

#[test]
fn signed_currency_prefix_and_suffix_share_one_pattern_owner() {
    let options = NumberFormatOptions {
        sign_display: SignDisplay::Always,
        ..currency("EUR", CurrencyDisplay::Symbol, CurrencySign::Standard)
    };
    let rendered = range("pt-PT", "2.9", "3.1", options);
    assert_eq!(rendered.to_text(), "+2,90 - 3,10\u{a0}€");
    let actual: Vec<_> = rendered
        .parts()
        .iter()
        .map(|part| (part.kind_name(), part.text(), part.source()))
        .collect();
    assert_eq!(
        actual,
        [
            ("plusSign", "+", RangePartSource::Shared),
            ("integer", "2", RangePartSource::Start),
            ("decimal", ",", RangePartSource::Start),
            ("fraction", "90", RangePartSource::Start),
            ("literal", " - ", RangePartSource::Shared),
            ("integer", "3", RangePartSource::End),
            ("decimal", ",", RangePartSource::End),
            ("fraction", "10", RangePartSource::End),
            ("literal", "\u{a0}", RangePartSource::Shared),
            ("currency", "€", RangePartSource::Shared),
        ]
    );
}

#[test]
fn opposite_signs_keep_the_complete_currency_pattern_on_each_endpoint() {
    let options = NumberFormatOptions {
        sign_display: SignDisplay::Always,
        ..currency("USD", CurrencyDisplay::Symbol, CurrencySign::Standard)
    };
    let rendered = range("en-US", "1", "-2", options);
    assert_eq!(rendered.to_text(), "+$1.00 – -$2.00");
    let actual: Vec<_> = rendered
        .parts()
        .iter()
        .filter(|part| matches!(part.kind_name(), "plusSign" | "minusSign" | "currency"))
        .map(|part| (part.kind_name(), part.text(), part.source()))
        .collect();
    assert_eq!(
        actual,
        [
            ("plusSign", "+", RangePartSource::Start),
            ("currency", "$", RangePartSource::Start),
            ("minusSign", "-", RangePartSource::End),
            ("currency", "$", RangePartSource::End),
        ]
    );
}

#[test]
fn accounting_brackets_stay_paired_with_their_currency() {
    let options = currency("USD", CurrencyDisplay::Symbol, CurrencySign::Accounting);
    let shared = range("en", "-1", "-2", options.clone());
    assert_eq!(shared.to_text(), "($1.00–2.00)");
    assert_eq!(
        shared.parts().first().unwrap().source(),
        RangePartSource::Shared
    );
    assert_eq!(
        shared.parts().last().unwrap().source(),
        RangePartSource::Shared
    );
    let mixed = range("en", "-1", "2", options);
    assert_eq!(mixed.to_text(), "($1.00) – $2.00");
    assert!(mixed
        .parts()
        .iter()
        .filter(|part| part.kind_name() == "currency")
        .all(|part| part.source() != RangePartSource::Shared));
}

#[test]
fn percent_patterns_use_owned_signs_without_collapsing_plain_signs() {
    let options = NumberFormatOptions {
        style: NumberStyle::Percent,
        precision: fraction(0, 0),
        ..options()
    };
    assert_eq!(
        range("en", "0.01", "0.02", options.clone()).to_text(),
        "1% – 2%"
    );
    let signed = NumberFormatOptions {
        sign_display: SignDisplay::Always,
        ..options
    };
    let rendered = range("en", "0.01", "0.02", signed);
    assert_eq!(rendered.to_text(), "+1–2%");
    assert!(rendered
        .parts()
        .iter()
        .filter(|part| matches!(part.kind_name(), "plusSign" | "percentSign"))
        .all(|part| part.source() == RangePartSource::Shared));
    let plain = range("en", "-1", "-2", super::options());
    assert_eq!(plain.to_text(), "-1 – -2");
    assert_eq!(
        plain
            .parts()
            .iter()
            .filter(|part| part.kind_name() == "minusSign")
            .count(),
        2
    );
}

#[test]
fn short_measurement_names_share_but_their_signs_and_notation_do_not() {
    let narrow = unit("meter", UnitDisplay::Narrow);
    assert_eq!(range("en", "3", "5", narrow.clone()).to_text(), "3–5m");
    assert_eq!(
        range("en", "-3", "-5", narrow.clone()).to_text(),
        "-3 – -5m"
    );
    let compact = NumberFormatOptions {
        notation: Notation::Compact(CompactDisplay::Short),
        precision: significant(1, 3),
        ..narrow
    };
    let rendered = range("en", "3000", "5000", compact);
    assert_eq!(rendered.to_text(), "3K – 5Km");
    assert_eq!(
        rendered
            .parts()
            .iter()
            .filter(|part| part.kind_name() == "compact")
            .count(),
        2
    );
    assert_eq!(
        rendered
            .parts()
            .iter()
            .filter(|part| part.kind_name() == "unit")
            .count(),
        1
    );
    assert!(rendered
        .parts()
        .iter()
        .filter(|part| part.kind_name() == "unit")
        .all(|part| part.source() == RangePartSource::Shared));
}

#[test]
fn incidental_bidi_marks_do_not_make_a_single_currency_symbol_collapsible() {
    // CLDR47 fa/arabext is LRM + currency + number, with no owned space.
    let rendered = range(
        "fa-u-nu-arabext",
        "3",
        "5",
        currency("USD", CurrencyDisplay::Symbol, CurrencySign::Standard),
    );
    let symbols: Vec<_> = rendered
        .parts()
        .iter()
        .filter(|part| part.kind_name() == "currency")
        .collect();
    assert_eq!(symbols.len(), 2);
    assert_eq!(
        (symbols[0].text(), symbols[0].source()),
        ("$", RangePartSource::Start)
    );
    assert_eq!(
        (symbols[1].text(), symbols[1].source()),
        ("$", RangePartSource::End)
    );
    let directions: Vec<_> = rendered
        .parts()
        .iter()
        .filter(|part| part.kind_name() == "literal" && part.text().contains('\u{200e}'))
        .collect();
    assert_eq!(directions.len(), 2);
    assert_eq!(directions[0].source(), RangePartSource::Start);
    assert_eq!(directions[1].source(), RangePartSource::End);
}

#[test]
fn scientific_exponents_remain_endpoint_specific() {
    let options = NumberFormatOptions {
        notation: Notation::Scientific,
        precision: significant(1, 3),
        ..options()
    };
    let rendered = range("en", "3000", "5000", options);
    assert_eq!(rendered.to_text(), "3E3–5E3");
    let exponents: Vec<_> = rendered
        .parts()
        .iter()
        .filter(|part| part.kind_name() == "exponentInteger")
        .collect();
    assert_eq!(exponents.len(), 2);
    assert_eq!(exponents[0].source(), RangePartSource::Start);
    assert_eq!(exponents[1].source(), RangePartSource::End);
}

#[test]
fn merged_separator_obeys_final_part_and_byte_limits() {
    let configuration = configuration(
        "en-US",
        NumberFormatOptions {
            precision: fraction(0, 0),
            ..currency("USD", CurrencyDisplay::Symbol, CurrencySign::Standard)
        },
    );
    let range = NumberRange::new(value("3"), value("5")).unwrap();
    let enough = PartitionLimits::new(
        NumericLimits::HOST_ABI,
        NonZeroU32::new(9).unwrap(),
        NonZeroU32::new(5).unwrap(),
    );
    let rendered = partition_number_range(&configuration, &range, profiles(), &enough).unwrap();
    assert_eq!(rendered.to_text(), "$3 – $5");
    assert_eq!(rendered.parts().len(), 5);
    assert_eq!(rendered.to_text().len(), 9);
    let too_few_parts = PartitionLimits::new(
        NumericLimits::HOST_ABI,
        NonZeroU32::new(9).unwrap(),
        NonZeroU32::new(4).unwrap(),
    );
    assert!(matches!(
        partition_number_range(&configuration, &range, profiles(), &too_few_parts),
        Err(NumberFormatKernelError::Resource(
            NumberPartitionResourceError::PartExtent { .. }
        ))
    ));
    let too_few_bytes = PartitionLimits::new(
        NumericLimits::HOST_ABI,
        NonZeroU32::new(8).unwrap(),
        NonZeroU32::new(5).unwrap(),
    );
    assert!(matches!(
        partition_number_range(&configuration, &range, profiles(), &too_few_bytes),
        Err(NumberFormatKernelError::Resource(
            NumberPartitionResourceError::OutputExtent { .. }
        ))
    ));
}

#[test]
fn literal_only_unit_forms_remain_complete_endpoint_values() {
    let rendered = range("ar-u-nu-latn", "2", "3", unit("meter", UnitDisplay::Short));
    let start: String = rendered
        .parts()
        .iter()
        .filter(|part| part.source() == RangePartSource::Start)
        .map(NumberRangePart::text)
        .collect();
    assert_eq!(start, "متران");
    assert!(rendered
        .parts()
        .iter()
        .any(|part| part.kind_name() == "integer"
            && part.text() == "3"
            && part.source() == RangePartSource::End));
    assert!(!rendered
        .parts()
        .iter()
        .any(|part| part.kind_name() == "unit" && part.source() == RangePartSource::Shared));
}

#[test]
fn medial_measurement_patterns_share_both_affixes_with_numeric_sources_intact() {
    let rendered = range("ja", "3", "5", unit("celsius", UnitDisplay::Long));
    assert_eq!(rendered.to_text(), "摂氏 3～5 度");
    let units: Vec<_> = rendered
        .parts()
        .iter()
        .filter(|part| part.kind_name() == "unit")
        .map(|part| (part.text(), part.source()))
        .collect();
    assert_eq!(
        units,
        [
            ("摂氏", RangePartSource::Shared),
            ("度", RangePartSource::Shared)
        ]
    );
    let numbers: Vec<_> = rendered
        .parts()
        .iter()
        .filter(|part| part.kind_name() == "integer")
        .map(|part| (part.text(), part.source()))
        .collect();
    assert_eq!(
        numbers,
        [("3", RangePartSource::Start), ("5", RangePartSource::End)]
    );
}
