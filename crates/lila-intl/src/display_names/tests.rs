use super::*;
use crate::display_names_protocol::*;
use crate::EmbeddedIntlProvider;
use std::sync::OnceLock;

struct Foundation {
    canonicalizer: EmbeddedIntlProvider,
    profiles: DisplayNamesProfiles,
}
fn foundation() -> &'static Foundation {
    static FOUNDATION: OnceLock<Foundation> = OnceLock::new();
    FOUNDATION.get_or_init(|| {
        let canonicalizer = EmbeddedIntlProvider::new().unwrap();
        let profiles = DisplayNamesProfiles::from_pinned_data(&canonicalizer).unwrap();
        Foundation {
            canonicalizer,
            profiles,
        }
    })
}
fn config(
    locale: &str,
    kind: DisplayNamesType,
    style: DisplayNamesStyle,
    fallback: DisplayNamesFallback,
    language: DisplayNamesLanguageDisplay,
) -> CheckedDisplayNamesConfiguration {
    CheckedDisplayNamesConfiguration::new(
        foundation()
            .profiles
            .admit(CanonicalLocaleId::from_data(locale).unwrap())
            .unwrap(),
        DisplayNamesSelection::from_options(kind, language),
        style,
        fallback,
    )
}
fn of(
    locale: &str,
    kind: DisplayNamesType,
    style: DisplayNamesStyle,
    fallback: DisplayNamesFallback,
    language: DisplayNamesLanguageDisplay,
    code: &str,
) -> Option<Box<str>> {
    let request = DisplayNameRequest::new(
        config(locale, kind, style, fallback, language),
        code.encode_utf16().collect(),
    )
    .unwrap();
    display_name(&request, &foundation().canonicalizer)
        .unwrap()
        .name()
        .map(Into::into)
}
fn expected(
    locale: &str,
    kind: DisplayNamesType,
    style: DisplayNamesStyle,
    code: &str,
    name: &str,
) {
    assert_eq!(
        of(
            locale,
            kind,
            style,
            DisplayNamesFallback::None,
            DisplayNamesLanguageDisplay::Dialect,
            code
        )
        .as_deref(),
        Some(name)
    );
}

#[test]
fn genuine_six_types_load_for_every_declared_locale_and_style() {
    let locales: Vec<_> = foundation()
        .profiles
        .available_locales()
        .map(|l| l.as_str())
        .collect();
    assert_eq!(
        locales,
        [
            "ar",
            "ar-EG",
            "de",
            "en",
            "en-US",
            "fr",
            "hi",
            "it",
            "ja",
            "ko",
            "zh",
            "zh-Hans",
            "zh-Hans-CN"
        ]
    );
    for locale in locales {
        for &style in DisplayNamesStyle::ALL {
            for (kind, code) in [
                (DisplayNamesType::Language, "fr"),
                (DisplayNamesType::Region, "US"),
                (DisplayNamesType::Script, "Hans"),
                (DisplayNamesType::Currency, "USD"),
                (DisplayNamesType::Calendar, "gregory"),
                (DisplayNamesType::DateTimeField, "year"),
            ] {
                assert!(!of(
                    locale,
                    kind,
                    style,
                    DisplayNamesFallback::None,
                    DisplayNamesLanguageDisplay::Dialect,
                    code
                )
                .unwrap()
                .is_empty());
            }
        }
    }
}

#[test]
fn enumeration_consumers_cover_the_real_currency_union_and_required_calendar_domain() {
    let currencies: std::collections::BTreeSet<_> =
        crate::number_format::embedded_number_profiles()
            .unwrap()
            .reachable_currency_codes()
            .collect();
    assert_eq!(currencies.len(), 307);
    let configuration = config(
        "en",
        DisplayNamesType::Currency,
        DisplayNamesStyle::Long,
        DisplayNamesFallback::None,
        DisplayNamesLanguageDisplay::Dialect,
    );
    let mut named = std::collections::BTreeSet::new();
    // The pinned enumeration consumer tests all 26^3 well-formed codes, so
    // checking only one known currency would miss an overbroad catalogue.
    for a in b'A'..=b'Z' {
        for b in b'A'..=b'Z' {
            for c in b'A'..=b'Z' {
                let bytes = [a, b, c];
                let code = DisplayNameCode::from_text(
                    DisplayNamesType::Currency,
                    core::str::from_utf8(&bytes).unwrap(),
                    &foundation().canonicalizer,
                )
                .unwrap();
                if profiles::display_name(&configuration, &code)
                    .unwrap()
                    .name()
                    .is_some()
                {
                    named.insert(bytes);
                }
            }
        }
    }
    assert_eq!(named, currencies);
    assert_eq!(crate::DateTimeCalendar::ALL.len(), 16);
    for &calendar in crate::DateTimeCalendar::ALL {
        for &style in DisplayNamesStyle::ALL {
            assert!(of(
                "en",
                DisplayNamesType::Calendar,
                style,
                DisplayNamesFallback::None,
                DisplayNamesLanguageDisplay::Dialect,
                calendar.as_str()
            )
            .is_some());
        }
    }
}

