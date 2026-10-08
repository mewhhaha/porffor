//! Selected real CLDR name records over one admitted IANA service owner.
use super::*;
use crate::{CustomProfileId, TimeZoneId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Descriptor {
    schema: u16,
    custom_id: String,
    source_sha256: [u8; 32],
    named_sha256: [u8; 32],
    named_time_zones: Vec<String>,
}
pub(crate) struct TimeZoneNameCatalogue<'a>(&'a [u8]);
impl TimeZoneNameCatalogue<'_> {
    pub(crate) fn bytes(&self) -> &[u8] {
        self.0
    }
}

fn derive(
    id: &CustomProfileId,
    named: &NamedTimeZoneDataImage,
) -> Result<Vec<u8>, IntlDataImageError> {
    if named.profile() != &IntlDataProfile::Custom(id.clone()) {
        return Err(IntlDataImageError::consumer(
            "time-zone names Custom foundation",
        ));
    }
    let selection = named.named_time_zone_selection().ok_or_else(|| {
        IntlDataImageError::consumer("time-zone name projection requires selected IANA data")
    })?;
    let _source = TimeZoneNames::from_json(PINNED_PAYLOAD, named.zones())
        .map_err(IntlDataImageError::consumer)?;
    let mut raw: serde_json::Value =
        serde_json::from_slice(PINNED_PAYLOAD).map_err(IntlDataImageError::consumer)?;
    let aliases = raw["rows"]["aliases"]
        .as_array()
        .ok_or_else(|| IntlDataImageError::consumer("time-zone alias rows"))?;
    let mut selected = BTreeSet::new();
    for pair in aliases {
        let name = pair[0]
            .as_str()
            .and_then(|name| TimeZoneId::parse(name).ok());
        if name
            .as_ref()
            .and_then(|name| named.zones_ref().lookup(name).ok())
            .is_some_and(|identity| {
                selection
                    .binary_search_by(|name| name.as_str().cmp(identity.primary_identifier()))
                    .is_ok()
            })
        {
            selected.insert(
                pair[1]
                    .as_str()
                    .ok_or_else(|| IntlDataImageError::consumer("time-zone alias target"))?
                    .to_owned(),
            );
        }
    }
    raw["rows"]["zones"].as_array_mut().unwrap().retain(|row| {
        row["identifier"]
            .as_str()
            .is_some_and(|name| selected.contains(name))
    });
    raw["rows"]["aliases"]
        .as_array_mut()
        .unwrap()
        .retain(|pair| pair[1].as_str().is_some_and(|name| selected.contains(name)));
    let mut used_meta = BTreeSet::new();
    for row in raw["rows"]["zones"].as_array().unwrap() {
        for period in row["periods"].as_array().unwrap() {
            used_meta.insert(period[2].as_str().unwrap().to_owned());
        }
    }
    raw["rows"]["metazones"]
        .as_array_mut()
        .unwrap()
        .retain(|row| {
            row["identifier"]
                .as_str()
                .is_some_and(|name| used_meta.contains(name))
        });
    let descriptor = Descriptor {
        schema: 1,
        custom_id: id.as_str().into(),
        source_sha256: Sha256::digest(PINNED_PAYLOAD).into(),
        named_sha256: *named.digest().as_bytes(),
        named_time_zones: selection
            .iter()
            .map(|name| name.as_str().to_owned())
            .collect(),
    };
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"LILATNP1");
    let descriptor = serde_json::to_vec(&descriptor).map_err(IntlDataImageError::consumer)?;
    bytes.extend_from_slice(
        &u32::try_from(descriptor.len())
            .map_err(IntlDataImageError::consumer)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&descriptor);
    bytes.extend_from_slice(&serde_json::to_vec(&raw).map_err(IntlDataImageError::consumer)?);
    Ok(bytes)
}
pub(super) fn produce(
    id: &CustomProfileId,
    named: &NamedTimeZoneDataImage,
) -> Result<Vec<u8>, IntlDataImageError> {
    derive(id, named)
}
pub(super) fn admit<'a>(
    bytes: &'a [u8],
    profile: &IntlDataProfile,
    named: &NamedTimeZoneDataImage,
) -> Result<TimeZoneNameCatalogue<'a>, IntlDataImageError> {
    let IntlDataProfile::Custom(id) = profile else {
        return Err(IntlDataImageError::consumer(
            "projected names require Custom",
        ));
    };
    if bytes.get(..8) != Some(b"LILATNP1".as_slice()) {
        return Err(IntlDataImageError::consumer(
            "time-zone name projection schema",
        ));
    }
    let end = bytes
        .get(8..12)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()) as usize)
        .filter(|length| *length <= 64 * 1024)
        .and_then(|length| length.checked_add(12))
        .filter(|end| *end < bytes.len())
        .ok_or_else(|| IntlDataImageError::consumer("time-zone name projection extent"))?;
    let _: Descriptor =
        serde_json::from_slice(&bytes[12..end]).map_err(IntlDataImageError::consumer)?;
    if bytes != derive(id, named)? {
        return Err(IntlDataImageError::consumer(
            "time-zone names differ from exact selected closure",
        ));
    }
    Ok(TimeZoneNameCatalogue(&bytes[end..]))
}
