//! ICU4X 2.0.0 provider overlay for CLDR's operational `search` collation.
//!
//! ICU4X's standard compiled provider omits search tailoring data. Keep its
//! normal collation data intact and route only `collator=search*` requests to
//! the generated CLDR 47 blob; the matching locale fallbacker lets regional
//! locales inherit the same language tailoring used during data generation.
//!
//! The pinned search metadata values are only `1` and `9`: neither tailored
//! diacritics nor custom reordering is requested. Root has a special export
//! discrepancy: its search tailoring data is present, but its metadata leaves
//! the tailored bit unset. ICU4X consequently skips that payload. For search
//! requests only, this adapter sets that bit when absent so root search data
//! is layered over the standard root collation. If refreshed metadata starts
//! requesting diacritics or reordering, add those marker payloads here too.

use std::sync::OnceLock;

use icu_collator::provider::{
    Baked as CollatorBaked, CollationDiacriticsV1, CollationJamoV1, CollationMetadata,
    CollationMetadataV1, CollationReorderingV1, CollationRootV1, CollationSpecialPrimariesV1,
    CollationTailoringV1,
};
use icu_locale::LocaleFallbacker;
use icu_normalizer::provider::{
    Baked as NormalizerBaked, NormalizerNfdDataV1, NormalizerNfdTablesV1,
};
use icu_provider::buf::{AsDeserializingBufferProvider, DeserializingBufferProvider};
use icu_provider::{DataError, DataMarker, DataPayload, DataProvider, DataRequest, DataResponse};
use icu_provider_adapters::fallback::LocaleFallbackProvider;
use icu_provider_blob::BlobDataProvider;

const SEARCH_TAILORED_MASK: u32 = 1 << 3;

type SearchDeserializingProvider = DeserializingBufferProvider<'static, BlobDataProvider>;

static SEARCH_DATA_PROVIDER: OnceLock<Result<BlobDataProvider, DataError>> = OnceLock::new();

pub(crate) struct CollatorDataProvider;

fn search_data_provider() -> Result<&'static BlobDataProvider, DataError> {
    match SEARCH_DATA_PROVIDER.get_or_init(|| {
        BlobDataProvider::try_new_from_static_blob(include_bytes!(
            "collation_search/generated/search.postcard"
        ))
    }) {
        Ok(provider) => Ok(provider),
        Err(error) => Err(*error),
    }
}

fn load_search_data<M>(request: DataRequest) -> Result<DataResponse<M>, DataError>
where
    M: DataMarker,
    SearchDeserializingProvider: DataProvider<M>,
{
    let blob_provider = search_data_provider()?;
    let provider = LocaleFallbackProvider::new(
        blob_provider.as_deserializing(),
        LocaleFallbacker::new().static_to_owned(),
    );
    DataProvider::<M>::load(&provider, request)
}

macro_rules! delegate_collator_data {
    ($marker:ty, $provider:expr) => {
        impl DataProvider<$marker> for CollatorDataProvider {
            fn load(&self, request: DataRequest) -> Result<DataResponse<$marker>, DataError> {
                DataProvider::<$marker>::load(&$provider, request)
            }
        }
    };
}

delegate_collator_data!(CollationRootV1, CollatorBaked);
delegate_collator_data!(CollationDiacriticsV1, CollatorBaked);
delegate_collator_data!(CollationJamoV1, CollatorBaked);
delegate_collator_data!(CollationReorderingV1, CollatorBaked);
delegate_collator_data!(CollationSpecialPrimariesV1, CollatorBaked);
delegate_collator_data!(NormalizerNfdDataV1, NormalizerBaked);
delegate_collator_data!(NormalizerNfdTablesV1, NormalizerBaked);

impl DataProvider<CollationMetadataV1> for CollatorDataProvider {
    fn load(&self, request: DataRequest) -> Result<DataResponse<CollationMetadataV1>, DataError> {
        if !is_search_request(&request) {
            return DataProvider::<CollationMetadataV1>::load(&CollatorBaked, request);
        }

        let mut response = load_search_data::<CollationMetadataV1>(request)?;
        let bits = response.payload.get().bits;
        if bits & SEARCH_TAILORED_MASK == 0 {
            response.payload = DataPayload::from_owned(CollationMetadata {
                bits: bits | SEARCH_TAILORED_MASK,
            });
        }
        Ok(response)
    }
}

impl DataProvider<CollationTailoringV1> for CollatorDataProvider {
    fn load(&self, request: DataRequest) -> Result<DataResponse<CollationTailoringV1>, DataError> {
        if is_search_request(&request) {
            load_search_data::<CollationTailoringV1>(request)
        } else {
            DataProvider::<CollationTailoringV1>::load(&CollatorBaked, request)
        }
    }
}

fn is_search_request(request: &DataRequest) -> bool {
    request.id.marker_attributes.as_str().starts_with("search")
}