#[test]
fn captured_widths_standalone_scripts_and_currency_names_remain_genuine() {
    for (style, region, year, week) in [
        (DisplayNamesStyle::Long, "United States", "year", "week"),
        (DisplayNamesStyle::Short, "US", "yr.", "wk."),
        (DisplayNamesStyle::Narrow, "US", "yr", "wk"),
    ] {
        expected("en", DisplayNamesType::Region, style, "us", region);
        expected("en", DisplayNamesType::DateTimeField, style, "year", year);
        expected(
            "en",
            DisplayNamesType::DateTimeField,
            style,
            "weekOfYear",
            week,
        );
        expected("en", DisplayNamesType::Currency, style, "usd", "US Dollar");
        expected(
            "en",
            DisplayNamesType::Script,
            style,
            "hans",
            "Simplified Han",
        );
        expected(
            "fr",
            DisplayNamesType::Currency,
            style,
            "USD",
            "dollar des États-Unis",
        );
    }
    expected(
        "fr",
        DisplayNamesType::Region,
        DisplayNamesStyle::Short,
        "US",
        "É.-U.",
    );
    expected(
        "fr",
        DisplayNamesType::DateTimeField,
        DisplayNamesStyle::Narrow,
        "year",
        "a",
    );
    expected(
        "ar",
        DisplayNamesType::Calendar,
        DisplayNamesStyle::Long,
        "GREGORY",
        "التقويم الميلادي",
    );
    expected(
        "zh",
        DisplayNamesType::Currency,
        DisplayNamesStyle::Long,
        "USD",
        "美元",
    );
}

#[test]
fn dialect_and_standard_compose_only_genuine_qualifier_names() {
    expected(
        "en",
        DisplayNamesType::Language,
        DisplayNamesStyle::Long,
        "en-GB",
        "British English",
    );
    expected(
        "en",
        DisplayNamesType::Language,
        DisplayNamesStyle::Short,
        "en-GB",
        "UK English",
    );
    assert_eq!(
        of(
            "en",
            DisplayNamesType::Language,
            DisplayNamesStyle::Long,
            DisplayNamesFallback::None,
            DisplayNamesLanguageDisplay::Standard,
            "en-GB"
        )
        .as_deref(),
        Some("English (United Kingdom)")
    );
    assert_eq!(
        of(
            "en",
            DisplayNamesType::Language,
            DisplayNamesStyle::Long,
            DisplayNamesFallback::None,
            DisplayNamesLanguageDisplay::Standard,
            "zh-Hans"
        )
        .as_deref(),
        Some("Chinese (Simplified)")
    );
    assert_eq!(
        of(
            "en",
            DisplayNamesType::Language,
            DisplayNamesStyle::Short,
            DisplayNamesFallback::None,
            DisplayNamesLanguageDisplay::Standard,
            "en-GB"
        )
        .as_deref(),
        Some("English (UK)")
    );
    expected(
        "en",
        DisplayNamesType::Language,
        DisplayNamesStyle::Long,
        "es-Cyrl-MX",
        "Mexican Spanish (Cyrillic)",
    );
    assert_eq!(
        of(
            "en",
            DisplayNamesType::Language,
            DisplayNamesStyle::Long,
            DisplayNamesFallback::None,
            DisplayNamesLanguageDisplay::Standard,
            "es-Cyrl-MX"
        )
        .as_deref(),
        Some("Spanish (Cyrillic, Mexico)")
    );
    // Short-only en_GB in Arabic must not inject English or short text into Long.
    expected(
        "ar",
        DisplayNamesType::Language,
        DisplayNamesStyle::Long,
        "en-GB",
        "الإنجليزية (المملكة المتحدة)",
    );
}

