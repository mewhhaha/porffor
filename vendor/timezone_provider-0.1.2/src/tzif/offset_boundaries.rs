//! Offset-change data built through the existing exact snapshot selector.

use alloc::vec::Vec;

use super::snapshot::MAX_EPOCH_SECONDS;
use super::{
    utils, DstTransitionInfoForYear, Seconds, TimeZoneProviderError, TimeZoneProviderResult, Tzif,
};

const MAX_WINDOW_SECONDS: i64 = 2 * 366 * 86_400;
const CYCLE_BASE_SECONDS: i64 = 64_060_588_800; // 4000-01-01T00:00Z
const CYCLE_SECONDS: i64 = 146_097 * 86_400;

/// Actual POSIX UTC offset changes over one complete Gregorian cycle.
/// An empty cycle proves that the footer's offset is constant. DST flags and
/// abbreviations alone never populate this list.
#[derive(Debug, Clone)]
pub struct PosixOffsetChangeCycle {
    relative_seconds: Vec<i64>,
}
impl PosixOffsetChangeCycle {
    pub const fn base_seconds(&self) -> i64 {
        CYCLE_BASE_SECONDS
    }
    pub const fn period_seconds(&self) -> i64 {
        CYCLE_SECONDS
    }
    pub fn relative_seconds(&self) -> &[i64] {
        &self.relative_seconds
    }
}

impl Tzif {
    /// Sorted, deduplicated real offset changes in a bounded inclusive window.
    /// Every returned second is checked at T-1 and T through `get`. This covers
    /// explicit records, rule spills and the actual table/footer handoff.
    pub fn offset_change_boundaries(
        &self,
        start: Seconds,
        end: Seconds,
    ) -> TimeZoneProviderResult<Vec<Seconds>> {
        validate_window(start, end)?;
        let block = self.get_data_block2()?;
        let first = block
            .transition_times
            .partition_point(|epoch| *epoch < start);
        let mut candidates: Vec<Seconds> = block.transition_times[first..]
            .iter()
            .copied()
            .take_while(|epoch| *epoch <= end)
            .collect();
        let last = block.transition_times.last().copied();
        let handoff = last
            .map(|last| {
                last.0
                    .checked_add(1)
                    .ok_or(TimeZoneProviderError::Range("TZif handoff overflow"))
            })
            .transpose()?;
        if let Some(handoff) = handoff {
            if start.0 <= handoff && handoff <= end.0 {
                candidates.push(Seconds(handoff));
            }
        }
        if let Some(footer) = self.posix_tz_string() {
            if let Some(daylight) = &footer.dst_info {
                let tail_start = handoff.map_or(start.0, |handoff| start.0.max(handoff));
                if tail_start <= end.0 {
                    for year in neighboring_years(tail_start, end.0)? {
                        let rules = DstTransitionInfoForYear::compute(footer, daylight, year);
                        for boundary in [rules.dst_start_seconds, rules.dst_end_seconds] {
                            if tail_start <= boundary.0 && boundary <= end {
                                candidates.push(boundary);
                            }
                        }
                    }
                }
            }
        }
        candidates.sort_unstable();
        candidates.dedup();
        let mut changes = Vec::new();
        for boundary in candidates {
            let before = self.get(&Seconds(boundary.0 - 1))?;
            let after = self.get(&boundary)?;
            if before.offset != after.offset {
                changes.push(boundary);
            }
        }
        Ok(changes)
    }

    /// Compile the footer independently of explicit-table activation. The
    /// exact same POSIX selector used by `get` proves each boundary. Rule dates
    /// repeat after 400 Gregorian years, including leap-century differences.
    pub fn posix_offset_change_cycle(
        &self,
    ) -> TimeZoneProviderResult<Option<PosixOffsetChangeCycle>> {
        let Some(footer) = self.posix_tz_string() else {
            return Ok(None);
        };
        let Some(daylight) = &footer.dst_info else {
            return Ok(Some(PosixOffsetChangeCycle {
                relative_seconds: Vec::new(),
            }));
        };
        let mut candidates = Vec::new();
        for year in 3999..=4401 {
            let rules = DstTransitionInfoForYear::compute(footer, daylight, year);
            for boundary in [rules.dst_start_seconds, rules.dst_end_seconds] {
                if (CYCLE_BASE_SECONDS..CYCLE_BASE_SECONDS + CYCLE_SECONDS).contains(&boundary.0) {
                    candidates.push(boundary.0);
                }
            }
        }
        candidates.sort_unstable();
        candidates.dedup();
        let mut relative_seconds = Vec::new();
        for boundary in candidates {
            let before = super::posix_snapshot::resolve(footer, boundary - 1)?;
            let after = super::posix_snapshot::resolve(footer, boundary)?;
            if before.offset != after.offset {
                relative_seconds.push(boundary - CYCLE_BASE_SECONDS);
            }
        }
        Ok(Some(PosixOffsetChangeCycle { relative_seconds }))
    }
}

