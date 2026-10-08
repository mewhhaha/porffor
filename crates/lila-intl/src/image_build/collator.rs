//! Export every exact identifier from the source-owned Collation/NFD cache.

use icu_collator::provider::*;
use icu_normalizer::provider::{NormalizerNfdDataV1, NormalizerNfdTablesV1};
use icu_provider::dynutil::UpcastDataPayload;
use icu_provider::export::{DataExporter, ExportMarker, FlushMetadata};
use icu_provider::prelude::*;
use icu_provider_blob::export::BlobExporter;
use lila_intl_collator_data::PinnedCollatorProvider;

pub fn export() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    {
        let mut exporter = BlobExporter::new_with_sink(Box::new(&mut bytes));
        macro_rules! marker {
            ($marker:ty) => {{
                let ids = IterableDataProvider::<$marker>::iter_ids(&PinnedCollatorProvider)?;
                if ids.is_empty() {
                    return Err(
                        concat!("missing cached identifiers for ", stringify!($marker)).into(),
                    );
                }
                for id in ids {
                    let response = DataProvider::<$marker>::load(
                        &PinnedCollatorProvider,
                        DataRequest {
                            id: id.as_borrowed(),
                            metadata: Default::default(),
                        },
                    )?;
                    // Export exact cache keys, never aliases materialized by fallback.
                    if response
                        .metadata
                        .locale
                        .as_ref()
                        .is_some_and(|locale| locale != &id.locale)
                    {
                        return Err(concat!(
                            "cached identifier unexpectedly fell back for ",
                            stringify!($marker)
                        )
                        .into());
                    }
                    let payload: DataPayload<ExportMarker> =
                        UpcastDataPayload::upcast(response.payload);
                    exporter.put_payload(<$marker>::INFO, id.as_borrowed(), &payload)?;
                }
                exporter.flush(<$marker>::INFO, FlushMetadata::default())?;
            }};
        }
        marker!(CollationRootV1);
        marker!(CollationTailoringV1);
        marker!(CollationMetadataV1);
        marker!(CollationDiacriticsV1);
        marker!(CollationJamoV1);
        marker!(CollationReorderingV1);
        marker!(CollationSpecialPrimariesV1);
        marker!(NormalizerNfdDataV1);
        marker!(NormalizerNfdTablesV1);
        exporter.close()?;
    }
    if bytes.is_empty() {
        return Err("Collator export produced no payload".into());
    }
    Ok(bytes)
}