#[test]
fn canonical_code_rules_keep_aliases_and_unknown_calendar_domain_separate() {
    let provider = &foundation().canonicalizer;
    for (kind, input, canonical) in [
        (DisplayNamesType::Language, "IW", "he"),
        (DisplayNamesType::Language, "sh", "sr-Latn"),
        (
            DisplayNamesType::Language,
            "zzzzzzzz-aBcD-aB",
            "zzzzzzzz-Abcd-AB",
        ),
        (DisplayNamesType::Region, "us", "US"),
        (DisplayNamesType::Region, "419", "419"),
        (DisplayNamesType::Script, "lATN", "Latn"),
        (DisplayNamesType::Currency, "uSd", "USD"),
        (DisplayNamesType::Calendar, "123-ABC-abc", "123-abc-abc"),
        (DisplayNamesType::Calendar, "ISLAMICC", "islamicc"),
    ] {
        let units: Vec<_> = input.encode_utf16().collect();
        assert_eq!(
            DisplayNameCode::parse(kind, &units, provider)
                .unwrap()
                .as_str(),
            canonical
        );
    }
    expected(
        "en",
        DisplayNamesType::Language,
        DisplayNamesStyle::Long,
        "iw",
        "Hebrew",
    );
    expected(
        "en",
        DisplayNamesType::Calendar,
        DisplayNamesStyle::Long,
        "ethioaa",
        "Ethiopic Amete Alem Calendar",
    );
    expected(
        "en",
        DisplayNamesType::Calendar,
        DisplayNamesStyle::Long,
        "islamicc",
        "Hijri Calendar (tabular, civil epoch)",
    );
}

#[test]
fn pinned_language_calendar_region_and_field_grammars_reject_invalid_codes() {
    let provider = &foundation().canonicalizer;
    for (kind, inputs) in [
        (
            DisplayNamesType::Language,
            &[
                "",
                "a",
                "abcdefghi",
                "en-u-hebrew",
                "aa-aaaa-bbbb",
                "aa-aaaaa-AAAAA",
                "aa-bb-cc",
                "1a",
                "aa-1a",
                "aa-1aa",
                "en-US-",
                "-en",
                "en--GB",
                "root",
                "abcd-GB",
                "abcd",
                "en_GB",
            ][..],
        ),
        (
            DisplayNamesType::Calendar,
            &[
                "00",
                "000000000",
                "-00000000",
                "00000000-",
                " abcdef",
                "abcdef ",
                "123_abc",
            ][..],
        ),
        (
            DisplayNamesType::Region,
            &[
                "00", "a", "aaa", "1111", "", "a01", "a1", "1a", "1a1", "-111", "111-", " aa",
                "aa ",
            ][..],
        ),
        (
            DisplayNamesType::Script,
            &["", "Lat", "Latnn", "L4tn", "éabc"][..],
        ),
        (
            DisplayNamesType::Currency,
            &["", "US", "USDD", "U5D", "u$d"][..],
        ),
        (
            DisplayNamesType::DateTimeField,
            &[
                "",
                "Year",
                "weekofyear",
                "dayperiod",
                "timezoneName",
                "millisecond",
            ][..],
        ),
    ] {
        for input in inputs {
            let units: Vec<_> = input.encode_utf16().collect();
            assert_eq!(
                DisplayNameCode::parse(kind, &units, provider),
                Err(DisplayNamesError::InvalidCode),
                "{kind:?} {input}"
            );
        }
    }
    for &field in DisplayNamesDateTimeField::ALL {
        let units: Vec<_> = field.name().encode_utf16().collect();
        assert_eq!(
            DisplayNameCode::parse(DisplayNamesType::DateTimeField, &units, provider)
                .unwrap()
                .as_str(),
            field.name()
        );
    }
    for language in ["ab", "cde", "zzzzzzzz"] {
        for script in ["", "-abcd"] {
            for region in ["", "-ab", "-123"] {
                for variant in ["", "-abcde", "-1abc", "-12345678", "-abcde-2345"] {
                    let text = format!("{language}{script}{region}{variant}");
                    let units: Vec<_> = text.encode_utf16().collect();
                    DisplayNameCode::parse(DisplayNamesType::Language, &units, provider).unwrap();
                }
            }
        }
    }
}

#[test]
fn missing_fields_return_canonical_code_or_undefined_without_partial_invention() {
    for (kind, input, canonical) in [
        (DisplayNamesType::Language, "zzzzzzzz-AB", "zzzzzzzz-AB"),
        (DisplayNamesType::Currency, "qQq", "QQQ"),
        (DisplayNamesType::Region, "qQ", "QQ"),
        (DisplayNamesType::Script, "QqQq", "Qqqq"),
        (DisplayNamesType::Calendar, "ABC-123", "abc-123"),
    ] {
        assert_eq!(
            of(
                "en",
                kind,
                DisplayNamesStyle::Long,
                DisplayNamesFallback::Code,
                DisplayNamesLanguageDisplay::Dialect,
                input
            )
            .as_deref(),
            Some(canonical)
        );
        assert_eq!(
            of(
                "en",
                kind,
                DisplayNamesStyle::Long,
                DisplayNamesFallback::None,
                DisplayNamesLanguageDisplay::Dialect,
                input
            ),
            None
        );
    }
    assert_eq!(
        of(
            "en",
            DisplayNamesType::Language,
            DisplayNamesStyle::Long,
            DisplayNamesFallback::None,
            DisplayNamesLanguageDisplay::Standard,
            "en-Qqqq"
        ),
        None
    );
}

