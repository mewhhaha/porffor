//! Exact POSIX-tail snapshots, including rules that cross UTC year boundaries.

use super::snapshot::MAX_EPOCH_SECONDS;
use super::{
    utils, DstTransitionInfo, DstTransitionInfoForYear, PosixTzString, Seconds,
    TimeZoneProviderError, TimeZoneProviderResult, TimeZoneTransitionInfo, TransitionDay,
    UtcOffsetSeconds,
};

/// RFC9636 3.3.1 (and RFC8536 3.3.1) permits perpetual daylight saving time.
/// The January start and December end coincide in UTC across adjacent years;
/// the standard type has no interval and must never become a snapshot.
fn all_year_daylight_saving(
    rules: &DstTransitionInfo,
    standard: UtcOffsetSeconds,
    daylight: UtcOffsetSeconds,
) -> bool {
    matches!(
        rules.start_date.day,
        TransitionDay::WithLeap(0) | TransitionDay::NoLeap(1)
    ) && rules.start_date.time == Seconds(0)
        && rules.end_date.day == TransitionDay::NoLeap(365)
        && rules.end_date.time.0 == 86_400 + daylight.0 - standard.0
}

pub(super) fn resolve(
    footer: &PosixTzString,
    seconds: i64,
) -> TimeZoneProviderResult<TimeZoneTransitionInfo> {
    // Lila queries the full Date/Temporal domain plus the complete two-year
    // localized-name context accepted by snapshot.rs. Reject a wider calendar
    // arithmetic domain explicitly instead of overflowing the shared utility.
    if !(-MAX_EPOCH_SECONDS..=MAX_EPOCH_SECONDS).contains(&seconds) {
        return Err(TimeZoneProviderError::Range(
            "POSIX snapshot epoch is outside the context domain",
        ));
    }
    let standard = UtcOffsetSeconds::from(&footer.std_info);
    let Some(rules) = &footer.dst_info else {
        return Ok(TimeZoneTransitionInfo {
            offset: standard,
            transition_epoch: None,
            is_dst: false,
        });
    };
    let daylight = UtcOffsetSeconds::from(&rules.variant_info);
    if all_year_daylight_saving(rules, standard, daylight) {
        return Ok(TimeZoneTransitionInfo {
            offset: daylight,
            transition_epoch: None,
            is_dst: true,
        });
    }

    let milliseconds = seconds
        .checked_mul(1000)
        .ok_or(TimeZoneProviderError::Range(
            "POSIX snapshot epoch overflow",
        ))?;
    let year = utils::epoch_time_to_iso_year(milliseconds);
    let previous_year = year
        .checked_sub(1)
        .ok_or(TimeZoneProviderError::Range("POSIX previous year overflow"))?;
    let next_year = year
        .checked_add(1)
        .ok_or(TimeZoneProviderError::Range("POSIX next year overflow"))?;
    let mut selected = None;
    // TZif v3 times can be signed and as large as 167:59:59. Rules nominally
    // in either adjacent year can therefore be the active UTC transition.
    // Parsed POSIX offsets (including the default DST adjustment) are
    // bounded by 26 hours, so an additional calendar year cannot intervene.
    for candidate_year in [previous_year, year, next_year] {
        let boundaries = DstTransitionInfoForYear::compute(footer, rules, candidate_year);
        // A start followed by an end at the same UTC instant has an empty
        // daylight interval: the end wins the tie. RFC-defined all-year DST
        // is a distinct rule form and was selected before this event scan.
        for (epoch, is_dst) in [
            (boundaries.dst_start_seconds, true),
            (boundaries.dst_end_seconds, false),
        ] {
            if epoch.0 <= seconds
                && selected
                    .as_ref()
                    .is_none_or(|previous: &TimeZoneTransitionInfo| {
                        previous
                            .transition_epoch
                            .is_some_and(|value| value < epoch.0 || (value == epoch.0 && !is_dst))
                    })
            {
                selected = Some(TimeZoneTransitionInfo {
                    offset: if is_dst { daylight } else { standard },
                    transition_epoch: Some(epoch.0),
                    is_dst,
                });
            }
        }
    }
    selected.ok_or(TimeZoneProviderError::Assert(
        "No preceding POSIX transition",
    ))
}

