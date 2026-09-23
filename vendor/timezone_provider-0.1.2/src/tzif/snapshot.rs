//! Exact offset/DST stability for bounded localized-name windows.

use super::{
    utils, DstTransitionInfoForYear, Seconds, TimeZoneProviderError, TimeZoneProviderResult,
    TimeZoneTransitionInfo, Tzif,
};

const MAX_WINDOW_SECONDS: i64 = 2 * 366 * 86_400;
const MAX_EPOCH_SECONDS: i64 = 8_640_000_000_000 + MAX_WINDOW_SECONDS;

fn same_snapshot(left: TimeZoneTransitionInfo, right: TimeZoneTransitionInfo) -> bool {
    (left.offset, left.is_dst) == (right.offset, right.is_dst)
}

impl Tzif {
    /// Whether offset and DST identity are constant on the inclusive interval.
    ///
    /// The domain covers Temporal/Date instants with up to two years of name
    /// context. A bounded window keeps POSIX rule enumeration finite; it is not
    /// an endpoint comparison, which would miss two intervening DST changes.
    pub fn offset_and_dst_are_constant(
        &self,
        start: Seconds,
        end: Seconds,
    ) -> TimeZoneProviderResult<bool> {
        if !(-MAX_EPOCH_SECONDS..=MAX_EPOCH_SECONDS).contains(&start.0)
            || !(-MAX_EPOCH_SECONDS..=MAX_EPOCH_SECONDS).contains(&end.0)
            || end.0 < start.0
            || end.0 - start.0 > MAX_WINDOW_SECONDS
        {
            return Err(TimeZoneProviderError::Range(
                "Invalid transition snapshot window",
            ));
        }
        let baseline = self.get(&start)?;
        let block = self.get_data_block2()?;
        let first = block
            .transition_times
            .partition_point(|epoch| *epoch <= start);
        for epoch in block.transition_times[first..]
            .iter()
            .take_while(|epoch| **epoch <= end)
        {
            if !same_snapshot(baseline, self.get(epoch)?) {
                return Ok(false);
            }
        }
        let tail_start = match block.transition_times.last() {
            Some(last) if end <= *last => return Ok(true),
            Some(last) => Seconds(start.0.max(last.0 + 1)),
            None => start,
        };
        if !same_snapshot(baseline, self.get(&tail_start)?)
            || !same_snapshot(baseline, self.get(&end)?)
        {
            return Ok(false);
        }
        let Some(footer) = self.posix_tz_string() else {
            // An empty transition table can use type zero for every instant.
            return Ok(true);
        };
        let Some(daylight) = &footer.dst_info else {
            return Ok(true);
        };
        // POSIX v3 transition times can spill into a neighboring calendar year.
        let first_year = utils::epoch_time_to_iso_year(tail_start.0 * 1000) - 1;
        let last_year = utils::epoch_time_to_iso_year(end.0 * 1000) + 1;
        for year in first_year..=last_year {
            let transitions = DstTransitionInfoForYear::compute(footer, daylight, year);
            for boundary in [transitions.dst_start_seconds, transitions.dst_end_seconds] {
                if tail_start < boundary && boundary <= end {
                    if !same_snapshot(baseline, self.get(&Seconds(boundary.0 - 1))?)
                        || !same_snapshot(baseline, self.get(&boundary)?)
                    {
                        return Ok(false);
                    }
                }
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tzif::data::tzif::{DataBlock, LocalTimeTypeRecord};

    fn zone(identifier: &str) -> Tzif {
        Tzif::from_bytes(jiff_tzdb::get(identifier).unwrap().1).unwrap()
    }

    #[test]
    fn every_snapshot_branch_carries_the_selected_dst_flag() {
        let mut value = zone("UTC");
        value.footer = None;
        value.data_block2 = Some(DataBlock {
            local_time_type_records: alloc::vec![LocalTimeTypeRecord {
                utoff: Seconds(123),
                is_dst: true,
                idx: 0,
            }],
            ..DataBlock::default()
        });
        assert!(value.get(&Seconds(0)).unwrap().is_dst);
        value
            .data_block2
            .as_mut()
            .unwrap()
            .transition_times
            .push(Seconds(10));
        value.data_block2.as_mut().unwrap().transition_types.push(0);
        assert!(value.get(&Seconds(9)).unwrap().is_dst);
        assert!(value.get(&Seconds(10)).unwrap().is_dst);
        assert!(!zone("UTC").get(&Seconds(64_060_588_800)).unwrap().is_dst);
        let new_york = zone("America/New_York");
        assert!(!new_york.get(&Seconds(64_060_588_800)).unwrap().is_dst);
        assert!(new_york.get(&Seconds(64_076_313_600)).unwrap().is_dst);
    }

    #[test]
    fn equal_endpoints_do_not_hide_intermediate_posix_dst_changes() {
        let new_york = zone("America/New_York");
        let start = Seconds(64_060_588_800); // 4000-01-01
        let end = Seconds(start.0 + 366 * 86_400);
        assert!(same_snapshot(
            new_york.get(&start).unwrap(),
            new_york.get(&end).unwrap()
        ));
        assert!(!new_york.offset_and_dst_are_constant(start, end).unwrap());
        assert!(zone("Asia/Kathmandu")
            .offset_and_dst_are_constant(start, end)
            .unwrap());
    }

    #[test]
    fn equal_offset_dst_change_and_closed_upper_boundary_are_observed() {
        let lisbon = zone("Europe/Lisbon");
        assert!(!lisbon
            .offset_and_dst_are_constant(Seconds(717_555_599), Seconds(717_555_600))
            .unwrap());
        assert!(lisbon
            .offset_and_dst_are_constant(Seconds(717_555_600), Seconds(717_555_601))
            .unwrap());
    }
}
