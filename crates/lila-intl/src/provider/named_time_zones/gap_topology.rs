//! Complete gap topology validation, performed before a zone is queryable.
//!
//! All inverse epochs for a local interval [lo, hi) lie inside
//! [lo - maxOffset, hi - minOffset). Every constant-offset UTC segment in
//! that whole window is translated and checked: none may intersect the gap.
//! Complete catalogue inverse checks then prove lo-1ns and hi have the sole
//! epochs T-1ns and T. A checked marker is retained with the immutable zone;
//! it cannot be manufactured by a policy consumer or a raw UTC jump alone.

use timezone_provider::tzif::{PosixOffsetChangeCycle, Tzif};
use tzif::data::time::Seconds;

use crate::{
    InvalidTimeZoneData, LocalTimeCoordinate, NamedTimeZoneOffsetSeconds, RawTimeZoneEpoch,
    TimeZoneInstant,
};

use super::{exact_query::ResolvedNamedZone, NamedTimeZone};

const NS: i128 = 1_000_000_000;
const DAY: i64 = 86_400;
// Same documented forward-selector context: Instant plus two leap years.
const SELECTOR_CONTEXT: i64 = 8_640_000_000_000 + 2 * 366 * DAY;

/// Constructor-only proof retained by the immutable catalogue record.
/// The marker has no raw/public constructor and is never detached from data.
pub(super) struct GapTopology {
    _checked: (),
}
impl GapTopology {
    pub(super) fn validate(
        transitions: &Tzif,
        offsets: &[NamedTimeZoneOffsetSeconds],
        cycle: Option<&PosixOffsetChangeCycle>,
    ) -> Result<Self, InvalidTimeZoneData> {
        let block = transitions
            .get_data_block2()
            .map_err(|_| InvalidTimeZoneData("gap topology lacks TZif block"))?;
        for boundary in &block.transition_times {
            certify_jump(transitions, offsets, boundary.0)?;
        }
        if let Some(last) = block.transition_times.last() {
            let handoff = last
                .0
                .checked_add(1)
                .ok_or(InvalidTimeZoneData("gap topology handoff overflow"))?;
            // With |offset|<day and gap length<=day, a gap's complete inverse
            // window can meet the join only if T is within four days. This
            // deliberately wider inclusive window covers every such event.
            let start = handoff
                .checked_sub(4 * DAY)
                .ok_or(InvalidTimeZoneData("gap topology join window overflow"))?;
            let end = handoff
                .checked_add(4 * DAY)
                .ok_or(InvalidTimeZoneData("gap topology join window overflow"))?;
            for boundary in true_boundaries(transitions, start, end)? {
                certify_jump(transitions, offsets, boundary.0)?;
            }
        }
        if let Some(cycle) = cycle {
            // A private TZif clone changes only table activation. `get` stays
            // the single authority; no second footer offset selector exists.
            let mut pure_footer = transitions.clone();
            let block = pure_footer
                .data_block2
                .as_mut()
                .ok_or(InvalidTimeZoneData("gap topology footer lacks block"))?;
            block.transition_times.clear();
            block.transition_types.clear();
            for &relative in cycle.relative_seconds() {
                let phase = cycle
                    .base_seconds()
                    .checked_add(relative)
                    .ok_or(InvalidTimeZoneData("gap topology cycle phase overflow"))?;
                certify_jump(&pure_footer, offsets, phase)?;
                // The complete 400-year cycle is the proof for every repeated
                // tail segment. Remote copies additionally validate calendar
                // arithmetic on both sides of the supported Instant domain.
                for edge in [
                    -TimeZoneInstant::limit_seconds(),
                    TimeZoneInstant::limit_seconds(),
                ] {
                    let repetition = (i128::from(edge) - i128::from(phase))
                        .div_euclid(i128::from(cycle.period_seconds()));
                    for shift in [0_i128, 1] {
                        let translated = i128::from(phase)
                            + (repetition + shift) * i128::from(cycle.period_seconds());
                        // Copies beyond the selector context are unnecessary:
                        // the base-cycle proof already covers those phases.
                        // Certify every relevant remote jump inside context.
                        if (-i128::from(SELECTOR_CONTEXT - 4 * DAY)
                            ..=i128::from(SELECTOR_CONTEXT - 4 * DAY))
                            .contains(&translated)
                        {
                            let translated = i64::try_from(translated).map_err(|_| {
                                InvalidTimeZoneData("gap topology remote phase overflow")
                            })?;
                            certify_jump(&pure_footer, offsets, translated)?;
                        }
                    }
                }
            }
        }
        Ok(Self { _checked: () })
    }
}

