use super::*;
use crate::number_format::options::LocaleMatcher;
use crate::number_format::{
    embedded_number_profiles_arc, NumberLocaleRequest, NumberPartKind, NumberProfiles,
    NumberingSystemOption,
};
use crate::CanonicalLocaleId;
use core::num::NonZeroU32;
use std::sync::Arc;

fn numbers() -> Arc<NumberProfiles> {
    embedded_number_profiles_arc().unwrap()
}
fn profiles() -> &'static RelativeProfiles {
    embedded_relative_profiles().unwrap()
}
fn locale(value: &str) -> CanonicalLocaleId {
    CanonicalLocaleId::from_data(value).unwrap()
}
fn configuration(
    value: &str,
    style: RelativeStyle,
    numeric: RelativeNumeric,
) -> RelativeTimeConfiguration {
    let request = NumberLocaleRequest {
        requested: vec![locale(value)].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
        numbering_system: None,
    };
    let resolved = profiles()
        .resolve_locale(&request, &numbers(), &PartitionLimits::HOST_ABI)
        .unwrap();
    RelativeTimeConfiguration::new(resolved, style, numeric, &numbers()).unwrap()
}
fn parts(
    configuration: &RelativeTimeConfiguration,
    value: f64,
    unit: RelativeUnit,
) -> RelativePartition {
    format_relative_time_parts(
        configuration,
        FiniteRelativeNumber::new(value).unwrap(),
        unit,
        profiles(),
        &PartitionLimits::HOST_ABI,
    )
    .unwrap()
}

#[test]
fn selected_number_image_rejects_foreign_auto_literal_configuration() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let image = crate::embedded_number_profiles_data_image().unwrap();
    let selected_image = crate::NumberProfilesDataImage::from_bytes(
        image.bytes(),
        &locale,
        &crate::embedded_list_data_image().unwrap(),
    )
    .unwrap();
    let selected = selected_image.profiles();
    let selected_relative = crate::RelativeTimeDataImage::from_bytes(
        crate::embedded_relative_time_data_image().unwrap().bytes(),
        &locale,
        &selected_image,
    )
    .unwrap();
    let original = numbers();
    assert!(!Arc::ptr_eq(&original, &selected));
    let configuration = configuration("en", RelativeStyle::Long, RelativeNumeric::Auto);
    let value = FiniteRelativeNumber::new(-1.0).unwrap();
    assert_eq!(
        format_relative_time_parts(
            &configuration,
            value,
            RelativeUnit::Day,
            profiles(),
            &PartitionLimits::HOST_ABI,
        )
        .unwrap()
        .to_text()
        .unwrap(),
        "yesterday"
    );
    assert_eq!(
        format_relative_time_parts(
            &configuration,
            value,
            RelativeUnit::Day,
            &selected_relative.profiles(),
            &PartitionLimits::HOST_ABI,
        ),
        Err(RelativeTimeError::Number(
            NumberFormatKernelError::InvalidResolvedLocale
        ))
    );
    assert_eq!(
        RelativeTimeConfiguration::new(
            configuration.locale().clone(),
            RelativeStyle::Long,
            RelativeNumeric::Auto,
            &selected,
        ),
        Err(RelativeTimeError::Number(
            NumberFormatKernelError::InvalidResolvedLocale
        ))
    );
}

