//! Collator profile admission and comparison consume immutable image payloads.

use crate::image::{DataImageComponent, DataImageEnvelope};
use crate::{
    CollatorProfiles, IntlDataDigest, IntlDataImageError, IntlDataProfile, LocaleDataImage,
};
use icu_collator::provider::*;
use icu_normalizer::provider::{NormalizerNfdDataV1, NormalizerNfdTablesV1};
use icu_provider::buf::{AsDeserializingBufferProvider, BufferMarker};
use icu_provider::prelude::*;
use icu_provider_adapters::fallback::LocaleFallbackProvider;
use icu_provider_blob::BlobDataProvider;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fmt;
use std::sync::{Arc, OnceLock};

mod projection;
pub(crate) use projection::CollatorCatalogue;

pub const INTL_COLLATOR_DATA_CUSTOM_SECTION: &str = "lila.intl-collator-data.v1";
const MARKERS: [&str; 10] = [
    "collation/root/v1",
    "collation/tailoring/v1",
    "collation/metadata/v1",
    "collation/diacritics/v1",
    "collation/jamo/v1",
    "collation/reordering/v1",
    "collation/special/primaries/v1",
    "normalizer/nfd/data/v1",
    "normalizer/nfd/tables/v1",
    "lila/collator/locale-inventory/v1",
];
const PINNED_BLOB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/collator-data.blob"));
const PINNED_LOCALES: &[u8] = include_bytes!("../data/collator-icu-2/locale-inventory.json");
const MAGIC: &[u8; 8] = b"LCLI0001";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLocaleInventory {
    schema: u32,
    algorithm: String,
    cldr_release: String,
    cldr_commit: String,
    candidate_source_canonical_sha256: String,
    candidate_source_payload_sha256: String,
    default_locale: String,
    locales: Box<[Box<str>]>,
}

#[derive(Debug)]
struct CollatorLocaleInventory(Box<[Box<str>]>);
impl CollatorLocaleInventory {
    fn from_bytes(bytes: &[u8]) -> Result<Self, IntlDataImageError> {
        let raw: RawLocaleInventory =
            serde_json::from_slice(bytes).map_err(IntlDataImageError::consumer)?;
        if raw.schema != 1
            || raw.algorithm != "collator-locale-inventory-cldr47-v1"
            || raw.cldr_release != "47.0.0"
            || raw.cldr_commit != "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c"
            || raw.candidate_source_canonical_sha256
                != "4bc17e2ed0a5a51a70ad342fa9bb7cf91af036db737775dedbf505204205699a"
            || raw.candidate_source_payload_sha256
                != "d6e02ff9770f2a8a06a3be507eb5906878ed04994f1c416fbf1f8437b8815254"
            || raw.default_locale != "en-US"
            || raw.locales.len() != 1082
            || !raw.locales.windows(2).all(|pair| pair[0] < pair[1])
            || raw
                .locales
                .binary_search_by(|locale| locale.as_ref().cmp("en-US"))
                .is_err()
        {
            return Err(IntlDataImageError::consumer(
                "Collator candidate locale source descriptor or inventory",
            ));
        }
        Ok(Self(raw.locales))
    }
}

