use std::collections::{btree_map::Entry, BTreeMap};
use std::sync::Arc;

use crate::NamedTimeZoneOffsetSeconds;
use timezone_provider::tzif::PosixOffsetChangeCycle;

use sha2::{Digest, Sha256};
use timezone_provider::tzif::Tzif;
use tzif::data::time::Seconds;

use crate::{
    InvalidTimeZoneData, LocalTimeCoordinate, NamedTimeZoneGap, NamedTimeZoneIdentity,
    TimeZoneEpochSeconds, TimeZoneId, UnknownTimeZone,
};

use super::time_zone_snapshot::{
    NamedTimeZoneTransition, TimeZoneVariant, STANDARD_TIME_STABILITY_WINDOW_SECONDS,
};

mod catalogue;
mod exact_query;
mod gap_topology;
pub(crate) use gap_topology::CertifiedNamedGapBoundary;
use gap_topology::GapTopology;
mod identity;
mod validation;

pub(super) const PROVIDER_DATA_SHA256: [u8; 32] = identity::PROVIDER_DATA_SHA256;

struct NamedTimeZone {
    identity: NamedTimeZoneIdentity,
    data: Arc<AdmittedNamedTimeZoneData>,
}

/// One immutable TZif payload and every proof required by its query consumers.
/// Catalogue spellings and primary identities remain on each named record.
struct AdmittedNamedTimeZoneData {
    transitions: Tzif,
    offsets: Vec<NamedTimeZoneOffsetSeconds>,
    tail_cycle: Option<PosixOffsetChangeCycle>,
    gap_topology: GapTopology,
}

impl AdmittedNamedTimeZoneData {
    fn from_data(transitions: Tzif) -> Result<Self, InvalidTimeZoneData> {
        validation::validate(&transitions)?;
        let offsets = exact_query::validated_offset_catalogue(&transitions)?;
        let tail_cycle = transitions
            .posix_offset_change_cycle()
            .map_err(|_| InvalidTimeZoneData("pinned POSIX offset cycle failed"))?;
        let gap_topology = GapTopology::validate(&transitions, &offsets, tail_cycle.as_ref())?;
        Ok(Self {
            transitions,
            offsets,
            tail_cycle,
            gap_topology,
        })
    }
}

impl NamedTimeZone {
    #[cfg(test)]
    fn from_data(
        identity: NamedTimeZoneIdentity,
        transitions: Tzif,
    ) -> Result<Self, InvalidTimeZoneData> {
        Ok(Self {
            identity,
            data: Arc::new(AdmittedNamedTimeZoneData::from_data(transitions)?),
        })
    }

    fn containing_gap(
        &self,
        local: LocalTimeCoordinate,
        boundary: i64,
    ) -> Result<Option<NamedTimeZoneGap>, InvalidTimeZoneData> {
        Ok(
            gap_topology::certified_containing_boundary(self, local, boundary)?
                .map(NamedTimeZoneGap::from_certified_boundary),
        )
    }
}

/// Immutable identifiers and transition records from the same IANA release.
pub(crate) struct NamedTimeZones {
    zones: BTreeMap<String, NamedTimeZone>,
    identities: BTreeMap<String, NamedTimeZoneIdentity>,
    selection: Option<Box<[TimeZoneId]>>,
}

impl NamedTimeZones {
    pub(crate) fn from_image_data(
        catalogue: &str,
        records: &[(&str, &[u8])],
    ) -> Result<Self, InvalidTimeZoneData> {
        Self::from_records(catalogue, records, None)
    }

    pub(crate) fn from_projection(
        catalogue: &crate::named_time_zone_image::projection::NamedZoneCatalogue<'_>,
    ) -> Result<Self, InvalidTimeZoneData> {
        Self::from_records(
            catalogue.catalogue(),
            catalogue.records(),
            Some(catalogue.selection()),
        )
    }

