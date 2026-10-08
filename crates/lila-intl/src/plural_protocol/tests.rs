use super::*;
use crate::number_format::numeric::{ObservedNumericInput, RoundingSettings};
use crate::number_format::{embedded_number_profiles_arc, PartitionLimits};
use crate::CanonicalLocaleId;
fn profiles() -> Arc<NumberProfiles> {
    embedded_number_profiles_arc().unwrap()
}
fn locale() -> PluralLocaleRequest {
    PluralLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data("en-US").unwrap()].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
    }
}
fn configuration() -> CheckedPluralConfiguration {
    CheckedPluralConfiguration::new(
        resolve_plural_locale(&locale(), &profiles(), &PartitionLimits::HOST_ABI).unwrap(),
        PluralType::Ordinal,
        Notation::Standard,
        RoundingSettings {
            precision: Precision::Fraction(FractionPrecision::Range(
                FractionDigitRange::new(
                    FractionDigitCount::new(0).unwrap(),
                    FractionDigitCount::new(3).unwrap(),
                )
                .unwrap(),
            )),
            minimum_integer_digits: IntegerDigitCount::new(1).unwrap(),
            mode: RoundingMode::HalfExpand,
            trailing_zero_display: TrailingZeroDisplay::Auto,
        },
    )
}
fn input() -> ObservedNumericInput {
    ObservedNumericInput::BigIntDecimal("100000000000000003".into())
}
#[test]
fn all_four_operation_shapes_round_trip_complete_messages() {
    let request = locale();
    assert_eq!(
        PluralLocaleRequest::decode(&request.encode().unwrap()).unwrap(),
        request
    );
    let resolved =
        resolve_plural_locale(&request, &profiles(), &PartitionLimits::HOST_ABI).unwrap();
    assert_eq!(
        ResolvedPluralLocale::decode(&resolved.encode().unwrap(), &profiles()).unwrap(),
        resolved
    );
    let supported = PluralSupportedLocalesRequest {
        requested: request.requested.clone(),
        matcher: request.matcher,
    };
    assert_eq!(
        PluralSupportedLocalesRequest::decode(&supported.encode().unwrap()).unwrap(),
        supported
    );
    let supported = supported_plural_locales(supported, &profiles()).unwrap();
    assert_eq!(
        PluralSupportedLocalesResult::decode(&supported.encode().unwrap()).unwrap(),
        supported
    );
    let scalar = SelectPluralRequest::new(configuration(), input());
    assert_eq!(
        SelectPluralRequest::decode(&scalar.encode().unwrap(), &profiles()).unwrap(),
        scalar
    );
    let range =
        SelectPluralRangeRequest::new(configuration(), input(), ObservedNumericInput::NegativeZero);
    assert_eq!(
        SelectPluralRangeRequest::decode(&range.encode().unwrap(), &profiles()).unwrap(),
        range
    );
    for category in PluralCategory::ALL {
        assert_eq!(
            PluralCategory::decode_scalar(&category.encode_scalar().unwrap()).unwrap(),
            category
        );
        assert_eq!(
            PluralCategory::decode_range(&category.encode_range().unwrap()).unwrap(),
            category
        );
        assert!(PluralCategory::decode_range(&category.encode_scalar().unwrap()).is_err());
    }
}
#[test]
fn masks_locale_association_header_and_trailing_bytes_are_checked() {
    for mask in [0, 1, 31, 64, u64::MAX] {
        assert!(PluralCategorySet::from_wire_mask(mask).is_none());
    }
    let resolved =
        resolve_plural_locale(&locale(), &profiles(), &PartitionLimits::HOST_ABI).unwrap();
    let mut bytes = resolved.encode().unwrap();
    let offset = bytes.len() - 8;
    bytes[offset..].copy_from_slice(&32u64.to_le_bytes());
    assert!(ResolvedPluralLocale::decode(&bytes, &profiles()).is_err());
    let original = locale().encode().unwrap();
    for length in 0..original.len() {
        assert!(PluralLocaleRequest::decode(&original[..length]).is_err());
    }
    let mut extra = original.clone();
    extra.push(0);
    assert!(PluralLocaleRequest::decode(&extra).is_err());
    let mut bad = original;
    bad[0] = 2;
    assert!(PluralLocaleRequest::decode(&bad).is_err());
    let mut category = PluralCategory::Other.encode_scalar().unwrap();
    category[16..].copy_from_slice(&6u64.to_le_bytes());
    assert!(PluralCategory::decode_scalar(&category).is_err());
}
#[test]
fn configuration_rejects_unknown_inactive_and_inconsistent_words() {
    let request = SelectPluralRequest::new(configuration(), input());
    let bytes = request.encode().unwrap();
    let offset = 16
        + 8
        + request.configuration().locale().resolved().as_str().len()
        + 8
        + request.configuration().locale().data().as_str().len();
    for (field, value) in [
        (W::Type, 0_u64),
        (W::Notation, 5),
        (W::CompactDisplay, 1),
        (W::MinimumInteger, 0),
        (W::MinimumInteger, 22),
        (W::Precision, 0),
        (W::MinimumSignificant, 1),
        (W::RoundingIncrement, 3),
        (W::RoundingIncrement, 2),
        (W::RoundingMode, 10),
        (W::TrailingZero, 0),
    ] {
        let mut bad = bytes.clone();
        let start = offset + field.offset() as usize;
        bad[start..start + 8].copy_from_slice(&value.to_le_bytes());
        assert!(
            SelectPluralRequest::decode(&bad, &profiles()).is_err(),
            "{field:?} {value}"
        );
    }
    let mut bad = bytes;
    bad.push(0);
    assert!(SelectPluralRequest::decode(&bad, &profiles()).is_err());
}
#[test]
fn primitive_numeric_input_codec_remains_lossless_and_bounded() {
    for input in [
        ObservedNumericInput::StringNumericLiteral(vec![0xD800].into_boxed_slice()),
        ObservedNumericInput::NegativeZero,
        ObservedNumericInput::NumberShortestDecimal("1.2345678901234567".into()),
        input(),
    ] {
        let request = SelectPluralRequest::new(configuration(), input);
        assert_eq!(
            SelectPluralRequest::decode(&request.encode().unwrap(), &profiles()).unwrap(),
            request
        );
    }
    let mut bytes = SelectPluralRequest::new(configuration(), input())
        .encode()
        .unwrap();
    let extent = bytes.len() - "100000000000000003".len() - 8;
    bytes[extent..extent + 8].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(SelectPluralRequest::decode(&bytes, &profiles()).is_err());
}

