//! A complete proof for one actual parsed POSIX footer, shared within a catalogue.

use std::sync::Arc;
use timezone_provider::tzif::{PosixOffsetChangeCycle, Tzif};
use tzif::data::posix::PosixTzString;

use super::{certify_footer_cycle, FooterTopologyCache};
use crate::{InvalidTimeZoneData, NamedTimeZoneOffsetSeconds};

/// A cycle cannot be obtained independently of its complete gap certificate.
pub(super) struct FooterTopology {
    footer: PosixTzString,
    cycle: PosixOffsetChangeCycle,
}

impl FooterTopology {
    pub(super) fn cycle(&self) -> &PosixOffsetChangeCycle {
        &self.cycle
    }
}

/// With no explicit transitions and a present footer, Tzif::get always calls
/// the existing POSIX selector. Its complete offset range is the footer's
/// negated standard/daylight offsets; historical local types cannot be selected.
/// Keeping this projection private prevents that smaller catalogue from being
/// used for an actual record's historical/handoff proof or inverse queries.
struct FooterOnlyZone {
    transitions: Tzif,
    offsets: Vec<NamedTimeZoneOffsetSeconds>,
}

impl FooterOnlyZone {
    fn new(transitions: &Tzif) -> Result<Self, InvalidTimeZoneData> {
        let footer = transitions
            .footer
            .as_ref()
            .ok_or(InvalidTimeZoneData("footer projection lacks its rule"))?;
        let offset = |seconds: i64| {
            NamedTimeZoneOffsetSeconds::from_data(
                seconds
                    .checked_neg()
                    .ok_or(InvalidTimeZoneData("POSIX offset negation overflow"))?,
            )
        };
        let mut offsets = vec![offset(footer.std_info.offset.0)?];
        if let Some(daylight) = &footer.dst_info {
            offsets.push(offset(daylight.variant_info.offset.0)?);
        }
        offsets.sort_unstable();
        offsets.dedup();
        let mut transitions = transitions.clone();
        let block = transitions
            .data_block2
            .as_mut()
            .ok_or(InvalidTimeZoneData("gap topology footer lacks block"))?;
        block.transition_times.clear();
        block.transition_types.clear();
        Ok(Self {
            transitions,
            offsets,
        })
    }
}

impl FooterTopologyCache {
    pub(super) fn admit(
        &mut self,
        transitions: &Tzif,
    ) -> Result<Option<Arc<FooterTopology>>, InvalidTimeZoneData> {
        let Some(footer) = &transitions.footer else {
            return Ok(None);
        };
        if let Some(admitted) = self.entries.iter().find(|entry| entry.footer == *footer) {
            return Ok(Some(Arc::clone(admitted)));
        }
        let pure = FooterOnlyZone::new(transitions)?;
        let cycle = pure
            .transitions
            .posix_offset_change_cycle()
            .map_err(|_| InvalidTimeZoneData("pinned POSIX offset cycle failed"))?
            .ok_or(InvalidTimeZoneData("footer projection lost its rule"))?;
        certify_footer_cycle(&pure.transitions, &pure.offsets, &cycle)?;
        let admitted = Arc::new(FooterTopology {
            footer: footer.clone(),
            cycle,
        });
        // Publish only after every phase and relevant remote copy succeeds.
        self.entries.push(Arc::clone(&admitted));
        Ok(Some(admitted))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use tzif::data::time::Seconds;

    fn zone(name: &str) -> Tzif {
        Tzif::from_bytes(jiff_tzdb::get(name).unwrap().1).unwrap()
    }

    #[test]
    fn footer_sharing_requires_the_entire_actual_parsed_rule() {
        let original = zone("America/New_York");
        let mut cache = FooterTopologyCache::default();
        let first = cache.admit(&original).unwrap().unwrap();
        let mut other_history = original.clone();
        let block = other_history.data_block2.as_mut().unwrap();
        block.transition_times.clear();
        block.transition_types.clear();
        block.local_time_type_records[0].utoff = Seconds(1234);
        let shared = cache.admit(&other_history).unwrap().unwrap();
        assert!(Arc::ptr_eq(&first, &shared));
        let mut changed = original.clone();
        changed.footer.as_mut().unwrap().std_info.name.push('X');
        let distinct = cache.admit(&changed).unwrap().unwrap();
        assert!(!Arc::ptr_eq(&first, &distinct));
        let mut changed = original.clone();
        changed
            .footer
            .as_mut()
            .unwrap()
            .dst_info
            .as_mut()
            .unwrap()
            .start_date
            .time = Seconds(10800);
        let distinct = cache.admit(&changed).unwrap().unwrap();
        assert!(!Arc::ptr_eq(&first, &distinct));
        assert_eq!(cache.entries.len(), 3);
        let mut invalid = original;
        invalid.footer.as_mut().unwrap().std_info.offset = Seconds(i64::MIN);
        assert!(cache.admit(&invalid).is_err());
        assert_eq!(cache.entries.len(), 3);
    }

    #[test]
    fn every_pinned_footer_preserves_the_uncached_full_catalogue_proof() {
        let mut cache = FooterTopologyCache::default();
        let mut seen = BTreeSet::new();
        let mut footers = Vec::new();
        let mut records = 0;
        for name in jiff_tzdb::available() {
            records += 1;
            let raw = jiff_tzdb::get(name).unwrap().1;
            if !seen.insert(raw) {
                continue;
            }
            let actual = Tzif::from_bytes(raw).unwrap();
            super::super::super::validation::validate(&actual).unwrap();
            let full_offsets =
                super::super::super::exact_query::validated_offset_catalogue(&actual).unwrap();
            let direct = actual.posix_offset_change_cycle().unwrap();
            let admitted = cache.admit(&actual).unwrap();
            match (&actual.footer, direct, admitted) {
                (Some(footer), Some(cycle), Some(admitted)) => {
                    if !footers.contains(footer) {
                        footers.push(footer.clone());
                    }
                    assert_eq!(admitted.cycle.base_seconds(), cycle.base_seconds());
                    assert_eq!(admitted.cycle.period_seconds(), cycle.period_seconds());
                    assert_eq!(admitted.cycle.relative_seconds(), cycle.relative_seconds());
                    let pure = FooterOnlyZone::new(&actual).unwrap();
                    assert!(pure
                        .offsets
                        .iter()
                        .all(|offset| full_offsets.contains(offset)));
                    // Retain the original exhaustive proof with every historical
                    // offset as an independent reference for all pinned inputs.
                    certify_footer_cycle(&pure.transitions, &full_offsets, &cycle).unwrap();
                }
                (None, None, None) => {}
                _ => panic!("footer/cycle/proof presence differs"),
            }
        }
        assert_eq!(records, 598);
        assert_eq!(seen.len(), 341);
        assert_eq!(cache.entries.len(), footers.len());
        assert!(cache.entries.len() < seen.len());
    }
}
