//! Temporal's time-zone operations over the pinned IANA records.
//!
//! [`ZoneRules`] is the one owner of "what does this zone do at this instant":
//! a fixed offset, or the named zone's TZif record read through the same
//! `Tzif::get` selector `Intl.DateTimeFormat` uses. Transitions are defined as
//! the instants where that selector's UTC offset changes, so a transition
//! search can never disagree with an offset lookup. Everything above it —
//! candidate exact times for a local date-time, disambiguation, start of day —
//! is written once against [`ZoneRules`] and handles both kinds of zone.

use timezone_provider::tzif::Tzif;
use tzif::data::posix::{PosixTzString, TransitionDate, TransitionDay};
use tzif::data::time::Seconds;

use crate::temporal_time_zone::{
    InvalidTemporalTimeZoneRequest, TemporalDisambiguation, TemporalOffsetMatch,
    TemporalOffsetMismatch, TemporalSeconds, TemporalTimeZone, TemporalTimeZoneAnswer,
    TemporalTimeZoneError, TemporalTimeZoneQuery, TemporalTimeZoneRangeError,
    TemporalTimeZoneRequest, TemporalTransitionDirection, TEMPORAL_EPOCH_SECONDS_LIMIT,
    TEMPORAL_ISO_DAYS_LIMIT,
};
use crate::InvalidTimeZoneData;

use super::named_time_zones::NamedTimeZones;

const SECONDS_PER_DAY: i64 = 86_400;
/// Instants and local times the kernel is asked about stay within the instant
/// range widened by the day windows the algorithms look through.
const QUERY_SECONDS_LIMIT: i64 = TEMPORAL_EPOCH_SECONDS_LIMIT + 8 * SECONDS_PER_DAY;

enum ZoneRules<'a> {
    Offset(i64),
    Named(&'a Tzif),
}

/// One maximal run of a single offset inside a search window.
#[derive(Debug, Clone, Copy)]
struct OffsetInterval {
    start: i64,
    end: i64,
    offset: i64,
}