#[test]
fn decoded_plural_requests_use_the_selected_number_image_owner() {
    use crate::number_format::numeric::NumericLimits;

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
        configuration(),
        ObservedNumericInput::BigIntDecimal(format!("{}03", "9".repeat(398)).into_boxed_str()),
    );
    let scalar = SelectPluralRequest::decode(&scalar.encode().unwrap(), &selected).unwrap();
    assert_eq!(
        select_plural_operation(scalar.clone(), &selected, &NumericLimits::HOST_ABI),
        Ok(PluralCategory::Few)
    );
    assert_eq!(
        select_plural_operation(scalar, &original, &NumericLimits::HOST_ABI),
        Err(PluralRulesOperationError::InvalidResolvedLocale)
    );

    let range = SelectPluralRangeRequest::new(
        configuration(),
        ObservedNumericInput::BigIntDecimal("4".into()),
        ObservedNumericInput::BigIntDecimal("1".into()),
    );
    let range = SelectPluralRangeRequest::decode(&range.encode().unwrap(), &selected).unwrap();
    assert_eq!(
        select_plural_range_operation(range.clone(), &selected, &NumericLimits::HOST_ABI),
        Ok(PluralCategory::One)
    );
    assert_eq!(
        select_plural_range_operation(range, &original, &NumericLimits::HOST_ABI),
        Err(PluralRulesOperationError::InvalidResolvedLocale)
    );
}