#[cfg(test)]
mod tests {
    use super::super::{CandidateEpochNanoseconds, IsoDateTime, Tzif};
    use super::*;
    use tzif::data::{posix::TransitionDate, tzif::LocalTimeTypeRecord};

    fn pinned(identifier: &str) -> Tzif {
        Tzif::from_bytes(jiff_tzdb::get(identifier).unwrap().1).unwrap()
    }

    fn epoch(year: i32, day: TransitionDay, time: i64) -> i64 {
        super::super::calculate_transition_seconds_for_year(
            year,
            TransitionDate {
                day,
                time: Seconds(time),
            },
            UtcOffsetSeconds(0),
        )
    }

    fn seasonal(start: TransitionDate, end: TransitionDate) -> Tzif {
        let mut value = pinned("UTC");
        let mut footer = pinned("America/New_York").footer.unwrap();
        footer.std_info.offset = Seconds(0);
        let daylight = footer.dst_info.as_mut().unwrap();
        daylight.variant_info.offset = Seconds(-3600);
        daylight.start_date = start;
        daylight.end_date = end;
        value.footer = Some(footer);
        value
    }

    #[test]
    fn pinned_all_year_dst_has_no_synthetic_december_standard_interval() {
        for identifier in ["Africa/Casablanca", "Africa/El_Aaiun"] {
            let value = pinned(identifier);
            for year in [2088, 2100, 4000] {
                let december_31 = epoch(year, TransitionDay::NoLeap(365), 0);
                for second_in_day in [0, 79_199, 79_200, 80_999, 86_399, 86_400] {
                    let selected = value.get(&Seconds(december_31 + second_in_day)).unwrap();
                    assert_eq!((selected.offset.0, selected.is_dst), (3600, true));
                }
                assert!(value
                    .offset_and_dst_are_constant(
                        Seconds(december_31),
                        Seconds(december_31 + 86_400)
                    )
                    .unwrap());
            }
        }
    }

    #[test]
    fn signed_start_time_selects_the_following_nominal_year() {
        let value = seasonal(
            TransitionDate {
                day: TransitionDay::NoLeap(1),
                time: Seconds(-7200),
            },
            TransitionDate {
                day: TransitionDay::NoLeap(2),
                time: Seconds(7200),
            },
        );
        let boundary = epoch(2026, TransitionDay::NoLeap(1), -7200);
        for (seconds, expected) in [(boundary - 1, 0), (boundary, 3600), (boundary + 1, 3600)] {
            let selected = value.get(&Seconds(seconds)).unwrap();
            assert_eq!(selected.offset.0, expected);
            if expected != 0 {
                assert_eq!(selected.transition_epoch, Some(boundary));
            }
        }
        assert!(!value
            .offset_and_dst_are_constant(Seconds(boundary - 1), Seconds(boundary))
            .unwrap());
    }

    #[test]
    fn end_time_above_twenty_four_hours_selects_the_previous_nominal_year() {
        let value = seasonal(
            TransitionDate {
                day: TransitionDay::NoLeap(100),
                time: Seconds(0),
            },
            TransitionDate {
                day: TransitionDay::NoLeap(365),
                time: Seconds(180_000),
            },
        );
        let boundary = epoch(2025, TransitionDay::NoLeap(365), 180_000) - 3600;
        for (seconds, expected) in [(boundary - 1, 3600), (boundary, 0), (boundary + 1, 0)] {
            let selected = value.get(&Seconds(seconds)).unwrap();
            assert_eq!(selected.offset.0, expected);
            if expected == 0 {
                assert_eq!(selected.transition_epoch, Some(boundary));
            }
        }
    }

