use std::sync::OnceLock;

use super::*;

pub(super) fn provider() -> &'static NamedTimeZones {
    static ZONES: OnceLock<NamedTimeZones> = OnceLock::new();
    ZONES.get_or_init(|| NamedTimeZones::from_pinned_data().unwrap())
}

#[test]
fn complete_pinned_catalogue_roundtrips_ascii_case_without_losing_aliases() {
    let zones = provider();
    assert_eq!(zones.zones.len(), 598);
    for identifier in jiff_tzdb::available() {
        for spelling in [
            identifier.to_owned(),
            identifier.to_ascii_lowercase(),
            identifier.to_ascii_uppercase(),
        ] {
            let identity = zones.lookup(&TimeZoneId::parse(spelling).unwrap()).unwrap();
            assert_eq!(identity.identifier(), identifier);
            let primary = zones
                .lookup(&TimeZoneId::parse(identity.primary_identifier()).unwrap())
                .unwrap();
            assert_eq!(primary.identifier(), primary.primary_identifier());
        }
    }
}

#[test]
fn geographic_primaries_preserve_country_and_backzone_ownership() {
    for (identifier, primary) in [
        ("Europe/Bratislava", "Europe/Bratislava"),
        ("Europe/Oslo", "Europe/Oslo"),
        ("Atlantic/Jan_Mayen", "Arctic/Longyearbyen"),
        ("Antarctica/South_Pole", "Antarctica/McMurdo"),
        ("Pacific/Truk", "Pacific/Chuuk"),
        ("Pacific/Yap", "Pacific/Chuuk"),
        ("Pacific/Ponape", "Pacific/Pohnpei"),
        ("Pacific/Johnston", "Pacific/Johnston"),
        ("America/Coral_Harbour", "America/Atikokan"),
        ("Africa/Timbuktu", "Africa/Bamako"),
        ("Iceland", "Atlantic/Reykjavik"),
        ("Australia/ACT", "Australia/Sydney"),
        ("Brazil/Acre", "America/Rio_Branco"),
        ("Navajo", "America/Denver"),
        ("America/Virgin", "America/St_Thomas"),
        ("Africa/Asmera", "Africa/Asmara"),
        ("Asia/Chungking", "Asia/Shanghai"),
        ("America/Coyhaique", "America/Coyhaique"),
        ("US/Eastern", "America/New_York"),
        ("Europe/Kiev", "Europe/Kyiv"),
    ] {
        let identity = provider()
            .lookup(&TimeZoneId::parse(identifier).unwrap())
            .unwrap();
        assert_eq!(
            (identity.identifier(), identity.primary_identifier()),
            (identifier, primary)
        );
    }
}

#[test]
fn utc_aliases_retain_normalized_identifier_and_have_one_primary() {
    for identifier in [
        "UTC",
        "Etc/UTC",
        "Etc/GMT",
        "GMT",
        "GMT0",
        "Etc/GMT+0",
        "Etc/GMT-0",
        "UCT",
        "Zulu",
        "Universal",
    ] {
        let identity = provider()
            .lookup(&TimeZoneId::parse(identifier).unwrap())
            .unwrap();
        assert_eq!(identity.identifier(), identifier);
        assert_eq!(identity.primary_identifier(), "UTC");
        for seconds in [-TimeZoneEpochSeconds::LIMIT, 0, TimeZoneEpochSeconds::LIMIT] {
            let snapshot = provider()
                .transition(&identity, TimeZoneEpochSeconds::new(seconds).unwrap())
                .unwrap();
            assert_eq!(snapshot.offset_seconds(), 0);
            assert_eq!(snapshot.variant(), TimeZoneVariant::Standard);
        }
    }
}

#[test]
fn unknown_input_and_inconsistent_internal_identity_have_distinct_errors() {
    for identifier in [
        "Europe/Not_A_Zone",
        "America/NewYork",
        "+01:00",
        "Etc/Unknown",
    ] {
        let input = TimeZoneId::parse(identifier).unwrap();
        assert_eq!(provider().lookup(&input).unwrap_err().time_zone(), &input);
    }
    let inconsistent = NamedTimeZoneIdentity::from_data("Europe/Paris", "UTC").unwrap();
    assert!(provider()
        .transition(&inconsistent, TimeZoneEpochSeconds::new(0).unwrap())
        .is_err());
}

#[test]
fn shared_transition_storage_never_merges_geographic_identities() {
    assert_ne!(PROVIDER_DATA_SHA256, [0; 32]);
    let berlin = &provider().zones["europe/berlin"];
    let oslo = &provider().zones["europe/oslo"];
    assert_eq!(
        jiff_tzdb::get("Europe/Berlin").unwrap().1,
        jiff_tzdb::get("Europe/Oslo").unwrap().1
    );
    assert_eq!(berlin.identity.primary_identifier(), "Europe/Berlin");
    assert_eq!(oslo.identity.primary_identifier(), "Europe/Oslo");
}
