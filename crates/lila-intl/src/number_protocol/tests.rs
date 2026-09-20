use super::*;

fn profiles() -> &'static NumberProfiles {
    embedded_number_profiles().unwrap()
}
fn locale_request() -> NumberLocaleRequest {
    NumberLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data("en-US-u-nu-arab").unwrap()]
            .into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
        numbering_system: Some(NumberingSystemOption::parse("latn").unwrap()),
    }
}
fn configuration() -> NumberFormatConfiguration {
    NumberFormatConfiguration {
        locale: resolve_number_locale(&locale_request(), profiles()).unwrap(),
        options: NumberFormatOptions {
            style: NumberStyle::Decimal,
            notation: Notation::Standard,
            minimum_integer_digits: IntegerDigitCount::new(1).unwrap(),
            precision: Precision::Fraction(FractionPrecision::Range(
                FractionDigitRange::new(
                    FractionDigitCount::new(0).unwrap(),
                    FractionDigitCount::new(3).unwrap(),
                )
                .unwrap(),
            )),
            rounding_mode: RoundingMode::HalfExpand,
            trailing_zero_display: TrailingZeroDisplay::Auto,
            grouping: Grouping::Auto,
            sign_display: SignDisplay::Auto,
        },
    }
}
fn set_word(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
fn configuration_offset(bytes: &[u8]) -> usize {
    let mut offset = NUMBER_WIRE_HEADER_BYTES as usize;
    for _ in 0..3 {
        let length = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap()) as usize;
        offset += 8 + length;
    }
    offset
}

#[test]
fn locale_messages_retain_canonical_public_identity_and_provider_proof() {
    let request = locale_request();
    assert_eq!(
        NumberLocaleRequest::decode(&request.encode().unwrap()).unwrap(),
        request
    );
    let locale = resolve_number_locale(&request, profiles()).unwrap();
    assert_eq!(locale.resolved().as_str(), "en-US");
    assert_eq!(locale.numbering_system().name(), "latn");
    assert_eq!(
        ResolvedNumberLocale::decode(&locale.encode().unwrap(), profiles()).unwrap(),
        locale
    );
    let request = NumberSupportedLocalesRequest {
        requested: vec![
            CanonicalLocaleId::from_data("en-US-u-nu-arab").unwrap(),
            CanonicalLocaleId::from_data("zxx").unwrap(),
        ]
        .into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
    };
    assert_eq!(
        NumberSupportedLocalesRequest::decode(&request.encode().unwrap()).unwrap(),
        request
    );
    let response = NumberSupportedLocalesResult {
        locales: filter_number_locales(&request, profiles()).unwrap(),
    };
    assert_eq!(response.locales.len(), 1);
    assert_eq!(response.locales[0].as_str(), "en-US-u-nu-arab");
    assert_eq!(
        NumberSupportedLocalesResult::decode(&response.encode().unwrap()).unwrap(),
        response
    );
    let mut wire = NumberWireWriter::new(
        IntlHostOp::ResolveNumberLocale,
        NumberWireDirection::Response,
    )
    .unwrap();
    for text in ["en-US", "fr", "latn"] {
        wire.text(text).unwrap();
    }
    assert!(ResolvedNumberLocale::decode(&wire.finish(), profiles()).is_err());
}

