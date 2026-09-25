use std::sync::OnceLock;

use super::*;
use crate::temporal_time_zone::TemporalOffsetMinutes;
use crate::TimeZoneId;

fn zones() -> &'static NamedTimeZones {
    static ZONES: OnceLock<NamedTimeZones> = OnceLock::new();
    ZONES.get_or_init(|| NamedTimeZones::from_pinned_data().unwrap())
}

fn ask(zone: &str, query: TemporalTimeZoneQuery) -> TemporalTimeZoneAnswer {
    let zone = TemporalTimeZone::parse_stored(zone).unwrap();
    answer(zones(), &TemporalTimeZoneRequest::new(zone, query)).unwrap()
}

fn named(zone: &str) -> ZoneRules<'static> {
    ZoneRules::Named(zones().rules(&TimeZoneId::parse(zone).unwrap()).unwrap())
}

/// Local seconds of an ISO date-time.
fn local(year: i64, month: i64, day: i64, hour: i64, minute: i64) -> i64 {
    days_from_civil(year, month, day) * SECONDS_PER_DAY + hour * 3_600 + minute * 60
}

fn whole(seconds: i64) -> TemporalSeconds {
    TemporalSeconds::new(seconds, false)
}

fn epoch_for(zone: &str, at: i64, disambiguation: TemporalDisambiguation) -> TemporalTimeZoneAnswer {
    ask(
        zone,
        TemporalTimeZoneQuery::EpochFor {
            local: whole(at),
            disambiguation,
        },
    )
}

fn seconds(answer: TemporalTimeZoneAnswer) -> i64 {
    match answer {
        TemporalTimeZoneAnswer::Seconds(seconds) => seconds,
        other => panic!("expected seconds, got {other:?}"),
    }
}

#[test]
fn civil_helpers_agree_on_known_dates() {
    assert_eq!(days_from_civil(1970, 1, 1), 0);
    assert_eq!(days_from_civil(2000, 3, 1), 11_017);
    assert_eq!(days_from_civil(-271_821, 4, 20), -100_000_000);
    assert_eq!(days_from_civil(275_760, 9, 13), 100_000_000);
    for days in [-100_000_001, -719_469, -1, 0, 59, 11_016, 100_000_001] {
        let year = year_of_epoch_seconds(days * SECONDS_PER_DAY);
        assert!(days_from_civil(year, 1, 1) <= days);
        assert!(days_from_civil(year + 1, 1, 1) > days);
    }
}

#[test]
fn new_york_repeated_and_skipped_local_times() {
    use TemporalDisambiguation::*;
    // 2017-11-05T01:30 happens at -04:00 and then at -05:00.
    let repeated = local(2017, 11, 5, 1, 30);
    assert_eq!(
        named("America/New_York").candidates(repeated).unwrap(),
        vec![repeated + 4 * 3_600, repeated + 5 * 3_600]
    );
    for (disambiguation, expected) in [
        (Compatible, repeated + 4 * 3_600),
        (Earlier, repeated + 4 * 3_600),
        (Later, repeated + 5 * 3_600),
    ] {
        assert_eq!(
            seconds(epoch_for("America/New_York", repeated, disambiguation)),
            expected
        );
    }
    assert_eq!(
        epoch_for("America/New_York", repeated, Reject),
        TemporalTimeZoneAnswer::RangeError(TemporalTimeZoneRangeError::Ambiguous)
    );
    // 2017-03-12T02:30 does not exist.
    let skipped = local(2017, 3, 12, 2, 30);
    assert!(named("America/New_York").candidates(skipped).unwrap().is_empty());
    // compatible/later: 03:30-04:00; earlier: 01:30-05:00.
    for (disambiguation, expected) in [
        (Compatible, skipped + 3_600 + 4 * 3_600),
        (Later, skipped + 3_600 + 4 * 3_600),
        (Earlier, skipped - 3_600 + 5 * 3_600),
    ] {
        assert_eq!(
            seconds(epoch_for("America/New_York", skipped, disambiguation)),
            expected
        );
    }
    assert_eq!(
        epoch_for("America/New_York", skipped, Reject),
        TemporalTimeZoneAnswer::RangeError(TemporalTimeZoneRangeError::Ambiguous)
    );
}