fn validate_window(start: Seconds, end: Seconds) -> TimeZoneProviderResult<()> {
    // T-1 must also be inside the selector's context.
    if start.0 <= -MAX_EPOCH_SECONDS
        || end.0 > MAX_EPOCH_SECONDS
        || end < start
        || end.0 - start.0 > MAX_WINDOW_SECONDS
    {
        return Err(TimeZoneProviderError::Range(
            "Invalid offset boundary window",
        ));
    }
    Ok(())
}
fn neighboring_years(
    start: i64,
    end: i64,
) -> TimeZoneProviderResult<core::ops::RangeInclusive<i32>> {
    let first = utils::epoch_time_to_iso_year(
        start
            .checked_mul(1000)
            .ok_or(TimeZoneProviderError::Range("POSIX year overflow"))?,
    )
    .checked_sub(1)
    .ok_or(TimeZoneProviderError::Range(
        "POSIX preceding year overflow",
    ))?;
    let last = utils::epoch_time_to_iso_year(
        end.checked_mul(1000)
            .ok_or(TimeZoneProviderError::Range("POSIX year overflow"))?,
    )
    .checked_add(1)
    .ok_or(TimeZoneProviderError::Range(
        "POSIX following year overflow",
    ))?;
    Ok(first..=last)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tzif::data::{
        posix::{TransitionDate, TransitionDay},
        tzif::LocalTimeTypeRecord,
    };
    fn zone(id: &str) -> Tzif {
        Tzif::from_bytes(jiff_tzdb::get(id).unwrap().1).unwrap()
    }

    #[test]
    fn offset_equal_records_and_perpetual_rules_have_no_fake_changes() {
        let lisbon = zone("Europe/Lisbon");
        assert!(lisbon
            .offset_change_boundaries(Seconds(717_555_599), Seconds(717_555_601))
            .unwrap()
            .is_empty());
        for id in ["UTC", "Africa/Casablanca", "Africa/El_Aaiun"] {
            assert!(zone(id)
                .posix_offset_change_cycle()
                .unwrap()
                .unwrap()
                .relative_seconds()
                .is_empty());
        }
        let mut metadata = zone("UTC");
        let block = metadata.data_block2.as_mut().unwrap();
        block.local_time_type_records.push(LocalTimeTypeRecord {
            utoff: Seconds(0),
            is_dst: true,
            idx: 0,
        });
        block.transition_times.push(Seconds(0));
        block.transition_types.push(1);
        assert!(metadata
            .offset_change_boundaries(Seconds(-1), Seconds(1))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn leap_only_rules_have_sparse_true_transitions_in_complete_cycle() {
        let mut value = zone("UTC");
        let mut footer = zone("America/New_York").footer.unwrap();
        footer.std_info.offset = Seconds(0);
        let dst = footer.dst_info.as_mut().unwrap();
        dst.variant_info.offset = Seconds(-3600);
        // These UTC boundaries coincide in ordinary years. In leap years the
        // end falls one day later, so there is a real one-day DST interval.
        dst.start_date = TransitionDate {
            day: TransitionDay::WithLeap(59),
            time: Seconds(0),
        };
        dst.end_date = TransitionDate {
            day: TransitionDay::NoLeap(60),
            time: Seconds(3600),
        };
        value.footer = Some(footer);
        let cycle = value.posix_offset_change_cycle().unwrap().unwrap();
        assert_eq!(cycle.relative_seconds().len(), 2 * 97);
        for relative in cycle.relative_seconds() {
            let t = cycle.base_seconds() + relative;
            assert_ne!(
                value.get(&Seconds(t - 1)).unwrap().offset,
                value.get(&Seconds(t)).unwrap().offset
            );
        }
    }

    #[test]
    fn seasonal_cycle_and_windows_agree_at_every_far_future_boundary() {
        for id in [
            "America/New_York",
            "Australia/Sydney",
            "Australia/Lord_Howe",
            "Europe/Dublin",
        ] {
            let value = zone(id);
            let cycle = value.posix_offset_change_cycle().unwrap().unwrap();
            assert_eq!(cycle.relative_seconds().len(), 800, "{id}");
            for relative in cycle.relative_seconds() {
                let t = cycle.base_seconds() + relative;
                assert_eq!(
                    value
                        .offset_change_boundaries(Seconds(t), Seconds(t))
                        .unwrap(),
                    [Seconds(t)]
                );
            }
        }
    }
}
