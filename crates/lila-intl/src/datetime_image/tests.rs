use super::*;
use crate::{CanonicalLocaleId, CustomProfileId, TimeZoneId, TimeZoneNameStyle, TimeZoneSelection};

fn selected(name: &str) -> (LocaleDataImage, NamedTimeZoneDataImage, DateTimeDataImage) {
    let profile = IntlDataProfile::Custom(CustomProfileId::parse(name).unwrap());
    let locale = LocaleDataImage::for_profile(profile.clone()).unwrap();
    let named = NamedTimeZoneDataImage::for_profile(profile.clone()).unwrap();
    let image = DateTimeDataImage::for_profile(profile, &locale, &named).unwrap();
    (locale, named, image)
}
fn plan(
    image: &DateTimeDataImage,
    requested: &str,
    calendar: Option<DateTimeCalendar>,
    selection: DateTimeStyleSelection,
) -> DateTimePlanRequest {
    DateTimePlanRequest {
        locale: image
            .resolve_locale(DateTimeLocaleRequest {
                requested: vec![CanonicalLocaleId::from_data(requested).unwrap()],
                matcher: DateTimeLocaleMatcher::Lookup,
                calendar: calendar
                    .map(|calendar| DateTimeKeyword::parse(calendar.as_str()).unwrap()),
                numbering_system: None,
                hour_cycle: DateTimeHourCyclePreference::Default,
            })
            .unwrap(),
        time_zone: TimeZoneSelection::Named(TimeZoneId::parse("UTC").unwrap()),
        selection,
        matcher: DateTimeFormatMatcher::Basic,
        required: DateTimeRequired::Any,
        defaults: DateTimeDefaults::Date,
    }
}
fn date(year: i32, month: u8, day: u8) -> DateTimeInput {
    DateTimeInput::Plain(
        DateTimePlainInput::new(
            DateTimeValueKind::PlainDate,
            DateTimeIsoFields {
                year,
                month,
                day,
                hour: 12,
                minute: 0,
                second: 0,
                nanosecond: 0,
            },
        )
        .unwrap(),
    )
}

#[test]
fn selected_iana_closure_prunes_localized_names_and_binds_dst_plans_to_available_transitions() {
    let (locale, full_named, full) = selected("selected-date-zone-closure");
    let IntlDataProfile::Custom(id) = full.profile() else {
        unreachable!()
    };
    let named = NamedTimeZoneDataImage::for_custom_projection(
        id,
        &[TimeZoneId::parse("US/Eastern").unwrap()],
    )
    .unwrap();
    let image = DateTimeDataImage::for_profile(full.profile().clone(), &locale, &named).unwrap();
    let envelope = DataImageEnvelope::decode(
        image.bytes(),
        DataImageComponent::DateTime,
        DATETIME_IMAGE_MARKERS,
    )
    .unwrap();
    let (native, kernels) = split_payload(envelope.blob()).unwrap();
    assert_eq!(kernels, PINNED_CALENDAR);
    let authority = LocaleCanonicalizationData::from_image(&locale).unwrap();
    let catalogue =
        projection::admit(native, image.profile(), &locale, &named, &authority).unwrap();
    let raw: serde_json::Value = serde_json::from_slice(catalogue.bytes()).unwrap();
    let source: serde_json::Value = serde_json::from_slice(PINNED_NATIVE).unwrap();
    for field in [
        "calendar_pool",
        "numbering_systems",
        "algorithmic_fields",
        "zone_geography",
    ] {
        assert_eq!(raw[field], source[field]);
    }
    for (pool, original) in raw["zone_name_pool"]
        .as_array()
        .unwrap()
        .iter()
        .zip(source["zone_name_pool"].as_array().unwrap())
    {
        assert!(
            pool["zones"].as_array().unwrap().len() < original["zones"].as_array().unwrap().len()
        );
        for field in ["zones", "metazones"] {
            for row in pool[field].as_array().unwrap() {
                assert_eq!(
                    row,
                    original[field]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|source| source["identifier"] == row["identifier"])
                        .unwrap()
                );
            }
        }
    }
    assert!(image.bytes().len() < full.bytes().len());
    assert_eq!(
        image.provider_ref().available_calendar_kernels(),
        DateTimeCalendar::ALL
    );
    assert_eq!(
        image
            .provider_ref()
            .available_numbering_system_kernels()
            .len(),
        78
    );
    let style = DateTimeStyleSelection::Components(DateTimeComponents {
        hour: Some(DateTimeNumericWidth::Numeric),
        minute: Some(DateTimeNumericWidth::Numeric),
        second: Some(DateTimeNumericWidth::Numeric),
        time_zone_name: Some(TimeZoneNameStyle::Long),
        ..Default::default()
    });
    let mut request = plan(&image, "fr-u-hc-h23", None, style.clone());
    request.time_zone = TimeZoneSelection::Named(TimeZoneId::parse("US/Eastern").unwrap());
    let actual = image.select_plan(request.clone()).unwrap().plan;
    let original = full.select_plan(request.clone()).unwrap().plan;
    for seconds in [1_609_459_200, 1_625_097_600, 1_710_053_999, 1_710_054_000] {
        assert_eq!(
            image
                .format_parts(DateTimeFormatRequest {
                    plan: actual.clone(),
                    input: instant(seconds)
                })
                .unwrap(),
            full.format_parts(DateTimeFormatRequest {
                plan: original.clone(),
                input: instant(seconds)
            })
            .unwrap()
        );
    }
    let range = DateTimeRangeRequest {
        plan: actual.clone(),
        start: instant(1_710_053_999),
        end: instant(1_710_054_000),
    };
    assert_eq!(
        image.format_range_parts(range.clone()).unwrap(),
        full.format_range_parts(DateTimeRangeRequest {
            plan: original.clone(),
            ..range
        })
        .unwrap()
    );
    request.time_zone = TimeZoneSelection::Named(TimeZoneId::parse("Europe/Paris").unwrap());
    assert!(matches!(
        image.select_plan(request),
        Err(DateTimeFormatError::UnavailableTimeZone(_))
    ));
    assert!(DateTimeDataImage::from_bytes(full.bytes(), &locale, &named).is_err());
    assert!(DateTimeDataImage::from_bytes(image.bytes(), &locale, &full_named).is_err());
    let systems = [crate::number_format::NumberingSystemOption::parse("deva").unwrap()];
    let composed = DateTimeDataImage::for_custom_numbering_projection(
        id,
        Some(&[LocaleId::parse("fr").unwrap()]),
        Some(&[DateTimeCalendar::Chinese]),
        &systems,
        &locale,
        &named,
    )
    .unwrap();
    let mut selected = plan(
        &composed,
        "fr-u-nu-deva",
        Some(DateTimeCalendar::Chinese),
        style,
    );
    selected.time_zone = TimeZoneSelection::Named(TimeZoneId::parse("America/New_York").unwrap());
    assert!(format(&composed, selected, instant(1_710_054_000))
        .to_formatted_string()
        .chars()
        .any(|digit| ('\u{0966}'..='\u{096f}').contains(&digit)));
    let decoded = DateTimeDataImage::from_bytes(image.bytes(), &locale, &named).unwrap();
    let weak = Arc::downgrade(&named.zones());
    drop(image);
    drop(named);
    drop(locale);
    drop(composed);
    assert!(weak.upgrade().is_some());
    assert_eq!(
        decoded
            .format_parts(DateTimeFormatRequest {
                plan: actual,
                input: instant(1_710_054_000)
            })
            .unwrap(),
        full.format_parts(DateTimeFormatRequest {
            plan: original,
            input: instant(1_710_054_000)
        })
        .unwrap()
    );
}

