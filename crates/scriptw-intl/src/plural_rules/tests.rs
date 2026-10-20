use super::*;
use crate::number_format::embedded_number_profiles_arc;
use crate::number_format::options::*;

fn profiles() -> Arc<NumberProfiles> {
    embedded_number_profiles_arc().unwrap()
}
fn string(value: &str) -> ObservedNumericInput {
    ObservedNumericInput::StringNumericLiteral(value.encode_utf16().collect())
}
fn configuration(locale: &str, kind: PluralType) -> CheckedPluralConfiguration {
    let request = PluralLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data(locale).unwrap()].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
    };
    CheckedPluralConfiguration::new(
        resolve_plural_locale(&request, &profiles(), &PartitionLimits::HOST_ABI).unwrap(),
        kind,
        Notation::Standard,
        RoundingSettings {
            precision: Precision::Fraction(FractionPrecision::Range(
                FractionDigitRange::new(
                    FractionDigitCount::new(0).unwrap(),
                    FractionDigitCount::new(3).unwrap(),
                )
                .unwrap(),
            )),
            mode: RoundingMode::HalfExpand,
            minimum_integer_digits: IntegerDigitCount::new(1).unwrap(),
            trailing_zero_display: TrailingZeroDisplay::Auto,
        },
    )
}
fn select(
    configuration: CheckedPluralConfiguration,
    input: ObservedNumericInput,
) -> PluralCategory {
    select_plural_operation(
        SelectPluralRequest::new(configuration, input),
        &profiles(),
        &NumericLimits::HOST_ABI,
    )
    .unwrap()
}
fn range(configuration: CheckedPluralConfiguration, start: &str, end: &str) -> PluralCategory {
    select_plural_range_operation(
        SelectPluralRangeRequest::new(configuration, string(start), string(end)),
        &profiles(),
        &NumericLimits::HOST_ABI,
    )
    .unwrap()
}
#[test]
fn ordinal_association_splits_old_cardinal_equivalent_profiles() {
    let english = configuration("en-Shaw", PluralType::Ordinal);
    let ido = configuration("io", PluralType::Ordinal);
    assert!(english
        .locale()
        .categories(PluralType::Ordinal)
        .contains(PluralCategory::Few));
    assert_eq!(ido.locale().categories(PluralType::Ordinal).wire_mask(), 32);
    assert_eq!(select(english, string("3")), PluralCategory::Few);
    assert_eq!(select(ido, string("3")), PluralCategory::Other);
}
#[test]
fn exact_bigint_modulo_differs_from_same_string_overflow() {
    let text = format!("{}03", "9".repeat(398));
    assert_eq!(
        select(
            configuration("en", PluralType::Ordinal),
            ObservedNumericInput::BigIntDecimal(text.clone().into_boxed_str())
        ),
        PluralCategory::Few
    );
    assert_eq!(
        select(configuration("en", PluralType::Ordinal), string(&text)),
        PluralCategory::Other
    );
}
#[test]
fn signed_rounding_visible_zeroes_and_integer_strip_feed_bare_operands() {
    let mut c = configuration("en", PluralType::Cardinal);
    c.rounding.precision = Precision::Fraction(FractionPrecision::Range(
        FractionDigitRange::new(
            FractionDigitCount::new(0).unwrap(),
            FractionDigitCount::new(0).unwrap(),
        )
        .unwrap(),
    ));
    c.rounding.mode = RoundingMode::Floor;
    assert_eq!(select(c.clone(), string("-1.2")), PluralCategory::Other);
    c.rounding.mode = RoundingMode::Ceil;
    assert_eq!(select(c.clone(), string("-1.2")), PluralCategory::One);
    c.rounding.precision = Precision::Fraction(FractionPrecision::Range(
        FractionDigitRange::new(
            FractionDigitCount::new(2).unwrap(),
            FractionDigitCount::new(2).unwrap(),
        )
        .unwrap(),
    ));
    assert_eq!(select(c.clone(), string("1")), PluralCategory::Other);
    c.rounding.trailing_zero_display = TrailingZeroDisplay::StripIfInteger;
    assert_eq!(select(c, string("1")), PluralCategory::One);
}
#[test]
fn all_signed_rounding_modes_use_shared_precision_authority() {
    let cases = [
        (RoundingMode::Ceil, "1"),
        (RoundingMode::Floor, "2"),
        (RoundingMode::Expand, "2"),
        (RoundingMode::Trunc, "1"),
        (RoundingMode::HalfCeil, "1"),
        (RoundingMode::HalfFloor, "2"),
        (RoundingMode::HalfExpand, "2"),
        (RoundingMode::HalfTrunc, "1"),
        (RoundingMode::HalfEven, "2"),
    ];
    for (mode, expected) in cases {
        let mut c = configuration("en", PluralType::Cardinal);
        c.rounding.precision = Precision::Fraction(FractionPrecision::Range(
            FractionDigitRange::new(
                FractionDigitCount::new(0).unwrap(),
                FractionDigitCount::new(0).unwrap(),
            )
            .unwrap(),
        ));
        c.rounding.mode = mode;
        let input = normalize_numeric_input(string("-1.5"), &NumericLimits::HOST_ABI).unwrap();
        let rounded = round_decimal(
            input.finite_value().unwrap(),
            &c.rounding,
            &NumericLimits::HOST_ABI,
        )
        .unwrap();
        assert_eq!(
            rounded.integer_digits(),
            expected
                .as_bytes()
                .iter()
                .map(|byte| byte - b'0')
                .collect::<Vec<_>>()
        );
        assert_eq!(
            select(c, string("-1.5")),
            if expected == "1" {
                PluralCategory::One
            } else {
                PluralCategory::Other
            }
        );
    }
}
#[test]
fn range_identity_is_unsigned_bare_string_not_category_or_input_equality() {
    assert_eq!(
        range(configuration("en", PluralType::Cardinal), "-1", "1"),
        PluralCategory::One
    );
    assert_eq!(
        range(configuration("sl", PluralType::Cardinal), "1", "1.0004"),
        PluralCategory::One
    );
    assert_eq!(
        select(configuration("sl", PluralType::Cardinal), string("101")),
        PluralCategory::One
    );
    assert_eq!(
        range(configuration("sl", PluralType::Cardinal), "1", "101"),
        PluralCategory::Few
    );
    assert_eq!(
        range(configuration("en", PluralType::Ordinal), "4", "1"),
        PluralCategory::One
    );
}
#[test]
fn range_admits_descending_negative_and_infinite_endpoints_rejecting_only_nan() {
    assert_eq!(
        range(configuration("en", PluralType::Cardinal), "2", "1"),
        PluralCategory::Other
    );
    assert_eq!(
        range(
            configuration("en", PluralType::Cardinal),
            "-Infinity",
            "Infinity"
        ),
        PluralCategory::Other
    );
    assert_eq!(
        range(
            configuration("en", PluralType::Cardinal),
            "Infinity",
            "Infinity"
        ),
        PluralCategory::Other
    );
    let result = select_plural_range_operation(
        SelectPluralRangeRequest::new(
            configuration("en", PluralType::Cardinal),
            string("NaN"),
            string("1"),
        ),
        &profiles(),
        &NumericLimits::HOST_ABI,
    );
    assert_eq!(result, Err(PluralRulesOperationError::NaNRangeEndpoint));
    for source in ["NaN", "Infinity", "-Infinity"] {
        assert_eq!(
            select(configuration("en", PluralType::Cardinal), string(source)),
            PluralCategory::Other
        );
    }
}
#[test]
fn compact_metadata_uses_final_bare_magnitude_without_scaling_operands() {
    let mut c = configuration("fr", PluralType::Cardinal);
    c.notation = Notation::Compact(CompactDisplay::Short);
    c.rounding.precision = Precision::Fraction(FractionPrecision::Range(
        FractionDigitRange::new(
            FractionDigitCount::new(0).unwrap(),
            FractionDigitCount::new(0).unwrap(),
        )
        .unwrap(),
    ));
    assert_eq!(select(c.clone(), string("999999.4")), PluralCategory::Other);
    assert_eq!(select(c.clone(), string("999999.5")), PluralCategory::Many);
    assert_eq!(select(c.clone(), string("1500000")), PluralCategory::Many);
    c.notation = Notation::Scientific;
    assert_eq!(select(c.clone(), string("1500000")), PluralCategory::Other);
    c.notation = Notation::Engineering;
    assert_eq!(select(c, string("1500000")), PluralCategory::Other);
}
#[test]
fn supported_locales_uses_actual_locale_set_without_type_or_numbering_dependency() {
    let request = PluralSupportedLocalesRequest {
        requested: vec![
            CanonicalLocaleId::from_data("en-US-u-nu-arab").unwrap(),
            CanonicalLocaleId::from_data("zxx").unwrap(),
        ]
        .into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
    };
    let result = supported_plural_locales(request, &profiles()).unwrap();
    assert_eq!(
        result
            .locales
            .iter()
            .map(CanonicalLocaleId::as_str)
            .collect::<Vec<_>>(),
        ["en-US-u-nu-arab"]
    );
    assert_eq!(
        configuration("en-US-u-nu-arab", PluralType::Cardinal)
            .locale()
            .resolved()
            .as_str(),
        "en-US"
    );
    assert_eq!(profiles().available_locales().len(), 1082);
}

