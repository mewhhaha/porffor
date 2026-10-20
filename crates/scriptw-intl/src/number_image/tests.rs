use super::*;
use crate::number_format::numeric::ObservedNumericInput;
use crate::number_format::options::*;
use crate::number_format::{
    resolve_number_locale, NumberFormatConfiguration, NumberLocaleRequest, PartitionLimits,
};
use crate::number_operation::{
    format_number_parts_operation, format_number_range_parts_operation, NumberFormatOperationError,
    NumberFormatRequest, NumberRangeFormatRequest,
};
use crate::{CanonicalLocaleId, CustomProfileId};

fn configuration(profiles: &Arc<NumberProfiles>) -> NumberFormatConfiguration {
    NumberFormatConfiguration {
        locale: resolve_number_locale(
            &NumberLocaleRequest {
                requested: vec![CanonicalLocaleId::from_data("en-US").unwrap()].into_boxed_slice(),
                matcher: LocaleMatcher::Lookup,
                numbering_system: None,
            },
            profiles,
        )
        .unwrap(),
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

#[test]
fn admitted_tables_own_dynamic_text_after_source_and_image_handles_drop() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let first = embedded_number_profiles_data_image().unwrap();
    let dynamic: Arc<[u8]> = first.bytes().as_ref().to_vec().into();
    let second = NumberProfilesDataImage::from_bytes(
        dynamic.clone(),
        &locale,
        &crate::embedded_list_data_image().unwrap(),
    )
    .unwrap();
    let profiles = second.profiles();
    let retained = configuration(&profiles);
    drop(dynamic);
    drop(second);
    let parts = format_number_parts_operation(
        NumberFormatRequest {
            configuration: retained,
            input: ObservedNumericInput::StringNumericLiteral("1234.5".encode_utf16().collect()),
        },
        &profiles,
        &PartitionLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(parts.to_text(), "1,234.5");
    assert_eq!(profiles.available_locales().len(), 1082);
    assert_eq!(profiles.numbering_systems().len(), 78);
    assert!(profiles
        .numbering_systems()
        .iter()
        .any(|name| name.as_ref() == "tols"));
}

#[test]
fn identical_payload_owners_cannot_exchange_scalar_or_nan_range_configurations() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let first = embedded_number_profiles_data_image().unwrap().profiles();
    let second = NumberProfilesDataImage::for_profile(IntlDataProfile::Minimal, &locale)
        .unwrap()
        .profiles();
    let retained = configuration(&first);
    let foreign = NumberFormatOperationError::Kernel(
        crate::number_format::NumberFormatKernelError::InvalidResolvedLocale,
    );
    assert_eq!(
        format_number_parts_operation(
            NumberFormatRequest {
                configuration: retained.clone(),
                input: ObservedNumericInput::NumberShortestDecimal("1".into()),
            },
            &second,
            &PartitionLimits::HOST_ABI
        ),
        Err(foreign.clone())
    );
    // Owner rejection precedes the normal NaN range response branch.
    assert_eq!(
        format_number_range_parts_operation(
            NumberRangeFormatRequest {
                configuration: retained,
                start: ObservedNumericInput::NumberShortestDecimal("NaN".into()),
                end: ObservedNumericInput::NumberShortestDecimal("1".into()),
            },
            &second,
            &PartitionLimits::HOST_ABI
        ),
        Err(foreign)
    );
}

#[test]
fn altered_self_consistent_native_payload_cannot_claim_pinned_source() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let mut payload = native_payload().unwrap();
    *payload.last_mut().unwrap() ^= 1;
    let framed = DataImageEnvelope::encode(
        DataImageComponent::NumberProfiles,
        &IntlDataProfile::Minimal,
        NUMBER_IMAGE_MARKERS,
        &payload,
    )
    .unwrap();
    assert!(matches!(
        NumberProfilesDataImage::from_bytes(
            framed,
            &locale,
            &crate::embedded_list_data_image().unwrap()
        ),
        Err(IntlDataImageError::Consumer(_))
    ));
}

#[test]
fn number_constructor_rejects_different_locale_profile_and_unavailable_conformance() {
    let minimal = embedded_number_profiles_data_image().unwrap();
    let locale = LocaleDataImage::for_profile(IntlDataProfile::Custom(
        CustomProfileId::parse("number-owner-proof").unwrap(),
    ))
    .unwrap();
    assert!(matches!(
        NumberProfilesDataImage::from_bytes(
            minimal.bytes(),
            &locale,
            &crate::ListDataImage::for_profile(locale.profile().clone(), &locale).unwrap()
        ),
        Err(IntlDataImageError::Consumer(_))
    ));
    assert!(matches!(
        NumberProfilesDataImage::for_profile(IntlDataProfile::Conformance, &locale),
        Err(IntlDataImageError::Consumer(_))
    ));
}