#[test]
fn selected_calendar_associations_compact_real_service_pools_and_keep_original_defaults() {
    let (locale, named, full) = selected("localized-calendar-subset");
    let IntlDataProfile::Custom(id) = full.profile() else {
        unreachable!()
    };
    let locales = [
        LocaleId::parse("zh").unwrap(),
        LocaleId::parse("ar-EG").unwrap(),
    ];
    let image = DateTimeDataImage::for_custom_data_projection(
        id,
        Some(&locales),
        &[DateTimeCalendar::Japanese, DateTimeCalendar::Chinese],
        &locale,
        &named,
    )
    .unwrap();
    let same = DateTimeDataImage::for_custom_data_projection(
        id,
        Some(&[locales[1].clone(), locales[0].clone()]),
        &[DateTimeCalendar::Chinese, DateTimeCalendar::Japanese],
        &locale,
        &named,
    )
    .unwrap();
    assert_eq!(image.bytes().as_ref(), same.bytes().as_ref());
    let envelope = DataImageEnvelope::decode(
        image.bytes(),
        DataImageComponent::DateTime,
        DATETIME_IMAGE_MARKERS,
    )
    .unwrap();
    let (native, kernels) = split_payload(envelope.blob()).unwrap();
    assert_eq!(kernels, PINNED_CALENDAR);
    let authority = LocaleCanonicalizationData::from_image(&locale).unwrap();
    let catalogue =
        projection::admit(native, image.profile(), &locale, &named, &authority).unwrap();
    let raw: serde_json::Value = serde_json::from_slice(catalogue.bytes()).unwrap();
    let source: serde_json::Value = serde_json::from_slice(PINNED_NATIVE).unwrap();
    assert_eq!(
        catalogue.calendar_types().unwrap(),
        [DateTimeCalendar::Chinese, DateTimeCalendar::Japanese]
    );
    for field in [
        "numbering_systems",
        "numbering_supplement",
        "era_supplement",
        "algorithmic_fields",
        "zone_geography",
    ] {
        assert_eq!(raw[field], source[field]);
    }
    let mut used = std::collections::BTreeSet::new();
    for row in raw["locales"].as_array().unwrap() {
        let original = source["locales"]
            .as_array()
            .unwrap()
            .iter()
            .find(|original| original["locale"] == row["locale"])
            .unwrap();
        let default = original["calendar_preferences"]
            .as_array()
            .unwrap()
            .iter()
            .find_map(|value| value.as_str().and_then(DateTimeCalendar::parse))
            .unwrap();
        let expected = original["calendar_refs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|reference| {
                reference[0].as_str().is_some_and(|name| {
                    name == default.as_str() || name == "chinese" || name == "japanese"
                })
            })
            .collect::<Vec<_>>();
        let retained = row["calendar_refs"].as_array().unwrap();
        assert_eq!(retained.len(), expected.len());
        for (reference, expected) in retained.iter().zip(expected) {
            assert_eq!(reference[0], expected[0]);
            let index = reference[1].as_u64().unwrap() as usize;
            used.insert(index);
            assert_eq!(
                raw["calendar_pool"][index],
                source["calendar_pool"][expected[1].as_u64().unwrap() as usize]
            );
        }
        assert_eq!(
            row["calendar_preferences"],
            original["calendar_preferences"]
        );
        assert_eq!(
            raw["zone_name_pool"][row["zone_name_ref"].as_u64().unwrap() as usize],
            source["zone_name_pool"][original["zone_name_ref"].as_u64().unwrap() as usize]
        );
    }
    assert_eq!(
        used.into_iter().collect::<Vec<_>>(),
        (0..raw["calendar_pool"].as_array().unwrap().len()).collect::<Vec<_>>()
    );
    let locale_only =
        DateTimeDataImage::for_custom_projection(id, &locales, &locale, &named).unwrap();
    assert!(image.bytes().len() < locale_only.bytes().len());
    for name in ["ar-EG", "en-US", "zh"] {
        for calendar in [
            None,
            Some(DateTimeCalendar::Chinese),
            Some(DateTimeCalendar::Japanese),
        ] {
            for input in [date(2023, 3, 22), date(2024, 2, 10)] {
                assert_eq!(
                    format(
                        &image,
                        plan(&image, name, calendar, date_fields()),
                        input.clone()
                    ),
                    format(&full, plan(&full, name, calendar, date_fields()), input)
                );
            }
        }
    }
}

