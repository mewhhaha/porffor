//! Deterministic complete export of the three consumed ICU List markers.

use icu_list::provider::{ListAndV1, ListOrV1, ListUnitV1};
use icu_provider::dynutil::UpcastDataPayload;
use icu_provider::export::{DataExporter, ExportMarker};
use icu_provider::prelude::*;
use icu_provider_blob::export::BlobExporter;

struct PinnedListProvider;
const _: () = {
    use icu_list_data::*;
    mod icu {
        pub use icu_list as list;
        pub use icu_locale as locale;
    }
    make_provider!(PinnedListProvider);
    impl_list_and_v1!(PinnedListProvider, ITER);
    impl_list_or_v1!(PinnedListProvider, ITER);
    impl_list_unit_v1!(PinnedListProvider, ITER);
};

pub(super) fn export() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let provider = PinnedListProvider;
    let mut output = Vec::new();
    {
        let mut exporter = BlobExporter::new_with_sink(Box::new(&mut output));
        macro_rules! export_marker {
            ($marker:ty) => {{
                let ids =
                    <PinnedListProvider as IterableDataProvider<$marker>>::iter_ids(&provider)?;
                if ids.is_empty() {
                    return Err("empty pinned List marker inventory".into());
                }
                // BTreeSet enumeration and BlobExporter's sorted final resources
                // make repeated exports independent of native hash-map ordering.
                for id in ids {
                    let response = <PinnedListProvider as DataProvider<$marker>>::load(
                        &provider,
                        DataRequest {
                            id: id.as_borrowed(),
                            ..Default::default()
                        },
                    )?;
                    if response.metadata.locale.is_some() {
                        return Err("pinned List inventory row unexpectedly used fallback".into());
                    }
                    let payload: DataPayload<ExportMarker> =
                        UpcastDataPayload::upcast(response.payload);
                    exporter.put_payload(<$marker>::INFO, id.as_borrowed(), &payload)?;
                }
                exporter.flush(<$marker>::INFO, Default::default())?;
            }};
        }
        export_marker!(ListAndV1);
        export_marker!(ListOrV1);
        export_marker!(ListUnitV1);
        exporter.close()?;
    }
    Ok(output)
}
