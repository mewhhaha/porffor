use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use timezone_provider::tzif::Tzif;
use tzif::data::time::Seconds;

use crate::{
    InvalidTimeZoneData, NamedTimeZoneIdentity, TimeZoneEpochSeconds, TimeZoneId, UnknownTimeZone,
};

use super::time_zone_snapshot::{
    NamedTimeZoneTransition, TimeZoneVariant, STANDARD_TIME_STABILITY_WINDOW_SECONDS,
};

mod catalogue;
mod identity;
mod validation;

pub(super) const PROVIDER_DATA_SHA256: [u8; 32] = identity::PROVIDER_DATA_SHA256;

struct NamedTimeZone {
    identity: NamedTimeZoneIdentity,
    transitions: Tzif,
}

/// Immutable identifiers and transition records from the same IANA release.
pub(super) struct NamedTimeZones {
    zones: BTreeMap<String, NamedTimeZone>,
}

impl NamedTimeZones {
    pub(super) fn from_pinned_data() -> Result<Self, InvalidTimeZoneData> {
        if jiff_tzdb::VERSION != Some("2026a") {
            return Err(InvalidTimeZoneData(
                "named transition archive is not IANA2026a",
            ));
        }
        let rows = catalogue::read()?;
        let mut zones = BTreeMap::new();
        for row in rows {
            let (normalized, bytes) = jiff_tzdb::get(row.identity.identifier()).ok_or(
                InvalidTimeZoneData("catalogue identifier has no transition record"),
            )?;
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
            let transitions = Tzif::from_bytes(bytes)
                .map_err(|_| InvalidTimeZoneData("invalid pinned TZif record"))?;
            validation::validate(&transitions)?;
            let key = row.identity.identifier().to_ascii_lowercase();
            if zones
                .insert(
                    key,
                    NamedTimeZone {
                        identity: row.identity,
                        transitions,
                    },
                )
                .is_some()
            {
                return Err(InvalidTimeZoneData("case-insensitive catalogue collision"));
            }
        }
        if zones.len() != jiff_tzdb::available().count() {
            return Err(InvalidTimeZoneData(
                "catalogue and transition archive extent differ",
            ));
        }
        for zone in zones.values() {
            let primary = zones
                .get(&zone.identity.primary_identifier().to_ascii_lowercase())
                .ok_or(InvalidTimeZoneData("catalogue primary is missing"))?;
            if primary.identity.identifier() != zone.identity.primary_identifier()
                || primary.identity.identifier() != primary.identity.primary_identifier()
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
        Ok(Self { zones })
    }

    pub(super) fn lookup(
        &self,
        identifier: &TimeZoneId,
    ) -> Result<NamedTimeZoneIdentity, UnknownTimeZone> {
        self.zones
            .get(&identifier.as_str().to_ascii_lowercase())
            .map(|zone| zone.identity.clone())
            .ok_or_else(|| UnknownTimeZone::new(identifier.clone()))
    }

    pub(super) fn transition(
        &self,
        identity: &NamedTimeZoneIdentity,
        epoch: TimeZoneEpochSeconds,
    ) -> Result<NamedTimeZoneTransition, InvalidTimeZoneData> {
        let zone = self
            .zones
            .get(&identity.identifier().to_ascii_lowercase())
            .ok_or(InvalidTimeZoneData("resolved named identifier is absent"))?;
        if &zone.identity != identity {
            return Err(InvalidTimeZoneData(
                "resolved named identity disagrees with catalogue",
            ));
        }
        let selected = zone
            .transitions
            .get(&Seconds(epoch.get()))
            .map_err(|_| InvalidTimeZoneData("pinned transition selection failed"))?;
        let offset_seconds = i32::try_from(selected.offset.0)
            .map_err(|_| InvalidTimeZoneData("selected offset is outside the TZif domain"))?;
        let variant = if selected.is_dst {
            TimeZoneVariant::Daylight
        } else {
            TimeZoneVariant::Standard
        };
        if variant == TimeZoneVariant::Standard
            && zone
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
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod transition_tests;