#[test]
fn localized_calendar_fallback_is_separate_from_full_kernel_and_locale_information_availability() {
    let (locale, named, full) = selected("calendar-kernel-service-separation");
    let IntlDataProfile::Custom(id) = full.profile() else {
        unreachable!()
    };
    let image = DateTimeDataImage::for_custom_data_projection(
        id,
        None,
        &[DateTimeCalendar::Chinese],
        &locale,
        &named,
    )
    .unwrap();
    let authority = LocaleCanonicalizationData::from_image(&locale).unwrap();
    let envelope = DataImageEnvelope::decode(
        image.bytes(),
        DataImageComponent::DateTime,
        DATETIME_IMAGE_MARKERS,
    )
    .unwrap();
    let (native, _) = split_payload(envelope.blob()).unwrap();
    let catalogue =
        projection::admit(native, image.profile(), &locale, &named, &authority).unwrap();
    let source: serde_json::Value = serde_json::from_slice(PINNED_NATIVE).unwrap();
    assert_eq!(
        catalogue.locales().len(),
        source["locales"].as_array().unwrap().len()
    );
    assert_eq!(
        image.provider_ref().available_calendar_kernels(),
        DateTimeCalendar::ALL
    );
    assert!(!image
        .provider_ref()
        .available_calendars()
        .unwrap()
        .contains(&DateTimeCalendar::Japanese));
    let selected = plan(&image, "en-US-u-ca-japanese", None, date_fields());
    let default = plan(&image, "en-US", None, date_fields());
    assert_eq!(selected.locale, default.locale);
    assert_eq!(
        plan(
            &image,
            "en-US",
            Some(DateTimeCalendar::Japanese),
            date_fields()
        )
        .locale,
        default.locale
    );
    let mut forged = default.clone();
    forged.locale.calendar = DateTimeCalendar::Japanese;
    forged.locale.locale = CanonicalLocaleId::from_data("en-US-u-ca-japanese").unwrap();
    assert!(matches!(
        image.select_plan(forged),
        Err(DateTimeFormatError::InvalidPlan(_))
    ));
    let information = crate::NativeLocaleInformationDataImage::for_profile(
        image.profile().clone(),
        &locale,
        &image,
    )
    .unwrap();
    let calendars = information
        .resolve_calendars(
            crate::LocaleCalendarsRequest::new(CanonicalLocaleId::from_data("th-TH").unwrap())
                .unwrap(),
        )
        .unwrap();
    assert_eq!(
        calendars
            .names()
            .iter()
            .map(|name| name.as_ref())
            .collect::<Vec<_>>(),
        ["buddhist", "gregory"]
    );
    assert!(information.uses_foundations(&locale, &image.provider()));
    let request = plan(&image, "zh", Some(DateTimeCalendar::Chinese), date_fields());
    let portable = image.select_plan(request.clone()).unwrap().plan;
    let expected = image
        .format_parts(DateTimeFormatRequest {
            plan: portable.clone(),
            input: date(2024, 2, 10),
        })
        .unwrap();
    let foreign = full
        .select_plan(plan(
            &full,
            "zh",
            Some(DateTimeCalendar::Chinese),
            date_fields(),
        ))
        .unwrap()
        .plan;
    assert!(image
        .format_parts(DateTimeFormatRequest {
            plan: foreign,
            input: date(2024, 2, 10)
        })
        .is_err());
    let redecoded = DateTimeDataImage::from_bytes(image.bytes(), &locale, &named).unwrap();
    drop(image);
    drop(information);
    drop(locale);
    drop(named);
    assert_eq!(
        redecoded
            .format_parts(DateTimeFormatRequest {
                plan: portable,
                input: date(2024, 2, 10)
            })
            .unwrap(),
        expected
    );
}