#[test]
fn numeric_transport_preserves_utf16_negative_zero_and_exact_intrinsic_text() {
    let inputs = [
        ObservedNumericInput::StringNumericLiteral(
            vec![0xd800, 0x0031, 0xdc00, 0x0000].into_boxed_slice(),
        ),
        ObservedNumericInput::StringNumericLiteral(
            "100000000000000001.25".encode_utf16().collect(),
        ),
        ObservedNumericInput::NumberShortestDecimal("1e+100".into()),
        ObservedNumericInput::NumberShortestDecimal("NaN".into()),
        ObservedNumericInput::NegativeZero,
        ObservedNumericInput::BigIntDecimal("-123456789012345678901234567890".into()),
    ];
    for input in inputs {
        let request = NumberFormatRequest {
            configuration: configuration(),
            input,
        };
        let bytes = request.encode().unwrap();
        assert_eq!(
            NumberFormatRequest::decode(&bytes, profiles()).unwrap(),
            request
        );
        for cut in [0, 7, 15, bytes.len() - 1] {
            assert!(NumberFormatRequest::decode(&bytes[..cut], profiles()).is_err());
        }
        let mut extra = bytes.clone();
        extra.push(0);
        assert!(NumberFormatRequest::decode(&extra, profiles()).is_err());
    }
    let range = NumberRangeFormatRequest {
        configuration: configuration(),
        start: ObservedNumericInput::NegativeZero,
        end: ObservedNumericInput::NumberShortestDecimal("0".into()),
    };
    assert_eq!(
        NumberRangeFormatRequest::decode(&range.encode().unwrap(), profiles()).unwrap(),
        range
    );
}

#[test]
fn every_configuration_word_and_active_domain_round_trips() {
    let fraction = FractionDigitRange::new(
        FractionDigitCount::new(2).unwrap(),
        FractionDigitCount::new(4).unwrap(),
    )
    .unwrap();
    let significant = SignificantDigitRange::new(
        SignificantDigitCount::new(2).unwrap(),
        SignificantDigitCount::new(6).unwrap(),
    )
    .unwrap();
    let styles = [
        NumberStyle::Decimal,
        NumberStyle::Percent,
        NumberStyle::Currency {
            code: CurrencyCode::parse("USD").unwrap(),
            display: CurrencyDisplay::Name,
            sign: CurrencySign::Accounting,
        },
        NumberStyle::Unit {
            identifier: UnitIdentifier::parse("meter-per-second").unwrap(),
            display: UnitDisplay::Long,
        },
    ];
    let precisions = [
        Precision::Fraction(FractionPrecision::Range(fraction)),
        Precision::Fraction(FractionPrecision::Increment {
            digits: FractionDigitCount::new(2).unwrap(),
            increment: NonUnitRoundingIncrement::TwentyFive,
        }),
        Precision::Significant(significant),
        Precision::More {
            fraction,
            significant,
        },
        Precision::Less {
            fraction,
            significant,
        },
    ];
    for style in styles {
        for precision in precisions {
            for notation in [
                Notation::Standard,
                Notation::Scientific,
                Notation::Engineering,
                Notation::Compact(CompactDisplay::Long),
            ] {
                let mut configuration = configuration();
                configuration.options.style = style.clone();
                configuration.options.precision = precision;
                configuration.options.notation = notation;
                configuration.options.rounding_mode = RoundingMode::HalfEven;
                configuration.options.trailing_zero_display = TrailingZeroDisplay::StripIfInteger;
                configuration.options.sign_display = SignDisplay::Negative;
                configuration.options.grouping = Grouping::MinTwo;
                let request = NumberFormatRequest {
                    configuration,
                    input: ObservedNumericInput::NegativeZero,
                };
                assert_eq!(
                    NumberFormatRequest::decode(&request.encode().unwrap(), profiles()).unwrap(),
                    request
                );
            }
        }
    }
}