#[test]
fn locale_resolution_strips_irrelevant_extensions_and_supported_lists_preserve_requests() {
    let request = DisplayNamesLocaleRequest {
        requested: [
            "fr-CA-u-nu-arab",
            "en-US-u-ca-hebrew",
            "zzzzzzzz",
            "fr-CA-u-nu-arab",
        ]
        .into_iter()
        .map(|s| CanonicalLocaleId::from_data(s).unwrap())
        .collect(),
        matcher: LocaleMatcher::Lookup,
    };
    assert_eq!(
        foundation()
            .profiles
            .resolve(&request)
            .unwrap()
            .resolved()
            .as_str(),
        "fr"
    );
    let result = foundation().profiles.supported_locales(&request);
    assert_eq!(
        result
            .locales
            .iter()
            .map(|l| l.as_str())
            .collect::<Vec<_>>(),
        ["fr-CA-u-nu-arab", "en-US-u-ca-hebrew"]
    );
    let empty = DisplayNamesLocaleRequest {
        requested: Box::new([]),
        matcher: LocaleMatcher::BestFit,
    };
    assert_eq!(
        foundation()
            .profiles
            .resolve(&empty)
            .unwrap()
            .resolved()
            .as_str(),
        "en-US"
    );
    assert!(foundation()
        .profiles
        .admit(CanonicalLocaleId::from_data("es").unwrap())
        .is_err());
}

#[test]
fn checked_profile_rejects_bad_identity_maps_domains_refs_and_unused_pools() {
    let original: serde_json::Value =
        serde_json::from_slice(profiles::DISPLAY_NAMES_PROFILE).unwrap();
    let bad = |mut value: serde_json::Value, change: fn(&mut serde_json::Value)| {
        change(&mut value);
        let raw = serde_json::from_value(value);
        assert!(
            raw.is_err()
                || DisplayNamesProfiles::from_raw(raw.unwrap(), &foundation().canonicalizer)
                    .is_err()
        );
    };
    for change in [
        (|v: &mut serde_json::Value| v["schema_version"] = 2.into()) as fn(&mut serde_json::Value),
        |v| v["cldr_commit"] = "wrong".into(),
        |v| v["default_locale"] = "en".into(),
        |v| {
            v["locales"].as_array_mut().unwrap().pop();
        },
        |v| v["locales"][0]["locale_pattern"] = "{0} {0} {1}".into(),
        |v| v["locales"][0]["styles"]["long"]["calendar"] = u32::MAX.into(),
        |v| {
            v["locales"][0]["styles"]["long"]["calendar"] =
                v["locales"][0]["styles"]["long"]["currency"].clone()
        },
        |v| v["locales"][0]["styles"]["long"]["calendar"] = true.into(),
        |v| {
            let row = v["name_pool"][0].clone();
            v["name_pool"].as_array_mut().unwrap().push(row);
        },
        |v| v["name_pool"][0]["entries"][0][1] = "↑↑↑".into(),
        |v| v["name_pool"][0]["entries"][0][0] = "!invalid!".into(),
        |v| {
            let pools = v["name_pool"].as_array_mut().unwrap();
            pools.swap(0, 1);
        },
        |v| v["unknown_source"] = true.into(),
    ] {
        bad(original.clone(), change);
    }
    let mut unused = original.clone();
    let pool = unused["name_pool"].as_array_mut().unwrap();
    let index = pool
        .iter()
        .position(|row| row["kind"] == "calendar")
        .unwrap();
    let mut row = pool[index].clone();
    row["entries"][0][1] = "Unused checked test record".into();
    pool.push(row);
    let mut indexed: Vec<_> = pool.drain(..).enumerate().collect();
    indexed.sort_by_key(|(_, row)| serde_json::to_vec(row).unwrap());
    let mut mapping = vec![0; indexed.len()];
    for (new, (old, _)) in indexed.iter().enumerate() {
        mapping[*old] = new;
    }
    *pool = indexed.into_iter().map(|(_, row)| row).collect();
    for locale in unused["locales"].as_array_mut().unwrap() {
        for style in locale["styles"].as_object_mut().unwrap().values_mut() {
            for reference in style.as_object_mut().unwrap().values_mut() {
                *reference = mapping[reference.as_u64().unwrap() as usize].into();
            }
        }
    }
    bad(unused, |_| {});
    let mut tampered = profiles::DISPLAY_NAMES_PROFILE.to_vec();
    tampered[0] ^= 1;
    assert!(DisplayNamesProfiles::from_json(&tampered, &foundation().canonicalizer).is_err());
}