#[test]
fn selected_numbering_symbols_preserve_actual_locale_defaults_and_calendar_override_kernels() {
    let (locale, named, full) = selected("date-numbering-service-closure");
    let IntlDataProfile::Custom(id) = full.profile() else {
        unreachable!()
    };
    let requested = [
        LocaleId::parse("fr").unwrap(),
        LocaleId::parse("ar-EG").unwrap(),
    ];
    let systems = [crate::number_format::NumberingSystemOption::parse("deva").unwrap()];
    let image = DateTimeDataImage::for_custom_numbering_projection(
        id,
        Some(&requested),
        Some(&[DateTimeCalendar::Chinese]),
        &systems,
        &locale,
        &named,
    )
    .unwrap();
    let envelope = DataImageEnvelope::decode(
        image.bytes(),
        DataImageComponent::DateTime,
        DATETIME_IMAGE_MARKERS,
    )
    .unwrap();
    let (native, kernels) = split_payload(envelope.blob()).unwrap();
    assert_eq!(kernels, PINNED_CALENDAR);
    let authority = LocaleCanonicalizationData::from_image(&locale).unwrap();
    let catalogue =
        projection::admit(native, image.profile(), &locale, &named, &authority).unwrap();
    let raw: serde_json::Value = serde_json::from_slice(catalogue.bytes()).unwrap();
    let source: serde_json::Value = serde_json::from_slice(PINNED_NATIVE).unwrap();
    assert_eq!(raw["numbering_systems"], source["numbering_systems"]);
    assert_eq!(raw["algorithmic_fields"], source["algorithmic_fields"]);
    assert_eq!(raw["numbering_supplement"], source["numbering_supplement"]);
    for row in raw["locales"].as_array().unwrap() {
        let original = source["locales"]
            .as_array()
            .unwrap()
            .iter()
            .find(|original| original["locale"] == row["locale"])
            .unwrap();
        for field in ["decimal_separators", "minus_signs"] {
            let expected = original[field]
                .as_array()
                .unwrap()
                .iter()
                .filter(|pair| pair[0] == original["default_numbering"] || pair[0] == "deva")
                .cloned()
                .collect::<Vec<_>>();
            assert_eq!(row[field], serde_json::Value::Array(expected));
        }
    }
    let baseline = DateTimeDataImage::for_custom_data_projection(
        id,
        Some(&requested),
        &[DateTimeCalendar::Chinese],
        &locale,
        &named,
    )
    .unwrap();
    assert!(image.bytes().len() < baseline.bytes().len());
    assert_eq!(
        image
            .provider_ref()
            .available_numbering_system_kernels()
            .len(),
        78
    );
    assert!(image
        .provider_ref()
        .available_numbering_system_kernels()
        .iter()
        .any(|name| name.as_ref() == "beng"));
    for (name, default) in [("fr", "latn"), ("en-US", "latn"), ("ar-EG", "arab")] {
        for system in ["deva", default] {
            let name = format!("{name}-u-nu-{system}");
            for calendar in [None, Some(DateTimeCalendar::Chinese)] {
                let actual = plan(&image, &name, calendar, date_fields());
                assert_eq!(actual.locale.numbering_system.as_str(), system);
                assert_eq!(
                    format(&image, actual, date(2024, 2, 10)),
                    format(
                        &full,
                        plan(&full, &name, calendar, date_fields()),
                        date(2024, 2, 10)
                    )
                );
            }
        }
        let omitted = plan(&image, &format!("{name}-u-nu-beng"), None, date_fields());
        assert_eq!(omitted.locale.numbering_system.as_str(), default);
        assert_eq!(omitted.locale.locale.as_str(), name);
        let mut forged = omitted.clone();
        forged.locale.numbering_system = DateTimeKeyword::parse("beng").unwrap();
        forged.locale.locale = CanonicalLocaleId::from_data(format!("{name}-u-nu-beng")).unwrap();
        assert!(image.select_plan(forged).is_err());
    }
    let devanagari = format(
        &image,
        plan(&image, "fr-u-nu-deva", None, date_fields()),
        date(2024, 2, 10),
    )
    .to_formatted_string();
    assert!(devanagari
        .chars()
        .any(|value| ('\u{0966}'..='\u{096f}').contains(&value)));
    let information = crate::NativeLocaleInformationDataImage::for_profile(
        image.profile().clone(),
        &locale,
        &image,
    )
    .unwrap();
    assert!(information.uses_foundations(&locale, &image.provider()));
    assert_eq!(image.numbering_system_selection(), Some(systems.as_slice()));
}
fn instant(seconds: i64) -> DateTimeInput {
    DateTimeInput::Exact(DateTimeExactInput::new(DateTimeValueKind::Instant, seconds, 0).unwrap())
}
fn date_fields() -> DateTimeStyleSelection {
    DateTimeStyleSelection::Components(DateTimeComponents {
        year: Some(DateTimeNumericWidth::Numeric),
        month: Some(DateTimeMonthWidth::Numeric),
        day: Some(DateTimeNumericWidth::Numeric),
        ..Default::default()
    })
}
fn format(
    image: &DateTimeDataImage,
    request: DateTimePlanRequest,
    input: DateTimeInput,
) -> DateTimeParts {
    image
        .format_parts(DateTimeFormatRequest {
            plan: image.select_plan(request).unwrap().plan,
            input,
        })
        .unwrap()
}
fn part(parts: &DateTimeParts, kind: DateTimePartKind) -> &str {
    &parts
        .parts
        .iter()
        .find(|part| part.kind == kind)
        .unwrap()
        .value
}

