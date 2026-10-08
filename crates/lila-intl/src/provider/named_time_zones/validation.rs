use timezone_provider::tzif::Tzif;
use tzif::data::posix::TransitionDay;

use crate::{InvalidTimeZoneData, NamedTimeZoneOffsetSeconds};

/// Tzif's selector assumes these invariants and otherwise defaults missing
/// records. Validate once before an immutable snapshot becomes queryable.
pub(super) fn validate(tzif: &Tzif) -> Result<(), InvalidTimeZoneData> {
    let block = tzif
        .get_data_block2()
        .map_err(|_| InvalidTimeZoneData("pinned TZif record lacks a 64-bit block"))?;
    if block.local_time_type_records.is_empty()
        || block.transition_times.len() != block.transition_types.len()
        || block
            .transition_times
            .windows(2)
            .any(|times| times[0] >= times[1])
        || block
            .transition_types
            .iter()
            .any(|&index| index >= block.local_time_type_records.len())
    {
        return Err(InvalidTimeZoneData(
            "invalid pinned TZif transition indices",
        ));
    }
    for record in &block.local_time_type_records {
        NamedTimeZoneOffsetSeconds::from_data(record.utoff.0)?;
    }
    if !block.leap_second_records.is_empty() {
        return Err(InvalidTimeZoneData(
            "pinned TZif uses leap-adjusted timestamps",
        ));
    }
    if !block.transition_times.is_empty() && tzif.footer.is_none() {
        return Err(InvalidTimeZoneData(
            "pinned TZif record lacks its future rule",
        ));
    }
    if let Some(footer) = &tzif.footer {
        NamedTimeZoneOffsetSeconds::from_data(
            footer
                .std_info
                .offset
                .0
                .checked_neg()
                .ok_or(InvalidTimeZoneData("POSIX offset negation overflow"))?,
        )?;
        if let Some(daylight) = &footer.dst_info {
            NamedTimeZoneOffsetSeconds::from_data(
                daylight
                    .variant_info
                    .offset
                    .0
                    .checked_neg()
                    .ok_or(InvalidTimeZoneData("POSIX offset negation overflow"))?,
            )?;
            for transition in [daylight.start_date, daylight.end_date] {
                let valid = match transition.day {
                    TransitionDay::NoLeap(day) => (1..=365).contains(&day),
                    TransitionDay::WithLeap(day) => day <= 365,
                    TransitionDay::Mwd(month, week, weekday) => {
                        (1..=12).contains(&month) && (1..=5).contains(&week) && weekday <= 6
                    }
                };
                if !valid || !(-604_799..=604_799).contains(&transition.time.0) {
                    return Err(InvalidTimeZoneData("invalid pinned POSIX transition date"));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broken_transition_indices_are_rejected_before_selection() {
        let (_, bytes) = jiff_tzdb::get("America/New_York").unwrap();
        let valid = Tzif::from_bytes(bytes).unwrap();
        let mut missing_types = valid.clone();
        missing_types
            .data_block2
            .as_mut()
            .unwrap()
            .local_time_type_records
            .clear();
        assert!(validate(&missing_types).is_err());
        let mut invalid_index = valid.clone();
        invalid_index.data_block2.as_mut().unwrap().transition_types[0] = usize::MAX;
        assert!(validate(&invalid_index).is_err());
        let mut no_future_rule = valid;
        no_future_rule.footer = None;
        assert!(validate(&no_future_rule).is_err());
    }
}