#[test]
fn primitive_codecs_preserve_utf16_and_reject_wrong_frames_before_native_work() {
    assert_eq!(
        DisplayNamesConfigurationWord::ALL.map(DisplayNamesConfigurationWord::offset),
        [0, 8, 16, 24]
    );
    assert_eq!(
        config(
            "en",
            DisplayNamesType::Language,
            DisplayNamesStyle::Short,
            DisplayNamesFallback::None,
            DisplayNamesLanguageDisplay::Standard
        )
        .wire_words(),
        [1, 2, 2, 2]
    );
    assert_eq!(
        config(
            "en",
            DisplayNamesType::Region,
            DisplayNamesStyle::Long,
            DisplayNamesFallback::Code,
            DisplayNamesLanguageDisplay::Standard
        )
        .wire_words(),
        [2, 1, 1, 0]
    );
    let profiles = &foundation().profiles;
    let request = DisplayNamesLocaleRequest {
        requested: [CanonicalLocaleId::from_data("fr-CA").unwrap()].into(),
        matcher: LocaleMatcher::Lookup,
    };
    let encoded = encode_display_names_locale_request(&request).unwrap();
    assert_eq!(
        decode_display_names_locale_request(&encoded).unwrap(),
        request
    );
    let resolved = profiles.resolve(&request).unwrap();
    assert_eq!(
        decode_display_names_locale_response(
            &encode_display_names_locale_response(&resolved).unwrap(),
            profiles
        )
        .unwrap()
        .resolved(),
        resolved.resolved()
    );
    assert_eq!(
        decode_display_names_supported_request(
            &encode_display_names_supported_request(&request).unwrap()
        )
        .unwrap(),
        request
    );
    let supported = profiles.supported_locales(&request);
    assert_eq!(
        decode_display_names_supported_response(
            &encode_display_names_supported_response(&supported).unwrap()
        )
        .unwrap(),
        supported
    );
    let request = DisplayNameRequest::new(
        config(
            "en",
            DisplayNamesType::Language,
            DisplayNamesStyle::Long,
            DisplayNamesFallback::Code,
            DisplayNamesLanguageDisplay::Dialect,
        ),
        [0xd800, 0x61, 0xdc00].into(),
    )
    .unwrap();
    let frame = encode_display_name_request(&request).unwrap();
    let decoded = decode_display_name_request(&frame, profiles).unwrap();
    assert_eq!(decoded.code(), request.code());
    assert_eq!(
        display_name(&decoded, &foundation().canonicalizer),
        Err(DisplayNamesError::InvalidCode)
    );
    for result in [
        DisplayNameResult::new(None).unwrap(),
        DisplayNameResult::new(Some("美元".into())).unwrap(),
        DisplayNameResult::new(Some("".into())).unwrap(),
    ] {
        assert_eq!(
            decode_display_name_response(&encode_display_name_response(&result).unwrap()).unwrap(),
            result
        );
    }
    for length in 0..frame.len() {
        assert!(decode_display_name_request(&frame[..length], profiles).is_err());
    }
    for (offset, value) in [(0, 2), (8, 54), (8, 59)] {
        let mut bad = frame.clone();
        bad[offset..offset + 8].copy_from_slice(&(value as u64).to_le_bytes());
        assert!(decode_display_name_request(&bad, profiles).is_err());
    }
    let mut trailing = frame.clone();
    trailing.push(0);
    assert!(decode_display_name_request(&trailing, profiles).is_err());
    let valid = DisplayNameRequest::new(
        config(
            "en",
            DisplayNamesType::Region,
            DisplayNamesStyle::Long,
            DisplayNamesFallback::Code,
            DisplayNamesLanguageDisplay::Standard,
        ),
        [0x75, 0x73].into(),
    )
    .unwrap();
    let mut wrong_slot = encode_display_name_request(&valid).unwrap();
    let slot = 16 + 8 + 2 + 24;
    wrong_slot[slot..slot + 8].copy_from_slice(&1u64.to_le_bytes());
    assert!(decode_display_name_request(&wrong_slot, profiles).is_err());
}