#[test]
fn selected_native_and_calendar_payloads_keep_real_lunar_era_and_cache_boundary_fields() {
    let (locale, named, original) = selected("date-native-owner");
    let bytes: Arc<[u8]> = original.bytes().as_ref().to_vec().into();
    let image = DateTimeDataImage::from_bytes(bytes.clone(), &locale, &named).unwrap();
    assert!(image.uses_locale(&locale));
    assert!(image.uses_named_time_zones(&named.zones()));
    drop(bytes);
    drop(original);
    drop(locale);
    drop(named);
    for &calendar in DateTimeCalendar::ALL {
        let parts = format(
            &image,
            plan(&image, "en-US", Some(calendar), date_fields()),
            date(2024, 2, 10),
        );
        assert!(!parts.to_formatted_string().is_empty(), "{calendar:?}");
    }
    let full = DateTimeStyleSelection::Styles(
        DateTimeStyles::new(Some(DateTimeStyle::Full), None).unwrap(),
    );
    let chinese = format(
        &image,
        plan(&image, "zh", Some(DateTimeCalendar::Chinese), full),
        date(2023, 3, 22),
    );
    assert_eq!(chinese.to_formatted_string(), "2023癸卯年闰二月初一星期三");
    let japanese = format(
        &image,
        plan(
            &image,
            "en-US",
            Some(DateTimeCalendar::Japanese),
            DateTimeStyleSelection::Components(DateTimeComponents {
                era: Some(DateTimeTextWidth::Long),
                ..match date_fields() {
                    DateTimeStyleSelection::Components(fields) => fields,
                    DateTimeStyleSelection::Styles(_) => unreachable!(),
                }
            }),
        ),
        date(2019, 5, 1),
    );
    assert_eq!(part(&japanese, DateTimePartKind::Year), "1");
    assert_eq!(part(&japanese, DateTimePartKind::Era), "Reiwa");
    for (year, iso) in [("1300", (1882, 11, 12)), ("1600", (2173, 12, 7))] {
        let parts = format(
            &image,
            plan(
                &image,
                "en-US",
                Some(DateTimeCalendar::IslamicUmalqura),
                date_fields(),
            ),
            date(iso.0, iso.1, iso.2),
        );
        assert_eq!(part(&parts, DateTimePartKind::Year), year);
        assert_eq!(part(&parts, DateTimePartKind::Month), "1");
        assert_eq!(part(&parts, DateTimePartKind::Day), "1");
    }
    let aliased = image
        .resolve_locale(DateTimeLocaleRequest {
            requested: Vec::new(),
            matcher: DateTimeLocaleMatcher::Lookup,
            calendar: Some(DateTimeKeyword::parse("islamicc").unwrap()),
            numbering_system: None,
            hour_cycle: DateTimeHourCyclePreference::Default,
        })
        .unwrap();
    assert_eq!(aliased.calendar, DateTimeCalendar::IslamicCivil);
}

#[test]
fn portable_plan_wire_uses_selected_data_identity_before_projection_and_range_kind_checks() {
    let (locale, named, first) = selected("date-plan-first");
    let same = DateTimeDataImage::from_bytes(first.bytes(), &locale, &named).unwrap();
    let (_, _, foreign) = selected("date-plan-foreign");
    let request = DateTimeFormatRequest {
        plan: first
            .select_plan(plan(&first, "en-US", None, date_fields()))
            .unwrap()
            .plan,
        input: date(2020, 1, 25),
    };
    let decoded = DateTimeFormatRequest::decode(&request.encode().unwrap()).unwrap();
    // An independent owner with identical admitted bytes accepts a portable plan.
    assert_eq!(
        same.format_parts(decoded.clone())
            .unwrap()
            .to_formatted_string(),
        "1/25/2020"
    );
    assert!(matches!(
        foreign.format_parts(decoded),
        Err(DateTimeFormatError::InvalidPlan(_))
    ));
    let range = DateTimeRangeRequest {
        plan: request.plan,
        start: request.input,
        end: instant(0),
    };
    assert_eq!(
        first.format_range_parts(range.clone()),
        Err(DateTimeFormatError::InputKindMismatch)
    );
    assert!(matches!(
        foreign.format_range_parts(DateTimeRangeRequest::decode(&range.encode().unwrap()).unwrap()),
        Err(DateTimeFormatError::InvalidPlan(_))
    ));
}

