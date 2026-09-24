use super::*;
use crate::provider::EmbeddedIntlProvider;
use crate::{IntlOperationProvider, LocaleInfo, SupportedValues, SupportedValuesRequest};

fn provider() -> EmbeddedIntlProvider {
    EmbeddedIntlProvider::new().expect("pinned provider is valid")
}

fn query(provider: &EmbeddedIntlProvider, query: LocaleInfoQuery, tag: &str) -> LocaleInfoResult {
    IntlOperationProvider::<LocaleInfo>::execute(
        provider,
        LocaleInfoRequest::new(query, CanonicalLocaleId::from_data(tag).unwrap()),
    )
    .unwrap_or_else(|error| panic!("{tag}: {error}"))
}

fn list(provider: &EmbeddedIntlProvider, kind: LocaleInfoQuery, tag: &str) -> Vec<String> {
    match query(provider, kind, tag) {
        LocaleInfoResult::Identifiers(values) => values.iter().map(|v| v.to_string()).collect(),
        other => panic!("{tag}: unexpected {other:?}"),
    }
}

fn week(provider: &EmbeddedIntlProvider, tag: &str) -> (u64, Vec<u64>) {
    match query(provider, LocaleInfoQuery::WeekInfo, tag) {
        LocaleInfoResult::Week(week) => (
            week.first_day().wire_code(),
            week.weekend().iter().map(|day| day.wire_code()).collect(),
        ),
        other => panic!("{tag}: unexpected {other:?}"),
    }
}

#[test]
fn locale_view_reads_base_subtags_and_complete_keyword_values() {
    let view = LocaleView::parse("sr-Latn-RS-1901-u-ca-islamic-civil-kn-x-u-ca-gregory").unwrap();
    assert_eq!(view.language, "sr");
    assert_eq!(view.script, Some("Latn"));
    assert_eq!(view.region, Some("RS"));
    assert_eq!(view.keyword("ca"), Some("islamic-civil"));
    assert_eq!(view.keyword("kn"), Some(""));
    assert_eq!(view.keyword("nu"), None);
    assert_eq!(
        view.without_unicode_extension(),
        "sr-Latn-RS-1901-x-u-ca-gregory"
    );
    let reserved = LocaleView::parse("abcde-419-t-en-u-fw-mon").unwrap();
    assert_eq!(reserved.language, "abcde");
    assert_eq!(reserved.region, Some("419"));
    assert_eq!(reserved.keyword("fw"), Some("mon"));
    assert_eq!(reserved.without_unicode_extension(), "abcde-419-t-en");
}

#[test]
fn region_preference_orders_override_region_subdivision_likely_and_world() {
    let provider = provider();
    // The TH/JP/IN/IR regions only differ through calendars that DateTimeFormat
    // supports; hour cycles distinguish every preference level observably.
    let cycles = |tag| list(&provider, LocaleInfoQuery::HourCycles, tag);
    assert_eq!(cycles("en-US-u-rg-gbzzzz-sd-gbeng"), cycles("en-GB"));
    assert_eq!(cycles("en-US-u-sd-gbeng"), cycles("en-US"));
    assert_eq!(cycles("en-u-sd-gbeng"), cycles("en-GB"));
    assert_eq!(cycles("en"), cycles("en-US"));
    assert_eq!(cycles("eo"), cycles("eo-001"));
    assert_ne!(cycles("en-GB"), cycles("en-US"));
    assert_ne!(cycles("en-US"), cycles("eo-001"));
    // An unavailable override falls back to the region, not to 001.
    assert_eq!(cycles("en-US-u-rg-aqzzzz"), cycles("en-US"));
}

#[test]
fn hour_cycles_prefer_language_region_time_data() {
    let provider = provider();
    let cycles = |tag| list(&provider, LocaleInfoQuery::HourCycles, tag);
    assert_eq!(cycles("fr-CA"), ["h23", "h12"]);
    assert_eq!(cycles("und-CA"), ["h12", "h23"]);
    assert_eq!(cycles("ja-JP"), ["h23", "h11", "h12"]);
    assert_eq!(cycles("en-u-hc-h24"), ["h24"]);
}

#[test]
fn calendars_are_filtered_to_date_time_format_calendars() {
    let provider = provider();
    let calendars = |tag| list(&provider, LocaleInfoQuery::Calendars, tag);
    assert_eq!(calendars("zh-TW"), ["gregory", "chinese"]);
    assert_eq!(calendars("zh"), ["gregory", "chinese"]);
    assert_eq!(calendars("en-US-u-rg-cnzzzz"), ["gregory", "chinese"]);
    assert_eq!(calendars("en"), ["gregory"]);
    assert_eq!(calendars("th-u-ca-buddhist"), ["buddhist"]);
}

