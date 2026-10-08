//! Complete native Duration rows derived from the pinned source and selected owners.

use super::{DurationDataImage, PINNED_PAYLOAD};
use crate::number_format::options::LocaleMatcher;
use crate::number_format::NumberLocaleRequest;
use crate::provider::LocaleCanonicalizationData;
use crate::{
    CanonicalLocaleId, CustomProfileId, DurationProfiles, IntlDataImageError, IntlDataProfile,
    ListDataImage, LocaleDataImage, LocaleId, NumberProfilesDataImage,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

const MAGIC: &[u8; 8] = b"LILADUP1";
const HEADER_BYTES: usize = 20;
const MAX_DESCRIPTOR_BYTES: usize = 16 * 1024;

/// Whole-source validation precedes selection, using its actual pinned Number
/// source and the supplied List foundation. No dependent image digest is needed.
pub(crate) fn number_dependency_locales(
    requested: Option<&[LocaleId]>,
    locale: &LocaleDataImage,
    lists: &ListDataImage,
    source_numbers: &crate::number_image::PinnedNumberSource,
) -> Result<Box<[Box<str>]>, IntlDataImageError> {
    if lists.profile() != locale.profile() || lists.locale_digest() != locale.digest() {
        return Err(IntlDataImageError::consumer(
            "Number dependency List and Locale foundations differ",
        ));
    }
    let full = DurationProfiles::from_image_data(
        PINNED_PAYLOAD,
        source_numbers.profiles(),
        &lists.profiles(),
    )
    .map_err(IntlDataImageError::consumer)?;
    let selected = match requested {
        Some(requested) => SelectedLocales::new(&full, requested, locale)?.0,
        None => full
            .available_locales()
            .cloned()
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    };
    Ok(selected
        .iter()
        .map(|name| name.as_str().into())
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

pub(crate) fn source_profile_sha256() -> [u8; 32] {
    Sha256::digest(PINNED_PAYLOAD).into()
}

struct SelectedLocales(Box<[CanonicalLocaleId]>);
impl SelectedLocales {
    fn new(
        full: &DurationProfiles,
        requested: &[LocaleId],
        locale: &LocaleDataImage,
    ) -> Result<Self, IntlDataImageError> {
        let available = full.available_locales().collect::<Vec<_>>();
        if requested.is_empty() || requested.len() > available.len() {
            return Err(IntlDataImageError::consumer(
                "Duration projection locale extent",
            ));
        }
        let authority =
            LocaleCanonicalizationData::from_image(locale).map_err(IntlDataImageError::consumer)?;
        let mut selected = BTreeSet::new();
        for requested in requested {
            let canonical = authority
                .canonicalize(requested)
                .map_err(IntlDataImageError::consumer)?;
            let index = available
                .binary_search_by(|name| name.as_str().cmp(canonical.as_str()))
                .map_err(|_| {
                    IntlDataImageError::consumer(format!(
                        "Duration projection locale {} is absent from the pinned catalogue",
                        canonical.as_str(),
                    ))
                })?;
            if !selected.insert((*available[index]).clone()) {
                return Err(IntlDataImageError::consumer(
                    "Duration projection contains duplicate canonical locales",
                ));
            }
        }
        let default = available
            .binary_search_by(|name| name.as_str().cmp("en-US"))
            .map_err(IntlDataImageError::consumer)?;
        selected.insert((*available[default]).clone());
        Ok(Self(
            selected.into_iter().collect::<Vec<_>>().into_boxed_slice(),
        ))
    }
}

/// The decoder receives selected names and their exact admitted bytes together.
/// No caller can mint this proof from an arbitrary row list or unrelated payload.
pub(crate) struct DurationCatalogue<'a> {
    bytes: &'a [u8],
    locales: Box<[CanonicalLocaleId]>,
}
impl DurationCatalogue<'_> {
    pub(crate) fn bytes(&self) -> &[u8] {
        self.bytes
    }
    pub(crate) fn locales(&self) -> &[CanonicalLocaleId] {
        &self.locales
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Association {
    duration_locale: String,
    number_locale: String,
    default_numbering_system: String,
    list_locale: String,
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
    list_image_sha256: [u8; 32],
    public_locales: Vec<String>,
    associations: Vec<Association>,
}

fn project(
    id: &CustomProfileId,
    requested: &[LocaleId],
    locale: &LocaleDataImage,
    numbers: &NumberProfilesDataImage,
    lists: &ListDataImage,
) -> Result<(Vec<u8>, SelectedLocales), IntlDataImageError> {
    DurationDataImage::check_foundations(
        &IntlDataProfile::Custom(id.clone()),
        locale,
        numbers,
        lists,
    )?;
    if <[u8; 32]>::from(Sha256::digest(PINNED_PAYLOAD))
        != crate::duration_format::DURATION_PROFILE_SHA256
    {
        return Err(IntlDataImageError::consumer(
            "Duration pinned source checksum mismatch",
        ));
    }
    let source_numbers = crate::number_image::pinned_number_source()?;
    let number_profiles = source_numbers.profiles();
    let list_profiles = lists.profiles();
    // Validate every original locale, unit, List witness and digital field before
    // deriving any subset. Retain the actual selected immutable foundations.
    let full = DurationProfiles::from_image_data(PINNED_PAYLOAD, number_profiles, &list_profiles)
        .map_err(IntlDataImageError::consumer)?;
    let selected = SelectedLocales::new(&full, requested, locale)?;
    let associations = selected
        .0
        .iter()
        .map(|name| {
            let resolved = full
                .resolve_locale(
                    &NumberLocaleRequest {
                        requested: vec![name.clone()].into_boxed_slice(),
                        matcher: LocaleMatcher::Lookup,
                        numbering_system: None,
                    },
                    number_profiles,
                )
                .map_err(IntlDataImageError::consumer)?;
            let list = list_profiles
                .resolve_duration_locale(name)
                .map_err(IntlDataImageError::consumer)?;
            Ok(Association {
                duration_locale: name.as_str().to_owned(),
                number_locale: resolved.resolved().as_str().to_owned(),
                default_numbering_system: resolved.numbering_system().to_owned(),
                list_locale: list.resolved().as_str().to_owned(),
            })
        })
        .collect::<Result<Vec<_>, IntlDataImageError>>()?;
    let descriptor = Descriptor {
        schema: 1,
        custom_id: id.as_str().to_owned(),
        default_locale: "en-US".to_owned(),
        full_profile_sha256: Sha256::digest(PINNED_PAYLOAD).into(),
        locale_image_sha256: *locale.digest().as_bytes(),
        number_image_sha256: *numbers.digest().as_bytes(),
        list_image_sha256: *lists.digest().as_bytes(),
        public_locales: selected
            .0
            .iter()
            .map(|name| name.as_str().to_owned())
            .collect(),
        associations,
    };
    let mut raw: serde_json::Value =
        serde_json::from_slice(PINNED_PAYLOAD).map_err(IntlDataImageError::consumer)?;
    let rows = raw["locales"]
        .as_array()
        .ok_or_else(|| IntlDataImageError::consumer("pinned Duration locale rows are absent"))?;
    let selected_rows = selected
        .0
        .iter()
        .map(|name| {
            rows.iter()
                .find(|row| row["locale"].as_str() == Some(name.as_str()))
                .cloned()
                .ok_or_else(|| IntlDataImageError::consumer("pinned Duration row is absent"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    raw["locales"] = serde_json::Value::Array(selected_rows);
    let bytes =
        serde_json::to_vec(&canonical_objects(raw)).map_err(IntlDataImageError::consumer)?;
    Ok((frame_payload(&descriptor, &bytes)?, selected))
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
    if descriptor.len() > MAX_DESCRIPTOR_BYTES || blob.len() > PINNED_PAYLOAD.len() {
        return Err(IntlDataImageError::consumer("Duration projection extent"));
    }
    let extent = HEADER_BYTES
        .checked_add(descriptor.len())
        .and_then(|n| n.checked_add(blob.len()))
        .filter(|&n| n <= crate::MAX_INTL_COMPONENT_IMAGE_BYTES)
        .ok_or_else(|| IntlDataImageError::consumer("Duration projection extent"))?;
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
    lists: &ListDataImage,
) -> Result<Vec<u8>, IntlDataImageError> {
    project(id, requested, locale, numbers, lists).map(|(bytes, _)| bytes)
}
pub(super) fn admit<'a>(
    payload: &'a [u8],
    profile: &IntlDataProfile,
    locale: &LocaleDataImage,
    numbers: &NumberProfilesDataImage,
    lists: &ListDataImage,
) -> Result<DurationCatalogue<'a>, IntlDataImageError> {
    let IntlDataProfile::Custom(id) = profile else {
        return Err(IntlDataImageError::consumer(
            "a projected Duration payload requires Custom",
        ));
    };
    let (descriptor, bytes) = split_payload(payload)?;
    let requested = descriptor
        .public_locales
        .iter()
        .map(|name| LocaleId::parse(name.as_str()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(IntlDataImageError::consumer)?;
    let (expected, selected) = project(id, &requested, locale, numbers, lists)?;
    // Canonical equality admits the entire native row closure and all dependency
    // associations, not self-described names, hashes or replacement templates.
    if payload != expected.as_slice() {
        return Err(IntlDataImageError::consumer(
            "Duration projection differs from its pinned derivation",
        ));
    }
    Ok(DurationCatalogue {
        bytes,
        locales: selected.0,
    })
}
fn split_payload(payload: &[u8]) -> Result<(Descriptor, &[u8]), IntlDataImageError> {
    if payload.len() < HEADER_BYTES || &payload[..8] != MAGIC {
        return Err(IntlDataImageError::consumer(
            "invalid Duration projection framing",
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
            descriptor_length <= MAX_DESCRIPTOR_BYTES && blob_length <= PINNED_PAYLOAD.len()
        })
        .ok_or_else(|| IntlDataImageError::consumer("invalid Duration projection extent"))?;
    if blob_start.checked_add(blob_length) != Some(payload.len()) {
        return Err(IntlDataImageError::consumer(
            "invalid Duration projection extent",
        ));
    }
    let descriptor = serde_json::from_slice(&payload[HEADER_BYTES..blob_start])
        .map_err(IntlDataImageError::consumer)?;
    Ok((descriptor, &payload[blob_start..]))
}

#[cfg(test)]
mod tests;
