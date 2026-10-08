//! A bounded selected frame is rederived from the complete pinned source before
//! its private catalogue or exact raw-row provider can be published.
use super::{split_payload, SegmenterImageProvider, PINNED_PAYLOAD};
use crate::provider::LocaleCanonicalizationData;
use crate::{
    CanonicalLocaleId, CustomProfileId, IntlDataImageError, IntlDataProfile, LocaleDataImage,
    LocaleId, SegmenterProfiles,
};
use icu_locale::LanguageIdentifier;
use icu_provider::buf::BufferMarker;
use icu_provider::prelude::*;
use icu_provider_blob::BlobDataProvider;
use icu_segmenter::provider::{SegmenterBreakSentenceOverrideV1, SegmenterBreakWordOverrideV1};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::sync::Arc;

mod rows;
pub(super) use rows::RawRows;
use rows::{Marker, RowKey};
const MAGIC: &[u8; 8] = b"LSEGP001";
const MAX_DESCRIPTOR_BYTES: usize = 64 * 1024;
const MODEL_PREFIXES: [&str; 4] = ["Burmese_", "Khmer_", "Lao_", "Thai_"];

#[derive(Debug)]
pub(crate) struct SegmenterCatalogue {
    public: Box<[CanonicalLocaleId]>,
}
impl SegmenterCatalogue {
    pub(crate) fn public_locales(&self) -> &[CanonicalLocaleId] {
        &self.public
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
    source_payload_sha256: String,
    source_descriptor_sha256: String,
    locale_digest: String,
    default_locale: String,
    public_locales: Vec<String>,
    rows: Vec<RowDescriptor>,
}
pub(super) struct ProjectedData {
    pub(super) bytes: Vec<u8>,
    pub(super) rows: RawRows,
    pub(super) catalogue: SegmenterCatalogue,
}
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(super) fn is_projection(bytes: &[u8]) -> bool {
    bytes.starts_with(MAGIC)
}

fn record_override<M: DataMarker>(
    provider: &SegmenterImageProvider,
    locale: &CanonicalLocaleId,
    keys: &mut BTreeSet<RowKey>,
) -> Result<(), IntlDataImageError>
where
    SegmenterImageProvider: DataProvider<M>,
{
    let language: LanguageIdentifier = locale
        .as_str()
        .parse()
        .map_err(IntlDataImageError::consumer)?;
    let requested = DataLocale::from(&language);
    let response = match DataProvider::<M>::load(
        provider,
        DataRequest {
            id: DataIdentifierBorrowed::for_locale(&requested),
            metadata: Default::default(),
        },
    ) {
        Ok(response) => response,
        Err(error) if error.kind == DataErrorKind::IdentifierNotFound => return Ok(()),
        Err(error) => return Err(IntlDataImageError::consumer(error)),
    };
    keys.insert(RowKey {
        marker: Marker::from_info(M::INFO)
            .ok_or_else(|| IntlDataImageError::consumer("foreign Segmenter override"))?,
        id: DataIdentifierCow::from_owned(
            DataMarkerAttributes::empty().to_owned(),
            response.metadata.locale.unwrap_or(requested),
        ),
    });
    Ok(())
}

fn global_keys(source: &BlobDataProvider) -> Result<BTreeSet<RowKey>, IntlDataImageError> {
    let mut keys = BTreeSet::new();
    for marker in Marker::ALL.into_iter().filter(|marker| marker.is_global()) {
        let ids =
            IterableDynamicDataProvider::<BufferMarker>::iter_ids_for_marker(source, marker.info())
                .map_err(IntlDataImageError::consumer)?;
        let valid = match marker {
            Marker::Grapheme | Marker::Word | Marker::Sentence => {
                ids.len() == 1
                    && ids
                        .iter()
                        .all(|id| id.locale.is_unknown() && id.marker_attributes.is_empty())
            }
            Marker::Dictionary => {
                ids.len() == 1
                    && ids.iter().all(|id| {
                        id.locale.is_unknown() && id.marker_attributes.as_str() == "cjdict"
                    })
            }
            Marker::Lstm => {
                ids.len() == 4
                    && ids.iter().all(|id| id.locale.is_unknown())
                    && MODEL_PREFIXES.iter().all(|prefix| {
                        ids.iter()
                            .filter(|id| id.marker_attributes.as_str().starts_with(*prefix))
                            .count()
                            == 1
                    })
            }
            Marker::WordOverride | Marker::SentenceOverride => unreachable!("global marker filter"),
        };
        if !valid {
            return Err(IntlDataImageError::consumer(
                "Segmenter exact global rule/model inventory",
            ));
        }
        keys.extend(ids.into_iter().map(|id| RowKey {
            marker,
            id: id.as_borrowed().into_owned(),
        }));
    }
    Ok(keys)
}
fn check_prefixes(source: &BlobDataProvider, rows: &RawRows) -> Result<(), IntlDataImageError> {
    // Both auto LSTM and auto dictionary constructors set this flag, including
    // the exact cjdict attribute. Check actual original Blob loads, not labels.
    for (marker, prefix) in MODEL_PREFIXES
        .into_iter()
        .map(|p| (Marker::Lstm, p))
        .chain([(Marker::Dictionary, "cjdict")])
    {
        let mut metadata = DataRequestMetadata::default();
        metadata.attributes_prefix_match = true;
        let request = DataRequest {
            id: DataIdentifierBorrowed::for_marker_attributes(
                DataMarkerAttributes::from_str_or_panic(prefix),
            ),
            metadata,
        };
        let full = source
            .load_data(marker.info(), request)
            .map_err(IntlDataImageError::consumer)?;
        let selected = rows
            .load_data(marker.info(), request)
            .map_err(IntlDataImageError::consumer)?;
        if full.payload.get() != selected.payload.get()
            || full.metadata.checksum != selected.metadata.checksum
            || full.metadata.buffer_format != selected.metadata.buffer_format
        {
            return Err(IntlDataImageError::consumer(
                "Segmenter model prefix differs from pinned Blob lookup",
            ));
        }
    }
    Ok(())
}

pub(super) fn project(
    id: &CustomProfileId,
    requested: &[LocaleId],
    locale: &LocaleDataImage,
) -> Result<ProjectedData, IntlDataImageError> {
    if requested.is_empty() || locale.profile() != &IntlDataProfile::Custom(id.clone()) {
        return Err(IntlDataImageError::consumer(
            "Segmenter projection requires nonempty locales and matching Custom Locale",
        ));
    }
    let (descriptor, blob) = split_payload(PINNED_PAYLOAD)?;
    let data = Arc::new(SegmenterImageProvider::checked(blob, locale)?);
    // Full source validation retains all17 original associations and all
    // mandatory tailored rows before authorizing any reduced publication.
    let full = SegmenterProfiles::from_image_data(data.clone(), descriptor)
        .map_err(IntlDataImageError::consumer)?;
    let available = full.available_locales().cloned().collect::<BTreeSet<_>>();
    let canonicalizer =
        LocaleCanonicalizationData::from_image(locale).map_err(IntlDataImageError::consumer)?;
    let mut public = BTreeSet::new();
    for requested in requested {
        let name = canonicalizer
            .canonicalize(requested)
            .map_err(IntlDataImageError::consumer)?;
        if !available.contains(&name) || !public.insert(name) {
            return Err(IntlDataImageError::consumer(
                "Segmenter projection locale is absent or canonically duplicate",
            ));
        }
    }
    public.insert(CanonicalLocaleId::from_data("en-US").map_err(IntlDataImageError::consumer)?);
    let catalogue = SegmenterCatalogue {
        public: public.into_iter().collect(),
    };
    let source =
        BlobDataProvider::try_new_from_blob(blob.into()).map_err(IntlDataImageError::consumer)?;
    let mut keys = global_keys(&source)?;
    for name in catalogue.public_locales() {
        record_override::<SegmenterBreakWordOverrideV1>(&data, name, &mut keys)?;
        record_override::<SegmenterBreakSentenceOverrideV1>(&data, name, &mut keys)?;
    }
    let rows = RawRows::from_pinned(&source, &keys)?;
    check_prefixes(&source, &rows)?;
    let description = Descriptor {
        schema: 1,
        custom_id: id.as_str().into(),
        source_payload_sha256: sha(PINNED_PAYLOAD),
        source_descriptor_sha256: sha(descriptor),
        locale_digest: locale
            .digest()
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect(),
        default_locale: "en-US".into(),
        public_locales: catalogue.public.iter().map(|n| n.as_str().into()).collect(),
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
    let description = serde_json::to_vec(&description).map_err(IntlDataImageError::consumer)?;
    if description.len() > MAX_DESCRIPTOR_BYTES {
        return Err(IntlDataImageError::consumer(
            "Segmenter projected descriptor extent",
        ));
    }
    let mut size = 12usize
        .checked_add(description.len())
        .ok_or_else(|| IntlDataImageError::consumer("Segmenter projected payload extent"))?;
    for (_, row) in rows.iter() {
        size = size
            .checked_add(row.payload.get().len())
            .ok_or_else(|| IntlDataImageError::consumer("Segmenter projected payload extent"))?;
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
            "Segmenter projection requires Custom profile",
        ));
    };
    let length = bytes
        .get(8..12)
        .ok_or_else(|| IntlDataImageError::consumer("Segmenter projection header"))?;
    let length = u32::from_le_bytes(length.try_into().expect("four-byte extent")) as usize;
    if !(1..=MAX_DESCRIPTOR_BYTES).contains(&length) {
        return Err(IntlDataImageError::consumer(
            "Segmenter projection descriptor extent",
        ));
    }
    let end = 12usize
        .checked_add(length)
        .ok_or_else(|| IntlDataImageError::consumer("Segmenter projection extent"))?;
    let descriptor: Descriptor =
        serde_json::from_slice(bytes.get(12..end).ok_or_else(|| {
            IntlDataImageError::consumer("Segmenter projection descriptor extent")
        })?)
        .map_err(IntlDataImageError::consumer)?;
    let requested = descriptor
        .public_locales
        .iter()
        .map(|name| LocaleId::parse(name.as_str()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(IntlDataImageError::consumer)?;
    let expected = project(id, &requested, locale)?;
    if bytes != expected.bytes {
        return Err(IntlDataImageError::consumer(
            "Segmenter projection differs from its complete pinned closure",
        ));
    }
    Ok(expected)
}

#[cfg(test)]
mod tests;