#[test]
fn selected_iana_owner_drives_dst_parts_and_range_and_rejects_foreign_foundation_installation() {
    let (locale, named, image) = selected("date-zone-owner");
    let other_locale = LocaleDataImage::from_bytes(locale.bytes()).unwrap();
    let other_named = NamedTimeZoneDataImage::from_bytes(named.bytes()).unwrap();
    assert!(!image.uses_locale(&other_locale));
    assert!(!image.uses_named_time_zones(&other_named.zones()));
    let mut request = plan(
        &image,
        "ar-EG-u-hc-h23",
        None,
        DateTimeStyleSelection::Components(DateTimeComponents {
            hour: Some(DateTimeNumericWidth::Numeric),
            minute: Some(DateTimeNumericWidth::Numeric),
            second: Some(DateTimeNumericWidth::Numeric),
            time_zone_name: Some(TimeZoneNameStyle::LongOffset),
            ..Default::default()
        }),
    );
    request.time_zone = TimeZoneSelection::Named(TimeZoneId::parse("America/New_York").unwrap());
    let selected = image.select_plan(request).unwrap();
    for (epoch, hour, zone) in [
        (1_710_053_999, "٠١", "غرينتش-٠٥:٠٠"),
        (1_710_054_000, "٠٣", "غرينتش-٠٤:٠٠"),
    ] {
        let parts = image
            .format_parts(DateTimeFormatRequest {
                plan: selected.plan.clone(),
                input: instant(epoch),
            })
            .unwrap();
        assert_eq!(part(&parts, DateTimePartKind::Hour), hour);
        assert_eq!(part(&parts, DateTimePartKind::TimeZoneName), zone);
    }
    let range = image
        .format_range_parts(DateTimeRangeRequest {
            plan: selected.plan,
            start: instant(1_710_053_999),
            end: instant(1_710_054_000),
        })
        .unwrap();
    for (source, expected) in [
        (DateTimeRangeSource::StartRange, "غرينتش-٠٥:٠٠"),
        (DateTimeRangeSource::EndRange, "غرينتش-٠٤:٠٠"),
    ] {
        assert!(range.parts.iter().any(|part| part.source == source
            && part.kind == DateTimePartKind::TimeZoneName
            && part.value == expected));
    }
}

#[test]
fn framed_altered_native_calendar_payloads_and_mixed_profiles_cannot_claim_locked_identity() {
    let locale = embedded_locale_data_image().unwrap();
    let named = embedded_named_time_zone_data_image().unwrap();
    let exact = payload();
    let (native, calendar) = split_payload(&exact).unwrap();
    assert_eq!(native, PINNED_NATIVE);
    assert_eq!(calendar, PINNED_CALENDAR);
    for offset in [16, 24 + PINNED_NATIVE.len()] {
        let mut altered = payload();
        altered[offset] ^= 1;
        let frame = DataImageEnvelope::encode(
            DataImageComponent::DateTime,
            &IntlDataProfile::Minimal,
            DATETIME_IMAGE_MARKERS,
            &altered,
        )
        .unwrap();
        assert!(matches!(
            DateTimeDataImage::from_bytes(frame, &locale, &named),
            Err(IntlDataImageError::Consumer(_))
        ));
    }
    let custom = IntlDataProfile::Custom(CustomProfileId::parse("date-mixed-profile").unwrap());
    assert!(matches!(
        DateTimeDataImage::for_profile(custom, &locale, &named),
        Err(IntlDataImageError::Consumer(_))
    ));
    assert!(matches!(
        DateTimeDataImage::for_profile(IntlDataProfile::Conformance, &locale, &named),
        Err(IntlDataImageError::Consumer(_))
    ));
}

fn projected(
    name: &str,
) -> (
    LocaleDataImage,
    NamedTimeZoneDataImage,
    DateTimeDataImage,
    DateTimeDataImage,
) {
    let (locale, named, full) = selected(name);
    let id = CustomProfileId::parse(name).unwrap();
    let image = DateTimeDataImage::for_custom_projection(
        &id,
        &[
            LocaleId::parse("zh").unwrap(),
            LocaleId::parse("ar-EG").unwrap(),
        ],
        &locale,
        &named,
    )
    .unwrap();
    (locale, named, full, image)
}