#[test]
fn malformed_closed_words_inactive_fields_and_numeric_spans_are_rejected() {
    let request = NumberFormatRequest {
        configuration: configuration(),
        input: ObservedNumericInput::NegativeZero,
    };
    let wire = request.encode().unwrap();
    let offset = configuration_offset(&wire);
    for word in NumberConfigurationWord::ALL {
        let mut corrupted = wire.clone();
        set_word(&mut corrupted, offset + word.index() * 8, u64::MAX);
        assert!(
            NumberFormatRequest::decode(&corrupted, profiles()).is_err(),
            "{word:?}"
        );
    }
    for word in [
        NumberConfigurationWord::CurrencyDisplay,
        NumberConfigurationWord::CurrencySign,
        NumberConfigurationWord::UnitDisplay,
        NumberConfigurationWord::CompactDisplay,
        NumberConfigurationWord::MinimumSignificant,
        NumberConfigurationWord::MaximumSignificant,
    ] {
        let mut corrupted = wire.clone();
        set_word(&mut corrupted, offset + word.index() * 8, 1);
        assert!(
            NumberFormatRequest::decode(&corrupted, profiles()).is_err(),
            "{word:?}"
        );
    }
    let numeric = offset + NUMBER_CONFIGURATION_WORDS * 8 + 8; // Empty style text.
    let mut payload_zero = wire.clone();
    set_word(&mut payload_zero, numeric + 8, 1);
    payload_zero.push(0);
    assert!(NumberFormatRequest::decode(&payload_zero, profiles()).is_err());
    let mut odd_utf16 = payload_zero;
    set_word(
        &mut odd_utf16,
        numeric,
        NumberNumericKind::String.wire_code(),
    );
    assert!(NumberFormatRequest::decode(&odd_utf16, profiles()).is_err());
    let mut wrong_operation = wire.clone();
    set_word(
        &mut wrong_operation,
        8,
        u64::from(IntlHostOp::FormatNumberRangeParts.code()) * 2,
    );
    assert!(NumberFormatRequest::decode(&wrong_operation, profiles()).is_err());
    let mut oversized_locale = wire;
    set_word(&mut oversized_locale, 16, u64::MAX);
    assert!(NumberFormatRequest::decode(&oversized_locale, profiles()).is_err());
}

#[test]
fn exact_inputs_share_scalar_and_range_partition_codecs_without_rounding_in_transport() {
    let request = NumberFormatRequest {
        configuration: configuration(),
        input: ObservedNumericInput::StringNumericLiteral(
            "100000000000000001.25".encode_utf16().collect(),
        ),
    };
    let request = NumberFormatRequest::decode(&request.encode().unwrap(), profiles()).unwrap();
    let response =
        format_number_parts_operation(request, profiles(), &PartitionLimits::HOST_ABI).unwrap();
    assert_eq!(response.to_text(), "100,000,000,000,000,001.25");
    assert_eq!(
        ScalarNumberPartition::decode(&response.encode().unwrap()).unwrap(),
        response
    );
    let request = NumberRangeFormatRequest {
        configuration: configuration(),
        start: ObservedNumericInput::NumberShortestDecimal("1".into()),
        end: ObservedNumericInput::NumberShortestDecimal("1".into()),
    };
    let response =
        format_number_range_parts_operation(request, profiles(), &PartitionLimits::HOST_ABI)
            .unwrap();
    assert_eq!(
        RangeNumberPartition::decode(&response.encode().unwrap()).unwrap(),
        response
    );
    let request = NumberRangeFormatRequest {
        configuration: configuration(),
        start: ObservedNumericInput::NumberShortestDecimal("NaN".into()),
        end: ObservedNumericInput::NumberShortestDecimal("2".into()),
    };
    assert_eq!(
        format_number_range_parts_operation(request, profiles(), &PartitionLimits::HOST_ABI),
        Err(NumberFormatOperationError::NaNRangeEndpoint)
    );
}

#[test]
fn range_approximately_sign_is_shared_and_cannot_be_a_scalar_kind() {
    let mut wire = NumberWireWriter::new(
        IntlHostOp::FormatNumberRangeParts,
        NumberWireDirection::Response,
    )
    .unwrap();
    wire.word(1).unwrap();
    wire.word(NUMBER_APPROXIMATELY_SIGN_CODE).unwrap();
    wire.word(RangePartSource::Start.wire_code()).unwrap();
    wire.text("~").unwrap();
    assert!(RangeNumberPartition::decode(&wire.finish()).is_err());
    let mut wire =
        NumberWireWriter::new(IntlHostOp::FormatNumberParts, NumberWireDirection::Response)
            .unwrap();
    wire.word(1).unwrap();
    wire.word(NUMBER_APPROXIMATELY_SIGN_CODE).unwrap();
    wire.text("~").unwrap();
    assert!(ScalarNumberPartition::decode(&wire.finish()).is_err());
}
