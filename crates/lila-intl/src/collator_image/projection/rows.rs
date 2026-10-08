//! Exact pinned Postcard rows, without typed reserialization or locale fallback.
use crate::IntlDataImageError;
use icu_collator::provider::*;
use icu_normalizer::provider::{NormalizerNfdDataV1, NormalizerNfdTablesV1};
use icu_provider::buf::{BufferFormat, BufferMarker};
use icu_provider::prelude::*;
use icu_provider_blob::BlobDataProvider;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Marker {
    Root,
    Tailoring,
    Metadata,
    Diacritics,
    Jamo,
    Reordering,
    SpecialPrimaries,
    NfdData,
    NfdTables,
}
impl Marker {
    const ALL: [Self; 9] = [
        Self::Root,
        Self::Tailoring,
        Self::Metadata,
        Self::Diacritics,
        Self::Jamo,
        Self::Reordering,
        Self::SpecialPrimaries,
        Self::NfdData,
        Self::NfdTables,
    ];
    pub(super) fn info(self) -> DataMarkerInfo {
        match self {
            Self::Root => CollationRootV1::INFO,
            Self::Tailoring => CollationTailoringV1::INFO,
            Self::Metadata => CollationMetadataV1::INFO,
            Self::Diacritics => CollationDiacriticsV1::INFO,
            Self::Jamo => CollationJamoV1::INFO,
            Self::Reordering => CollationReorderingV1::INFO,
            Self::SpecialPrimaries => CollationSpecialPrimariesV1::INFO,
            Self::NfdData => NormalizerNfdDataV1::INFO,
            Self::NfdTables => NormalizerNfdTablesV1::INFO,
        }
    }
    pub(super) fn from_info(info: DataMarkerInfo) -> Option<Self> {
        Self::ALL.into_iter().find(|marker| marker.info() == info)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct RowKey {
    pub(super) marker: Marker,
    pub(super) id: DataIdentifierCow<'static>,
}

/// Only the source projector can create this provider from exact pinned loads.
/// Locale fallback remains exclusively in the existing ICU adapter above it.
#[derive(Debug)]
pub(in crate::collator_image) struct RawRows {
    rows: BTreeMap<RowKey, DataResponse<BufferMarker>>,
}
impl RawRows {
    pub(super) fn from_pinned(
        source: &BlobDataProvider,
        keys: &BTreeSet<RowKey>,
    ) -> Result<Self, IntlDataImageError> {
        let mut rows = BTreeMap::new();
        for key in keys {
            let response = source
                .load_data(
                    key.marker.info(),
                    DataRequest {
                        id: key.id.as_borrowed(),
                        metadata: Default::default(),
                    },
                )
                .map_err(IntlDataImageError::consumer)?;
            if response.metadata.locale.is_some()
                || response.metadata.buffer_format != Some(BufferFormat::Postcard1)
                || response.payload.get().is_empty()
                || (key.marker.info().is_singleton
                    && (!key.id.locale.is_unknown() || !key.id.marker_attributes.is_empty()))
            {
                return Err(IntlDataImageError::consumer(
                    "Collator projection is not an exact pinned Postcard row",
                ));
            }
            rows.insert(
                key.clone(),
                DataResponse {
                    metadata: response.metadata,
                    payload: DataPayload::from_owned_buffer(
                        response.payload.get().to_vec().into_boxed_slice(),
                    ),
                },
            );
        }
        Ok(Self { rows })
    }
    pub(super) fn iter(&self) -> impl Iterator<Item = (&RowKey, &DataResponse<BufferMarker>)> {
        self.rows.iter()
    }
    pub(in crate::collator_image) fn metadata_ids(&self) -> BTreeSet<DataIdentifierCow<'static>> {
        self.rows
            .keys()
            .filter(|key| key.marker == Marker::Metadata)
            .map(|key| key.id.clone())
            .collect()
    }
}
impl DynamicDataProvider<BufferMarker> for RawRows {
    fn load_data(
        &self,
        info: DataMarkerInfo,
        request: DataRequest,
    ) -> Result<DataResponse<BufferMarker>, DataError> {
        let marker = Marker::from_info(info)
            .ok_or_else(|| DataErrorKind::MarkerNotFound.with_req(info, request))?;
        if (info.is_singleton && !request.id.locale.is_unknown())
            || request.metadata.attributes_prefix_match
        {
            return Err(DataErrorKind::InvalidRequest.with_req(info, request));
        }
        self.rows
            .get(&RowKey {
                marker,
                id: request.id.into_owned(),
            })
            .map(|response| DataResponse {
                metadata: response.metadata.clone(),
                payload: response.payload.clone(),
            })
            .ok_or_else(|| DataErrorKind::IdentifierNotFound.with_req(info, request))
    }
}
