use super::*;
use crate::embedded_list_data_image;
use crate::number_format::options::LocaleMatcher;
use crate::number_format::{embedded_number_profiles_arc, NumberLocaleRequest, PartitionLimits};
fn admitted() -> &'static DurationProfiles {
    embedded_duration_profiles().unwrap()
}
fn setup(locale: &str, options: DurationOptions) -> CheckedDurationConfiguration {
    let numbers = &embedded_number_profiles_arc().unwrap();
    let profiles = admitted();
    let resolved = profiles
        .resolve_locale(
            &NumberLocaleRequest {
                requested: vec![crate::CanonicalLocaleId::from_data(locale).unwrap()]
                    .into_boxed_slice(),
                matcher: LocaleMatcher::Lookup,
                numbering_system: None,
            },
            numbers,
        )
        .unwrap();
    CheckedDurationConfiguration::new(resolved, options).unwrap()
}
fn format(locale: &str, options: DurationOptions, fields: [f64; 10]) -> DurationPartition {
    format_duration_parts(
        &setup(locale, options),
        &DurationRecord::from_number_fields(fields).unwrap(),
        admitted(),
        &PartitionLimits::HOST_ABI,
    )
    .unwrap()
}
fn digital() -> DurationOptions {
    DurationOptions {
        style: DurationStyle::Digital,
        ..Default::default()
    }
}
#[test]
fn finite_integral_uniform_sign_and_calendar_field_bounds_are_checked_once() {
    for value in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN, 0.5] {
        let mut fields = [0.0; 10];
        fields[6] = value;
        assert!(DurationRecord::from_number_fields(fields).is_err())
    }
    assert!(DurationRecord::from_number_fields([
        1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
    ])
    .is_err());
    for index in 0..3 {
        let mut fields = [0.0; 10];
        fields[index] = 4_294_967_296.0;
        assert_eq!(
            DurationRecord::from_number_fields(fields),
            Err(DurationError::InvalidBounds)
        );
        fields[index] -= 1.0;
        assert!(DurationRecord::from_number_fields(fields).is_ok())
    }
}
#[test]
fn normalized_day_time_limit_includes_exact_day_and_subsecond_contributions() {
    let mut fields = [0.0; 10];
    fields[6] = 9_007_199_254_740_991.0;
    fields[9] = 999_999_999.0;
    assert!(DurationRecord::from_number_fields(fields).is_ok());
    fields[9] = 1_000_000_000.0;
    assert_eq!(
        DurationRecord::from_number_fields(fields),
        Err(DurationError::InvalidBounds)
    );
    fields = [0.0; 10];
    fields[3] = 104_249_991_374.0;
    assert!(DurationRecord::from_number_fields(fields).is_ok());
    fields[3] = 104_249_991_375.0;
    assert_eq!(
        DurationRecord::from_number_fields(fields),
        Err(DurationError::InvalidBounds)
    );
    fields = [0.0; 10];
    fields[9] = f64::MAX;
    assert_eq!(
        DurationRecord::from_number_fields(fields),
        Err(DurationError::InvalidBounds)
    );
}
#[test]
fn ieee_unsafe_integer_subseconds_are_composed_as_exact_mathematical_values() {
    let result = format(
        "en",
        digital(),
        [
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            4_503_599_627_370_497_000.0,
            4_503_599_627_370_495_000_000.0,
            0.0,
        ],
    );
    assert_eq!(result.to_text().unwrap(), "0:00:9007199254740991.975424");
    let result = format(
        "en",
        digital(),
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 10_000_000.0, 0.0, 0.0, 1.0],
    );
    assert_eq!(result.to_text().unwrap(), "0:00:10000000.000000001");
}
#[test]
fn nanoseconds_max_safe_integer_preserves_all_nine_fraction_places() {
    assert_eq!(
        format(
            "en",
            digital(),
            [
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                2.0,
                3.0,
                9_007_199_254_740_991.0
            ]
        )
        .to_text()
        .unwrap(),
        "0:00:9007200.256743991"
    );
}
#[test]
fn negative_zero_input_is_neutral_but_first_displayed_zero_owns_negative_sign() {
    assert!(!DurationRecord::from_number_fields([-0.0; 10])
        .unwrap()
        .negative());
    let result = format(
        "en",
        digital(),
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0],
    );
    assert_eq!(result.to_text().unwrap(), "-0:00:01");
    assert_eq!(
        result
            .parts()
            .iter()
            .filter(|p| p.kind() == crate::number_format::NumberPartKind::MinusSign)
            .count(),
        1
    );
}
#[test]
fn genuine_serbian_digital_separators_survive_number_and_list_composition() {
    let result = format(
        "sr",
        digital(),
        [0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 0.0, 0.0, 0.0],
    );
    assert_eq!(result.to_text().unwrap(), "1.02.03");
    let separators = result
        .parts()
        .iter()
        .filter(|p| p.unit().is_none())
        .map(|p| p.text())
        .collect::<Vec<_>>();
    assert_eq!(separators, [".", "."]);
}
#[test]
fn numeric_padding_and_auto_zero_hour_minute_bridge_are_checked_in_configuration() {
    let mut options = digital();
    options.units[4].display = Some(DurationDisplay::Auto);
    options.units[5].display = Some(DurationDisplay::Auto);
    assert_eq!(
        format(
            "en",
            options.clone(),
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0]
        )
        .to_text()
        .unwrap(),
        "03"
    );
    assert_eq!(
        format(
            "en",
            options,
            [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 3.0, 0.0, 0.0, 0.0]
        )
        .to_text()
        .unwrap(),
        "1:00:03"
    );
}
#[test]
fn fractional_precision_truncates_without_binary64_rounding_and_keeps_requested_zeroes() {
    let mut options = digital();
    options.fractional_digits = Some(DurationFractionalDigits::new(3).unwrap());
    assert_eq!(
        format(
            "en",
            options.clone(),
            [0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 123.0, 999.0, 999.0]
        )
        .to_text()
        .unwrap(),
        "1:02:03.123"
    );
    assert_eq!(
        format(
            "en",
            options,
            [0.0, 0.0, 0.0, 0.0, -1.0, -2.0, -3.0, 0.0, 0.0, 0.0]
        )
        .to_text()
        .unwrap(),
        "-1:02:03.000"
    );
    assert!(DurationFractionalDigits::new(10).is_err());
}
#[test]
fn incompatible_text_numeric_and_fractional_option_transitions_reject_at_construction() {
    for (index, style) in [
        (0, DurationUnitStyle::Numeric),
        (7, DurationUnitStyle::TwoDigit),
        (5, DurationUnitStyle::Long),
    ] {
        let mut options = digital();
        options.units[index].style = Some(style);
        let locale = setup("en", digital()).locale().clone();
        assert!(CheckedDurationConfiguration::new(locale, options).is_err());
    }
    let mut options = digital();
    options.units[7].display = Some(DurationDisplay::Always);
    assert!(
        CheckedDurationConfiguration::new(setup("en", digital()).locale().clone(), options)
            .is_err()
    );
}
#[test]
fn textual_subsecond_absorption_uses_shared_unit_partition_and_cardinal_rounding() {
    let mut options = DurationOptions {
        style: DurationStyle::Long,
        ..Default::default()
    };
    options.units[7].style = Some(DurationUnitStyle::Numeric);
    let result = format(
        "en",
        options,
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0],
    );
    assert_eq!(result.to_text().unwrap(), "1.001 seconds");
    assert!(result
        .parts()
        .iter()
        .all(|p| p.unit() == Some(DurationUnit::Second)));
}
#[test]
fn parts_preserve_unit_ownership_and_only_list_and_digital_literals_have_no_unit() {
    let options = DurationOptions {
        style: DurationStyle::Long,
        ..Default::default()
    };
    let result = format(
        "en",
        options,
        [1.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    );
    assert_eq!(result.to_text().unwrap(), "1 year, 2 months");
    assert!(result
        .parts()
        .iter()
        .any(|p| p.kind() == crate::number_format::NumberPartKind::Unit
            && p.unit() == Some(DurationUnit::Year)));
    assert_eq!(
        result
            .parts()
            .iter()
            .filter(|p| p.unit().is_none())
            .map(|p| p.text())
            .collect::<Vec<_>>(),
        [", "]
    );
}
#[test]
fn all15_locales_are_admitted_by_actual_shared_number_and_list_owners() {
    let profiles = crate::DurationDataImage::from_bytes(
        crate::embedded_duration_data_image().unwrap().bytes(),
        &crate::embedded_locale_data_image().unwrap(),
        &crate::embedded_number_profiles_data_image().unwrap(),
        &embedded_list_data_image().unwrap(),
    )
    .unwrap()
    .profiles();
    assert_eq!(
        profiles
            .available_locales()
            .map(|p| p.as_str())
            .collect::<Vec<_>>(),
        profiles::LOCALES
    );
    for locale in profiles::LOCALES {
        let _ = format(
            locale,
            digital(),
            [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        );
    }
    assert_eq!(
        setup("sr-Thai-RS", digital()).locale().resolved().as_str(),
        "sr"
    );
    assert_eq!(setup("zh-CN", digital()).locale().resolved().as_str(), "zh");
}

#[test]
fn selected_duration_catalogue_captures_the_actual_selected_list_profiles() {
    use crate::list_format::{FormatListPartsRequest, ListFormatOperationError};
    let locale = crate::embedded_locale_data_image().unwrap();
    let numbers = crate::embedded_number_profiles_data_image().unwrap();
    let default_lists = embedded_list_data_image().unwrap();
    let selected_lists = crate::ListDataImage::from_bytes(default_lists.bytes(), &locale).unwrap();
    let image = crate::DurationDataImage::for_profile(
        crate::IntlDataProfile::Minimal,
        &locale,
        &numbers,
        &selected_lists,
    )
    .unwrap();
    let profiles = image.profiles();
    let locale = profiles
        .resolve_locale(
            &NumberLocaleRequest {
                requested: vec![crate::CanonicalLocaleId::from_data("en").unwrap()]
                    .into_boxed_slice(),
                matcher: LocaleMatcher::Lookup,
                numbering_system: None,
            },
            &numbers.profiles(),
        )
        .unwrap();
    let configuration = CheckedDurationConfiguration::new(
        locale,
        DurationOptions {
            style: DurationStyle::Long,
            ..Default::default()
        },
    )
    .unwrap();
    let request = FormatListPartsRequest::new(
        configuration.list.clone(),
        vec![vec![0x31].into_boxed_slice(), vec![0x32].into_boxed_slice()].into_boxed_slice(),
    )
    .unwrap();
    assert!(selected_lists
        .profiles()
        .format_parts(request.clone())
        .is_ok());
    assert!(matches!(
        default_lists.profiles().format_parts(request),
        Err(ListFormatOperationError::InvalidResolvedLocale)
    ));
    let record =
        DurationRecord::from_number_fields([1.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
            .unwrap();
    assert_eq!(
        format_duration_parts(
            &configuration,
            &record,
            &profiles,
            &PartitionLimits::HOST_ABI
        )
        .unwrap()
        .to_text()
        .unwrap(),
        "1 year, 2 months"
    );
}
#[test]
fn all78_numbering_systems_are_consumed_by_real_numeric_partitions() {
    let numbers = &embedded_number_profiles_arc().unwrap();
    assert_eq!(numbers.numbering_systems().len(), 78);
    for system in numbers.numbering_systems() {
        let configuration = setup(&format!("en-u-nu-{system}"), digital());
        assert_eq!(configuration.locale().numbering_system(), system.as_ref());
        let output = format_duration_parts(
            &configuration,
            &DurationRecord::from_number_fields([0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 0.0, 0.0, 0.0])
                .unwrap(),
            admitted(),
            &PartitionLimits::HOST_ABI,
        )
        .unwrap();
        assert_eq!(
            output
                .parts()
                .iter()
                .filter(|p| p.kind() == crate::number_format::NumberPartKind::Integer)
                .count(),
            3
        );
    }
}
#[test]
fn malformed_profile_domain_and_digital_metadata_reject_before_owner_publication() {
    let numbers = &embedded_number_profiles_arc().unwrap();
    let lists = &embedded_list_data_image().unwrap().profiles();
    let raw = include_str!("generated/profile.json");
    for damaged in [
        raw.replacen("\"en-US\"", "\"en\"", 1),
        raw.replacen("h:mm:ss", "h:mm:ss z", 1),
    ] {
        assert!(DurationProfiles::from_json(&damaged, numbers, lists).is_err())
    }
    let mut damaged: serde_json::Value = serde_json::from_str(raw).unwrap();
    damaged["locales"][0]["units"][0]["patterns"][0] = "{0}{0} years".into();
    assert!(DurationProfiles::from_json(&damaged.to_string(), numbers, lists).is_err());
}
#[test]
fn complete_zero_auto_partition_and_resource_limits_have_honest_extents() {
    let record = DurationRecord::from_number_fields([0.0; 10]).unwrap();
    let result = format_duration_parts(
        &setup("en", DurationOptions::default()),
        &record,
        admitted(),
        &PartitionLimits::HOST_ABI,
    )
    .unwrap();
    assert!(result.parts().is_empty());
    assert_eq!(result.to_text().unwrap(), "");
    let limits = PartitionLimits::new(
        crate::number_format::numeric::NumericLimits::HOST_ABI,
        core::num::NonZeroU32::new(1).unwrap(),
        core::num::NonZeroU32::new(1).unwrap(),
    );
    let record =
        DurationRecord::from_number_fields([0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 0.0, 0.0, 0.0])
            .unwrap();
    assert!(format_duration_parts(&setup("en", digital()), &record, admitted(), &limits).is_err());
}

#[test]
fn normative_hours_override_applies_to_every_valid_text_style_without_forging_locale_data() {
    use super::configuration::{validated_width, EffectiveStyle};
    // All fifteen genuine captured rows currently have false. Test the primary
    // algorithm's Boolean domain directly, without inventing a locale profile.
    for style in [
        DurationUnitStyle::Long,
        DurationUnitStyle::Short,
        DurationUnitStyle::Narrow,
    ] {
        assert_eq!(
            validated_width(
                DurationUnit::Hour,
                EffectiveStyle::Text(style),
                DurationDisplay::Auto,
                None,
                true
            )
            .unwrap(),
            EffectiveStyle::Numeric { two_digit: true }
        );
        assert_eq!(
            validated_width(
                DurationUnit::Hour,
                EffectiveStyle::Text(style),
                DurationDisplay::Auto,
                None,
                false
            )
            .unwrap(),
            EffectiveStyle::Text(style)
        );
    }
}
#[test]
fn invalid_previous_and_next_unit_styles_reject_before_width_override() {
    use super::configuration::{validated_width, EffectiveStyle};
    assert_eq!(
        validated_width(
            DurationUnit::Hour,
            EffectiveStyle::Text(DurationUnitStyle::Long),
            DurationDisplay::Auto,
            Some(EffectiveStyle::Fractional),
            true
        ),
        Err(DurationError::InvalidOptions)
    );
    let hours = validated_width(
        DurationUnit::Hour,
        EffectiveStyle::Text(DurationUnitStyle::Long),
        DurationDisplay::Always,
        None,
        true,
    )
    .unwrap();
    assert_eq!(
        validated_width(
            DurationUnit::Minute,
            EffectiveStyle::Text(DurationUnitStyle::Short),
            DurationDisplay::Always,
            Some(hours),
            false
        ),
        Err(DurationError::InvalidOptions)
    );
    assert_eq!(
        validated_width(
            DurationUnit::Minute,
            EffectiveStyle::Numeric { two_digit: false },
            DurationDisplay::Always,
            Some(hours),
            false
        )
        .unwrap(),
        EffectiveStyle::Numeric { two_digit: true }
    );
}