impl ZoneRules<'_> {
    fn offset_at(&self, seconds: i64) -> Result<i64, TemporalTimeZoneError> {
        match self {
            Self::Offset(offset) => Ok(*offset),
            Self::Named(rules) => {
                let selected = rules
                    .get(&Seconds(seconds))
                    .map_err(|_| InvalidTimeZoneData("pinned transition selection failed"))?;
                let offset = selected.offset.0;
                if offset.unsigned_abs() >= SECONDS_PER_DAY as u64 {
                    return Err(InvalidTimeZoneData("pinned offset is not within one day").into());
                }
                Ok(offset)
            }
        }
    }

    fn is_transition(&self, seconds: i64) -> Result<bool, TemporalTimeZoneError> {
        Ok(self.offset_at(seconds - 1)? != self.offset_at(seconds)?)
    }

    /// The first instant strictly after `after` where the offset changes.
    fn next_transition(&self, after: i64) -> Result<Option<i64>, TemporalTimeZoneError> {
        let Self::Named(rules) = self else {
            return Ok(None);
        };
        let block = rules
            .get_data_block2()
            .map_err(|_| InvalidTimeZoneData("pinned TZif record lacks a 64-bit block"))?;
        let times = &block.transition_times;
        let mut index = times.partition_point(|time| time.0 <= after);
        while index < times.len() {
            if table_offset_changes(block, index) {
                return Ok(Some(times[index].0));
            }
            index += 1;
        }
        self.next_footer_transition(after, times.last().map(|time| time.0))
    }

    /// The last instant strictly before `before` where the offset changes.
    fn previous_transition(&self, before: i64) -> Result<Option<i64>, TemporalTimeZoneError> {
        let Self::Named(rules) = self else {
            return Ok(None);
        };
        let block = rules
            .get_data_block2()
            .map_err(|_| InvalidTimeZoneData("pinned TZif record lacks a 64-bit block"))?;
        let times = &block.transition_times;
        let last_explicit = times.last().map(|time| time.0);
        if let Some(found) = self.previous_footer_transition(before, last_explicit)? {
            return Ok(Some(found));
        }
        let mut index = times.partition_point(|time| time.0 < before);
        while index > 0 {
            index -= 1;
            if table_offset_changes(block, index) {
                return Ok(Some(times[index].0));
            }
        }
        Ok(None)
    }

    /// Candidate change points of the footer rule after the explicit table:
    /// the rule's own transitions for the UTC years around `around`, the UTC
    /// year boundaries the selector evaluates the rule by, and the hand-off
    /// right after the last explicit transition. Only candidates where the
    /// selected offset really changes are transitions.
    fn footer_candidates(
        rules: &Tzif,
        around: i64,
        last_explicit: Option<i64>,
        years: core::ops::RangeInclusive<i64>,
    ) -> Vec<i64> {
        let mut candidates = Vec::with_capacity(12);
        if let Some(last) = last_explicit {
            candidates.push(last + 1);
        }
        let Some(footer) = rules.posix_tz_string() else {
            return candidates;
        };
        if footer.dst_info.is_none() {
            return candidates;
        }
        let year = year_of_epoch_seconds(around);
        for offset in years {
            let year = year + offset;
            candidates.push(days_from_civil(year, 1, 1) * SECONDS_PER_DAY);
            if let Some((start, end)) = footer_rule_instants(footer, year) {
                candidates.push(start);
                candidates.push(end);
            }
        }
        candidates
    }

    fn next_footer_transition(
        &self,
        after: i64,
        last_explicit: Option<i64>,
    ) -> Result<Option<i64>, TemporalTimeZoneError> {
        let Self::Named(rules) = self else {
            return Ok(None);
        };
        let floor = last_explicit.map_or(after, |last| after.max(last));
        // A daylight rule changes the offset at least once a year, so a
        // four-year span holds the next change whenever one exists.
        let mut candidates = Self::footer_candidates(rules, floor, last_explicit, -1..=3);
        candidates.retain(|candidate| *candidate > floor && *candidate <= QUERY_SECONDS_LIMIT);
        candidates.sort_unstable();
        candidates.dedup();
        for candidate in candidates {
            if self.is_transition(candidate)? {
                return Ok(Some(candidate));
            }
        }
        Ok(None)
    }

    fn previous_footer_transition(
        &self,
        before: i64,
        last_explicit: Option<i64>,
    ) -> Result<Option<i64>, TemporalTimeZoneError> {
        let Self::Named(rules) = self else {
            return Ok(None);
        };
        if last_explicit.is_some_and(|last| before <= last + 1) {
            return Ok(None);
        }
        let mut candidates = Self::footer_candidates(rules, before, last_explicit, -3..=1);
        candidates.retain(|candidate| {
            *candidate < before
                && *candidate >= -QUERY_SECONDS_LIMIT
                && last_explicit.is_none_or(|last| *candidate > last)
        });
        candidates.sort_unstable_by(|left, right| right.cmp(left));
        candidates.dedup();
        for candidate in candidates {
            if self.is_transition(candidate)? {
                return Ok(Some(candidate));
            }
        }
        Ok(None)
    }

    /// Runs of one offset covering the UTC window `[low, high)`.
    fn intervals(&self, low: i64, high: i64) -> Result<Vec<OffsetInterval>, TemporalTimeZoneError> {
        let mut intervals = Vec::with_capacity(4);
        let mut start = low;
        loop {
            let offset = self.offset_at(start)?;
            let next = self.next_transition(start)?.filter(|next| *next < high);
            let end = next.unwrap_or(high);
            intervals.push(OffsetInterval { start, end, offset });
            match next {
                Some(next) => start = next,
                None => return Ok(intervals),
            }
        }
    }

    /// `GetNamedTimeZoneEpochNanoseconds` in whole seconds (fixed offsets
    /// have their single candidate): every exact time whose wall clock is
    /// `local`, ascending. Every offset is strictly inside a day, so every
    /// candidate lies within a day of `local`.
    fn candidates(&self, local: i64) -> Result<Vec<i64>, TemporalTimeZoneError> {
        if let Self::Offset(offset) = self {
            return Ok(vec![local - offset]);
        }
        let mut result = Vec::with_capacity(2);
        for interval in self.intervals(local - SECONDS_PER_DAY, local + SECONDS_PER_DAY + 1)? {
            let candidate = local - interval.offset;
            if candidate >= interval.start && candidate < interval.end {
                result.push(candidate);
            }
        }
        result.sort_unstable();
        result.dedup();
        Ok(result)
    }

    /// The offsets in effect on either side of the skipped local time
    /// `local`: `DisambiguatePossibleEpochNanoseconds` steps 6-13, and the
    /// UTC instant where the first local time after the gap begins.
    fn gap(&self, local: i64) -> Result<(i64, OffsetInterval), TemporalTimeZoneError> {
        let intervals = self.intervals(local - SECONDS_PER_DAY, local + SECONDS_PER_DAY + 1)?;
        let before = intervals
            .iter()
            .filter(|interval| interval.end + interval.offset <= local)
            .max_by_key(|interval| interval.end + interval.offset);
        let after = intervals
            .iter()
            .filter(|interval| interval.start + interval.offset > local)
            .min_by_key(|interval| interval.start + interval.offset);
        match (before, after) {
            (Some(before), Some(after)) => Ok((before.offset, *after)),
            _ => Err(InvalidTimeZoneData("skipped local time has no neighbouring local times").into()),
        }
    }
}

