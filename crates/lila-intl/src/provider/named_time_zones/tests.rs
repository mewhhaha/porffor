use super::*;

const CATALOGUE: &str = include_str!("../../../data/iana-tzdb-2026a/catalogue.tsv");

fn pinned_records() -> Vec<(&'static str, &'static [u8])> {
    CATALOGUE
        .lines()
        .map(|line| {
            let (name, _) = line.split_once('\t').unwrap();
            (name, jiff_tzdb::get(name).unwrap().1)
        })
        .collect()
}

pub(super) fn provider() -> &'static NamedTimeZones {
    crate::named_time_zone_image::embedded_named_time_zone_data_image_ref()
        .unwrap()
        .zones_ref()
}

#[test]
fn complete_pinned_catalogue_roundtrips_ascii_case_without_losing_aliases() {
    let zones = provider();
    assert_eq!(zones.zones.len(), 598);
    let mut data_by_payload = BTreeMap::new();
    for identifier in jiff_tzdb::available() {
        let zone = &zones.zones[&identifier.to_ascii_lowercase()];
        let bytes = jiff_tzdb::get(identifier).unwrap().1;
        if let Some(previous) = data_by_payload.insert(bytes, &zone.data) {
            assert!(
                Arc::ptr_eq(previous, &zone.data),
                "identical payload must retain one admitted owner: {identifier}"
            );
        }
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
    assert_eq!(data_by_payload.len(), 341);
    let admitted_owners = zones
        .zones
        .values()
        .map(|zone| Arc::as_ptr(&zone.data))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        admitted_owners.len(),
        data_by_payload.len(),
        "distinct payloads must retain distinct admission proofs"
    );
    for data in data_by_payload.values() {
        let retained_records = zones
            .zones
            .values()
            .filter(|zone| Arc::ptr_eq(data, &zone.data))
            .count();
        assert_eq!(
            Arc::strong_count(data),
            retained_records,
            "only named records retain admitted data after construction"
        );
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
    let zones = provider();
    let berlin = &zones.zones["europe/berlin"];
    let oslo = &zones.zones["europe/oslo"];
    assert_eq!(
        jiff_tzdb::get("Europe/Berlin").unwrap().1,
        jiff_tzdb::get("Europe/Oslo").unwrap().1
    );
    assert!(Arc::ptr_eq(&berlin.data, &oslo.data));
    assert_eq!(berlin.identity.identifier(), "Europe/Berlin");
    assert_eq!(berlin.identity.primary_identifier(), "Europe/Berlin");
    assert_eq!(oslo.identity.identifier(), "Europe/Oslo");
    assert_eq!(oslo.identity.primary_identifier(), "Europe/Oslo");

    let new_york = &zones.zones["america/new_york"];
    let eastern = &zones.zones["us/eastern"];
    assert!(Arc::ptr_eq(&new_york.data, &eastern.data));
    assert!(!Arc::ptr_eq(&berlin.data, &new_york.data));
    assert_eq!(new_york.identity.identifier(), "America/New_York");
    assert_eq!(eastern.identity.identifier(), "US/Eastern");
    assert_eq!(eastern.identity.primary_identifier(), "America/New_York");
    for seconds in [-2_208_988_800, 1_615_705_200, 64_060_588_800] {
        let epoch = TimeZoneEpochSeconds::new(seconds).unwrap();
        for (left, right) in [(berlin, oslo), (new_york, eastern)] {
            assert_eq!(
                zones.transition(&left.identity, epoch).unwrap(),
                zones.transition(&right.identity, epoch).unwrap()
            );
        }
    }
}

#[test]
fn shared_payload_reuse_keeps_each_record_spelling_and_digest_checks() {
    let records = pinned_records();
    assert_eq!(records[0].0, "Africa/Abidjan");
    assert_eq!(records[1].0, "Africa/Accra");
    assert_eq!(records[0].1, records[1].1);

    // The first row admits this payload before the second row tries reuse.
    let mut wrong_spelling = records.clone();
    wrong_spelling[1].0 = "africa/accra";
    assert_eq!(
        NamedTimeZones::from_image_data(CATALOGUE, &wrong_spelling).err(),
        Some(InvalidTimeZoneData(
            "catalogue and transition spelling differ"
        ))
    );

    let mut malformed = records[1].1.to_vec();
    malformed[..4].copy_from_slice(b"BAD!");
    assert!(Tzif::from_bytes(&malformed).is_err());
    for replacement in [jiff_tzdb::get("UTC").unwrap().1, malformed.as_slice()] {
        assert_ne!(replacement, records[1].1);
        let altered = records
            .iter()
            .map(|&(name, bytes)| {
                (
                    name,
                    if name == "Africa/Accra" {
                        replacement
                    } else {
                        bytes
                    },
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            NamedTimeZones::from_image_data(CATALOGUE, &altered).err(),
            Some(InvalidTimeZoneData(
                "pinned transition record digest mismatch"
            ))
        );
    }
}

#[test]
fn distinct_transition_data_requires_full_admission_before_sharing() {
    let mut invalid = Tzif::from_bytes(jiff_tzdb::get("America/New_York").unwrap().1).unwrap();
    invalid.data_block2.as_mut().unwrap().transition_types[0] = usize::MAX;
    assert!(matches!(
        AdmittedNamedTimeZoneData::from_data(invalid),
        Err(InvalidTimeZoneData(
            "invalid pinned TZif transition indices"
        ))
    ));
}

#[test]
fn sharing_does_not_make_an_unselected_geographic_primary_available() {
    let selected = [
        TimeZoneId::parse("Europe/Berlin").unwrap(),
        TimeZoneId::parse("UTC").unwrap(),
    ];
    let rows = catalogue::read(CATALOGUE).unwrap();
    let records = pinned_records()
        .into_iter()
        .zip(rows)
        .filter_map(|(record, row)| {
            selected
                .iter()
                .any(|name| name.as_str() == row.identity.primary_identifier())
                .then_some(record)
        })
        .collect::<Vec<_>>();
    let zones = NamedTimeZones::from_records(CATALOGUE, &records, Some(&selected)).unwrap();
    assert_eq!(zones.identifiers().count(), 598);
    assert!(!Arc::ptr_eq(
        &zones.zones["europe/berlin"].data,
        &provider().zones["europe/berlin"].data
    ));
    for identifier in ["Europe/Berlin", "Etc/UTC", "UTC"] {
        let identity = zones
            .lookup(&TimeZoneId::parse(identifier).unwrap())
            .unwrap();
        zones
            .transition(&identity, TimeZoneEpochSeconds::new(0).unwrap())
            .unwrap();
    }
    let identity = zones
        .lookup(&TimeZoneId::parse("Europe/Oslo").unwrap())
        .unwrap();
    assert_eq!(identity.primary_identifier(), "Europe/Oslo");
    assert!(matches!(
        zones.transition(&identity, TimeZoneEpochSeconds::new(0).unwrap()),
        Err(crate::NamedTimeZoneDataError::UnavailableIdentifier(name))
            if name.as_str() == "Europe/Oslo"
    ));
}
