//! Exact selected transition closure over the original full IANA identity owner.
use super::*;
use crate::{CustomProfileId, TimeZoneId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

const MAGIC: &[u8; 8] = b"LILATZP1";
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Descriptor {
    schema: u16,
    custom_id: String,
    full_payload_sha256: [u8; 32],
    named_time_zones: Vec<String>,
}

pub(crate) struct NamedZoneCatalogue<'a> {
    native: NativePayload<'a>,
    bytes: &'a [u8],
    selection: Box<[TimeZoneId]>,
}
impl<'a> NamedZoneCatalogue<'a> {
    pub(crate) fn catalogue(&self) -> &'a str {
        self.native.catalogue
    }
    pub(crate) fn records(&self) -> &[(&'a str, &'a [u8])] {
        &self.native.records
    }
    pub(crate) fn selection(&self) -> &[TimeZoneId] {
        &self.selection
    }
    pub(super) fn native(&self) -> &'a [u8] {
        self.bytes
    }
}

fn put(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), IntlDataImageError> {
    output.extend_from_slice(
        &u32::try_from(bytes.len())
            .map_err(IntlDataImageError::consumer)?
            .to_le_bytes(),
    );
    output.extend_from_slice(bytes);
    Ok(())
}
fn derive(
    id: &CustomProfileId,
    requested: &[TimeZoneId],
) -> Result<(Vec<u8>, Box<[TimeZoneId]>), IntlDataImageError> {
    let full = NamedTimeZoneDataImage::for_profile(IntlDataProfile::Custom(id.clone()))?;
    if requested.is_empty() || requested.len() > 598 {
        return Err(IntlDataImageError::consumer("named-zone selection extent"));
    }
    let mut selected = BTreeSet::new();
    for name in requested {
        let identity = full
            .zones_ref()
            .lookup(name)
            .map_err(IntlDataImageError::consumer)?;
        if !selected.insert(identity.primary_identifier().to_owned()) {
            return Err(IntlDataImageError::consumer(
                "duplicate canonical named-zone selection",
            ));
        }
    }
    selected.insert("UTC".into());
    let source = read_native(PINNED_PAYLOAD)?;
    let records = source
        .records
        .iter()
        .copied()
        .filter(|(name, _)| {
            let name = TimeZoneId::parse(*name).expect("admitted IANA identifier");
            selected.contains(
                full.zones_ref()
                    .lookup(&name)
                    .expect("admitted IANA identity")
                    .primary_identifier(),
            )
        })
        .collect::<Vec<_>>();
    let mut native = Vec::new();
    native.extend_from_slice(super::MAGIC);
    for field in [
        source.catalogue.as_bytes(),
        source.zone_tab.as_bytes(),
        source.regions.as_bytes(),
    ] {
        put(&mut native, field)?;
    }
    native.extend_from_slice(
        &u32::try_from(records.len())
            .map_err(IntlDataImageError::consumer)?
            .to_le_bytes(),
    );
    for (name, bytes) in records {
        put(&mut native, name.as_bytes())?;
        put(&mut native, bytes)?;
    }
    let descriptor = Descriptor {
        schema: 1,
        custom_id: id.as_str().to_owned(),
        full_payload_sha256: Sha256::digest(PINNED_PAYLOAD).into(),
        named_time_zones: selected.iter().cloned().collect(),
    };
    let mut payload = Vec::new();
    payload.extend_from_slice(MAGIC);
    put(
        &mut payload,
        &serde_json::to_vec(&descriptor).map_err(IntlDataImageError::consumer)?,
    )?;
    put(&mut payload, &native)?;
    if payload.len() > crate::MAX_INTL_COMPONENT_IMAGE_BYTES {
        return Err(IntlDataImageError::consumer("named-zone projection extent"));
    }
    let selection = selected
        .into_iter()
        .map(|name| TimeZoneId::parse(name).map_err(IntlDataImageError::consumer))
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();
    Ok((payload, selection))
}
pub(super) fn produce(
    id: &CustomProfileId,
    zones: &[TimeZoneId],
) -> Result<Vec<u8>, IntlDataImageError> {
    derive(id, zones).map(|(payload, _)| payload)
}
pub(super) fn admit<'a>(
    payload: &'a [u8],
    profile: &IntlDataProfile,
) -> Result<NamedZoneCatalogue<'a>, IntlDataImageError> {
    let IntlDataProfile::Custom(id) = profile else {
        return Err(IntlDataImageError::consumer(
            "projected named zones require Custom",
        ));
    };
    let mut reader = Reader {
        bytes: payload,
        position: 0,
    };
    if reader.take(8)? != MAGIC {
        return Err(IntlDataImageError::consumer("named-zone projection schema"));
    }
    let descriptor_bytes = reader.bytes()?;
    if descriptor_bytes.len() > 64 * 1024 {
        return Err(IntlDataImageError::consumer("named-zone descriptor extent"));
    }
    let descriptor: Descriptor =
        serde_json::from_slice(descriptor_bytes).map_err(IntlDataImageError::consumer)?;
    let native = reader.bytes()?;
    if reader.position != payload.len() {
        return Err(IntlDataImageError::consumer(
            "trailing named-zone projection",
        ));
    }
    let requested = descriptor
        .named_time_zones
        .iter()
        .map(|name| TimeZoneId::parse(name.as_str()).map_err(IntlDataImageError::consumer))
        .collect::<Result<Vec<_>, _>>()?;
    let (expected, selection) = derive(id, &requested)?;
    if expected != payload {
        return Err(IntlDataImageError::consumer(
            "named-zone projection differs from exact pinned closure",
        ));
    }
    Ok(NamedZoneCatalogue {
        native: read_native(native)?,
        bytes: native,
        selection,
    })
}

#[cfg(test)]
mod tests;