#[test]
fn selected_number_image_rejects_foreign_scalar_and_range_proofs() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let image = crate::embedded_number_profiles_data_image().unwrap();
    let selected = crate::NumberProfilesDataImage::from_bytes(
        image.bytes(),
        &locale,
        &crate::embedded_list_data_image().unwrap(),
    )
    .unwrap()
    .profiles();
    let original = profiles();
    assert!(!Arc::ptr_eq(&original, &selected));

    let scalar = SelectPluralRequest::new(
        configuration("en", PluralType::Ordinal),
        ObservedNumericInput::BigIntDecimal(format!("{}03", "9".repeat(398)).into_boxed_str()),
    );
    assert_eq!(
        select_plural_operation(scalar.clone(), &original, &NumericLimits::HOST_ABI),
        Ok(PluralCategory::Few)
    );
    assert_eq!(
        select_plural_operation(scalar, &selected, &NumericLimits::HOST_ABI),
        Err(PluralRulesOperationError::InvalidResolvedLocale)
    );

    let range = SelectPluralRangeRequest::new(
        configuration("sl", PluralType::Cardinal),
        string("1"),
        string("101"),
    );
    assert_eq!(
        select_plural_range_operation(range.clone(), &original, &NumericLimits::HOST_ABI),
        Ok(PluralCategory::Few)
    );
    assert_eq!(
        select_plural_range_operation(range, &selected, &NumericLimits::HOST_ABI),
        Err(PluralRulesOperationError::InvalidResolvedLocale)
    );
    let nonfinite = SelectPluralRangeRequest::new(
        configuration("en", PluralType::Cardinal),
        string("NaN"),
        string("Infinity"),
    );
    assert_eq!(
        select_plural_range_operation(nonfinite, &selected, &NumericLimits::HOST_ABI),
        Err(PluralRulesOperationError::InvalidResolvedLocale)
    );
}