/// Whether the explicit transition at `index` changes the selected offset.
fn table_offset_changes(block: &tzif::data::tzif::DataBlock, index: usize) -> bool {
    let record = |type_index: usize| block.local_time_type_records[type_index].utoff.0;
    let previous = if index == 0 {
        record(0)
    } else {
        record(block.transition_types[index - 1])
    };
    previous != record(block.transition_types[index])
}

pub(super) fn answer(
    zones: &NamedTimeZones,
    request: &TemporalTimeZoneRequest,
) -> Result<TemporalTimeZoneAnswer, TemporalTimeZoneError> {
    let rules = match request.zone() {
        TemporalTimeZone::Offset(minutes) => ZoneRules::Offset(minutes.seconds()),
        TemporalTimeZone::Named(identifier) => ZoneRules::Named(
            zones
                .rules(identifier)
                .ok_or_else(|| TemporalTimeZoneError::UnknownNamedZone(identifier.clone()))?,
        ),
    };
    match request.query() {
        TemporalTimeZoneQuery::OffsetAt { epoch } => {
            require_query_domain(epoch.seconds())?;
            Ok(TemporalTimeZoneAnswer::Seconds(rules.offset_at(epoch.seconds())?))
        }
        TemporalTimeZoneQuery::EpochFor {
            local,
            disambiguation,
        } => epoch_for(&rules, local, disambiguation),
        TemporalTimeZoneQuery::EpochForOffset {
            local,
            offset_nanoseconds,
            mismatch,
            matching,
            disambiguation,
        } => epoch_for_offset(
            &rules,
            local,
            offset_nanoseconds,
            mismatch,
            matching,
            disambiguation,
        ),
        TemporalTimeZoneQuery::StartOfDay { local_midnight } => {
            start_of_day(&rules, local_midnight)
        }
        TemporalTimeZoneQuery::Transition { epoch, direction } => {
            require_query_domain(epoch.seconds())?;
            let found = match direction {
                TemporalTransitionDirection::Next => rules
                    .next_transition(epoch.seconds())?
                    .filter(|seconds| *seconds <= TEMPORAL_EPOCH_SECONDS_LIMIT),
                // Strictly before `seconds + remainder`: a transition at the
                // whole second itself precedes a non-zero remainder.
                TemporalTransitionDirection::Previous => rules
                    .previous_transition(epoch.seconds() + i64::from(epoch.has_subsecond()))?
                    .filter(|seconds| *seconds >= -TEMPORAL_EPOCH_SECONDS_LIMIT),
            };
            Ok(found.map_or(
                TemporalTimeZoneAnswer::NoTransition,
                TemporalTimeZoneAnswer::Seconds,
            ))
        }
    }
}

fn require_query_domain(seconds: i64) -> Result<(), TemporalTimeZoneError> {
    if seconds.unsigned_abs() > QUERY_SECONDS_LIMIT as u64 {
        return Err(TemporalTimeZoneError::InvalidRequest(
            InvalidTemporalTimeZoneRequest("exact time is outside the Temporal domain"),
        ));
    }
    Ok(())
}

const OUT_OF_RANGE: TemporalTimeZoneAnswer =
    TemporalTimeZoneAnswer::RangeError(TemporalTimeZoneRangeError::OutOfRange);

/// `IsValidEpochNanoseconds` of `seconds` plus a remainder.
fn is_valid_epoch(seconds: i64, has_subsecond: bool) -> bool {
    seconds >= -TEMPORAL_EPOCH_SECONDS_LIMIT
        && (seconds < TEMPORAL_EPOCH_SECONDS_LIMIT
            || (seconds == TEMPORAL_EPOCH_SECONDS_LIMIT && !has_subsecond))
}