    fn from_records(
        catalogue: &str,
        records: &[(&str, &[u8])],
        selection: Option<&[TimeZoneId]>,
    ) -> Result<Self, InvalidTimeZoneData> {
        let rows = catalogue::read(catalogue)?;
        if rows.len() != 598 || (selection.is_none() && records.len() != rows.len()) {
            return Err(InvalidTimeZoneData("named image catalogue/record extent"));
        }
        let mut zones = BTreeMap::new();
        let mut identities = BTreeMap::new();
        // This map exists only for this construction and borrows the actual
        // record bytes. Equality compares the complete payload, so even a
        // digest collision cannot reuse another payload's admission proofs.
        let mut admitted: BTreeMap<&[u8], Arc<AdmittedNamedTimeZoneData>> = BTreeMap::new();
        let mut records = records.iter();
        for row in rows {
            let key = row.identity.identifier().to_ascii_lowercase();
            if identities
                .insert(key.clone(), row.identity.clone())
                .is_some()
            {
                return Err(InvalidTimeZoneData("case-insensitive catalogue collision"));
            }
            let retained = selection.is_none_or(|selected| {
                selected
                    .binary_search_by(|name| name.as_str().cmp(row.identity.primary_identifier()))
                    .is_ok()
            });
            if !retained {
                continue;
            }
            let &(normalized, bytes) = records
                .next()
                .ok_or(InvalidTimeZoneData("selected transition record missing"))?;
            if normalized != row.identity.identifier() {
                return Err(InvalidTimeZoneData(
                    "catalogue and transition spelling differ",
                ));
            }
            let actual_digest: [u8; 32] = Sha256::digest(bytes).into();
            if actual_digest != row.tzif_digest {
                return Err(InvalidTimeZoneData(
                    "pinned transition record digest mismatch",
                ));
            }
            let data = match admitted.entry(bytes) {
                Entry::Occupied(entry) => Arc::clone(entry.get()),
                Entry::Vacant(entry) => {
                    let transitions = Tzif::from_bytes(bytes)
                        .map_err(|_| InvalidTimeZoneData("invalid pinned TZif record"))?;
                    Arc::clone(
                        entry.insert(Arc::new(AdmittedNamedTimeZoneData::from_data(transitions)?)),
                    )
                }
            };
            if zones
                .insert(
                    key,
                    NamedTimeZone {
                        identity: row.identity,
                        data,
                    },
                )
                .is_some()
            {
                return Err(InvalidTimeZoneData("case-insensitive catalogue collision"));
            }
        }
        if records.next().is_some() {
            return Err(InvalidTimeZoneData("extra selected transition record"));
        }
        for identity in identities.values() {
            let primary = identities
                .get(&identity.primary_identifier().to_ascii_lowercase())
                .ok_or(InvalidTimeZoneData("catalogue primary is missing"))?;
            if primary.identifier() != identity.primary_identifier()
                || primary.identifier() != primary.primary_identifier()
            {
                return Err(InvalidTimeZoneData("catalogue primary is not terminal"));
            }
        }
        let utc = zones
            .get("utc")
            .ok_or(InvalidTimeZoneData("UTC identity is absent"))?;
        if utc.identity.identifier() != "UTC" || utc.identity.primary_identifier() != "UTC" {
            return Err(InvalidTimeZoneData("UTC primary identity is invalid"));
        }
        Ok(Self {
            zones,
            identities,
            selection: selection.map(|names| names.to_vec().into_boxed_slice()),
        })
    }

    /// Enumerate admitted catalogue spellings, including aliases, from the
    /// same immutable owner used by lookup and transition operations.
    pub(crate) fn identifiers(&self) -> impl Iterator<Item = &str> {
        self.identities
            .values()
            .map(NamedTimeZoneIdentity::identifier)
    }

    pub(crate) fn lookup(
        &self,
        identifier: &TimeZoneId,
    ) -> Result<NamedTimeZoneIdentity, UnknownTimeZone> {
        self.identities
            .get(&identifier.as_str().to_ascii_lowercase())
            .cloned()
            .ok_or_else(|| UnknownTimeZone::new(identifier.clone()))
    }

    pub(crate) fn transition(
        &self,
        identity: &NamedTimeZoneIdentity,
        epoch: TimeZoneEpochSeconds,
    ) -> Result<NamedTimeZoneTransition, crate::NamedTimeZoneDataError> {
        self.require_available(identity)?;
        let zone = self
            .zones
            .get(&identity.identifier().to_ascii_lowercase())
            .ok_or(InvalidTimeZoneData("resolved named identifier is absent"))?;
        if &zone.identity != identity {
            return Err(
                InvalidTimeZoneData("resolved named identity disagrees with catalogue").into(),
            );
        }
        let selected = exact_query::ResolvedNamedZone::new(zone).snapshot(epoch.get())?;
        let offset_seconds = NamedTimeZoneOffsetSeconds::from_data(selected.offset.0)?.seconds();
        let variant = if selected.is_dst {
            TimeZoneVariant::Daylight
        } else {
            TimeZoneVariant::Standard
        };
        if variant == TimeZoneVariant::Standard
            && zone
                .data
                .transitions
                .offset_and_dst_are_constant(
                    Seconds(epoch.get() - STANDARD_TIME_STABILITY_WINDOW_SECONDS),
                    Seconds(epoch.get() + STANDARD_TIME_STABILITY_WINDOW_SECONDS),
                )
                .map_err(|_| {
                    InvalidTimeZoneData("pinned standard-time stability selection failed")
                })?
        {
            return Ok(NamedTimeZoneTransition::stable_standard(offset_seconds));
        }
        Ok(NamedTimeZoneTransition::new(offset_seconds, variant))
    }
    pub(crate) fn named_time_zone_selection(&self) -> Option<&[TimeZoneId]> {
        self.selection.as_deref()
    }
    pub(crate) fn require_available(
        &self,
        identity: &NamedTimeZoneIdentity,
    ) -> Result<(), crate::NamedTimeZoneDataError> {
        let name = TimeZoneId::parse(identity.identifier())
            .map_err(|_| InvalidTimeZoneData("invalid resolved zone identity"))?;
        let actual = self
            .lookup(&name)
            .map_err(|_| crate::NamedTimeZoneDataError::UnknownIdentifier(name.clone()))?;
        if &actual != identity {
            return Err(
                InvalidTimeZoneData("resolved named identity disagrees with catalogue").into(),
            );
        }
        if !self
            .zones
            .contains_key(&identity.identifier().to_ascii_lowercase())
        {
            return Err(crate::NamedTimeZoneDataError::UnavailableIdentifier(name));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod transition_tests;