    #[test]
    fn same_utc_start_and_end_has_no_daylight_interval() {
        let value = seasonal(
            TransitionDate {
                day: TransitionDay::Mwd(3, 2, 0),
                time: Seconds(7200),
            },
            TransitionDate {
                day: TransitionDay::Mwd(3, 2, 0),
                time: Seconds(10800),
            },
        );
        let boundary = epoch(2025, TransitionDay::Mwd(3, 2, 0), 7200);
        for seconds in [
            epoch(2025, TransitionDay::NoLeap(1), 0),
            boundary - 1,
            boundary,
            boundary + 1,
            epoch(2025, TransitionDay::NoLeap(180), 0),
            epoch(2025, TransitionDay::NoLeap(365), 0),
        ] {
            let selected = value.get(&Seconds(seconds)).unwrap();
            assert_eq!((selected.offset.0, selected.is_dst), (0, false));
        }
    }

    #[test]
    fn contextual_selector_keeps_the_full_epoch_range_and_rejects_wider_arithmetic() {
        let value = pinned("America/New_York");
        let footer = value.footer.as_ref().unwrap();
        let limit = 8_640_000_000_000_i64;
        let context = 2 * 366 * 86_400;
        for seconds in [-limit - context, -limit, limit, limit + context] {
            assert!(resolve(footer, seconds).is_ok());
        }
        for seconds in [
            -limit - context - 1,
            limit + context + 1,
            i64::MIN,
            i64::MAX,
        ] {
            assert!(resolve(footer, seconds).is_err());
        }
    }

    #[test]
    fn prior_year_transition_metadata_uses_the_pre_transition_offset() {
        let january = epoch(4000, TransitionDay::NoLeap(1), 0);
        for (identifier, expected_dst) in [("America/New_York", false), ("Australia/Sydney", true)]
        {
            let value = pinned(identifier);
            let footer = value.footer.as_ref().unwrap();
            let previous =
                DstTransitionInfoForYear::compute(footer, footer.dst_info.as_ref().unwrap(), 3999);
            let selected = value.get(&Seconds(january)).unwrap();
            assert_eq!(selected.is_dst, expected_dst);
            assert_eq!(
                selected.transition_epoch,
                Some(if expected_dst {
                    previous.dst_start_seconds.0
                } else {
                    previous.dst_end_seconds.0
                })
            );
        }
    }

    #[test]
    fn negative_fractional_epoch_and_local_inputs_floor_before_selection() {
        let mut value = pinned("UTC");
        value.footer = None;
        let block = value.data_block2.as_mut().unwrap();
        block.local_time_type_records = alloc::vec![
            LocalTimeTypeRecord {
                utoff: Seconds(0),
                is_dst: false,
                idx: 0
            },
            LocalTimeTypeRecord {
                utoff: Seconds(3600),
                is_dst: false,
                idx: 0
            },
        ];
        block.transition_times = alloc::vec![Seconds(0)];
        block.transition_types = alloc::vec![1];
        assert_eq!(
            value
                .transition_nanoseconds_for_utc_epoch_nanoseconds(-1)
                .unwrap()
                .0,
            0
        );
        assert_eq!(
            value
                .transition_nanoseconds_for_utc_epoch_nanoseconds(0)
                .unwrap()
                .0,
            3600
        );
        let local = IsoDateTime {
            year: 1969,
            month: 12,
            day: 31,
            hour: 23,
            minute: 59,
            second: 59,
            millisecond: 999,
            microsecond: 999,
            nanosecond: 999,
        };
        let CandidateEpochNanoseconds::One(candidate) = value
            .candidate_nanoseconds_for_local_epoch_nanoseconds(local)
            .unwrap()
        else {
            panic!("the nanosecond before the local gap must have exactly one candidate")
        };
        assert_eq!(candidate.ns.0, -1);
        assert_eq!(candidate.offset.0, 0);
    }
}