fn pinned_payload() -> Result<Vec<u8>, IntlDataImageError> {
    let size = 24usize
        .checked_add(PINNED_LOCALES.len())
        .and_then(|size| size.checked_add(PINNED_BLOB.len()))
        .ok_or_else(|| IntlDataImageError::consumer("Collator data payload extent"))?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(IntlDataImageError::consumer)?;
    bytes.extend_from_slice(MAGIC);
    for payload in [PINNED_LOCALES, PINNED_BLOB] {
        bytes.extend_from_slice(
            &u64::try_from(payload.len())
                .map_err(IntlDataImageError::consumer)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(payload);
    }
    Ok(bytes)
}

fn admit_payload(bytes: &[u8]) -> Result<(&[u8], CollatorLocaleInventory), IntlDataImageError> {
    let fail = || IntlDataImageError::consumer("Collator data payload framing or locked source");
    if bytes.get(..8) != Some(MAGIC.as_slice()) {
        return Err(fail());
    }
    let mut position = 8usize;
    let mut inputs = [&[][..]; 2];
    for (input, pinned) in inputs.iter_mut().zip([PINNED_LOCALES, PINNED_BLOB]) {
        let end = position.checked_add(8).ok_or_else(fail)?;
        let size = bytes.get(position..end).ok_or_else(fail)?;
        let size = usize::try_from(u64::from_le_bytes(
            size.try_into().expect("eight-byte extent"),
        ))
        .map_err(IntlDataImageError::consumer)?;
        position = end;
        let end = position.checked_add(size).ok_or_else(fail)?;
        *input = bytes.get(position..end).ok_or_else(fail)?;
        if *input != pinned {
            return Err(fail());
        }
        position = end;
    }
    if position != bytes.len() {
        return Err(fail());
    }
    Ok((inputs[1], CollatorLocaleInventory::from_bytes(inputs[0])?))
}

/// Only the image constructor can create a provider after decoding every real
/// marker identifier. Its fallback authority is retained with the typed data.
pub(crate) struct CollatorImageProvider {
    provider: LocaleFallbackProvider<CollatorBacking>,
    metadata_ids: BTreeSet<DataIdentifierCow<'static>>,
    locale: LocaleDataImage,
    locale_inventory: CollatorLocaleInventory,
    recordings: Option<Arc<projection::Recordings>>,
}

#[derive(Debug)]
enum CollatorBacking {
    Full(BlobDataProvider),
    Projected(projection::RawRows),
}
impl DynamicDataProvider<BufferMarker> for CollatorBacking {
    fn load_data(
        &self,
        marker: DataMarkerInfo,
        request: DataRequest,
    ) -> Result<DataResponse<BufferMarker>, DataError> {
        match self {
            Self::Full(provider) => provider.load_data(marker, request),
            Self::Projected(provider) => provider.load_data(marker, request),
        }
    }
}
impl fmt::Debug for CollatorImageProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CollatorImageProvider")
            .field("metadata_identifiers", &self.metadata_ids.len())
            .field("locale_digest", &self.locale.digest())
            .field("candidate_locales", &self.locale_inventory.0.len())
            .finish()
    }
}
impl CollatorImageProvider {
    fn checked(
        blob: &[u8],
        locale: &LocaleDataImage,
        locale_inventory: CollatorLocaleInventory,
    ) -> Result<Self, IntlDataImageError> {
        let provider = BlobDataProvider::try_new_from_blob(blob.into())
            .map_err(IntlDataImageError::consumer)?;
        macro_rules! admit {
            ($marker:ty) => {{
                let ids = IterableDynamicDataProvider::<BufferMarker>::iter_ids_for_marker(
                    &provider,
                    <$marker>::INFO,
                )
                .map_err(IntlDataImageError::consumer)?;
                if ids.is_empty()
                    || (<$marker>::INFO.is_singleton
                        && (ids.len() != 1
                            || !ids.iter().all(|id| {
                                id.locale.is_unknown() && id.marker_attributes.is_empty()
                            })))
                {
                    return Err(IntlDataImageError::consumer(concat!(
                        "missing or invalid exact identifiers for ",
                        stringify!($marker)
                    )));
                }
                for id in ids {
                    DataProvider::<$marker>::load(
                        &provider.as_deserializing(),
                        DataRequest {
                            id: id.as_borrowed(),
                            metadata: Default::default(),
                        },
                    )
                    .map_err(IntlDataImageError::consumer)?;
                }
            }};
        }
        admit!(CollationRootV1);
        admit!(CollationTailoringV1);
        admit!(CollationMetadataV1);
        admit!(CollationDiacriticsV1);
        admit!(CollationJamoV1);
        admit!(CollationReorderingV1);
        admit!(CollationSpecialPrimariesV1);
        admit!(NormalizerNfdDataV1);
        admit!(NormalizerNfdTablesV1);
        let metadata_ids = IterableDynamicDataProvider::<BufferMarker>::iter_ids_for_marker(
            &provider,
            CollationMetadataV1::INFO,
        )
        .map_err(IntlDataImageError::consumer)?
        .into_iter()
        .map(|id| id.as_borrowed().into_owned())
        .collect();
        Ok(Self {
            provider: LocaleFallbackProvider::new(
                CollatorBacking::Full(provider),
                locale.fallbacker(),
            ),
            metadata_ids,
            locale: locale.clone(),
            locale_inventory,
            recordings: None,
        })
    }
    fn projected(
        rows: projection::RawRows,
        locale: &LocaleDataImage,
    ) -> Result<Self, IntlDataImageError> {
        let metadata_ids = rows.metadata_ids();
        Ok(Self {
            provider: LocaleFallbackProvider::new(
                CollatorBacking::Projected(rows),
                locale.fallbacker(),
            ),
            metadata_ids,
            locale: locale.clone(),
            locale_inventory: CollatorLocaleInventory::from_bytes(PINNED_LOCALES)?,
            recordings: None,
        })
    }
    pub(crate) fn metadata_ids(&self) -> &BTreeSet<DataIdentifierCow<'static>> {
        &self.metadata_ids
    }
    /// Tests the exact admitted metadata domain under the same ICU fallback
    /// configuration as the real load. A projected provider consults only its
    /// retained rows; this does not make a profile available without admission.
    pub(crate) fn supports_metadata_id(&self, id: &DataIdentifierCow<'static>) -> bool {
        let fallbacker = self.locale.fallbacker();
        let mut locales = fallbacker
            .for_config(CollationMetadataV1::INFO.fallback_config)
            .fallback_for(id.locale);
        let mut candidate = id.clone();
        loop {
            candidate.locale = *locales.get();
            if self.metadata_ids.contains(&candidate) {
                return true;
            }
            if locales.get().is_unknown() {
                return false;
            }
            locales.step();
        }
    }
    pub(crate) fn locale_image(&self) -> &LocaleDataImage {
        &self.locale
    }
    pub(crate) fn candidate_locales(&self) -> impl ExactSizeIterator<Item = &str> {
        self.locale_inventory.0.iter().map(Box::as_ref)
    }
}
macro_rules! image_marker {
    ($marker:ty) => {
        impl DataProvider<$marker> for CollatorImageProvider {
            fn load(&self, request: DataRequest) -> Result<DataResponse<$marker>, DataError> {
                let response =
                    DataProvider::<$marker>::load(&self.provider.as_deserializing(), request)?;
                if let Some(recordings) = &self.recordings {
                    recordings.record(<$marker>::INFO, request, &response.metadata)?;
                }
                Ok(response)
            }
        }
    };
}
image_marker!(CollationRootV1);
image_marker!(CollationTailoringV1);
image_marker!(CollationMetadataV1);
image_marker!(CollationDiacriticsV1);
image_marker!(CollationJamoV1);
image_marker!(CollationReorderingV1);
image_marker!(CollationSpecialPrimariesV1);
image_marker!(NormalizerNfdDataV1);
image_marker!(NormalizerNfdTablesV1);

