//! Exact native row projection from the admitted pinned RelativeTime catalogue.

use super::PINNED_PROFILE;
use crate::number_format::options::LocaleMatcher;
use crate::number_format::{NumberLocaleRequest, PartitionLimits};
use crate::provider::LocaleCanonicalizationData;
use crate::{
    CanonicalLocaleId, CustomProfileId, IntlDataImageError, IntlDataProfile, LocaleDataImage,
    LocaleId, NumberProfilesDataImage, RelativeProfiles,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

const MAGIC: &[u8; 8] = b"LILARTP1";
const HEADER_BYTES: usize = 20;
const MAX_DESCRIPTOR_BYTES: usize = 16 * 1024;

/// The same complete source decoder and canonical selection used by the real
/// image producer determine its Number rows before that image exists.
pub(crate) fn number_dependency_locales(
    requested: Option<&[LocaleId]>,
    locale: &LocaleDataImage,
    source_numbers: &crate::number_image::PinnedNumberSource,
) -> Result<Box<[Box<str>]>, IntlDataImageError> {
    let source = core::str::from_utf8(PINNED_PROFILE).map_err(IntlDataImageError::consumer)?;
    let full = RelativeProfiles::from_json(source, source_numbers.profiles())
        .map_err(IntlDataImageError::consumer)?;
    let selected = match requested {
        Some(requested) => RelativeCatalogue::projected(&full, requested, locale)?.locales,
        None => full.available_locales().to_vec().into_boxed_slice(),
    };
    Ok(selected
        .iter()
        .map(|name| (*name).into())
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

pub(crate) fn source_profile_sha256() -> [u8; 32] {
    Sha256::digest(PINNED_PROFILE).into()
}

/// Only this producer can derive a selected domain from the complete admitted
/// pinned catalogue. The decoder cannot accept a caller's unchecked row list.
pub(crate) struct RelativeCatalogue {
    locales: Box<[&'static str]>,
}
impl RelativeCatalogue {
    fn projected(
        full: &RelativeProfiles,
        requested: &[LocaleId],
        locale: &LocaleDataImage,
    ) -> Result<Self, IntlDataImageError> {
        if requested.is_empty() || requested.len() > full.available_locales().len() {
            return Err(IntlDataImageError::consumer(
                "RelativeTime projection locale extent",
            ));
        }
        let canonicalizer =
            LocaleCanonicalizationData::from_image(locale).map_err(IntlDataImageError::consumer)?;
        let mut selected = BTreeSet::new();
        for requested in requested {
            let canonical = canonicalizer
                .canonicalize(requested)
                .map_err(IntlDataImageError::consumer)?;
            let slot = full
                .available_locales()
                .binary_search(&canonical.as_str())
                .map_err(|_| {
                    IntlDataImageError::consumer(format!(
                        "RelativeTime projection locale {} is absent from the pinned catalogue",
                        canonical.as_str(),
                    ))
                })?;
            if !selected.insert(full.available_locales()[slot]) {
                return Err(IntlDataImageError::consumer(
                    "RelativeTime projection contains duplicate canonical locales",
                ));
            }
        }
        let default = full
            .available_locales()
            .binary_search(&"en-US")
            .map_err(IntlDataImageError::consumer)?;
        selected.insert(full.available_locales()[default]);
        Ok(Self {
            locales: selected.into_iter().collect::<Vec<_>>().into_boxed_slice(),
        })
    }
    pub(crate) fn locales(&self) -> &[&'static str] {
        &self.locales
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NumberAssociation {
    relative_locale: String,
    number_locale: String,
    default_numbering_system: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Descriptor {
    schema: u16,
    custom_id: String,
    default_locale: String,
    full_profile_sha256: [u8; 32],
    locale_image_sha256: [u8; 32],
    number_image_sha256: [u8; 32],
    public_locales: Vec<String>,
    number_associations: Vec<NumberAssociation>,
}

fn project(
    id: &CustomProfileId,
    requested: &[LocaleId],
    locale: &LocaleDataImage,
    numbers: &NumberProfilesDataImage,
) -> Result<(Vec<u8>, RelativeCatalogue), IntlDataImageError> {
    let profile = IntlDataProfile::Custom(id.clone());
    if locale.profile() != &profile
        || numbers.profile() != &profile
        || numbers.locale_digest() != locale.digest()
    {
        return Err(IntlDataImageError::consumer(
            "RelativeTime projection, Locale and Number foundations differ",
        ));
    }
    let source_numbers = crate::number_image::pinned_number_source()?;
    let number_profiles = source_numbers.profiles();
    let source = core::str::from_utf8(PINNED_PROFILE).map_err(IntlDataImageError::consumer)?;
    // The same complete field/category/plural decoder admits the source before
    // any row is selected. Public support never comes from Number's larger set.
    let full = RelativeProfiles::from_json(source, number_profiles)
        .map_err(IntlDataImageError::consumer)?;
    let catalogue = RelativeCatalogue::projected(&full, requested, locale)?;
    let number_associations = catalogue
        .locales()
        .iter()
        .map(|name| {
            let request = NumberLocaleRequest {
                requested: vec![
                    CanonicalLocaleId::from_data(*name).map_err(IntlDataImageError::consumer)?
                ]
                .into_boxed_slice(),
                matcher: LocaleMatcher::Lookup,
                numbering_system: None,
            };
            let resolved = full
                .resolve_locale(&request, number_profiles, &PartitionLimits::HOST_ABI)
                .map_err(IntlDataImageError::consumer)?;
            Ok(NumberAssociation {
                relative_locale: (*name).to_owned(),
                number_locale: resolved.formatting().as_str().to_owned(),
                default_numbering_system: resolved.numbering_system().to_owned(),
            })
        })
        .collect::<Result<Vec<_>, IntlDataImageError>>()?;
    let descriptor = Descriptor {
        schema: 1,
        custom_id: id.as_str().to_owned(),
        default_locale: "en-US".to_owned(),
        full_profile_sha256: Sha256::digest(PINNED_PROFILE).into(),
        locale_image_sha256: *locale.digest().as_bytes(),
        number_image_sha256: *numbers.digest().as_bytes(),
        public_locales: catalogue
            .locales()
            .iter()
            .map(|name| (*name).to_owned())
            .collect(),
        number_associations,
    };
    let mut raw: serde_json::Value =
        serde_json::from_slice(PINNED_PROFILE).map_err(IntlDataImageError::consumer)?;
    let rows = raw["locales"].as_array().ok_or_else(|| {
        IntlDataImageError::consumer("pinned RelativeTime source locale rows are absent")
    })?;
    let selected_rows = catalogue
        .locales()
        .iter()
        .map(|name| {
            rows.iter()
                .find(|row| row["locale"].as_str() == Some(*name))
                .cloned()
                .ok_or_else(|| IntlDataImageError::consumer("pinned RelativeTime row is absent"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    raw["locales"] = serde_json::Value::Array(selected_rows);
    // Make object key order independent of serde_json's preserve_order feature.
    let blob = serde_json::to_vec(&canonical_objects(raw)).map_err(IntlDataImageError::consumer)?;
    Ok((frame_payload(&descriptor, &blob)?, catalogue))
}

fn canonical_objects(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(object) => {
            let mut entries = object.into_iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|a, b| a.0.cmp(&b.0));
            serde_json::Value::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, canonical_objects(value)))
                    .collect(),
            )
        }
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(canonical_objects).collect())
        }
        value => value,
    }
}

fn frame_payload(descriptor: &Descriptor, blob: &[u8]) -> Result<Vec<u8>, IntlDataImageError> {
    let descriptor = serde_json::to_vec(descriptor).map_err(IntlDataImageError::consumer)?;
    if descriptor.len() > MAX_DESCRIPTOR_BYTES || blob.len() > PINNED_PROFILE.len() {
        return Err(IntlDataImageError::consumer(
            "RelativeTime projection extent",
        ));
    }
    let extent = HEADER_BYTES
        .checked_add(descriptor.len())
        .and_then(|n| n.checked_add(blob.len()))
        .filter(|&n| n <= crate::MAX_INTL_COMPONENT_IMAGE_BYTES)
        .ok_or_else(|| IntlDataImageError::consumer("RelativeTime projection extent"))?;
    let mut payload = Vec::new();
    payload
        .try_reserve_exact(extent)
        .map_err(IntlDataImageError::consumer)?;
    payload.extend_from_slice(MAGIC);
    payload.extend_from_slice(
        &u32::try_from(descriptor.len())
            .map_err(IntlDataImageError::consumer)?
            .to_le_bytes(),
    );
    payload.extend_from_slice(
        &u64::try_from(blob.len())
            .map_err(IntlDataImageError::consumer)?
            .to_le_bytes(),
    );
    payload.extend_from_slice(&descriptor);
    payload.extend_from_slice(blob);
    Ok(payload)
}

pub(super) fn produce(
    id: &CustomProfileId,
    requested: &[LocaleId],
    locale: &LocaleDataImage,
    numbers: &NumberProfilesDataImage,
) -> Result<Vec<u8>, IntlDataImageError> {
    project(id, requested, locale, numbers).map(|(payload, _)| payload)
}

pub(super) fn admit<'a>(
    payload: &'a [u8],
    profile: &IntlDataProfile,
    locale: &LocaleDataImage,
    numbers: &NumberProfilesDataImage,
) -> Result<(&'a [u8], RelativeCatalogue), IntlDataImageError> {
    let IntlDataProfile::Custom(id) = profile else {
        return Err(IntlDataImageError::consumer(
            "a projected RelativeTime payload requires Custom",
        ));
    };
    let (descriptor, blob) = split_payload(payload)?;
    let requested = descriptor
        .public_locales
        .iter()
        .map(|name| LocaleId::parse(name.as_str()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(IntlDataImageError::consumer)?;
    let (expected, catalogue) = project(id, &requested, locale, numbers)?;
    // Exact equality proves canonical metadata, selected complete rows, original
    // source pins, actual Number associations and absence of replacement data.
    if payload != expected.as_slice() {
        return Err(IntlDataImageError::consumer(
            "RelativeTime projection differs from its pinned derivation",
        ));
    }
    Ok((blob, catalogue))
}

fn split_payload(payload: &[u8]) -> Result<(Descriptor, &[u8]), IntlDataImageError> {
    if payload.len() < HEADER_BYTES || &payload[..8] != MAGIC {
        return Err(IntlDataImageError::consumer(
            "invalid RelativeTime projection framing",
        ));
    }
    let descriptor_length = u32::from_le_bytes(
        payload[8..12]
            .try_into()
            .map_err(IntlDataImageError::consumer)?,
    ) as usize;
    let blob_length = usize::try_from(u64::from_le_bytes(
        payload[12..20]
            .try_into()
            .map_err(IntlDataImageError::consumer)?,
    ))
    .map_err(IntlDataImageError::consumer)?;
    let blob_start = HEADER_BYTES
        .checked_add(descriptor_length)
        .filter(|_| {
            descriptor_length <= MAX_DESCRIPTOR_BYTES && blob_length <= PINNED_PROFILE.len()
        })
        .ok_or_else(|| IntlDataImageError::consumer("invalid RelativeTime projection extent"))?;
    if blob_start.checked_add(blob_length) != Some(payload.len()) {
        return Err(IntlDataImageError::consumer(
            "invalid RelativeTime projection extent",
        ));
    }
    let descriptor = serde_json::from_slice(&payload[HEADER_BYTES..blob_start])
        .map_err(IntlDataImageError::consumer)?;
    Ok((descriptor, &payload[blob_start..]))
}

#[cfg(test)]
mod tests;