#[test]
fn week_info_uses_region_week_data_and_the_first_day_keyword() {
    let provider = provider();
    assert_eq!(week(&provider, "en"), (7, vec![6, 7]));
    assert_eq!(week(&provider, "fa"), (6, vec![5]));
    assert_eq!(
        week(&provider, "fa-JP-u-rg-afzzzz-sd-inka"),
        (6, vec![4, 5])
    );
    assert_eq!(week(&provider, "fa-u-sd-inka"), (7, vec![7]));
    assert_eq!(week(&provider, "en-US-u-rg-dezzzz"), (1, vec![6, 7]));
    assert_eq!(week(&provider, "eo"), (1, vec![6, 7]));
    assert_eq!(week(&provider, "en-u-fw-wed"), (3, vec![6, 7]));
}

#[test]
fn collations_follow_collator_locale_lookup() {
    let provider = provider();
    let collations = |tag| list(&provider, LocaleInfoQuery::Collations, tag);
    assert_eq!(collations("de-CH"), ["emoji", "eor", "phonebk"]);
    assert_eq!(collations("es"), ["emoji", "eor", "trad"]);
    assert_eq!(
        collations("zh-Hant-TW"),
        ["emoji", "eor", "pinyin", "stroke", "unihan", "zhuyin"]
    );
    for tag in ["und", "und-US", "und-Latn-US", "qfz", "qtz-CN"] {
        assert_eq!(collations(tag), ["emoji", "eor"], "{tag}");
    }
    assert_eq!(collations("de-u-co-phonebk"), ["phonebk"]);
}

#[test]
fn numbering_systems_time_zones_and_direction() {
    let provider = provider();
    assert_eq!(
        list(&provider, LocaleInfoQuery::NumberingSystems, "ar-EG"),
        ["arab"]
    );
    assert_eq!(
        list(&provider, LocaleInfoQuery::NumberingSystems, "en"),
        ["latn"]
    );
    assert_eq!(
        list(&provider, LocaleInfoQuery::NumberingSystems, "qfz"),
        ["latn"]
    );
    assert_eq!(
        list(&provider, LocaleInfoQuery::NumberingSystems, "en-u-nu-thai"),
        ["thai"]
    );
    let zones = list(&provider, LocaleInfoQuery::TimeZones, "en-US");
    assert!(zones.contains(&"America/New_York".to_string()));
    assert!(zones.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(
        query(&provider, LocaleInfoQuery::TimeZones, "en"),
        LocaleInfoResult::Undefined
    );
    assert_eq!(
        list(&provider, LocaleInfoQuery::TimeZones, "en-419"),
        Vec::<String>::new()
    );
    for (tag, expected) in [
        (
            "en",
            LocaleInfoResult::Direction(TextDirection::LeftToRight),
        ),
        (
            "ar",
            LocaleInfoResult::Direction(TextDirection::RightToLeft),
        ),
        (
            "he",
            LocaleInfoResult::Direction(TextDirection::RightToLeft),
        ),
        (
            "en-Arab",
            LocaleInfoResult::Direction(TextDirection::RightToLeft),
        ),
        ("en-Zyyy", LocaleInfoResult::Undefined),
    ] {
        assert_eq!(
            query(&provider, LocaleInfoQuery::TextDirection, tag),
            expected,
            "{tag}"
        );
    }
}

#[test]
fn supported_values_are_sorted_canonical_lists() {
    let provider = provider();
    for key in SupportedValuesKey::ALL {
        let result = IntlOperationProvider::<SupportedValues>::execute(
            &provider,
            SupportedValuesRequest::new(*key),
        )
        .unwrap();
        assert!(!result.values().is_empty(), "{key:?}");
    }
    let values = |key| {
        IntlOperationProvider::<SupportedValues>::execute(
            &provider,
            SupportedValuesRequest::new(key),
        )
        .unwrap()
        .values()
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
    };
    assert_eq!(
        values(SupportedValuesKey::Calendar),
        ["chinese", "gregory", "iso8601"]
    );
    let collations = values(SupportedValuesKey::Collation);
    assert!(!collations.contains(&"standard".to_string()));
    assert!(!collations.contains(&"search".to_string()));
    let zones = values(SupportedValuesKey::TimeZone);
    for zone in ["UTC", "Etc/GMT+1", "Etc/GMT-14", "America/New_York"] {
        assert!(zones.contains(&zone.to_string()), "{zone}");
    }
    assert!(!zones.contains(&"US/Pacific".to_string()));
    assert!(values(SupportedValuesKey::Currency).contains(&"EUR".to_string()));
    assert!(values(SupportedValuesKey::NumberingSystem).contains(&"latn".to_string()));
    assert!(values(SupportedValuesKey::Unit).contains(&"mile-scandinavian".to_string()));
}
