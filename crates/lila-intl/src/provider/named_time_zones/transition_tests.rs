use tzif::data::tzif::LocalTimeTypeRecord;

use super::tests::provider;
use super::*;
use crate::provider::time_zone_snapshot::StandardTimeStability;

fn snapshot(identifier: &str, seconds: i64) -> (i32, TimeZoneVariant) {
    let identity = provider()
        .lookup(&TimeZoneId::parse(identifier).unwrap())
        .unwrap();
    let selected = provider()
        .transition(&identity, TimeZoneEpochSeconds::new(seconds).unwrap())
        .unwrap();
    (selected.offset_seconds(), selected.variant())
}

fn expected(record: LocalTimeTypeRecord) -> (i32, TimeZoneVariant) {
    (
        i32::try_from(record.utoff.0).unwrap(),
        if record.is_dst {
            TimeZoneVariant::Daylight
        } else {
            TimeZoneVariant::Standard
        },
    )
}

#[test]
fn every_explicit_transition_uses_the_selected_records_offset_and_variant() {
    for zone in provider().zones.values() {
        let block = zone.transitions.get_data_block2().unwrap();
        for (index, epoch) in block.transition_times.iter().enumerate() {
            let record = block.local_time_type_records[block.transition_types[index]];
            assert_eq!(
                snapshot(zone.identity.identifier(), epoch.0),
                expected(record),
                "{} at {}",
                zone.identity.identifier(),
                epoch.0
            );
            let previous = if index == 0 {
                block.local_time_type_records[0]
            } else {
                block.local_time_type_records[block.transition_types[index - 1]]
            };
            assert_eq!(
                snapshot(zone.identity.identifier(), epoch.0 - 1),
                expected(previous),
                "{} before {}",
                zone.identity.identifier(),
                epoch.0
            );
        }
    }
}

#[test]
fn historical_seconds_initial_records_and_zero_offset_dst_are_preserved() {
    use TimeZoneVariant::{Daylight, Standard};
    assert_eq!(snapshot("Europe/Paris", -2_208_988_800), (561, Standard));
    assert_eq!(snapshot("Europe/Paris", -1_855_958_962), (561, Standard));
    assert_eq!(snapshot("Europe/Paris", -1_855_958_961), (0, Standard));
    // Lisbon changed designation and DST status without changing its offset.
    assert_eq!(snapshot("Europe/Lisbon", 717_555_599), (3_600, Daylight));
    assert_eq!(snapshot("Europe/Lisbon", 717_555_600), (3_600, Standard));
    for zone in provider().zones.values() {
        let block = zone.transitions.get_data_block2().unwrap();
        if !block.transition_times.is_empty() {
            assert_eq!(
                snapshot(zone.identity.identifier(), -TimeZoneEpochSeconds::LIMIT),
                expected(block.local_time_type_records[0])
            );
        }
        // Full-domain lookup must remain defined after all explicit records.
        let identity = &zone.identity;
        provider()
            .transition(
                identity,
                TimeZoneEpochSeconds::new(TimeZoneEpochSeconds::LIMIT).unwrap(),
            )
            .unwrap();
    }
}

#[test]
fn seasonal_tails_cover_northern_southern_half_hour_and_rearguard_dst() {
    use TimeZoneVariant::{Daylight, Standard};
    let winter = 64_060_588_800; // 4000-01-01T00:00Z
    let summer = 64_076_313_600; // 4000-07-01T00:00Z
    for (zone, january, july) in [
        ("America/New_York", (-18_000, Standard), (-14_400, Daylight)),
        ("Australia/Sydney", (39_600, Daylight), (36_000, Standard)),
        (
            "Australia/Lord_Howe",
            (39_600, Daylight),
            (37_800, Standard),
        ),
        ("Europe/Dublin", (0, Standard), (3_600, Daylight)),
        ("Asia/Kathmandu", (20_700, Standard), (20_700, Standard)),
    ] {
        assert_eq!(snapshot(zone, winter), january, "{zone} January");
        assert_eq!(snapshot(zone, summer), july, "{zone} July");
    }
    assert_eq!(
        snapshot("America/Coyhaique", 1_751_328_000),
        (-10_800, Standard)
    );
}

#[test]
fn dateline_jumps_and_floor_second_transition_neighbors_are_exact() {
    use TimeZoneVariant::{Daylight, Standard};
    assert_eq!(snapshot("Pacific/Apia", 1_325_239_199), (-36_000, Daylight));
    assert_eq!(snapshot("Pacific/Apia", 1_325_239_200), (50_400, Daylight));
    for (zone, transition, before, after) in [
        (
            "America/Los_Angeles",
            1_772_964_000_i64,
            (-28_800, Standard),
            (-25_200, Daylight),
        ),
        (
            "America/Los_Angeles",
            1_793_523_600_i64,
            (-25_200, Daylight),
            (-28_800, Standard),
        ),
        (
            "Europe/Paris",
            -1_855_958_961_i64,
            (561, Standard),
            (0, Standard),
        ),
    ] {
        for (milliseconds, expected) in [
            (transition * 1000 - 1, before),
            (transition * 1000, after),
            (transition * 1000 + 1, after),
        ] {
            let epoch = TimeZoneEpochSeconds::from_milliseconds(milliseconds).unwrap();
            assert_eq!(
                snapshot(zone, epoch.get()),
                expected,
                "{zone} at {milliseconds}ms"
            );
        }
    }
}

#[test]
fn generic_name_standard_fallback_requires_the_whole_window_to_be_stable() {
    for (identifier, seconds, expected) in [
        (
            "America/New_York",
            64_060_588_800,
            StandardTimeStability::NotProven,
        ),
        (
            "Australia/Lord_Howe",
            64_076_313_600,
            StandardTimeStability::NotProven,
        ),
        (
            "Asia/Kathmandu",
            64_060_588_800,
            StandardTimeStability::Stable,
        ),
        (
            "UTC",
            TimeZoneEpochSeconds::LIMIT,
            StandardTimeStability::Stable,
        ),
        (
            "UTC",
            -TimeZoneEpochSeconds::LIMIT,
            StandardTimeStability::Stable,
        ),
        (
            "Europe/Lisbon",
            717_555_600,
            StandardTimeStability::NotProven,
        ),
    ] {
        let identity = provider()
            .lookup(&TimeZoneId::parse(identifier).unwrap())
            .unwrap();
        let selected = provider()
            .transition(&identity, TimeZoneEpochSeconds::new(seconds).unwrap())
            .unwrap();
        assert_eq!(selected.standard_time_stability(), expected, "{identifier}");
        if expected == StandardTimeStability::Stable {
            assert_eq!(selected.variant(), TimeZoneVariant::Standard);
        }
    }
}
