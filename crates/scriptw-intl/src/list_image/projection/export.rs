//! Exact typed export of the recorded successful physical List rows.
use super::Marker;
use crate::IntlDataImageError;
use icu_list::provider::{ListAndV1, ListOrV1, ListUnitV1};
use icu_provider::dynutil::UpcastDataPayload;
use icu_provider::export::{DataExporter, ExportMarker};
use icu_provider::prelude::*;
use icu_provider_blob::export::BlobExporter;
use icu_provider_blob::BlobDataProvider;
use std::collections::BTreeSet;

pub(super) fn export_rows(
    full: &BlobDataProvider,
    rows: &BTreeSet<(Marker, DataIdentifierCow<'static>)>,
) -> Result<Vec<u8>, IntlDataImageError> {
    let mut blob = Vec::new();
    {
        let mut exporter = BlobExporter::new_with_sink(Box::new(&mut blob));
        macro_rules! export_marker {
            ($marker:ty, $kind:ident) => {{
                for (_, id) in rows.iter().filter(|(kind, _)| *kind == Marker::$kind) {
                    let response: DataResponse<$marker> = full
                        .as_deserializing()
                        .load(DataRequest {
                            id: id.as_borrowed(),
                            ..Default::default()
                        })
                        .map_err(IntlDataImageError::consumer)?;
                    let payload: DataPayload<ExportMarker> =
                        UpcastDataPayload::upcast(response.payload);
                    exporter
                        .put_payload(<$marker>::INFO, id.as_borrowed(), &payload)
                        .map_err(IntlDataImageError::consumer)?;
                }
                exporter
                    .flush(<$marker>::INFO, Default::default())
                    .map_err(IntlDataImageError::consumer)?;
            }};
        }
        export_marker!(ListAndV1, And);
        export_marker!(ListOrV1, Or);
        export_marker!(ListUnitV1, Unit);
        exporter.close().map_err(IntlDataImageError::consumer)?;
    }
    Ok(blob)
}