#[test]
fn transitions_match_known_tzdb_values() {
    let dst_start_2017 = local(2017, 3, 12, 7, 0);
    let dst_end_2017 = local(2017, 11, 5, 6, 0);
    let next = |zone: &str, at: TemporalSeconds| {
        ask(
            zone,
            TemporalTimeZoneQuery::Transition {
                epoch: at,
                direction: TemporalTransitionDirection::Next,
            },
        )
    };
    let previous = |zone: &str, at: TemporalSeconds| {
        ask(
            zone,
            TemporalTimeZoneQuery::Transition {
                epoch: at,
                direction: TemporalTransitionDirection::Previous,
            },
        )
    };
    assert_eq!(
        next("America/New_York", whole(local(2017, 1, 1, 0, 0))),
        TemporalTimeZoneAnswer::Seconds(dst_start_2017)
    );
    // Strictly after: a transition instant finds the following one.
    assert_eq!(
        next("America/New_York", whole(dst_start_2017)),
        TemporalTimeZoneAnswer::Seconds(dst_end_2017)
    );
    assert_eq!(
        next("America/New_York", TemporalSeconds::new(dst_start_2017 - 1, true)),
        TemporalTimeZoneAnswer::Seconds(dst_start_2017)
    );
    // Strictly before: only a non-zero remainder puts the instant after it.
    assert_eq!(
        previous("America/New_York", whole(dst_end_2017)),
        TemporalTimeZoneAnswer::Seconds(dst_start_2017)
    );
    assert_eq!(
        previous("America/New_York", TemporalSeconds::new(dst_end_2017, true)),
        TemporalTimeZoneAnswer::Seconds(dst_end_2017)
    );
    // Beyond the explicit records the footer rule continues.
    assert_eq!(
        next("America/New_York", whole(local(2100, 1, 1, 0, 0))),
        TemporalTimeZoneAnswer::Seconds(local(2100, 3, 14, 7, 0))
    );
    // The last transition before the representable end.
    let last = seconds(previous(
        "America/New_York",
        whole(TEMPORAL_EPOCH_SECONDS_LIMIT),
    ));
    assert_eq!(
        next("America/New_York", whole(last)),
        TemporalTimeZoneAnswer::NoTransition
    );
    for zone in ["UTC", "Etc/GMT+5", "+05:30"] {
        for direction in [
            TemporalTransitionDirection::Next,
            TemporalTransitionDirection::Previous,
        ] {
            assert_eq!(
                ask(
                    zone,
                    TemporalTimeZoneQuery::Transition {
                        epoch: whole(0),
                        direction,
                    }
                ),
                TemporalTimeZoneAnswer::NoTransition,
                "{zone}"
            );
        }
    }
    // Europe/London's first transition leaves local mean time in 1847.
    assert_eq!(
        previous("Europe/London", whole(local(1900, 1, 1, 0, 0))),
        TemporalTimeZoneAnswer::Seconds(-3_852_662_325)
    );
    assert_eq!(
        previous("Europe/London", whole(-3_852_662_325)),
        TemporalTimeZoneAnswer::NoTransition
    );
}

#[test]
fn start_of_day_uses_the_transition_when_midnight_is_skipped() {
    // America/Sao_Paulo skipped 2018-11-04T00:00 (-03:00 to -02:00).
    let midnight = local(2018, 11, 4, 0, 0);
    assert_eq!(
        ask(
            "America/Sao_Paulo",
            TemporalTimeZoneQuery::StartOfDay {
                local_midnight: midnight
            }
        ),
        TemporalTimeZoneAnswer::Seconds(midnight + 3 * 3_600)
    );
    // America/St_Johns 2010-11-07 starts twice; the first start wins.
    let repeated = local(2010, 11, 7, 0, 0);
    assert_eq!(
        ask(
            "America/St_Johns",
            TemporalTimeZoneQuery::StartOfDay {
                local_midnight: repeated
            }
        ),
        TemporalTimeZoneAnswer::Seconds(repeated + 2 * 3_600 + 1_800)
    );
}

