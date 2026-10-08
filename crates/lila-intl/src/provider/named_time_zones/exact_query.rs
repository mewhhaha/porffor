//! One forward authority for Intl snapshots and exact named-zone data queries.

use timezone_provider::tzif::{TimeZoneTransitionInfo, Tzif};
use tzif::data::time::Seconds;

use super::{NamedTimeZone, NamedTimeZones};
use crate::{
    FindNamedTimeZoneTransitionRequest, FindNamedTimeZoneTransitionResult, InvalidTimeZoneData,
    LocalTimeCoordinate, NamedTimeZoneCandidate, NamedTimeZoneDataError,
    NamedTimeZoneOffsetRequest, NamedTimeZoneOffsetSeconds, NamedTimeZoneTransitionDirection,
    PossibleNamedTimeZoneEpochsRequest, PossibleNamedTimeZoneEpochsResult, RawTimeZoneEpoch,
    SortedNamedTimeZoneCandidates, TimeZoneId, TimeZoneInstant,
};

const NANOS_PER_SECOND: i128 = 1_000_000_000;

/// A borrow tied to this catalogue record; identity and data cannot diverge.
pub(super) struct ResolvedNamedZone<'a>(&'a NamedTimeZone);
impl<'a> ResolvedNamedZone<'a> {
    pub(super) const fn new(zone: &'a NamedTimeZone) -> Self {
        Self(zone)
    }
    pub(super) fn snapshot(
        &self,
        seconds: i64,
    ) -> Result<TimeZoneTransitionInfo, InvalidTimeZoneData> {
        self.0
            .data
            .transitions
            .get(&Seconds(seconds))
            .map_err(|_| InvalidTimeZoneData("pinned transition selection failed"))
    }
    fn offset(&self, seconds: i64) -> Result<NamedTimeZoneOffsetSeconds, InvalidTimeZoneData> {
        NamedTimeZoneOffsetSeconds::from_data(self.snapshot(seconds)?.offset.0)
    }
    fn candidates(
        &self,
        local: LocalTimeCoordinate,
    ) -> Result<Vec<NamedTimeZoneCandidate>, InvalidTimeZoneData> {
        let mut candidates = Vec::new();
        for &tested in &self.0.data.offsets {
            let epoch = RawTimeZoneEpoch::from_nanoseconds(
                local.nanoseconds() - i128::from(tested.seconds()) * NANOS_PER_SECOND,
            )?;
            let actual = self.offset(epoch.seconds())?;
            if actual == tested {
                candidates.push(NamedTimeZoneCandidate::from_forward_match(
                    local, epoch, tested, actual,
                )?);
            }
        }
        candidates.sort_unstable_by_key(|candidate| candidate.epoch());
        candidates.dedup_by_key(|candidate| candidate.epoch());
        Ok(candidates)
    }
    fn inverse(
        &self,
        local: LocalTimeCoordinate,
    ) -> Result<PossibleNamedTimeZoneEpochsResult, InvalidTimeZoneData> {
        let candidates = self.candidates(local)?;
        if !candidates.is_empty() {
            return Ok(PossibleNamedTimeZoneEpochsResult::Candidates(
                SortedNamedTimeZoneCandidates::from_sorted(candidates, self.0.data.offsets.len())?,
            ));
        }
        let min = self
            .0
            .data
            .offsets
            .first()
            .ok_or(InvalidTimeZoneData("empty offset catalogue"))?
            .seconds();
        let max = self
            .0
            .data
            .offsets
            .last()
            .ok_or(InvalidTimeZoneData("empty offset catalogue"))?
            .seconds();
        let start = local.seconds() - i64::from(max);
        let end = local.seconds() - i64::from(min);
        let boundaries = self
            .0
            .data
            .transitions
            .offset_change_boundaries(Seconds(start), Seconds(end))
            .map_err(|_| InvalidTimeZoneData("pinned gap boundary enumeration failed"))?;
        let mut witness = None;
        for boundary in boundaries {
            if let Some(gap) = self.0.containing_gap(local, boundary.0)? {
                // The immutable owner derives both offsets from its certified
                // record. No detached marker or independent tuple crosses the
                // gap constructor; Wasm still owns disambiguation and checks.
                if witness.replace(gap).is_some() {
                    return Err(InvalidTimeZoneData(
                        "local input has conflicting gap witnesses",
                    ));
                }
            }
        }
        witness
            .map(PossibleNamedTimeZoneEpochsResult::Gap)
            .ok_or(InvalidTimeZoneData(
                "empty inverse has no actual UTC gap witness",
            ))
    }
    fn transition(
        &self,
        request: &FindNamedTimeZoneTransitionRequest,
    ) -> Result<FindNamedTimeZoneTransitionResult, InvalidTimeZoneData> {
        let instant = request.instant();
        let direction = request.direction();
        let block = self
            .0
            .data
            .transitions
            .get_data_block2()
            .map_err(|_| InvalidTimeZoneData("validated TZif block disappeared"))?;
        let mut selected = None;
        for epoch in &block.transition_times {
            self.consider_transition(epoch.0, instant, direction, &mut selected)?;
        }
        let handoff = block
            .transition_times
            .last()
            .map(|last| {
                last.0
                    .checked_add(1)
                    .ok_or(InvalidTimeZoneData("TZif tail handoff overflow"))
            })
            .transpose()?;
        if let Some(handoff) = handoff {
            self.consider_transition(handoff, instant, direction, &mut selected)?;
        }
        if let Some(cycle) = &self.0.data.tail_cycle {
            let period = i128::from(cycle.period_seconds());
            let base = i128::from(cycle.base_seconds());
            // The nearest phase in the requested direction is obtained with
            // Euclidean floor/ceil division. Enforce tail activation before
            // selection, so a rule cannot overwrite historical table data.
            let query_ns = instant.nanoseconds();
            for &relative in cycle.relative_seconds() {
                let phase = base + i128::from(relative);
                let candidate = match direction {
                    NamedTimeZoneTransitionDirection::Next => {
                        let lower_ns = handoff.map_or(query_ns, |start| {
                            query_ns.max(i128::from(start) * NANOS_PER_SECOND - 1)
                        });
                        let repetition = (lower_ns - phase * NANOS_PER_SECOND)
                            .div_euclid(period * NANOS_PER_SECOND)
                            + 1;
                        phase + repetition * period
                    }
                    NamedTimeZoneTransitionDirection::Previous => {
                        let repetition = (query_ns - 1 - phase * NANOS_PER_SECOND)
                            .div_euclid(period * NANOS_PER_SECOND);
                        let candidate = phase + repetition * period;
                        if handoff.is_some_and(|start| candidate < i128::from(start)) {
                            continue;
                        }
                        candidate
                    }
                };
                let candidate = i64::try_from(candidate)
                    .map_err(|_| InvalidTimeZoneData("POSIX transition epoch overflow"))?;
                self.consider_transition(candidate, instant, direction, &mut selected)?;
            }
        }
        FindNamedTimeZoneTransitionResult::from_seconds(selected)
    }
    fn consider_transition(
        &self,
        seconds: i64,
        query: TimeZoneInstant,
        direction: NamedTimeZoneTransitionDirection,
        selected: &mut Option<i64>,
    ) -> Result<(), InvalidTimeZoneData> {
        // The spec returns null when the nearest real transition is outside
        // Instant bounds. Prove this arithmetically before asking the bounded
        // forward selector; sparse rules may lie decades beyond its context.
        if !(-TimeZoneInstant::limit_seconds()..=TimeZoneInstant::limit_seconds())
            .contains(&seconds)
        {
            return Ok(());
        }
        let ns = i128::from(seconds) * NANOS_PER_SECOND;
        let eligible = match direction {
            NamedTimeZoneTransitionDirection::Next => {
                ns > query.nanoseconds() && selected.is_none_or(|previous| seconds < previous)
            }
            NamedTimeZoneTransitionDirection::Previous => {
                ns < query.nanoseconds() && selected.is_none_or(|previous| seconds > previous)
            }
        };
        if eligible && self.offset(seconds - 1)? != self.offset(seconds)? {
            *selected = Some(seconds);
        }
        Ok(())
    }
}