#[derive(Debug)]
struct AdmittedCollatorData {
    envelope: DataImageEnvelope,
    profiles: Arc<CollatorProfiles>,
    locale_digest: IntlDataDigest,
}

/// Framing alone cannot create this owner: all typed payloads and the complete
/// actual sort/search profile catalogue must load through its image provider.
#[derive(Debug, Clone)]
pub struct CollatorDataImage(Arc<AdmittedCollatorData>);
impl CollatorDataImage {
    pub(crate) fn require_complete_source(&self) -> Result<(), IntlDataImageError> {
        if self.0.envelope.blob() != pinned_payload()?.as_slice() {
            return Err(IntlDataImageError::IncompleteConformance);
        }
        Ok(())
    }

    pub fn for_custom_projection(
        id: &crate::CustomProfileId,
        public_locales: &[crate::LocaleId],
        locale: &LocaleDataImage,
    ) -> Result<Self, IntlDataImageError> {
        let profile = IntlDataProfile::Custom(id.clone());
        if locale.profile() != &profile {
            return Err(IntlDataImageError::consumer(
                "Collator projection and Locale profiles differ",
            ));
        }
        let projected = projection::project(id, public_locales, locale)?;
        Self::from_bytes(
            DataImageEnvelope::encode(
                DataImageComponent::Collator,
                &profile,
                &MARKERS,
                &projected.bytes,
            )?,
            locale,
        )
    }
    pub fn for_profile(
        profile: IntlDataProfile,
        locale: &LocaleDataImage,
    ) -> Result<Self, IntlDataImageError> {
        Self::from_bytes(
            DataImageEnvelope::encode(
                DataImageComponent::Collator,
                &profile,
                &MARKERS,
                &pinned_payload()?,
            )?,
            locale,
        )
    }
    pub fn from_bytes(
        bytes: Arc<[u8]>,
        locale: &LocaleDataImage,
    ) -> Result<Self, IntlDataImageError> {
        let envelope = DataImageEnvelope::decode(bytes, DataImageComponent::Collator, &MARKERS)?;
        if envelope.profile() != locale.profile() {
            return Err(IntlDataImageError::consumer(
                "Collator and Locale image profiles differ",
            ));
        }
        let profiles = if projection::is_projection(envelope.blob()) {
            let projected = projection::admit(envelope.blob(), envelope.profile(), locale)?;
            let data = Arc::new(CollatorImageProvider::projected(projected.rows, locale)?);
            Arc::new(
                CollatorProfiles::from_projection(data, &projected.catalogue)
                    .map_err(IntlDataImageError::consumer)?,
            )
        } else {
            let (blob, locale_inventory) = admit_payload(envelope.blob())?;
            let data = Arc::new(CollatorImageProvider::checked(
                blob,
                locale,
                locale_inventory,
            )?);
            Arc::new(CollatorProfiles::from_image(data).map_err(IntlDataImageError::consumer)?)
        };
        Ok(Self(Arc::new(AdmittedCollatorData {
            envelope,
            profiles,
            locale_digest: locale.digest(),
        })))
    }
    pub fn bytes(&self) -> Arc<[u8]> {
        self.0.envelope.bytes()
    }
    pub fn digest(&self) -> IntlDataDigest {
        self.0.envelope.digest()
    }
    pub fn profile(&self) -> &IntlDataProfile {
        self.0.envelope.profile()
    }
    pub fn profiles(&self) -> Arc<CollatorProfiles> {
        self.0.profiles.clone()
    }
    pub(crate) fn locale_digest(&self) -> IntlDataDigest {
        self.0.locale_digest
    }
}

pub fn embedded_collator_data_image() -> Result<CollatorDataImage, IntlDataImageError> {
    static IMAGE: OnceLock<Result<CollatorDataImage, IntlDataImageError>> = OnceLock::new();
    IMAGE
        .get_or_init(|| {
            let locale = crate::embedded_locale_data_image()?;
            CollatorDataImage::for_profile(IntlDataProfile::Minimal, &locale)
        })
        .clone()
}

#[cfg(test)]
mod tests;