#[test]
fn relative_units_accept_only_eight_singular_and_plural_spellings() {
    assert_eq!(RelativeUnit::ALL.len(), 8);
    for unit in RelativeUnit::ALL {
        assert_eq!(RelativeUnit::from_observed(unit.name()), Some(*unit));
        assert_eq!(
            RelativeUnit::from_observed(&format!("{}s", unit.name())),
            Some(*unit)
        );
        assert_eq!(RelativeUnit::from_wire_code(unit.wire_code()), Some(*unit));
    }
    for rejected in [
        "",
        "dayss",
        "Day",
        "seconds ",
        "millisecond",
        "years\0",
        "fortnight",
    ] {
        assert_eq!(RelativeUnit::from_observed(rejected), None);
    }
    assert_eq!(RelativeUnit::from_wire_code(0), None);
    assert_eq!(RelativeUnit::from_wire_code(9), None);
}
#[test]
fn finite_relative_values_preserve_signed_zero_and_ieee_boundaries() {
    for value in [
        0.0,
        -0.0,
        f64::from_bits(1),
        -f64::from_bits(1),
        f64::MAX,
        -f64::MAX,
    ] {
        let checked = FiniteRelativeNumber::new(value).unwrap();
        assert_eq!(checked.bits(), value.to_bits());
        assert_eq!(FiniteRelativeNumber::from_bits(checked.bits()), Ok(checked));
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            FiniteRelativeNumber::new(value),
            Err(RelativeTimeError::NonFinite)
        );
        assert_eq!(
            FiniteRelativeNumber::from_bits(value.to_bits()),
            Err(RelativeTimeError::NonFinite)
        );
    }
}
#[test]
fn captured_relative_profiles_cover_all_fourteen_locales_units_and_styles() {
    assert_eq!(profiles().available_locales().len(), 14);
    for locale in profiles().available_locales() {
        for style in RelativeStyle::ALL {
            let configuration = configuration(locale, *style, RelativeNumeric::Always);
            for unit in RelativeUnit::ALL {
                for value in [-2.0, -1.0, -0.0, 0.0, 1.0, 2.0, 1.25] {
                    let result = parts(&configuration, value, *unit);
                    assert!(
                        !result.to_text().unwrap().is_empty(),
                        "{locale} {style:?} {unit:?}"
                    );
                    for part in result.parts() {
                        assert!(!part.text().is_empty());
                        if let Some(actual_unit) = part.unit() {
                            assert_eq!(actual_unit, *unit);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn polish_relative_locale_uses_existing_decimal_plural_and_grouping_owners() {
    let requested = vec![locale("pl-PL"), locale("pl-PL-u-nu-latn")];
    assert_eq!(
        profiles()
            .supported_locales(
                &requested,
                LocaleMatcher::Lookup,
                &PartitionLimits::HOST_ABI
            )
            .unwrap()
            .as_ref(),
        requested.as_slice()
    );
    for requested_locale in ["pl", "pl-PL", "pl-PL-u-nu-latn"] {
        let configuration = configuration(
            requested_locale,
            RelativeStyle::Long,
            RelativeNumeric::Always,
        );
        assert_eq!(configuration.locale().formatting().as_str(), "pl");
        assert_eq!(configuration.locale().numbering_system(), "latn");
        assert_eq!(
            configuration.locale().resolved().as_str(),
            if requested_locale.contains("-u-") {
                "pl-u-nu-latn"
            } else {
                "pl"
            }
        );
        let four_digits = parts(&configuration, 1000.0, RelativeUnit::Second);
        assert_eq!(four_digits.to_text().unwrap(), "za 1000 sekund");
        assert_eq!(
            four_digits
                .parts()
                .iter()
                .map(|part| (part.kind(), part.text(), part.unit()))
                .collect::<Vec<_>>(),
            vec![
                (NumberPartKind::Literal, "za ", None),
                (NumberPartKind::Integer, "1000", Some(RelativeUnit::Second)),
                (NumberPartKind::Literal, " sekund", None),
            ]
        );
        let five_digits = parts(&configuration, 10000.0, RelativeUnit::Second);
        assert_eq!(five_digits.to_text().unwrap(), "za 10\u{a0}000 sekund");
        assert_eq!(
            five_digits
                .parts()
                .iter()
                .map(|part| (part.kind(), part.text(), part.unit()))
                .collect::<Vec<_>>(),
            vec![
                (NumberPartKind::Literal, "za ", None),
                (NumberPartKind::Integer, "10", Some(RelativeUnit::Second)),
                (NumberPartKind::Group, "\u{a0}", Some(RelativeUnit::Second)),
                (NumberPartKind::Integer, "000", Some(RelativeUnit::Second)),
                (NumberPartKind::Literal, " sekund", None),
            ]
        );
        let fraction = parts(&configuration, 1.25, RelativeUnit::Second);
        assert_eq!(fraction.to_text().unwrap(), "za 1,25 sekundy");
        assert_eq!(
            fraction
                .parts()
                .iter()
                .map(|part| (part.kind(), part.text(), part.unit()))
                .collect::<Vec<_>>(),
            vec![
                (NumberPartKind::Literal, "za ", None),
                (NumberPartKind::Integer, "1", Some(RelativeUnit::Second)),
                (NumberPartKind::Decimal, ",", Some(RelativeUnit::Second)),
                (NumberPartKind::Fraction, "25", Some(RelativeUnit::Second)),
                (NumberPartKind::Literal, " sekundy", None),
            ]
        );
    }
}

#[test]
fn polish_relative_patterns_preserve_categories_auto_and_direction_in_all_styles() {
    for (style, one, few, many, fractional) in [
        (
            RelativeStyle::Long,
            "sekundę",
            "sekundy",
            "sekund",
            "sekundy",
        ),
        (RelativeStyle::Short, "sek.", "sek.", "sek.", "sek."),
        (RelativeStyle::Narrow, "s", "s", "s", "s"),
    ] {
        let always = configuration("pl-PL", style, RelativeNumeric::Always);
        for (value, number, suffix) in [
            (1.0, "1", one),
            (2.0, "2", few),
            (4.0, "4", few),
            (5.0, "5", many),
            (12.0, "12", many),
            (22.0, "22", few),
            (1000.0, "1000", many),
            (10000.0, "10\u{a0}000", many),
            (1.25, "1,25", fractional),
        ] {
            assert_eq!(
                parts(&always, value, RelativeUnit::Second)
                    .to_text()
                    .unwrap(),
                format!("za {number} {suffix}")
            );
            assert_eq!(
                parts(&always, -value, RelativeUnit::Second)
                    .to_text()
                    .unwrap(),
                format!("{number} {suffix} temu")
            );
        }
        assert_eq!(
            parts(&always, 0.0, RelativeUnit::Second).to_text().unwrap(),
            format!("za 0 {many}")
        );
        assert_eq!(
            parts(&always, -0.0, RelativeUnit::Second)
                .to_text()
                .unwrap(),
            format!("0 {many} temu")
        );
        let auto = configuration("pl-PL", style, RelativeNumeric::Auto);
        for zero in [0.0, -0.0] {
            let result = parts(&auto, zero, RelativeUnit::Second);
            assert_eq!(result.parts(), &[RelativePart::Literal("teraz".into())]);
        }
        let (yesterday, today) = if style == RelativeStyle::Narrow {
            ("wcz.", "dziś")
        } else {
            ("wczoraj", "dzisiaj")
        };
        for (value, text) in [(-1.0, yesterday), (0.0, today), (1.0, "jutro")] {
            let result = parts(&auto, value, RelativeUnit::Day);
            assert_eq!(result.parts(), &[RelativePart::Literal(text.into())]);
        }
        assert_eq!(
            parts(&auto, 1.25, RelativeUnit::Second).to_text().unwrap(),
            format!("za 1,25 {fractional}")
        );
    }
}
#[test]
fn numeric_always_matches_pinned_english_eight_unit_vectors() {
    // Pinned Test262 en-us-numeric-always.js format and formatToParts vectors.
    let configuration = configuration("en-US", RelativeStyle::Long, RelativeNumeric::Always);
    for unit in RelativeUnit::ALL {
        let name = unit.name();
        assert_eq!(
            parts(&configuration, 1000.0, *unit).to_text().unwrap(),
            format!("in 1,000 {name}s")
        );
        assert_eq!(
            parts(&configuration, 1.0, *unit).to_text().unwrap(),
            format!("in 1 {name}")
        );
        assert_eq!(
            parts(&configuration, -1.0, *unit).to_text().unwrap(),
            format!("1 {name} ago")
        );
        assert_eq!(
            parts(&configuration, -1000.0, *unit).to_text().unwrap(),
            format!("1,000 {name}s ago")
        );
        let result = parts(&configuration, 1000.0, *unit);
        let signature: Vec<_> = result
            .parts()
            .iter()
            .map(|part| (part.kind(), part.text(), part.unit()))
            .collect();
        assert_eq!(
            signature,
            vec![
                (NumberPartKind::Literal, "in ", None),
                (NumberPartKind::Integer, "1", Some(*unit)),
                (NumberPartKind::Group, ",", Some(*unit)),
                (NumberPartKind::Integer, "000", Some(*unit)),
                (NumberPartKind::Literal, result.parts()[4].text(), None),
            ]
        );
        assert_eq!(result.parts()[4].text(), format!(" {name}s"));
    }
}
#[test]
fn signed_zero_uses_past_numeric_pattern_and_auto_zero_literal() {
    let always = configuration("en-US", RelativeStyle::Long, RelativeNumeric::Always);
    assert_eq!(
        parts(&always, 0.0, RelativeUnit::Day).to_text().unwrap(),
        "in 0 days"
    );
    assert_eq!(
        parts(&always, -0.0, RelativeUnit::Day).to_text().unwrap(),
        "0 days ago"
    );
    let auto = configuration("en-US", RelativeStyle::Long, RelativeNumeric::Auto);
    for zero in [0.0, -0.0] {
        let result = parts(&auto, zero, RelativeUnit::Day);
        assert_eq!(result.parts(), &[RelativePart::Literal("today".into())]);
    }
    assert_eq!(
        parts(&auto, -1.0, RelativeUnit::Day).to_text().unwrap(),
        "yesterday"
    );
    assert_eq!(
        parts(&auto, 1.0, RelativeUnit::Day).to_text().unwrap(),
        "tomorrow"
    );
    assert_eq!(
        parts(&auto, 1.25, RelativeUnit::Day).to_text().unwrap(),
        "in 1.25 days"
    );
}
#[test]
fn arabic_one_and_two_are_genuine_literal_only_numeric_patterns() {
    let configuration = configuration("ar", RelativeStyle::Long, RelativeNumeric::Always);
    assert_eq!(
        parts(&configuration, 1.0, RelativeUnit::Day).parts(),
        &[RelativePart::Literal("خلال يوم واحد".into())]
    );
    assert_eq!(
        parts(&configuration, 2.0, RelativeUnit::Day).parts(),
        &[RelativePart::Literal("خلال يومين".into())]
    );
    assert_eq!(
        parts(&configuration, -1.0, RelativeUnit::Day).parts(),
        &[RelativePart::Literal("قبل يوم واحد".into())]
    );
    let zero = parts(&configuration, 0.0, RelativeUnit::Day);
    assert!(zero
        .parts()
        .iter()
        .any(|part| part.unit() == Some(RelativeUnit::Day)));
    assert!(zero.to_text().unwrap().starts_with("خلال "));
}
#[test]
fn cardinal_choice_and_decimal_parts_share_default_three_digit_rounding() {
    let configuration = configuration("en-US", RelativeStyle::Long, RelativeNumeric::Always);
    assert_eq!(
        parts(&configuration, 1.0004, RelativeUnit::Day)
            .to_text()
            .unwrap(),
        "in 1 day"
    );
    assert_eq!(
        parts(&configuration, 1.0005, RelativeUnit::Day)
            .to_text()
            .unwrap(),
        "in 1.001 days"
    );
    assert_eq!(
        parts(&configuration, -0.0001, RelativeUnit::Day)
            .to_text()
            .unwrap(),
        "0 days ago"
    );
    assert_eq!(
        parts(&configuration, 123456.78, RelativeUnit::Day)
            .to_text()
            .unwrap(),
        "in 123,456.78 days"
    );
}
#[test]
fn relative_styles_and_french_auto_offsets_keep_genuine_text() {
    let short = configuration("en-US", RelativeStyle::Short, RelativeNumeric::Always);
    let narrow = configuration("en-US", RelativeStyle::Narrow, RelativeNumeric::Always);
    assert_eq!(
        parts(&short, 3.0, RelativeUnit::Day).to_text().unwrap(),
        "in 3 days"
    );
    assert_eq!(
        parts(&narrow, 3.0, RelativeUnit::Day).to_text().unwrap(),
        "in 3d"
    );
    let french = configuration("fr", RelativeStyle::Long, RelativeNumeric::Auto);
    assert_eq!(
        parts(&french, -2.0, RelativeUnit::Day).to_text().unwrap(),
        "avant-hier"
    );
    assert_eq!(
        parts(&french, 2.0, RelativeUnit::Day).to_text().unwrap(),
        "après-demain"
    );
}
#[test]
fn resolver_filters_service_inventory_before_shared_number_resolution() {
    let en_gb = configuration("en-GB", RelativeStyle::Long, RelativeNumeric::Always);
    assert_eq!(en_gb.locale().formatting().as_str(), "en");
    let unsupported = configuration("zz-ZZ", RelativeStyle::Long, RelativeNumeric::Always);
    assert_eq!(unsupported.locale().formatting().as_str(), "en-US");
    let requested = vec![
        locale("en-GB"),
        locale("zz-ZZ"),
        locale("fr-u-nu-latn"),
        locale("zh-Hant-TW"),
    ];
    let supported = profiles()
        .supported_locales(
            &requested,
            LocaleMatcher::Lookup,
            &PartitionLimits::HOST_ABI,
        )
        .unwrap();
    assert_eq!(
        supported.as_ref(),
        &[
            requested[0].clone(),
            requested[2].clone(),
            requested[3].clone()
        ]
    );
}
#[test]
fn unicode_numbering_and_explicit_option_keep_shared_precedence() {
    let request = NumberLocaleRequest {
        requested: vec![locale("en-GB-u-nu-arab")].into_boxed_slice(),
        matcher: LocaleMatcher::BestFit,
        numbering_system: None,
    };
    let resolved = profiles()
        .resolve_locale(&request, &numbers(), &PartitionLimits::HOST_ABI)
        .unwrap();
    assert_eq!(resolved.resolved().as_str(), "en-u-nu-arab");
    assert_eq!(resolved.numbering_system(), "arab");
    let config = RelativeTimeConfiguration::new(
        resolved,
        RelativeStyle::Long,
        RelativeNumeric::Always,
        &numbers(),
    )
    .unwrap();
    assert!(parts(&config, 12.0, RelativeUnit::Day)
        .to_text()
        .unwrap()
        .contains("١٢"));
    let override_request = NumberLocaleRequest {
        numbering_system: Some(NumberingSystemOption::parse("latn").unwrap()),
        ..request
    };
    let overridden = profiles()
        .resolve_locale(&override_request, &numbers(), &PartitionLimits::HOST_ABI)
        .unwrap();
    assert_eq!(overridden.resolved().as_str(), "en");
    assert_eq!(overridden.numbering_system(), "latn");
}
#[test]
fn finite_extremes_use_shared_decimal_kernel_without_nonfinite_parts() {
    let configuration = configuration("en-US", RelativeStyle::Long, RelativeNumeric::Always);
    for value in [f64::from_bits(1), -f64::from_bits(1), f64::MAX, -f64::MAX] {
        let result = parts(&configuration, value, RelativeUnit::Second);
        assert!(!result.to_text().unwrap().is_empty());
        assert!(result.parts().iter().all(|part| !matches!(
            part.kind(),
            NumberPartKind::Infinity | NumberPartKind::NaN | NumberPartKind::MinusSign
        )));
    }
    // These exercise the finite binary64 -> ECMAScript decimal spelling ->
    // shared rounding/partition bridge, including both exponential thresholds.
    for (value, expected) in [
        (f64::from_bits(1), "in 0 seconds"),
        (-f64::from_bits(1), "0 seconds ago"),
        (1e-7, "in 0 seconds"),
        (1e-6, "in 0 seconds"),
        (-1e-7, "0 seconds ago"),
        (-1e-6, "0 seconds ago"),
        (1e20, "in 100,000,000,000,000,000,000 seconds"),
        (1e21, "in 1,000,000,000,000,000,000,000 seconds"),
        (1e23, "in 100,000,000,000,000,000,000,000 seconds"),
        (-1e21, "1,000,000,000,000,000,000,000 seconds ago"),
    ] {
        assert_eq!(
            parts(&configuration, value, RelativeUnit::Second)
                .to_text()
                .unwrap(),
            expected,
            "finite spelling bridge for bits {:#018x}",
            value.to_bits()
        );
    }
}
#[test]
fn combined_pattern_and_numeric_parts_enforce_bytes_and_part_limits() {
    let configuration = configuration("en-US", RelativeStyle::Long, RelativeNumeric::Always);
    let value = FiniteRelativeNumber::new(1.0).unwrap();
    let bytes = PartitionLimits::new(
        PartitionLimits::HOST_ABI.numeric(),
        NonZeroU32::new(7).unwrap(),
        NonZeroU32::new(100).unwrap(),
    );
    assert!(format_relative_time_parts(
        &configuration,
        value,
        RelativeUnit::Day,
        profiles(),
        &bytes
    )
    .is_err());
    let count = PartitionLimits::new(
        PartitionLimits::HOST_ABI.numeric(),
        NonZeroU32::new(100).unwrap(),
        NonZeroU32::new(2).unwrap(),
    );
    assert!(format_relative_time_parts(
        &configuration,
        value,
        RelativeUnit::Day,
        profiles(),
        &count
    )
    .is_err());
}
#[test]
fn malformed_relative_profile_domains_and_patterns_are_rejected() {
    let source: serde_json::Value =
        serde_json::from_str(include_str!("generated/profile.json")).unwrap();
    let reject = |value: serde_json::Value| {
        assert!(RelativeProfiles::from_json(&value.to_string(), &numbers()).is_err())
    };
    let mut wrong = source.clone();
    wrong["cldr_commit"] = "unreviewed".into();
    reject(wrong);
    let mut wrong = source.clone();
    wrong["locales"][0]["locale"] = "en-GB".into();
    reject(wrong);
    let mut wrong = source.clone();
    wrong["locales"][0]["fields"][1] = wrong["locales"][0]["fields"][0].clone();
    reject(wrong);
    for pattern in ["{1} days", "{0} and {0}", "{0}{", "", "x\0y"] {
        let mut wrong = source.clone();
        wrong["locales"][0]["fields"][0]["past"][0] = pattern.into();
        reject(wrong);
    }
    let mut wrong = source.clone();
    wrong["locales"][0]["fields"][0]["relative"] =
        serde_json::json!([{"offset":3,"value":"future"}]);
    reject(wrong);
    let mut wrong = source.clone();
    wrong["locales"][0]["fields"][0]["relative"] =
        serde_json::json!([{"offset":0,"value":"today"},{"offset":0,"value":"today"}]);
    reject(wrong);
    let mut wrong = source;
    wrong["locales"][0]["fields"][0]["relative"] = serde_json::json!([{"offset":0,"value":"{0}"}]);
    reject(wrong);
}
