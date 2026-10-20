use super::*;

fn resolve(requested: &str, matcher: LocaleMatcher) -> ResolvedNumberLocale {
    resolve_number_locale(
        &NumberLocaleRequest {
            requested: vec![canonical(requested)].into_boxed_slice(),
            matcher,
            numbering_system: None,
        },
        profiles(),
    )
    .unwrap()
}

#[test]
fn best_fit_uses_selected_likely_subtags_without_replacing_exact_or_lookup_associations() {
    for (requested, selected) in [
        ("zh-TW", "zh-Hant-TW"),
        ("zh-HK", "zh-Hant-HK"),
        ("zh-MO", "zh-Hant-MO"),
        ("en-US", "en-US"),
        ("zh-Hant", "zh-Hant"),
        ("en-GB-fonipa", "en-GB"),
    ] {
        assert_eq!(
            resolve(requested, LocaleMatcher::BestFit)
                .formatting()
                .as_str(),
            selected,
            "{requested}"
        );
    }
    assert_eq!(
        resolve("zh-TW", LocaleMatcher::Lookup)
            .formatting()
            .as_str(),
        "zh"
    );
    let resolved = resolve("zh-TW-u-nu-hanidec", LocaleMatcher::BestFit);
    assert_eq!(resolved.resolved().as_str(), "zh-Hant-TW-u-nu-hanidec");
    assert_eq!(resolved.numbering_system().name(), "hanidec");
    let supported = filter_number_locales(
        &NumberSupportedLocalesRequest {
            requested: vec![canonical("zh-TW-u-nu-hanidec"), canonical("zz-ZZ")].into_boxed_slice(),
            matcher: LocaleMatcher::BestFit,
        },
        profiles(),
    )
    .unwrap();
    assert_eq!(
        supported
            .iter()
            .map(crate::CanonicalLocaleId::as_str)
            .collect::<Vec<_>>(),
        ["zh-TW-u-nu-hanidec"]
    );
}

#[test]
fn best_fit_reaches_traditional_number_symbols_compact_and_unit_patterns() {
    for (source, options, expected) in [
        ("NaN", options(), "非數值"),
        (
            "987654321",
            NumberFormatOptions {
                notation: Notation::Compact(CompactDisplay::Short),
                precision: significant(1, 2),
                ..options()
            },
            "9.9億",
        ),
        (
            "-987",
            unit("kilometer-per-hour", UnitDisplay::Short),
            "-987 公里/小時",
        ),
    ] {
        let result = partition_number(
            &NumberFormatConfiguration {
                locale: resolve("zh-TW", LocaleMatcher::BestFit),
                options,
            },
            &value(source),
            profiles(),
            &PartitionLimits::HOST_ABI,
        )
        .unwrap();
        assert_eq!(result.to_text(), expected);
    }
}