#[test]
fn offset_matching_prefers_exact_then_rounded_minutes() {
    use TemporalOffsetMatch::{Exactly, Minutes};
    use TemporalOffsetMismatch::{Prefer, Reject};
    // Africa/Monrovia used -00:44:30 in 1970.
    let at = local(1970, 1, 1, 0, 0);
    let query = |offset_nanoseconds, mismatch, matching| {
        ask(
            "Africa/Monrovia",
            TemporalTimeZoneQuery::EpochForOffset {
                local: whole(at),
                offset_nanoseconds,
                mismatch,
                matching,
                disambiguation: TemporalDisambiguation::Compatible,
            },
        )
    };
    let exact = -(44 * 60 + 30) * 1_000_000_000;
    let rounded = -45 * 60 * 1_000_000_000;
    assert_eq!(
        query(exact, Reject, Exactly),
        TemporalTimeZoneAnswer::Seconds(at + 44 * 60 + 30)
    );
    assert_eq!(
        query(rounded, Reject, Minutes),
        TemporalTimeZoneAnswer::Seconds(at + 44 * 60 + 30)
    );
    assert_eq!(
        query(rounded, Reject, Exactly),
        TemporalTimeZoneAnswer::RangeError(TemporalTimeZoneRangeError::OffsetMismatch)
    );
    assert_eq!(
        query(0, Prefer, Exactly),
        TemporalTimeZoneAnswer::Seconds(at + 44 * 60 + 30)
    );
}

#[test]
fn offset_zones_use_the_balanced_date_and_instant_limits() {
    let zone = "-05:30";
    let minutes = TemporalOffsetMinutes::new(-330).unwrap();
    assert_eq!(minutes.seconds(), -19_800);
    assert_eq!(
        seconds(epoch_for(zone, 0, TemporalDisambiguation::Reject)),
        19_800
    );
    assert_eq!(
        ask(zone, TemporalTimeZoneQuery::OffsetAt { epoch: whole(0) }),
        TemporalTimeZoneAnswer::Seconds(-19_800)
    );
    // The last local time whose instant is valid, and the first after it.
    let last_local = TEMPORAL_EPOCH_SECONDS_LIMIT - 19_800;
    assert_eq!(
        seconds(epoch_for(zone, last_local, TemporalDisambiguation::Compatible)),
        TEMPORAL_EPOCH_SECONDS_LIMIT
    );
    assert_eq!(
        ask(
            zone,
            TemporalTimeZoneQuery::EpochFor {
                local: TemporalSeconds::new(last_local, true),
                disambiguation: TemporalDisambiguation::Compatible,
            }
        ),
        TemporalTimeZoneAnswer::RangeError(TemporalTimeZoneRangeError::OutOfRange)
    );
    assert_eq!(
        epoch_for(zone, last_local + 1, TemporalDisambiguation::Compatible),
        TemporalTimeZoneAnswer::RangeError(TemporalTimeZoneRangeError::OutOfRange)
    );
}

#[test]
fn named_candidates_at_the_instant_limits_are_range_checked() {
    let lower = -TEMPORAL_EPOCH_SECONDS_LIMIT;
    // UTC is a named zone whose only offset is zero.
    assert_eq!(
        seconds(epoch_for("UTC", lower, TemporalDisambiguation::Compatible)),
        lower
    );
    assert_eq!(
        epoch_for("UTC", lower - 1, TemporalDisambiguation::Compatible),
        TemporalTimeZoneAnswer::RangeError(TemporalTimeZoneRangeError::OutOfRange)
    );
    assert_eq!(
        epoch_for("UTC", i64::MAX / 2, TemporalDisambiguation::Compatible),
        TemporalTimeZoneAnswer::RangeError(TemporalTimeZoneRangeError::OutOfRange)
    );
}

