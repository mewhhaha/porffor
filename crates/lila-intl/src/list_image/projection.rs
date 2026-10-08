//! Deterministic projection of actual pinned List loads, including Duration's
//! private associations. Neither a source label nor a self-consistent digest
//! admits replacement data: readers recompute these exact bytes.

use super::PINNED_BLOB;
use crate::duration_format::DurationProfiles;
use crate::provider::LocaleCanonicalizationData;
use crate::{
    CanonicalLocaleId, CustomListProfile, IntlDataImageError, IntlDataProfile, ListProfiles,
    LocaleDataImage,
};
use icu_list::provider::{ListAndV1, ListOrV1, ListUnitV1};
use icu_provider::prelude::*;
use icu_provider_adapters::fallback::LocaleFallbackProvider;
use icu_provider_blob::BlobDataProvider;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::BTreeSet;

mod export;

const MAGIC: &[u8; 8] = b"LILALIP1";
const HEADER_BYTES: usize = 20;
const MAX_DESCRIPTOR_BYTES: usize = 512 * 1024;

/// The only constructors derive both domains from the actual full pinned
/// inventory. Consumers cannot supply an arbitrary dependency or public set.
pub(crate) struct ListCatalogue {
    public: Box<[CanonicalLocaleId]>,
    required: Box<[CanonicalLocaleId]>,
    duration: Box<[(CanonicalLocaleId, CanonicalLocaleId)]>,
}

impl ListCatalogue {
    pub(crate) fn full(locale: &LocaleDataImage) -> Result<Self, IntlDataImageError> {
        let image = BlobDataProvider::try_new_from_static_blob(PINNED_BLOB)
            .map_err(IntlDataImageError::consumer)?;
        let canonicalizer = locale.canonicalizer();
        let mut names = BTreeSet::new();
        macro_rules! add_ids {
            ($marker:ty) => {
                let ids = image
                    .iter_ids_for_marker(<$marker>::INFO)
                    .map_err(IntlDataImageError::consumer)?;
                if ids.is_empty() {
                    return Err(IntlDataImageError::consumer(
                        "empty image List marker inventory",
                    ));
                }
                for id in ids {
                    let mut locale: icu_locale::Locale = id
                        .locale
                        .to_string()
                        .parse()
                        .map_err(IntlDataImageError::consumer)?;
                    canonicalizer.canonicalize(&mut locale);
                    names.insert(
                        CanonicalLocaleId::from_data(locale.to_string())
                            .map_err(IntlDataImageError::consumer)?,
                    );
                }
            };
        }
        add_ids!(ListAndV1);
        add_ids!(ListOrV1);
        add_ids!(ListUnitV1);
        names.insert(CanonicalLocaleId::from_data("en-US").map_err(IntlDataImageError::consumer)?);
        let names = names.into_iter().collect::<Vec<_>>().into_boxed_slice();
        let duration = Self::duration_associations_from(&names)?;
        Ok(Self {
            public: names.clone(),
            required: names,
            duration,
        })
    }

    fn projected(
        full: &Self,
        selection: &CustomListProfile,
        locale: &LocaleDataImage,
    ) -> Result<Self, IntlDataImageError> {
        let canonicalizer =
            LocaleCanonicalizationData::from_image(locale).map_err(IntlDataImageError::consumer)?;
        let mut public = BTreeSet::new();
        for requested in selection.requested_locales() {
            let canonical = canonicalizer
                .canonicalize(requested)
                .map_err(IntlDataImageError::consumer)?;
            if full.public.binary_search(&canonical).is_err() {
                return Err(IntlDataImageError::consumer(format!(
                    "List projection locale {} is absent from the pinned public inventory",
                    canonical.as_str(),
                )));
            }
            if !public.insert(canonical) {
                return Err(IntlDataImageError::consumer(
                    "List projection contains duplicate canonical locales",
                ));
            }
        }
        public.insert(CanonicalLocaleId::from_data("en-US").map_err(IntlDataImageError::consumer)?);
        let mut required = public.clone();
        for (_, resolved) in &full.duration {
            required.insert(resolved.clone());
        }
        Ok(Self {
            public: public.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            required: required.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            duration: full.duration.clone(),
        })
    }