/// `GetPossibleEpochNanoseconds`, or the RangeError it throws.
fn possible(
    rules: &ZoneRules<'_>,
    local: TemporalSeconds,
) -> Result<Result<Vec<i64>, TemporalTimeZoneAnswer>, TemporalTimeZoneError> {
    // Every candidate is within a day of `local`; beyond that margin none can
    // be a valid exact time, and the rules are not consulted.
    if local.seconds().unsigned_abs() > (TEMPORAL_EPOCH_SECONDS_LIMIT + 2 * SECONDS_PER_DAY) as u64
    {
        return Ok(Err(OUT_OF_RANGE));
    }
    if let ZoneRules::Offset(offset) = rules {
        // `CheckISODaysRange` of the balanced date, then the instant range.
        let balanced = local.seconds() - offset;
        if balanced.div_euclid(SECONDS_PER_DAY).unsigned_abs() > TEMPORAL_ISO_DAYS_LIMIT as u64 {
            return Ok(Err(OUT_OF_RANGE));
        }
    }
    let candidates = rules.candidates(local.seconds())?;
    if candidates
        .iter()
        .any(|candidate| !is_valid_epoch(*candidate, local.has_subsecond()))
    {
        return Ok(Err(OUT_OF_RANGE));
    }
    Ok(Ok(candidates))
}

fn epoch_for(
    rules: &ZoneRules<'_>,
    local: TemporalSeconds,
    disambiguation: TemporalDisambiguation,
) -> Result<TemporalTimeZoneAnswer, TemporalTimeZoneError> {
    let candidates = match possible(rules, local)? {
        Ok(candidates) => candidates,
        Err(answer) => return Ok(answer),
    };
    disambiguate(rules, &candidates, local, disambiguation)
}

/// `DisambiguatePossibleEpochNanoseconds`.
fn disambiguate(
    rules: &ZoneRules<'_>,
    candidates: &[i64],
    local: TemporalSeconds,
    disambiguation: TemporalDisambiguation,
) -> Result<TemporalTimeZoneAnswer, TemporalTimeZoneError> {
    let direction = match (candidates, disambiguation) {
        ([only], _) => return Ok(TemporalTimeZoneAnswer::Seconds(*only)),
        (_, TemporalDisambiguation::Reject) => {
            return Ok(TemporalTimeZoneAnswer::RangeError(
                TemporalTimeZoneRangeError::Ambiguous,
            ))
        }
        ([first, _, ..], TemporalDisambiguation::Earlier | TemporalDisambiguation::Compatible) => {
            return Ok(TemporalTimeZoneAnswer::Seconds(*first))
        }
        ([_, .., last], TemporalDisambiguation::Later) => {
            return Ok(TemporalTimeZoneAnswer::Seconds(*last))
        }
        ([], TemporalDisambiguation::Earlier) => GapShift::Earlier,
        ([], TemporalDisambiguation::Compatible | TemporalDisambiguation::Later) => {
            GapShift::Later
        }
    };
    // Steps 6-26: shift the local time by the size of the gap and take the
    // outermost candidate on that side.
    let (offset_before, after) = rules.gap(local.seconds())?;
    let shift = after.offset - offset_before;
    let shifted = match direction {
        GapShift::Earlier => local.seconds() - shift,
        GapShift::Later => local.seconds() + shift,
    };
    let shifted = TemporalSeconds::new(shifted, local.has_subsecond());
    let candidates = match possible(rules, shifted)? {
        Ok(candidates) => candidates,
        Err(answer) => return Ok(answer),
    };
    let chosen = match direction {
        GapShift::Earlier => candidates.first(),
        GapShift::Later => candidates.last(),
    };
    chosen
        .copied()
        .map(TemporalTimeZoneAnswer::Seconds)
        .ok_or_else(|| {
            InvalidTimeZoneData("local time moved across a gap is still skipped").into()
        })
}

/// Which side of a skipped local time `DisambiguatePossibleEpochNanoseconds`
/// resolves to.
#[derive(Clone, Copy)]
enum GapShift {
    Earlier,
    Later,
}