#[test]
fn projected_native_rows_compact_both_pools_and_keep_all_global_data_and_calendar_associations() {
    let (locale, named, full, image) = projected("date-pool-projection");
    assert!(image.bytes().len() < full.bytes().len());
    let envelope = DataImageEnvelope::decode(
        image.bytes(),
        DataImageComponent::DateTime,
        DATETIME_IMAGE_MARKERS,
    )
    .unwrap();
    let (native, calendars) = split_payload(envelope.blob()).unwrap();
    assert_eq!(calendars, PINNED_CALENDAR);
    let authority = LocaleCanonicalizationData::from_image(&locale).unwrap();
    let catalogue =
        projection::admit(native, image.profile(), &locale, &named, &authority).unwrap();
    let raw: serde_json::Value = serde_json::from_slice(catalogue.bytes()).unwrap();
    let source: serde_json::Value = serde_json::from_slice(PINNED_NATIVE).unwrap();
    assert_eq!(
        raw["selector"]["locales"],
        serde_json::json!(["ar-EG", "en-US", "zh"])
    );
    assert_eq!(raw["locales"].as_array().unwrap().len(), 3);
    for pool in ["calendar_pool", "zone_name_pool"] {
        assert!(raw[pool].as_array().unwrap().len() < source[pool].as_array().unwrap().len());
    }
    for field in [
        "numbering_systems",
        "numbering_supplement",
        "era_supplement",
        "algorithmic_fields",
        "zone_geography",
    ] {
        assert_eq!(raw[field], source[field], "{field}");
    }
    for row in raw["locales"].as_array().unwrap() {
        assert_eq!(
            row["calendar_refs"].as_array().unwrap().len(),
            DateTimeCalendar::ALL.len()
        );
    }
    assert_eq!(
        image
            .provider_ref()
            .available_calendars()
            .unwrap()
            .as_slice(),
        DateTimeCalendar::ALL
    );
    let supported = image
        .supported_locales(DateTimeSupportedLocalesRequest {
            requested: ["fr", "ar-EG", "zh-Hans-CN", "en-US"]
                .into_iter()
                .map(|name| CanonicalLocaleId::from_data(name).unwrap())
                .collect(),
            matcher: DateTimeLocaleMatcher::Lookup,
        })
        .unwrap();
    assert_eq!(
        supported
            .locales
            .iter()
            .map(CanonicalLocaleId::as_str)
            .collect::<Vec<_>>(),
        ["ar-EG", "zh-Hans-CN", "en-US"]
    );
    assert_eq!(
        plan(&image, "zh-Hans-CN", None, date_fields())
            .locale
            .data_locale
            .as_str(),
        "zh"
    );
    assert_eq!(
        plan(&image, "fr", None, date_fields())
            .locale
            .data_locale
            .as_str(),
        "en-US"
    );
}

#[test]
fn projected_calendars_numbering_and_lunar_fields_match_the_retained_complete_source() {
    let (_, _, full, image) = projected("date-kernel-projection");
    for &calendar in DateTimeCalendar::ALL {
        for locale in ["en-US", "ar-EG", "zh"] {
            assert_eq!(
                format(
                    &image,
                    plan(&image, locale, Some(calendar), date_fields()),
                    date(2024, 2, 10)
                ),
                format(
                    &full,
                    plan(&full, locale, Some(calendar), date_fields()),
                    date(2024, 2, 10)
                ),
                "{locale}/{calendar:?}",
            );
        }
    }
    let source: serde_json::Value = serde_json::from_slice(PINNED_NATIVE).unwrap();
    for row in source["numbering_systems"].as_array().unwrap() {
        let numbering = DateTimeKeyword::parse(row["identifier"].as_str().unwrap()).unwrap();
        let resolve = |owner: &DateTimeDataImage| {
            owner
                .resolve_locale(DateTimeLocaleRequest {
                    requested: vec![CanonicalLocaleId::from_data("en-US").unwrap()],
                    matcher: DateTimeLocaleMatcher::Lookup,
                    calendar: None,
                    numbering_system: Some(numbering.clone()),
                    hour_cycle: DateTimeHourCyclePreference::Default,
                })
                .unwrap()
        };
        let mut projected_plan = plan(&image, "en-US", None, date_fields());
        projected_plan.locale = resolve(&image);
        let mut full_plan = plan(&full, "en-US", None, date_fields());
        full_plan.locale = resolve(&full);
        assert_eq!(
            format(&image, projected_plan, date(2020, 1, 25)),
            format(&full, full_plan, date(2020, 1, 25))
        );
    }
    let full_style = DateTimeStyleSelection::Styles(
        DateTimeStyles::new(Some(DateTimeStyle::Full), None).unwrap(),
    );
    assert_eq!(
        format(
            &image,
            plan(&image, "zh", Some(DateTimeCalendar::Chinese), full_style),
            date(2023, 3, 22)
        )
        .to_formatted_string(),
        "2023癸卯年闰二月初一星期三"
    );
    assert_eq!(
        format(
            &image,
            plan(&image, "ar-EG", None, date_fields()),
            date(2020, 1, 25)
        )
        .to_formatted_string(),
        "٢٥\u{200f}/١\u{200f}/٢٠٢٠"
    );
}