/// Every change of the selected offset is found by the transition search, in
/// both directions, and nothing else is: explicit records, the footer rule,
/// and the hand-off between them.
#[test]
fn transition_search_agrees_with_the_offset_selector_for_every_zone() {
    let start = local(1800, 1, 1, 0, 0);
    let end = local(2060, 1, 1, 0, 0);
    for identifier in jiff_tzdb::available() {
        let rules = named(identifier);
        let mut transitions = Vec::new();
        let mut cursor = start;
        while let Some(found) = rules.next_transition(cursor).unwrap() {
            if found > end {
                break;
            }
            assert!(found > cursor, "{identifier} at {cursor}");
            assert!(rules.is_transition(found).unwrap(), "{identifier} at {found}");
            transitions.push(found);
            cursor = found;
        }
        // Between consecutive transitions the offset is the same at both ends
        // (a single missed change would show there) and at evenly spaced
        // probes (a missed change-and-return would show there).
        let mut previous_edge = start;
        for edge in transitions.iter().copied().chain([end]) {
            let offset = rules.offset_at(previous_edge).unwrap();
            for step in 0..32 {
                let probe = previous_edge + (edge - previous_edge) * step / 32;
                assert_eq!(
                    rules.offset_at(probe).unwrap(),
                    offset,
                    "{identifier}: unexplained change before {probe}"
                );
            }
            assert_eq!(rules.offset_at(edge - 1).unwrap(), offset, "{identifier} before {edge}");
            previous_edge = edge;
        }
        for pair in transitions.windows(2) {
            assert_eq!(
                rules.previous_transition(pair[1]).unwrap(),
                Some(pair[0]),
                "{identifier} before {}",
                pair[1]
            );
            assert_eq!(
                rules.previous_transition(pair[0] + 1).unwrap(),
                Some(pair[0]),
                "{identifier} just after {}",
                pair[0]
            );
        }
    }
}

#[test]
fn every_candidate_has_its_own_offset_around_every_transition() {
    for identifier in ["America/New_York", "Australia/Lord_Howe", "Antarctica/Casey", "Europe/Dublin"] {
        let rules = named(identifier);
        let mut cursor = local(1990, 1, 1, 0, 0);
        while let Some(found) = rules.next_transition(cursor).unwrap() {
            if found > local(2040, 1, 1, 0, 0) {
                break;
            }
            for delta in [-7_200, -1, 0, 1, 1_800, 7_200] {
                let at = found + rules.offset_at(found).unwrap() + delta;
                for candidate in rules.candidates(at).unwrap() {
                    assert_eq!(
                        candidate + rules.offset_at(candidate).unwrap(),
                        at,
                        "{identifier} local {at}"
                    );
                }
            }
            cursor = found;
        }
    }
}

// The ABI carries arbitrary i64 words. Reject extreme exact times and return
// the specified RangeError for local times before doing transition arithmetic.
#[test]
fn extreme_wire_seconds_never_overflow_the_time_zone_kernel() {
    for identifier in ["America/New_York", "UTC", "+23:59", "-23:59"] {
        for seconds in [i64::MIN, i64::MAX] {
            let at = TemporalSeconds::new(seconds, true);
            for query in [
                TemporalTimeZoneQuery::OffsetAt { epoch: at },
                TemporalTimeZoneQuery::Transition {
                    epoch: at,
                    direction: TemporalTransitionDirection::Next,
                },
                TemporalTimeZoneQuery::Transition {
                    epoch: at,
                    direction: TemporalTransitionDirection::Previous,
                },
                TemporalTimeZoneQuery::EpochFor {
                    local: at,
                    disambiguation: TemporalDisambiguation::Compatible,
                },
                TemporalTimeZoneQuery::EpochForOffset {
                    local: at,
                    offset_nanoseconds: 0,
                    mismatch: TemporalOffsetMismatch::Reject,
                    matching: TemporalOffsetMatch::Exactly,
                    disambiguation: TemporalDisambiguation::Compatible,
                },
                TemporalTimeZoneQuery::StartOfDay {
                    local_midnight: seconds,
                },
            ] {
                let request = TemporalTimeZoneRequest::new(
                    TemporalTimeZone::parse_stored(identifier).unwrap(),
                    query,
                );
                let decoded = TemporalTimeZoneRequest::decode(&request.encode()).unwrap();
                let actual = answer(zones(), &decoded);
                match query {
                    TemporalTimeZoneQuery::OffsetAt { .. }
                    | TemporalTimeZoneQuery::Transition { .. } => {
                        assert!(matches!(
                            actual,
                            Err(TemporalTimeZoneError::InvalidRequest(_))
                        ));
                    }
                    TemporalTimeZoneQuery::EpochFor { .. }
                    | TemporalTimeZoneQuery::EpochForOffset { .. }
                    | TemporalTimeZoneQuery::StartOfDay { .. } => {
                        assert_eq!(actual.unwrap(), OUT_OF_RANGE);
                    }
                }
            }
        }
    }
}