/// `InterpretISODateTimeOffset` steps 7-14.
fn epoch_for_offset(
    rules: &ZoneRules<'_>,
    local: TemporalSeconds,
    offset_nanoseconds: i64,
    mismatch: TemporalOffsetMismatch,
    matching: TemporalOffsetMatch,
    disambiguation: TemporalDisambiguation,
) -> Result<TemporalTimeZoneAnswer, TemporalTimeZoneError> {
    if local.seconds().div_euclid(SECONDS_PER_DAY).unsigned_abs() > TEMPORAL_ISO_DAYS_LIMIT as u64 {
        return Ok(OUT_OF_RANGE);
    }
    let candidates = match possible(rules, local)? {
        Ok(candidates) => candidates,
        Err(answer) => return Ok(answer),
    };
    for candidate in &candidates {
        // The candidate keeps the local remainder, so its offset is exactly
        // the whole-second difference.
        let candidate_offset = local.seconds() - candidate;
        if candidate_offset * 1_000_000_000 == offset_nanoseconds {
            return Ok(TemporalTimeZoneAnswer::Seconds(*candidate));
        }
        match matching {
            TemporalOffsetMatch::Exactly => {}
            TemporalOffsetMatch::Minutes => {
                if round_half_expand(candidate_offset, 60) * 1_000_000_000 == offset_nanoseconds {
                    return Ok(TemporalTimeZoneAnswer::Seconds(*candidate));
                }
            }
        }
    }
    match mismatch {
        TemporalOffsetMismatch::Reject => Ok(TemporalTimeZoneAnswer::RangeError(
            TemporalTimeZoneRangeError::OffsetMismatch,
        )),
        TemporalOffsetMismatch::Prefer => {
            disambiguate(rules, &candidates, local, disambiguation)
        }
    }
}

/// `GetStartOfDay`.
fn start_of_day(
    rules: &ZoneRules<'_>,
    local_midnight: i64,
) -> Result<TemporalTimeZoneAnswer, TemporalTimeZoneError> {
    let local = TemporalSeconds::new(local_midnight, false);
    let candidates = match possible(rules, local)? {
        Ok(candidates) => candidates,
        Err(answer) => return Ok(answer),
    };
    if let Some(first) = candidates.first() {
        return Ok(TemporalTimeZoneAnswer::Seconds(*first));
    }
    // Midnight is skipped: the day starts with the first local time after the
    // gap, which is the transition itself.
    let (_, after) = rules.gap(local_midnight)?;
    if !is_valid_epoch(after.start, false) {
        return Ok(OUT_OF_RANGE);
    }
    Ok(TemporalTimeZoneAnswer::Seconds(after.start))
}

/// `RoundNumberToIncrement(value, increment, half-expand)`.
fn round_half_expand(value: i64, increment: i64) -> i64 {
    let quotient = value / increment;
    let remainder = value % increment;
    if remainder.abs() * 2 >= increment {
        (quotient + value.signum()) * increment
    } else {
        quotient * increment
    }
}

/// The footer rule's daylight start and end instants in `year`, with the
/// arithmetic the TZif selector uses: a rule date's local time is converted
/// with the offset in effect before the change.
fn footer_rule_instants(footer: &PosixTzString, year: i64) -> Option<(i64, i64)> {
    let daylight = footer.dst_info.as_ref()?;
    // POSIX offsets count seconds west of UTC.
    let standard = -footer.std_info.offset.0;
    let daylight_offset = -daylight.variant_info.offset.0;
    Some((
        rule_instant(year, daylight.start_date, standard),
        rule_instant(year, daylight.end_date, daylight_offset),
    ))
}

fn rule_instant(year: i64, date: TransitionDate, utc_offset: i64) -> i64 {
    let year_start = days_from_civil(year, 1, 1);
    let leap = is_leap_year(year);
    let day_of_year = match date.day {
        TransitionDay::NoLeap(day) => {
            let day = i64::from(day);
            if day > 59 {
                day - 1 + i64::from(leap)
            } else {
                day - 1
            }
        }
        TransitionDay::WithLeap(day) => i64::from(day),
        TransitionDay::Mwd(month, week, weekday) => {
            let month = i64::from(month);
            let first = days_from_civil(year, month, 1);
            // 1970-01-01 was a Thursday; weekday 0 is Sunday.
            let first_weekday = (first + 4).rem_euclid(7);
            let mut day = (i64::from(week) - 1) * 7 + (i64::from(weekday) - first_weekday).rem_euclid(7);
            if day >= days_in_month(year, month) {
                day -= 7;
            }
            first - year_start + day
        }
    };
    (year_start + day_of_year) * SECONDS_PER_DAY + date.time.0 - utc_offset
}

fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Days since 1970-01-01 of a proleptic Gregorian date.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_index = (month + 9) % 12;
    let day_of_year = (153 * month_index + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn year_of_epoch_seconds(seconds: i64) -> i64 {
    let days = seconds.div_euclid(SECONDS_PER_DAY) + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let year = year_of_era + era * 400;
    if month_index >= 10 {
        year + 1
    } else {
        year
    }
}

#[cfg(test)]
mod tests;