/// A containing change derived from one immutable, certified zone. Independent
/// transition/offset tuples cannot manufacture this private-field witness.
pub(crate) struct CertifiedNamedGapBoundary {
    transition: RawTimeZoneEpoch,
    before: NamedTimeZoneOffsetSeconds,
    after: NamedTimeZoneOffsetSeconds,
}

impl CertifiedNamedGapBoundary {
    pub(crate) fn transition_seconds(&self) -> i64 {
        self.transition.seconds()
    }
    pub(crate) fn before(&self) -> NamedTimeZoneOffsetSeconds {
        self.before
    }
    pub(crate) fn after(&self) -> NamedTimeZoneOffsetSeconds {
        self.after
    }
}

pub(super) fn certified_containing_boundary(
    zone: &NamedTimeZone,
    local: LocalTimeCoordinate,
    boundary: i64,
) -> Result<Option<CertifiedNamedGapBoundary>, InvalidTimeZoneData> {
    // The shared data constructor validates this certificate before publication;
    // these exact snapshots come from that same immutable record and selector.
    let _certificate = &zone.data.gap_topology;
    let resolved = ResolvedNamedZone::new(zone);
    let predecessor = boundary
        .checked_sub(1)
        .ok_or(InvalidTimeZoneData("gap topology predecessor overflow"))?;
    let before = NamedTimeZoneOffsetSeconds::from_data(resolved.snapshot(predecessor)?.offset.0)?;
    let after = NamedTimeZoneOffsetSeconds::from_data(resolved.snapshot(boundary)?.offset.0)?;
    let utc = i128::from(boundary) * NS;
    if after <= before
        || local.nanoseconds() < utc + i128::from(before.seconds()) * NS
        || local.nanoseconds() >= utc + i128::from(after.seconds()) * NS
    {
        return Ok(None);
    }
    // A published certificate proves the complete containing gap and both
    // sole nearest endpoints, rather than merely this local jump's shape.
    if i64::from(after.seconds()) - i64::from(before.seconds()) > DAY {
        return Err(InvalidTimeZoneData("gap topology jump exceeds one day"));
    }
    let transition = RawTimeZoneEpoch::from_nanoseconds(utc)?;
    Ok(Some(CertifiedNamedGapBoundary {
        transition,
        before,
        after,
    }))
}

