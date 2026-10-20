//! A projected frame contains only actual consumed pinned raw ICU rows.
//! Admission reconstructs the complete payload; source bytes are never replaced
//! by typed reserialization, and the provider retains no full-blob fallback.
use super::{CollatorImageProvider, CollatorLocaleInventory, PINNED_BLOB, PINNED_LOCALES};
use crate::provider::LocaleCanonicalizationData;
use crate::{
    CanonicalLocaleId, CollatorProfiles, CustomProfileId, IntlDataImageError, IntlDataProfile,
    LocaleDataImage, LocaleId,
};
use icu_provider::prelude::*;
use icu_provider_blob::BlobDataProvider;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

mod rows;
pub(super) use rows::RawRows;
use rows::{Marker, RowKey};
const MAGIC: &[u8; 8] = b"LCLP0001";
const MAX_DESCRIPTOR_BYTES: usize = 512 * 1024;

/// Both public and private domains originate in complete pinned constructors.
/// Fields cannot be supplied by callers or minted from arbitrary descriptor rows.
#[derive(Debug)]
pub(crate) struct CollatorCatalogue {
    public: Box<[CanonicalLocaleId]>,
    preferences: Box<[(CanonicalLocaleId, Box<[Box<str>]>)]>,
    roots: Box<[Box<str>]>,
}
impl CollatorCatalogue {
    pub(crate) fn public_locales(&self) -> &[CanonicalLocaleId] {
        &self.public
    }
    pub(crate) fn preference_rows(&self) -> &[(CanonicalLocaleId, Box<[Box<str>]>)] {
        &self.preferences
    }
}