#[test]
fn projected_range_parts_and_portable_plans_retain_actual_foundations_after_input_drop() {
    let (locale, named, full, original) = projected("date-range-projection");
    let image = DateTimeDataImage::from_bytes(original.bytes(), &locale, &named).unwrap();
    assert!(image.uses_locale(&locale));
    assert!(image.uses_named_time_zones(&named.zones()));
    let mut request = plan(
        &image,
        "ar-EG-u-hc-h23",
        None,
        DateTimeStyleSelection::Components(DateTimeComponents {
            hour: Some(DateTimeNumericWidth::Numeric),
            minute: Some(DateTimeNumericWidth::Numeric),
            second: Some(DateTimeNumericWidth::Numeric),
            time_zone_name: Some(TimeZoneNameStyle::LongOffset),
            ..Default::default()
        }),
    );
    request.time_zone = TimeZoneSelection::Named(TimeZoneId::parse("America/New_York").unwrap());
    let selected = image.select_plan(request.clone()).unwrap();
    let full_plan = full.select_plan(request).unwrap().plan;
    let range = DateTimeRangeRequest {
        plan: selected.plan.clone(),
        start: instant(1_710_053_999),
        end: instant(1_710_054_000),
    };
    let expected = full
        .format_range_parts(DateTimeRangeRequest {
            plan: full_plan,
            ..range.clone()
        })
        .unwrap();
    assert_eq!(image.format_range_parts(range.clone()).unwrap(), expected);
    assert!(matches!(
        full.format_range_parts(range.clone()),
        Err(DateTimeFormatError::InvalidPlan(_))
    ));
    let scalar = DateTimeFormatRequest {
        plan: selected.plan,
        input: range.end,
    };
    drop(locale);
    drop(named);
    drop(full);
    drop(original);
    let parts = image
        .format_parts(DateTimeFormatRequest::decode(&scalar.encode().unwrap()).unwrap())
        .unwrap();
    assert_eq!(part(&parts, DateTimePartKind::Hour), "٠٣");
    assert_eq!(part(&parts, DateTimePartKind::TimeZoneName), "غرينتش-٠٤:٠٠");
    assert_eq!(
        image
            .format_range_parts(DateTimeRangeRequest::decode(&range.encode().unwrap()).unwrap())
            .unwrap(),
        expected
    );
}

#[test]
fn projected_admission_rederives_exact_rows_pools_and_foundation_identity_before_publication() {
    let (locale, named, full, image) = projected("date-admission-projection");
    let id = CustomProfileId::parse("date-admission-projection").unwrap();
    for names in [vec![], vec!["es"], vec!["AR-eg", "ar-EG"]] {
        let names = names
            .into_iter()
            .map(|name| LocaleId::parse(name).unwrap())
            .collect::<Vec<_>>();
        assert!(DateTimeDataImage::for_custom_projection(&id, &names, &locale, &named).is_err());
    }
    assert_eq!(
        DateTimeDataImage::from_bytes(full.bytes(), &locale, &named)
            .unwrap()
            .bytes(),
        full.bytes()
    );
    let requested = [
        LocaleId::parse("ar-EG").unwrap(),
        LocaleId::parse("zh").unwrap(),
    ];
    let minimal_locale = embedded_locale_data_image().unwrap();
    let minimal_named = embedded_named_time_zone_data_image().unwrap();
    assert!(
        DateTimeDataImage::for_custom_projection(&id, &requested, &minimal_locale, &named).is_err()
    );
    assert!(
        DateTimeDataImage::for_custom_projection(&id, &requested, &locale, &minimal_named).is_err()
    );
    let (other_locale, other_named, _) = selected("date-other-foundation");
    assert!(DateTimeDataImage::from_bytes(image.bytes(), &other_locale, &other_named).is_err());

    let envelope = DataImageEnvelope::decode(
        image.bytes(),
        DataImageComponent::DateTime,
        DATETIME_IMAGE_MARKERS,
    )
    .unwrap();
    let (native, _) = split_payload(envelope.blob()).unwrap();
    let descriptor_end = 20 + u32::from_le_bytes(native[8..12].try_into().unwrap()) as usize;
    let raw: serde_json::Value = serde_json::from_slice(&native[descriptor_end..]).unwrap();
    for damage in 0..4 {
        let mut raw = raw.clone();
        match damage {
            0 => raw["selector"]["locales"] = serde_json::json!(["en-US"]),
            1 => {
                let extra = raw["calendar_pool"][0].clone();
                raw["calendar_pool"].as_array_mut().unwrap().push(extra);
            }
            2 => raw["locales"][0]["zone_name_ref"] = serde_json::json!(999),
            3 => {
                raw["zone_name_pool"][0]["patterns"]["gmtFormat"] = serde_json::json!("altered {0}")
            }
            _ => unreachable!(),
        }
        let changed = serde_json::to_vec(&raw).unwrap();
        let mut framed_native = native[..descriptor_end].to_vec();
        framed_native[12..20].copy_from_slice(&(changed.len() as u64).to_le_bytes());
        framed_native.extend_from_slice(&changed);
        let bytes = DataImageEnvelope::encode(
            DataImageComponent::DateTime,
            image.profile(),
            DATETIME_IMAGE_MARKERS,
            &payload_with_native(&framed_native).unwrap(),
        )
        .unwrap();
        assert!(
            DateTimeDataImage::from_bytes(bytes, &locale, &named).is_err(),
            "damage {damage}"
        );
    }
}