fn selected(transitions: &Tzif, seconds: i64) -> Result<i64, InvalidTimeZoneData> {
    if !(-SELECTOR_CONTEXT..=SELECTOR_CONTEXT).contains(&seconds) {
        return Err(InvalidTimeZoneData("gap topology exceeds selector context"));
    }
    let selected = transitions
        .get(&Seconds(seconds))
        .map_err(|_| InvalidTimeZoneData("gap topology forward selection failed"))?;
    Ok(i64::from(
        NamedTimeZoneOffsetSeconds::from_data(selected.offset.0)?.seconds(),
    ))
}
fn true_boundaries(
    transitions: &Tzif,
    start: i64,
    end: i64,
) -> Result<Vec<Seconds>, InvalidTimeZoneData> {
    transitions
        .offset_change_boundaries(Seconds(start), Seconds(end))
        .map_err(|_| InvalidTimeZoneData("gap topology segment enumeration failed"))
}
fn sole_epoch(
    transitions: &Tzif,
    offsets: &[NamedTimeZoneOffsetSeconds],
    local_ns: i128,
    expected_ns: i128,
) -> Result<(), InvalidTimeZoneData> {
    let mut found = None;
    for offset in offsets {
        let epoch_ns = local_ns - i128::from(offset.seconds()) * NS;
        let seconds = i64::try_from(epoch_ns.div_euclid(NS))
            .map_err(|_| InvalidTimeZoneData("gap topology endpoint overflow"))?;
        if selected(transitions, seconds)? == i64::from(offset.seconds()) {
            if found.replace(epoch_ns).is_some() {
                return Err(InvalidTimeZoneData("gap topology endpoint is ambiguous"));
            }
        }
    }
    if found != Some(expected_ns) {
        return Err(InvalidTimeZoneData("gap topology endpoint is not nearest"));
    }
    Ok(())
}
fn certify_jump(
    transitions: &Tzif,
    offsets: &[NamedTimeZoneOffsetSeconds],
    boundary: i64,
) -> Result<(), InvalidTimeZoneData> {
    let before_epoch = boundary
        .checked_sub(1)
        .ok_or(InvalidTimeZoneData("gap topology predecessor overflow"))?;
    let before = selected(transitions, before_epoch)?;
    let after = selected(transitions, boundary)?;
    if after <= before {
        return Ok(());
    }
    if after - before > DAY {
        return Err(InvalidTimeZoneData("gap topology jump exceeds one day"));
    }
    let min = offsets
        .first()
        .ok_or(InvalidTimeZoneData(
            "gap topology offset catalogue is empty",
        ))?
        .seconds();
    let max = offsets.last().unwrap().seconds();
    let lower = boundary
        .checked_add(before)
        .ok_or(InvalidTimeZoneData("gap topology local boundary overflow"))?;
    let upper = boundary
        .checked_add(after)
        .ok_or(InvalidTimeZoneData("gap topology local boundary overflow"))?;
    let start = lower
        .checked_sub(i64::from(max))
        .ok_or(InvalidTimeZoneData("gap topology inverse window overflow"))?;
    let end = upper
        .checked_sub(i64::from(min))
        .ok_or(InvalidTimeZoneData("gap topology inverse window overflow"))?;
    let mut cursor = start;
    for stop in true_boundaries(transitions, start, end)?
        .into_iter()
        .map(|value| value.0)
        .filter(|value| start < *value && *value < end)
        .chain(core::iter::once(end))
    {
        let offset = selected(transitions, cursor)?;
        let segment_lower = i128::from(cursor) + i128::from(offset);
        let segment_upper = i128::from(stop) + i128::from(offset);
        if segment_lower < i128::from(upper) && segment_upper > i128::from(lower) {
            return Err(InvalidTimeZoneData(
                "gap topology interval is not globally empty",
            ));
        }
        cursor = stop;
    }
    sole_epoch(
        transitions,
        offsets,
        i128::from(lower) * NS - 1,
        i128::from(boundary) * NS - 1,
    )?;
    sole_epoch(
        transitions,
        offsets,
        i128::from(upper) * NS,
        i128::from(boundary) * NS,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tzif::data::tzif::{DataBlock, LocalTimeTypeRecord};

    fn data(before: i64, after: i64, boundary: i64, footer: i64) -> Tzif {
        let mut value = Tzif::from_bytes(jiff_tzdb::get("UTC").unwrap().1).unwrap();
        value.data_block2 = Some(DataBlock {
            local_time_type_records: [before, after]
                .into_iter()
                .map(|offset| LocalTimeTypeRecord {
                    utoff: Seconds(offset),
                    is_dst: false,
                    idx: 0,
                })
                .collect(),
            transition_times: vec![Seconds(boundary)],
            transition_types: vec![1],
            ..DataBlock::default()
        });
        value.footer.as_mut().unwrap().std_info.offset = Seconds(-footer);
        value
    }
    fn checked(value: &Tzif) -> Result<GapTopology, InvalidTimeZoneData> {
        super::super::validation::validate(value)?;
        let offsets = super::super::exact_query::validated_offset_catalogue(value)?;
        let cycle = value.posix_offset_change_cycle().unwrap();
        GapTopology::validate(value, &offsets, cycle.as_ref())
    }

    #[test]
    fn footer_handoff_cannot_fill_an_explicit_gap_interior() {
        // Table T0 jumps to100, but the actual T1 footer handoff returns to0.
        // Epoch[1,100) fills the proposed local[0,100) gap. A jump and sampled
        // inverse emptiness are insufficient: publication must reject it.
        let error = checked(&data(0, 100, 0, 0)).err().unwrap();
        assert_eq!(error.0, "gap topology interval is not globally empty");
    }
    #[test]
    fn a_gap_larger_than_one_day_has_no_spec_endpoint_certificate() {
        let error = checked(&data(-50_000, 50_000, 0, 50_000)).err().unwrap();
        assert_eq!(error.0, "gap topology jump exceeds one day");
    }
    #[test]
    fn extreme_unqueryable_table_arithmetic_rejects_without_wrapping() {
        assert!(checked(&data(0, 100, i64::MIN, 100)).is_err());
        assert!(checked(&data(0, 100, i64::MAX, 100)).is_err());
    }
}