    fn duration_associations_from(
        names: &[CanonicalLocaleId],
    ) -> Result<Box<[(CanonicalLocaleId, CanonicalLocaleId)]>, IntlDataImageError> {
        let default = names
            .binary_search_by(|locale| locale.as_str().cmp("en-US"))
            .map_err(IntlDataImageError::consumer)?;
        DurationProfiles::required_list_locales()
            .iter()
            .map(|requested| {
                let index = Self::matching(requested, |candidate| {
                    names
                        .binary_search_by(|locale| locale.as_str().cmp(candidate))
                        .ok()
                })
                .unwrap_or(default);
                Ok((
                    CanonicalLocaleId::from_data(*requested)
                        .map_err(IntlDataImageError::consumer)?,
                    names[index].clone(),
                ))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    /// The same prefix search owns ordinary resolution and projection dependency
    /// selection. It never substitutes ICU data fallback for public support.
    pub(crate) fn matching(
        requested: &str,
        mut lookup: impl FnMut(&str) -> Option<usize>,
    ) -> Option<usize> {
        let mut candidate = requested;
        loop {
            if let Some(index) = lookup(candidate) {
                return Some(index);
            }
            let position = candidate.rfind('-')?;
            candidate = &candidate[..position];
            if candidate
                .rsplit('-')
                .next()
                .is_some_and(|part| part.len() == 1)
            {
                let position = candidate.rfind('-')?;
                candidate = &candidate[..position];
            }
        }
    }

    pub(crate) fn public_locales(&self) -> &[CanonicalLocaleId] {
        &self.public
    }
    pub(crate) fn required_locales(&self) -> &[CanonicalLocaleId] {
        &self.required
    }
    pub(crate) fn duration_associations(&self) -> &[(CanonicalLocaleId, CanonicalLocaleId)] {
        &self.duration
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
enum Marker {
    #[serde(rename = "list/and/v1")]
    And,
    #[serde(rename = "list/or/v1")]
    Or,
    #[serde(rename = "list/unit/v1")]
    Unit,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    marker: Marker,
    attributes: String,
    locale: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Association {
    duration_locale: String,
    list_locale: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Descriptor {
    schema: u16,
    custom_id: String,
    default_locale: String,
    full_list_blob_sha256: [u8; 32],
    locale_image_sha256: [u8; 32],
    duration_profile_sha256: [u8; 32],
    public_locales: Vec<String>,
    duration_associations: Vec<Association>,
    rows: Vec<Row>,
}

struct RecordingProvider<'a> {
    image: &'a BlobDataProvider,
    rows: RefCell<BTreeSet<(Marker, DataIdentifierCow<'static>)>>,
}
macro_rules! record_marker {
    ($marker:ty, $kind:ident) => {
        impl DataProvider<$marker> for RecordingProvider<'_> {
            fn load(&self, request: DataRequest) -> Result<DataResponse<$marker>, DataError> {
                let response: DataResponse<$marker> =
                    self.image.as_deserializing().load(request)?;
                if response.metadata.locale.is_some() {
                    return Err(DataError::custom(
                        "pinned List row unexpectedly used fallback",
                    ));
                }
                self.rows
                    .borrow_mut()
                    .insert((Marker::$kind, request.id.into_owned()));
                Ok(response)
            }
        }
    };
}
record_marker!(ListAndV1, And);
record_marker!(ListOrV1, Or);
record_marker!(ListUnitV1, Unit);

fn check_profile(
    selection: &CustomListProfile,
    locale: &LocaleDataImage,
) -> Result<(), IntlDataImageError> {
    let profile = IntlDataProfile::Custom(selection.id().clone());
    if locale.profile() != &profile {
        return Err(IntlDataImageError::consumer(
            "List projection and Locale profiles differ",
        ));
    }
    Ok(())
}

fn project(
    selection: &CustomListProfile,
    locale: &LocaleDataImage,
) -> Result<(Vec<u8>, ListCatalogue), IntlDataImageError> {
    check_profile(selection, locale)?;
    let full = BlobDataProvider::try_new_from_static_blob(PINNED_BLOB)
        .map_err(IntlDataImageError::consumer)?;
    let catalogue = ListCatalogue::projected(&ListCatalogue::full(locale)?, selection, locale)?;
    let recorder = RecordingProvider {
        image: &full,
        rows: RefCell::new(BTreeSet::new()),
    };
    // Recording is below the actual fallback adapter, so identifiers are the
    // physical successful marker/width/locale rows consumed by all nine loads.
    let provider = LocaleFallbackProvider::new(&recorder, locale.fallbacker());
    let admitted = ListProfiles::from_data_provider(&provider, &catalogue)
        .map_err(IntlDataImageError::consumer)?;
    drop(admitted);
    drop(provider);
    let rows = recorder.rows.into_inner();
    let descriptor = Descriptor {
        schema: 1,
        custom_id: selection.id().as_str().to_owned(),
        default_locale: "en-US".to_owned(),
        full_list_blob_sha256: Sha256::digest(PINNED_BLOB).into(),
        locale_image_sha256: *locale.digest().as_bytes(),
        duration_profile_sha256: crate::duration_format::DURATION_PROFILE_SHA256,
        public_locales: catalogue
            .public
            .iter()
            .map(|locale| locale.as_str().to_owned())
            .collect(),
        duration_associations: catalogue
            .duration
            .iter()
            .map(|(requested, resolved)| Association {
                duration_locale: requested.as_str().to_owned(),
                list_locale: resolved.as_str().to_owned(),
            })
            .collect(),
        rows: rows
            .iter()
            .map(|(marker, id)| Row {
                marker: *marker,
                attributes: id.marker_attributes.as_str().to_owned(),
                locale: id.locale.to_string(),
            })
            .collect(),
    };
    let blob = export::export_rows(&full, &rows)?;
    Ok((frame_payload(&descriptor, &blob)?, catalogue))
}

fn frame_payload(descriptor: &Descriptor, blob: &[u8]) -> Result<Vec<u8>, IntlDataImageError> {
    let descriptor = serde_json::to_vec(descriptor).map_err(IntlDataImageError::consumer)?;
    if descriptor.len() > MAX_DESCRIPTOR_BYTES {
        return Err(IntlDataImageError::consumer(
            "List projection descriptor exceeds its extent",
        ));
    }
    let extent = HEADER_BYTES
        .checked_add(descriptor.len())
        .and_then(|n| n.checked_add(blob.len()))
        .filter(|&n| n <= crate::MAX_INTL_COMPONENT_IMAGE_BYTES)
        .ok_or_else(|| {
            IntlDataImageError::consumer("List projection payload exceeds its extent")
        })?;
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
    selection: &CustomListProfile,
    locale: &LocaleDataImage,
) -> Result<Vec<u8>, IntlDataImageError> {
    project(selection, locale).map(|(payload, _)| payload)
}

pub(super) fn admit(
    payload: &[u8],
    profile: &IntlDataProfile,
    locale: &LocaleDataImage,
) -> Result<(BlobDataProvider, ListCatalogue), IntlDataImageError> {
    let IntlDataProfile::Custom(id) = profile else {
        return Err(IntlDataImageError::consumer(
            "a projected List payload requires Custom",
        ));
    };
    let (descriptor, blob) = split_payload(payload)?;
    let requested = descriptor
        .public_locales
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let selection =
        CustomListProfile::new(id.clone(), &requested).map_err(IntlDataImageError::consumer)?;
    let (expected, catalogue) = project(&selection, locale)?;
    // This comparison binds the descriptor, exact complete row set, conditional
    // pattern bytes, canonical export layout and absence of extra marker data.
    if payload != expected.as_slice() {
        return Err(IntlDataImageError::consumer(
            "List projection differs from its pinned derivation",
        ));
    }
    let image =
        BlobDataProvider::try_new_from_blob(blob.into()).map_err(IntlDataImageError::consumer)?;
    Ok((image, catalogue))
}

fn split_payload(payload: &[u8]) -> Result<(Descriptor, &[u8]), IntlDataImageError> {
    if payload.len() < HEADER_BYTES || &payload[..8] != MAGIC {
        return Err(IntlDataImageError::consumer(
            "invalid List projection framing",
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
        .filter(|_| descriptor_length <= MAX_DESCRIPTOR_BYTES)
        .ok_or_else(|| IntlDataImageError::consumer("invalid List descriptor extent"))?;
    if blob_start.checked_add(blob_length) != Some(payload.len()) {
        return Err(IntlDataImageError::consumer(
            "invalid List projection payload extent",
        ));
    }
    let descriptor: Descriptor = serde_json::from_slice(&payload[HEADER_BYTES..blob_start])
        .map_err(IntlDataImageError::consumer)?;
    Ok((descriptor, &payload[blob_start..]))
}

#[cfg(test)]
mod tests;
