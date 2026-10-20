//! Exact pinned raw buffers with the original ICU identifier-prefix behavior.
use crate::IntlDataImageError;
use icu_provider::buf::{BufferFormat, BufferMarker};
use icu_provider::prelude::*;
use icu_provider_blob::BlobDataProvider;
use icu_segmenter::provider::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Marker {
    Grapheme,
    Word,
    Sentence,
    WordOverride,
    SentenceOverride,
    Dictionary,
    Lstm,
}
impl Marker {
    pub(super) const ALL: [Self; 7] = [
        Self::Grapheme,
        Self::Word,
        Self::Sentence,
        Self::WordOverride,
        Self::SentenceOverride,
        Self::Dictionary,
        Self::Lstm,
    ];
    pub(super) fn info(self) -> DataMarkerInfo {
        match self {
            Self::Grapheme => SegmenterBreakGraphemeClusterV1::INFO,
            Self::Word => SegmenterBreakWordV1::INFO,
            Self::Sentence => SegmenterBreakSentenceV1::INFO,
            Self::WordOverride => SegmenterBreakWordOverrideV1::INFO,
            Self::SentenceOverride => SegmenterBreakSentenceOverrideV1::INFO,
            Self::Dictionary => SegmenterDictionaryAutoV1::INFO,
            Self::Lstm => SegmenterLstmAutoV1::INFO,
        }
    }
    pub(super) fn from_info(info: DataMarkerInfo) -> Option<Self> {
        Self::ALL.into_iter().find(|marker| marker.info() == info)
    }
    pub(super) fn is_global(self) -> bool {
        match self {
            Self::Grapheme | Self::Word | Self::Sentence | Self::Dictionary | Self::Lstm => true,
            Self::WordOverride | Self::SentenceOverride => false,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct RowKey {
    pub(super) marker: Marker,
    pub(super) id: DataIdentifierCow<'static>,
}

/// Only the complete pinned projector can mint a raw provider. No full provider
/// remains behind a projected load, including optional missing override loads.
#[derive(Debug)]
pub(in crate::segmenter_image) struct RawRows {
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
                    "Segmenter projection is not an exact pinned Postcard row",
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
    #[cfg(test)]
    pub(super) fn remove(&mut self, key: &RowKey) {
        self.rows.remove(key);
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
        if info.is_singleton && !request.id.locale.is_unknown() {
            return Err(DataErrorKind::InvalidRequest.with_req(info, request));
        }
        let key = RowKey {
            marker,
            id: request.id.into_owned(),
        };
        // BlobSchema first takes an exact value, then the first lexicographic
        // descendant for a nonempty attribute prefix. The pinned model domain
        // has one row per consumed script prefix; admission proves its bytes.
        let response = self
            .rows
            .get(&key)
            .or_else(|| {
                (request.metadata.attributes_prefix_match && !key.id.marker_attributes.is_empty())
                    .then(|| {
                        self.rows
                            .iter()
                            .find(|(candidate, _)| {
                                candidate.marker == marker
                                    && candidate.id.locale == key.id.locale
                                    && candidate
                                        .id
                                        .marker_attributes
                                        .as_str()
                                        .starts_with(key.id.marker_attributes.as_str())
                            })
                            .map(|(_, response)| response)
                    })
                    .flatten()
            })
            .ok_or_else(|| DataErrorKind::IdentifierNotFound.with_req(info, request))?;
        Ok(DataResponse {
            metadata: response.metadata.clone(),
            payload: response.payload.clone(),
        })
    }
}