#[derive(Debug, Default)]
pub(super) struct Recordings(Mutex<BTreeSet<RowKey>>);
impl Recordings {
    pub(super) fn record(
        &self,
        info: DataMarkerInfo,
        request: DataRequest,
        metadata: &DataResponseMetadata,
    ) -> Result<(), DataError> {
        let marker = Marker::from_info(info)
            .ok_or_else(|| DataError::custom("foreign Collator projection marker"))?;
        if request.metadata.attributes_prefix_match {
            return Err(DataError::custom(
                "Collator projection cannot record a prefix identifier",
            ));
        }
        let id = DataIdentifierCow::from_owned(
            request.id.marker_attributes.to_owned(),
            metadata
                .locale
                .clone()
                .unwrap_or_else(|| request.id.locale.clone()),
        );
        self.0
            .lock()
            .map_err(|_| DataError::custom("poisoned Collator projection recorder"))?
            .insert(RowKey { marker, id });
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RowDescriptor {
    marker: Marker,
    locale: String,
    attributes: String,
    bytes: usize,
    sha256: String,
    checksum: Option<u64>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Descriptor {
    schema: u8,
    custom_id: String,
    source_blob_sha256: String,
    source_inventory_sha256: String,
    locale_digest: String,
    default_locale: String,
    public_locales: Vec<String>,
    locale_preferences: Vec<(String, Vec<String>)>,
    root_preferences: Vec<String>,
    rows: Vec<RowDescriptor>,
}

pub(super) struct ProjectedData {
    pub(super) bytes: Vec<u8>,
    pub(super) rows: RawRows,
    pub(super) catalogue: CollatorCatalogue,
}
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(super) fn is_projection(bytes: &[u8]) -> bool {
    bytes.starts_with(MAGIC)
}

pub(super) fn project(
    id: &CustomProfileId,
    requested: &[LocaleId],
    locale: &LocaleDataImage,
) -> Result<ProjectedData, IntlDataImageError> {
    if requested.is_empty() || locale.profile() != &IntlDataProfile::Custom(id.clone()) {
        return Err(IntlDataImageError::consumer(
            "Collator projection needs nonempty locales and matching Custom Locale",
        ));
    }
    // The full catalogue validates the original 1,082 candidate associations.
    // Only its small preference authority survives in the projected image.
    let full_data = Arc::new(CollatorImageProvider::checked(
        PINNED_BLOB,
        locale,
        CollatorLocaleInventory::from_bytes(PINNED_LOCALES)?,
    )?);
    let full = CollatorProfiles::from_image(full_data).map_err(IntlDataImageError::consumer)?;
    let full_names = full.available_locales().cloned().collect::<BTreeSet<_>>();
    let canonicalizer =
        LocaleCanonicalizationData::from_image(locale).map_err(IntlDataImageError::consumer)?;
    let mut public = BTreeSet::new();
    for requested in requested {
        let canonical = canonicalizer
            .canonicalize(requested)
            .map_err(IntlDataImageError::consumer)?;
        if !full_names.contains(&canonical) || !public.insert(canonical) {
            return Err(IntlDataImageError::consumer(
                "Collator projection locale is absent or canonically duplicate",
            ));
        }
    }
    public.insert(CanonicalLocaleId::from_data("en-US").map_err(IntlDataImageError::consumer)?);
    let catalogue = CollatorCatalogue {
        public: public.into_iter().collect(),
        preferences: full
            .preference_rows()
            .map(|(name, values)| (name.clone(), values.to_vec().into_boxed_slice()))
            .collect(),
        roots: full
            .locale_sort_collations(
                &CanonicalLocaleId::from_data("qaa").map_err(IntlDataImageError::consumer)?,
            )
            .map_err(IntlDataImageError::consumer)?,
    };
    let recorder = Arc::new(Recordings::default());
    let mut recorded = CollatorImageProvider::checked(
        PINNED_BLOB,
        locale,
        CollatorLocaleInventory::from_bytes(PINNED_LOCALES)?,
    )?;
    recorded.recordings = Some(recorder.clone());
    // Existing complete profile admission consumes default/search, all actual
    // sort types, root emoji/eor and conditional numeric/shifted payloads.
    let selected = CollatorProfiles::from_projection(Arc::new(recorded), &catalogue)
        .map_err(IntlDataImageError::consumer)?;
    drop(selected);
    let keys = recorder
        .0
        .lock()
        .map_err(|_| IntlDataImageError::consumer("poisoned Collator projection recorder"))?;
    let source = BlobDataProvider::try_new_from_static_blob(PINNED_BLOB)
        .map_err(IntlDataImageError::consumer)?;
    let rows = RawRows::from_pinned(&source, &keys)?;
    let descriptor = Descriptor {
        schema: 1,
        custom_id: id.as_str().to_owned(),
        source_blob_sha256: sha(PINNED_BLOB),
        source_inventory_sha256: sha(PINNED_LOCALES),
        locale_digest: locale
            .digest()
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        default_locale: "en-US".into(),
        public_locales: catalogue
            .public
            .iter()
            .map(|name| name.as_str().to_owned())
            .collect(),
        locale_preferences: catalogue
            .preferences
            .iter()
            .map(|(name, values)| {
                (
                    name.as_str().to_owned(),
                    values.iter().map(ToString::to_string).collect(),
                )
            })
            .collect(),
        root_preferences: catalogue.roots.iter().map(ToString::to_string).collect(),
        rows: rows
            .iter()
            .map(|(key, response)| RowDescriptor {
                marker: key.marker,
                locale: key.id.locale.to_string(),
                attributes: key.id.marker_attributes.to_string(),
                bytes: response.payload.get().len(),
                sha256: sha(response.payload.get()),
                checksum: response.metadata.checksum,
            })
            .collect(),
    };
    let description = serde_json::to_vec(&descriptor).map_err(IntlDataImageError::consumer)?;
    if description.len() > MAX_DESCRIPTOR_BYTES {
        return Err(IntlDataImageError::consumer(
            "Collator projection descriptor extent",
        ));
    }
    let mut size = 12usize
        .checked_add(description.len())
        .ok_or_else(|| IntlDataImageError::consumer("Collator projection payload extent"))?;
    for (_, response) in rows.iter() {
        size = size
            .checked_add(response.payload.get().len())
            .ok_or_else(|| IntlDataImageError::consumer("Collator projection payload extent"))?;
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(IntlDataImageError::consumer)?;
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(
        &u32::try_from(description.len())
            .map_err(IntlDataImageError::consumer)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&description);
    for (_, response) in rows.iter() {
        bytes.extend_from_slice(response.payload.get());
    }
    Ok(ProjectedData {
        bytes,
        rows,
        catalogue,
    })
}

pub(super) fn admit(
    bytes: &[u8],
    profile: &IntlDataProfile,
    locale: &LocaleDataImage,
) -> Result<ProjectedData, IntlDataImageError> {
    let IntlDataProfile::Custom(id) = profile else {
        return Err(IntlDataImageError::consumer(
            "Collator projection requires a Custom profile",
        ));
    };
    let length = bytes
        .get(8..12)
        .ok_or_else(|| IntlDataImageError::consumer("Collator projection header"))?;
    let length = u32::from_le_bytes(length.try_into().expect("four-byte extent")) as usize;
    if length > MAX_DESCRIPTOR_BYTES {
        return Err(IntlDataImageError::consumer(
            "Collator projection descriptor extent",
        ));
    }
    let end = 12usize
        .checked_add(length)
        .ok_or_else(|| IntlDataImageError::consumer("Collator projection extent"))?;
    let raw: Descriptor =
        serde_json::from_slice(bytes.get(12..end).ok_or_else(|| {
            IntlDataImageError::consumer("Collator projection descriptor extent")
        })?)
        .map_err(IntlDataImageError::consumer)?;
    let requested = raw
        .public_locales
        .iter()
        .map(|name| LocaleId::parse(name.as_str()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(IntlDataImageError::consumer)?;
    let expected = project(id, &requested, locale)?;
    if bytes != expected.bytes {
        return Err(IntlDataImageError::consumer(
            "Collator projection differs from its complete pinned closure",
        ));
    }
    Ok(expected)
}

#[cfg(test)]
mod tests;