pub(super) fn validated_offset_catalogue(
    transitions: &Tzif,
) -> Result<Vec<NamedTimeZoneOffsetSeconds>, InvalidTimeZoneData> {
    let block = transitions
        .get_data_block2()
        .map_err(|_| InvalidTimeZoneData("missing offset catalogue block"))?;
    let mut offsets = Vec::new();
    for record in &block.local_time_type_records {
        offsets.push(NamedTimeZoneOffsetSeconds::from_data(record.utoff.0)?);
    }
    if let Some(footer) = transitions.posix_tz_string() {
        let actual = |seconds: i64| {
            seconds
                .checked_neg()
                .ok_or(InvalidTimeZoneData("POSIX offset negation overflow"))
                .and_then(NamedTimeZoneOffsetSeconds::from_data)
        };
        offsets.push(actual(footer.std_info.offset.0)?);
        if let Some(daylight) = &footer.dst_info {
            offsets.push(actual(daylight.variant_info.offset.0)?);
        }
    }
    offsets.sort_unstable();
    offsets.dedup();
    if offsets.is_empty() {
        return Err(InvalidTimeZoneData("empty named offset catalogue"));
    }
    Ok(offsets)
}

impl NamedTimeZones {
    fn resolved(
        &self,
        identifier: &TimeZoneId,
    ) -> Result<ResolvedNamedZone<'_>, NamedTimeZoneDataError> {
        let identity = self
            .lookup(identifier)
            .map_err(|_| NamedTimeZoneDataError::UnknownIdentifier(identifier.clone()))?;
        self.require_available(&identity)?;
        self.zones
            .get(&identifier.as_str().to_ascii_lowercase())
            .map(ResolvedNamedZone::new)
            .ok_or_else(|| NamedTimeZoneDataError::UnknownIdentifier(identifier.clone()))
    }
    pub(in crate::provider) fn exact_offset(
        &self,
        request: &NamedTimeZoneOffsetRequest,
    ) -> Result<NamedTimeZoneOffsetSeconds, NamedTimeZoneDataError> {
        Ok(self
            .resolved(request.identifier())?
            .offset(request.instant().seconds())?)
    }
    pub(in crate::provider) fn possible_epochs(
        &self,
        request: &PossibleNamedTimeZoneEpochsRequest,
    ) -> Result<PossibleNamedTimeZoneEpochsResult, NamedTimeZoneDataError> {
        Ok(self
            .resolved(request.identifier())?
            .inverse(request.local())?)
    }
    pub(in crate::provider) fn find_transition(
        &self,
        request: &FindNamedTimeZoneTransitionRequest,
    ) -> Result<FindNamedTimeZoneTransitionResult, NamedTimeZoneDataError> {
        Ok(self.resolved(request.identifier())?.transition(request)?)
    }
}

#[cfg(test)]
mod tests;
