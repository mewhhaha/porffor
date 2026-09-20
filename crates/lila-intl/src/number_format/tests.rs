use super::numeric::*;
use super::options::*;
use super::plural_rules::{CardinalCategory, PluralSelectionPurpose};
use super::*;
use core::num::{NonZeroU32, NonZeroU64};

fn profiles() -> &'static NumberProfiles {
    embedded_number_profiles().unwrap()
}
fn canonical(value: &str) -> crate::CanonicalLocaleId {
    crate::CanonicalLocaleId::from_data(value).unwrap()
}
fn value(source: &str) -> IntlMathematicalValue {
    normalize_numeric_input(
        ObservedNumericInput::StringNumericLiteral(source.encode_utf16().collect()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap()
}
fn fraction(minimum: u8, maximum: u8) -> Precision {
    Precision::Fraction(FractionPrecision::Range(
        FractionDigitRange::new(
            FractionDigitCount::new(minimum).unwrap(),
            FractionDigitCount::new(maximum).unwrap(),
        )
        .unwrap(),
    ))
}
fn significant(minimum: u8, maximum: u8) -> Precision {
    Precision::Significant(
        SignificantDigitRange::new(
            SignificantDigitCount::new(minimum).unwrap(),
            SignificantDigitCount::new(maximum).unwrap(),
        )
        .unwrap(),
    )
}
fn options() -> NumberFormatOptions {
    NumberFormatOptions {
        style: NumberStyle::Decimal,
        notation: Notation::Standard,
        minimum_integer_digits: IntegerDigitCount::new(1).unwrap(),
        precision: fraction(0, 3),
        rounding_mode: RoundingMode::HalfExpand,
        trailing_zero_display: TrailingZeroDisplay::Auto,
        grouping: Grouping::Auto,
        sign_display: SignDisplay::Auto,
    }
}
fn configuration(locale: &str, options: NumberFormatOptions) -> NumberFormatConfiguration {
    NumberFormatConfiguration {
        locale: resolve_number_locale(
            &NumberLocaleRequest {
                requested: vec![canonical(locale)].into_boxed_slice(),
                matcher: LocaleMatcher::Lookup,
                numbering_system: None,
            },
            profiles(),
        )
        .unwrap(),
        options,
    }
}
fn scalar(locale: &str, source: &str, options: NumberFormatOptions) -> ScalarNumberPartition {
    let result = partition_number(
        &configuration(locale, options),
        &value(source),
        profiles(),
        &PartitionLimits::HOST_ABI,
    )
    .unwrap();
    let joined: String = result.parts().iter().map(NumberPart::text).collect();
    assert_eq!(joined, result.to_text());
    assert!(result.parts().iter().all(|part| !part.text().is_empty()));
    result
}
fn range(
    locale: &str,
    start: &str,
    end: &str,
    options: NumberFormatOptions,
) -> RangeNumberPartition {
    let result = partition_number_range(
        &configuration(locale, options),
        &NumberRange::new(value(start), value(end)).unwrap(),
        profiles(),
        &PartitionLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(
        result
            .parts()
            .iter()
            .map(NumberRangePart::text)
            .collect::<String>(),
        result.to_text()
    );
    result
}
fn unit(name: &str, display: UnitDisplay) -> NumberFormatOptions {
    NumberFormatOptions {
        style: NumberStyle::Unit {
            identifier: UnitIdentifier::parse(name).unwrap(),
            display,
        },
        ..options()
    }
}
fn currency(code: &str, display: CurrencyDisplay, sign: CurrencySign) -> NumberFormatOptions {
    NumberFormatOptions {
        style: NumberStyle::Currency {
            code: CurrencyCode::parse(code).unwrap(),
            display,
            sign,
        },
        precision: fraction(2, 2),
        ..options()
    }
}

#[test]
fn complete_locale_inventory_and_currency_precision_share_one_profile_authority() {
    let profiles = profiles();
    assert_eq!(profiles.available_locales().len(), 1082);
    assert_eq!(profiles.numbering_systems().len(), 77);
    for locale in profiles.available_locales() {
        let selected = configuration(locale, options());
        assert_eq!(selected.locale.resolved().as_str(), *locale);
        assert_eq!(selected.locale.formatting().as_str(), *locale);
        assert!(!partition_number(
            &selected,
            &value("1234.5"),
            profiles,
            &PartitionLimits::HOST_ABI
        )
        .unwrap()
        .parts()
        .is_empty());
    }
    let fractions = profiles.currency_fractions();
    for (code, expected) in [("JPY", 0), ("USD", 2), ("KWD", 3), ("CLF", 4), ("ZZZ", 2)] {
        assert_eq!(
            fractions.digits(&CurrencyCode::parse(code).unwrap()).get(),
            expected
        );
    }
    assert_eq!(fractions.default_digits().get(), 2);
    assert!(fractions
        .overrides()
        .windows(2)
        .all(|pair| pair[0].code() < pair[1].code()));
}

#[test]
fn locale_resolution_preserves_selected_identity_and_relevant_nu_only() {
    let request = |name: Option<&str>| NumberLocaleRequest {
        requested: vec![canonical("fr-CA-u-ca-gregory-nu-arab")].into_boxed_slice(),
        matcher: LocaleMatcher::BestFit,
        numbering_system: name.map(|name| NumberingSystemOption::parse(name).unwrap()),
    };
    let selected = resolve_number_locale(&request(None), profiles()).unwrap();
    assert_eq!(selected.resolved().as_str(), "fr-CA-u-nu-arab");
    assert_eq!(selected.formatting().as_str(), "fr-CA");
    assert_eq!(selected.numbering_system().name(), "arab");
    assert_eq!(
        resolve_number_locale(&request(Some("ARAB")), profiles())
            .unwrap()
            .resolved()
            .as_str(),
        "fr-CA-u-nu-arab"
    );
    assert_eq!(
        resolve_number_locale(&request(Some("latn")), profiles())
            .unwrap()
            .resolved()
            .as_str(),
        "fr-CA"
    );
    assert_eq!(
        resolve_number_locale(&request(Some("unknown")), profiles())
            .unwrap()
            .numbering_system()
            .name(),
        "arab"
    );
    let supported = filter_number_locales(
        &NumberSupportedLocalesRequest {
            requested: vec![canonical("fr-CA-u-ca-gregory-nu-arab"), canonical("zz-ZZ")]
                .into_boxed_slice(),
            matcher: LocaleMatcher::Lookup,
        },
        profiles(),
    )
    .unwrap();
    assert_eq!(
        supported
            .iter()
            .map(|locale| locale.as_str())
            .collect::<Vec<_>>(),
        ["fr-CA-u-ca-gregory-nu-arab"]
    );
    assert_eq!(
        configuration("zz-ZZ", options()).locale.resolved().as_str(),
        "en-US"
    );
}

#[test]
fn restored_locale_proof_rejects_cross_field_forgery() {
    for (resolved, formatting, numbering) in [
        ("fr", "en", "latn"),
        ("fr-u-nu-arab", "fr", "latn"),
        ("fr-u-ca-gregory", "fr", "latn"),
        ("fr-u-nu-arab-ca-gregory", "fr", "arab"),
        ("zz", "zz", "latn"),
        ("en", "en", "roman"),
    ] {
        assert!(ResolvedNumberLocale::from_resolved(
            canonical(resolved),
            canonical(formatting),
            numbering,
            profiles()
        )
        .is_err());
    }
    for name in ["", "ab", "arab_foo", "arab-", "traditional", "aßcd"] {
        assert!(NumberingSystemOption::parse(name).is_err());
    }
    assert_eq!(
        NumberingSystemOption::parse("ARAB-FOO").unwrap().name(),
        "arab-foo"
    );
}

#[test]
fn localized_grouping_decimal_symbols_and_minimum_grouping() {
    assert_eq!(
        scalar("en", "1234567.5", options()).to_text(),
        "1,234,567.5"
    );
    assert_eq!(
        scalar("hi", "1234567.5", options()).to_text(),
        "12,34,567.5"
    );
    assert_eq!(
        scalar("fr", "1234567.5", options()).to_text(),
        "1\u{202f}234\u{202f}567,5"
    );
    assert_eq!(
        scalar("de", "1234567.5", options()).to_text(),
        "1.234.567,5"
    );
    assert_eq!(scalar("es", "1234", options()).to_text(), "1234");
    assert_eq!(
        scalar(
            "es",
            "1234",
            NumberFormatOptions {
                grouping: Grouping::Always,
                ..options()
            }
        )
        .to_text(),
        "1.234"
    );
    assert_eq!(
        scalar(
            "en",
            "1234",
            NumberFormatOptions {
                grouping: Grouping::MinTwo,
                ..options()
            }
        )
        .to_text(),
        "1234"
    );
}

#[test]
fn every_numeric_digit_alphabet_is_selected_without_mapping_measurement_text() {
    for name in profiles().numbering_systems() {
        let locale = format!("en-u-nu-{name}");
        let rendered = scalar(
            &locale,
            "1234567890",
            NumberFormatOptions {
                grouping: Grouping::Never,
                ..options()
            },
        );
        let (_, system) = profiles().system(name).unwrap();
        let expected: String = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0]
            .into_iter()
            .map(|index| system.digits[index])
            .collect();
        assert_eq!(rendered.to_text(), expected, "{name}");
    }
    assert_eq!(
        scalar("en-u-nu-hanidec", "123", options()).to_text(),
        "一二三"
    );
    assert_eq!(scalar("en-u-nu-mathbold", "12", options()).to_text(), "𝟏𝟐");
    let rendered = scalar(
        "en-u-nu-arab",
        "12",
        currency("USD", CurrencyDisplay::Code, CurrencySign::Standard),
    );
    assert!(rendered
        .parts()
        .iter()
        .any(|part| part.kind() == NumberPartKind::Currency && part.text() == "USD"));
    assert!(rendered
        .parts()
        .iter()
        .any(|part| part.kind() == NumberPartKind::Integer && part.text() == "١٢"));
}

#[test]
fn exact_strings_bigints_and_signed_zero_remain_exact_in_parts() {
    let rendered = scalar("en", "9007199254740993.125", options());
    assert_eq!(rendered.to_text(), "9,007,199,254,740,993.125");
    let mathematical = normalize_numeric_input(
        ObservedNumericInput::BigIntDecimal("999999999999999999999999999999999999".into()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(
        partition_number(
            &configuration(
                "en",
                NumberFormatOptions {
                    grouping: Grouping::Never,
                    ..options()
                }
            ),
            &mathematical,
            profiles(),
            &PartitionLimits::HOST_ABI
        )
        .unwrap()
        .to_text(),
        "999999999999999999999999999999999999"
    );
    for (display, expected) in [
        (SignDisplay::Auto, "-0"),
        (SignDisplay::Always, "-0"),
        (SignDisplay::Never, "0"),
        (SignDisplay::ExceptZero, "0"),
        (SignDisplay::Negative, "0"),
    ] {
        assert_eq!(
            scalar(
                "en",
                "-0",
                NumberFormatOptions {
                    sign_display: display,
                    ..options()
                }
            )
            .to_text(),
            expected
        );
    }
    assert_eq!(scalar("en", "-0.0001", options()).to_text(), "-0");
}

#[test]
fn nonfinite_signs_and_measurement_parts_follow_the_same_partition() {
    for (source, expected) in [("NaN", "NaN"), ("Infinity", "∞"), ("-Infinity", "-∞")] {
        assert_eq!(scalar("en", source, options()).to_text(), expected);
    }
    assert_eq!(
        scalar(
            "en",
            "NaN",
            NumberFormatOptions {
                sign_display: SignDisplay::Always,
                ..options()
            }
        )
        .to_text(),
        "+NaN"
    );
    assert_eq!(
        scalar(
            "en",
            "NaN",
            NumberFormatOptions {
                sign_display: SignDisplay::ExceptZero,
                ..options()
            }
        )
        .to_text(),
        "NaN"
    );
    assert_eq!(
        scalar("en", "Infinity", unit("meter", UnitDisplay::Long)).to_text(),
        "∞ meters"
    );
    assert_eq!(
        scalar(
            "en",
            "-Infinity",
            currency("USD", CurrencyDisplay::Symbol, CurrencySign::Accounting)
        )
        .to_text(),
        "($∞)"
    );
}

#[test]
fn percent_scaling_is_exclusive_to_percent_style() {
    assert_eq!(
        scalar(
            "en",
            "0.125",
            NumberFormatOptions {
                style: NumberStyle::Percent,
                ..options()
            }
        )
        .to_text(),
        "12.5%"
    );
    assert_eq!(
        scalar("en", "0.125", unit("percent", UnitDisplay::Short)).to_text(),
        "0.125%"
    );
}

#[test]
fn scientific_and_engineering_exponents_use_exact_digits_and_never_group_mantissas() {
    assert_eq!(
        scalar(
            "en",
            "1234567",
            NumberFormatOptions {
                notation: Notation::Scientific,
                precision: significant(1, 4),
                ..options()
            }
        )
        .to_text(),
        "1.235E6"
    );
    assert_eq!(
        scalar(
            "en",
            "1234567",
            NumberFormatOptions {
                notation: Notation::Engineering,
                precision: significant(1, 4),
                ..options()
            }
        )
        .to_text(),
        "1.235E6"
    );
    assert_eq!(
        scalar(
            "en",
            "12345",
            NumberFormatOptions {
                notation: Notation::Engineering,
                precision: significant(1, 4),
                ..options()
            }
        )
        .to_text(),
        "12.35E3"
    );
    assert_eq!(
        scalar(
            "en",
            "0.0012",
            NumberFormatOptions {
                notation: Notation::Scientific,
                ..options()
            }
        )
        .to_text(),
        "1.2E-3"
    );
    let rendered = scalar(
        "en-u-nu-mathbold",
        "0.0012",
        NumberFormatOptions {
            notation: Notation::Scientific,
            ..options()
        },
    );
    assert!(rendered
        .parts()
        .iter()
        .any(|part| part.kind() == NumberPartKind::ExponentInteger && part.text() == "𝟑"));
}

#[test]
fn compact_literal_forms_and_per_plural_normal_fallback_preserve_value() {
    let compact = || NumberFormatOptions {
        notation: Notation::Compact(CompactDisplay::Long),
        precision: significant(1, 3),
        grouping: Grouping::MinTwo,
        ..options()
    };
    assert_eq!(scalar("fr", "1000", compact()).to_text(), "mille");
    assert_eq!(scalar("it", "1000", compact()).to_text(), "mille");
    assert_eq!(scalar("fr", "2000", compact()).to_text(), "2 mille");
    assert_eq!(
        scalar(
            "vec",
            "1000",
            NumberFormatOptions {
                notation: Notation::Compact(CompactDisplay::Short),
                ..compact()
            }
        )
        .to_text(),
        "1000"
    );
    assert_eq!(
        scalar(
            "en",
            "999500",
            NumberFormatOptions {
                notation: Notation::Compact(CompactDisplay::Short),
                ..compact()
            }
        )
        .to_text(),
        "1M"
    );
}

#[test]
fn compact_pattern_operands_do_not_reuse_expanded_source_operands() {
    let table =
        CompactExponentTable::new(vec![CompactExponentRow::new(3, 3).unwrap()].into_boxed_slice())
            .unwrap();
    let source = value("1000");
    let rounded = format_decimal(
        source.finite_value().unwrap(),
        &DecimalFormatSettings::from_options(
            &NumberFormatOptions {
                notation: Notation::Compact(CompactDisplay::Long),
                precision: significant(1, 3),
                ..options()
            },
            &table,
        ),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(
        rounded
            .mantissa_plural_operands()
            .operand(PluralOperand::N)
            .compare_integer(1),
        core::cmp::Ordering::Equal
    );
    assert_eq!(
        rounded
            .mantissa_plural_operands()
            .operand(PluralOperand::E)
            .compare_integer(0),
        core::cmp::Ordering::Equal
    );
    assert_eq!(
        rounded
            .plural_operands()
            .operand(PluralOperand::N)
            .compare_integer(1000),
        core::cmp::Ordering::Equal
    );
    assert_eq!(
        rounded
            .plural_operands()
            .operand(PluralOperand::E)
            .compare_integer(3),
        core::cmp::Ordering::Equal
    );
}

#[test]
fn currency_symbols_names_accounting_and_alpha_spacing_use_pinned_rows() {
    assert_eq!(
        scalar(
            "en",
            "12",
            currency("USD", CurrencyDisplay::Symbol, CurrencySign::Standard)
        )
        .to_text(),
        "$12.00"
    );
    assert_eq!(
        scalar(
            "en",
            "-12",
            currency("USD", CurrencyDisplay::Symbol, CurrencySign::Accounting)
        )
        .to_text(),
        "($12.00)"
    );
    assert_eq!(
        scalar(
            "en",
            "12",
            currency("USD", CurrencyDisplay::Code, CurrencySign::Standard)
        )
        .to_text(),
        "USD\u{a0}12.00"
    );
    assert_eq!(
        scalar(
            "fr",
            "12",
            currency("EUR", CurrencyDisplay::Symbol, CurrencySign::Standard)
        )
        .to_text(),
        "12,00\u{a0}€"
    );
    assert_eq!(
        scalar(
            "en",
            "1",
            NumberFormatOptions {
                precision: fraction(0, 0),
                ..currency("USD", CurrencyDisplay::Name, CurrencySign::Standard)
            }
        )
        .to_text(),
        "1 US dollar"
    );
    assert_eq!(
        scalar(
            "en",
            "2",
            NumberFormatOptions {
                precision: fraction(0, 0),
                ..currency("USD", CurrencyDisplay::Name, CurrencySign::Standard)
            }
        )
        .to_text(),
        "2 US dollars"
    );
    assert_eq!(
        scalar(
            "en",
            "12",
            currency("ZZZ", CurrencyDisplay::Symbol, CurrencySign::Standard)
        )
        .to_text(),
        "ZZZ\u{a0}12.00"
    );
    let ckb = scalar(
        "ckb",
        "12",
        currency("IQD", CurrencyDisplay::Name, CurrencySign::Standard),
    );
    assert!(ckb
        .parts()
        .iter()
        .any(|part| part.kind() == NumberPartKind::Currency && part.text() == "دیناری عێراقی"));
    assert_eq!(
        configuration("ckb", options()).locale.formatting().as_str(),
        "ckb"
    );
}

#[test]
fn numbering_extensions_do_not_change_currency_name_placement_locale() {
    let rendered = scalar(
        "de-u-nu-arab",
        "12",
        currency("USD", CurrencyDisplay::Name, CurrencySign::Standard),
    );
    assert!(rendered.to_text().ends_with(" US-Dollar"));
    assert!(rendered
        .parts()
        .iter()
        .any(|part| part.kind() == NumberPartKind::Integer && part.text() == "١٢"));
}

#[test]
fn unit_forms_keep_numeric_omission_and_medial_placeholders() {
    assert_eq!(
        scalar("ar", "1", unit("meter", UnitDisplay::Long)).to_text(),
        "متر"
    );
    assert_eq!(
        scalar("ar", "2", unit("meter", UnitDisplay::Long)).to_text(),
        "2 متر"
    );
    let short_dual = scalar("ar", "2", unit("meter", UnitDisplay::Short));
    assert_eq!(short_dual.to_text(), "متران");
    assert!(!short_dual
        .parts()
        .iter()
        .any(|part| part.kind() == NumberPartKind::Integer));
    let singular_day = scalar("zu", "1", unit("day", UnitDisplay::Narrow));
    assert_eq!(singular_day.to_text(), "1");
    assert!(!singular_day
        .parts()
        .iter()
        .any(|part| part.kind() == NumberPartKind::Unit));
    assert_eq!(
        scalar("zu", "2", unit("day", UnitDisplay::Narrow)).to_text(),
        "2 suku"
    );
    assert_eq!(
        scalar("ja", "3", unit("celsius", UnitDisplay::Long)).to_text(),
        "摂氏 3 度"
    );
    assert_eq!(
        scalar("ja", "3", unit("meter-per-celsius", UnitDisplay::Long)).to_text(),
        "3 メートル毎摂氏"
    );
    assert_eq!(
        scalar("ja", "3", unit("celsius-per-meter", UnitDisplay::Long)).to_text(),
        "摂氏 3 度/メートル"
    );
    assert_eq!(
        configuration("ja", options()).locale.resolved().as_str(),
        "ja"
    );
}

#[test]
fn sanctioned_per_compositions_are_complete_for_every_width() {
    for display in [UnitDisplay::Short, UnitDisplay::Narrow, UnitDisplay::Long] {
        for numerator in SingleUnit::ALL {
            for denominator in SingleUnit::ALL {
                let options = NumberFormatOptions {
                    style: NumberStyle::Unit {
                        identifier: UnitIdentifier::Per {
                            numerator: *numerator,
                            denominator: *denominator,
                        },
                        display,
                    },
                    ..options()
                };
                assert!(!scalar("ja", "3", options).parts().is_empty());
            }
        }
    }
}

#[test]
fn range_collapse_reselects_plural_and_retains_endpoint_sources() {
    let rendered = range("en", "1", "2", unit("meter", UnitDisplay::Long));
    assert_eq!(rendered.to_text(), "1–2 meters");
    assert!(rendered
        .parts()
        .iter()
        .any(|part| part.source() == RangePartSource::Start && part.text() == "1"));
    assert!(rendered
        .parts()
        .iter()
        .any(|part| part.source() == RangePartSource::End && part.text() == "2"));
    assert!(rendered
        .parts()
        .iter()
        .any(|part| part.source() == RangePartSource::Shared && part.text() == "meters"));
    assert_eq!(
        range(
            "en",
            "1",
            "2",
            currency("USD", CurrencyDisplay::Symbol, CurrencySign::Standard)
        )
        .to_text(),
        "$1.00 – $2.00"
    );
    assert_eq!(
        range(
            "en",
            "1",
            "2",
            currency("USD", CurrencyDisplay::Code, CurrencySign::Standard)
        )
        .to_text(),
        "USD\u{a0}1.00–2.00"
    );
}

#[test]
fn range_keeps_notation_and_plain_signs_and_shares_owned_currency_spacing() {
    let compact = NumberFormatOptions {
        notation: Notation::Compact(CompactDisplay::Short),
        precision: significant(1, 3),
        ..options()
    };
    let rendered = range("en", "3000", "5000", compact);
    assert_eq!(rendered.to_text(), "3K – 5K");
    assert_eq!(
        rendered
            .parts()
            .iter()
            .filter(|part| part.kind_name() == "compact")
            .count(),
        2
    );
    let rendered = range(
        "ar-u-nu-latn",
        "3",
        "5",
        currency("USD", CurrencyDisplay::NarrowSymbol, CurrencySign::Standard),
    );
    let labels: Vec<_> = rendered
        .parts()
        .iter()
        .filter(|part| part.kind_name() == "currency")
        .collect();
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].text(), "US$");
    assert_eq!(labels[0].source(), RangePartSource::Shared);
    let rendered = range(
        "ar-u-nu-latn",
        "3",
        "5",
        currency("EUR", CurrencyDisplay::NarrowSymbol, CurrencySign::Standard),
    );
    assert!(rendered.parts().iter().any(|part| {
        part.kind_name() == "literal"
            && part.text().chars().any(|character| character == '\u{200f}')
    }));
    assert_eq!(
        rendered
            .parts()
            .iter()
            .filter(|part| part.kind_name() == "currency")
            .count(),
        1
    );
    assert!(rendered
        .parts()
        .iter()
        .filter(|part| part.kind_name() == "currency")
        .all(|part| part.source() == RangePartSource::Shared));
    assert_eq!(
        range("en", "-1", "-2", options())
            .parts()
            .iter()
            .filter(|part| part.kind_name() == "minusSign")
            .count(),
        2
    );
}

#[test]
fn approximation_uses_shared_parts_and_sign_or_accounting_position() {
    assert_eq!(
        range(
            "en",
            "1.001",
            "1.002",
            NumberFormatOptions {
                precision: fraction(0, 2),
                ..options()
            }
        )
        .to_text(),
        "~1"
    );
    assert_eq!(range("en", "-1", "-1", options()).to_text(), "~-1");
    assert_eq!(
        range(
            "en",
            "1",
            "1",
            currency("USD", CurrencyDisplay::Symbol, CurrencySign::Standard)
        )
        .to_text(),
        "~$1.00"
    );
    assert_eq!(
        range(
            "en",
            "-1",
            "-1",
            currency("USD", CurrencyDisplay::Symbol, CurrencySign::Accounting)
        )
        .to_text(),
        "~($1.00)"
    );
    let rendered = range("fr", "1", "1", options());
    assert_eq!(rendered.to_text(), "≃1");
    assert!(rendered
        .parts()
        .iter()
        .all(|part| part.source() == RangePartSource::Shared));
    assert_eq!(rendered.parts()[0].kind_name(), "approximatelySign");
}

#[test]
fn descending_infinite_and_signed_zero_ranges_keep_input_order() {
    let descending = range("en", "5", "3", options());
    assert_eq!(descending.to_text(), "5–3");
    assert_eq!(descending.parts()[0].source(), RangePartSource::Start);
    assert_eq!(
        range("en", "Infinity", "Infinity", options()).to_text(),
        "~∞"
    );
    let rendered = range("en", "-0", "0", options());
    assert!(rendered
        .parts()
        .iter()
        .any(|part| part.kind_name() == "minusSign" && part.source() == RangePartSource::Start));
    assert!(!rendered
        .parts()
        .iter()
        .any(|part| part.kind_name() == "approximatelySign"));
}

#[test]
fn partition_limits_fail_before_truncation_and_apply_to_combined_ranges() {
    let configuration = configuration("en", options());
    let bytes = PartitionLimits::new(
        NumericLimits::HOST_ABI,
        NonZeroU32::new(3).unwrap(),
        NonZeroU32::new(99).unwrap(),
    );
    assert!(matches!(
        partition_number(&configuration, &value("1234"), profiles(), &bytes),
        Err(NumberFormatKernelError::Resource(
            NumberPartitionResourceError::OutputExtent { .. }
        ))
    ));
    let parts = PartitionLimits::new(
        NumericLimits::HOST_ABI,
        NonZeroU32::new(999).unwrap(),
        NonZeroU32::new(1).unwrap(),
    );
    assert!(matches!(
        partition_number(&configuration, &value("1.2"), profiles(), &parts),
        Err(NumberFormatKernelError::Resource(
            NumberPartitionResourceError::PartExtent { .. }
        ))
    ));
    assert!(partition_number_range(
        &configuration,
        &NumberRange::new(value("12"), value("34")).unwrap(),
        profiles(),
        &bytes
    )
    .is_err());
}

#[test]
fn measurement_names_use_the_reviewed_notation_specific_exact_operand_view() {
    for (locale, single_unit, compact_unit, single_currency, compact_currency, compact_number) in [
        ("en", "meter", "meters", "US dollar", "US dollars", "1K"),
        (
            "ru",
            "метр",
            "метров",
            "доллар США",
            "долларов США",
            "1\u{a0}тыс.",
        ),
    ] {
        for notation in [Notation::Scientific, Notation::Engineering] {
            let unit_options = NumberFormatOptions {
                notation,
                precision: significant(1, 3),
                ..unit("meter", UnitDisplay::Long)
            };
            assert_eq!(
                scalar(locale, "1000", unit_options).to_text(),
                format!("1E3 {single_unit}")
            );
            let currency_options = NumberFormatOptions {
                notation,
                precision: significant(1, 3),
                ..currency("USD", CurrencyDisplay::Name, CurrencySign::Standard)
            };
            assert_eq!(
                scalar(locale, "1000", currency_options).to_text(),
                format!("1E3 {single_currency}")
            );
        }
        let unit_options = NumberFormatOptions {
            notation: Notation::Compact(CompactDisplay::Short),
            precision: significant(1, 3),
            ..unit("meter", UnitDisplay::Long)
        };
        assert_eq!(
            scalar(locale, "1000", unit_options).to_text(),
            format!("{compact_number} {compact_unit}")
        );
        let currency_options = NumberFormatOptions {
            notation: Notation::Compact(CompactDisplay::Short),
            precision: significant(1, 3),
            ..currency("USD", CurrencyDisplay::Name, CurrencySign::Standard)
        };
        assert_eq!(
            scalar(locale, "1000", currency_options).to_text(),
            format!("{compact_number} {compact_currency}")
        );
    }
}

mod plural_samples;

#[test]
fn all_pinned_cldr_cardinal_sample_endpoints_match_exact_rust_operands() {
    assert_eq!(plural_samples::SAMPLES.len(), 1719);
    for &(rule, spelling, expected) in plural_samples::SAMPLES {
        let (mantissa, exponent) = spelling
            .split_once('c')
            .map_or((spelling, 0), |(mantissa, exponent)| {
                (mantissa, exponent.parse::<u32>().unwrap())
            });
        let visible = mantissa
            .split_once('.')
            .map_or(0, |(_, fraction)| fraction.len()) as u8;
        let source = if exponent == 0 {
            value(mantissa)
        } else {
            value(&format!("{mantissa}e{exponent}"))
        };
        let exponents = CompactExponentTable::new(if exponent == 0 {
            Box::default()
        } else {
            vec![CompactExponentRow::new(exponent, exponent).unwrap()].into_boxed_slice()
        })
        .unwrap();
        let format_options = NumberFormatOptions {
            precision: fraction(visible, visible),
            notation: if exponent == 0 {
                Notation::Standard
            } else {
                Notation::Compact(CompactDisplay::Short)
            },
            grouping: Grouping::Never,
            ..options()
        };
        let rounded = format_decimal(
            source.finite_value().unwrap(),
            &DecimalFormatSettings::from_options(&format_options, &exponents),
            &NumericLimits::HOST_ABI,
        )
        .unwrap();
        let observed = profiles()
            .rules(super::profiles::PluralRulesId(rule))
            .select(rounded.plural_operands());
        assert_eq!(observed, expected, "rule {rule}: {spelling}");
    }
}

#[test]
fn exact_plural_modulus_handles_large_values_without_binary64() {
    let source = value("999999999999999999999999999999999999999999999999999999991");
    let rounded = round_decimal(
        source.finite_value().unwrap(),
        &RoundingSettings::from(&NumberFormatOptions {
            precision: fraction(0, 0),
            ..options()
        }),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(
        rounded
            .plural_operands()
            .operand(PluralOperand::I)
            .modulo(NonZeroU64::new(100).unwrap())
            .compare_integer(91),
        core::cmp::Ordering::Equal
    );
    assert_eq!(
        profiles()
            .rules(profiles().profile("ru").unwrap().plural_rules)
            .select(PluralSelectionPurpose::Measurement.operands(&rounded)),
        CardinalCategory::One
    );
}

#[path = "tests/range_policy.rs"]
mod range_policy;

#[path = "tests/range_ownership.rs"]
mod range_ownership;
