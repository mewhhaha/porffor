//! Deterministic singleton export from the exact locked ICU Locale provider.

use icu_locale::provider::{
    Baked, LocaleAliasesV1, LocaleLikelySubtagsExtendedV1, LocaleLikelySubtagsLanguageV1,
    LocaleLikelySubtagsScriptRegionV1, LocaleParentsV1,
};
use icu_provider::dynutil::UpcastDataPayload;
use icu_provider::export::{DataExporter, ExportMarker, FlushMetadata};
use icu_provider::prelude::*;
use icu_provider_blob::export::BlobExporter;

pub fn export() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    {
        let mut exporter = BlobExporter::new_with_sink(Box::new(&mut bytes));
        macro_rules! singleton {
            ($marker:ty) => {
                let response = DataProvider::<$marker>::load(&Baked, Default::default())?;
                let payload: DataPayload<ExportMarker> =
                    UpcastDataPayload::upcast(response.payload);
                exporter.put_payload(<$marker>::INFO, Default::default(), &payload)?;
                exporter.flush(<$marker>::INFO, FlushMetadata::default())?;
            };
        }
        singleton!(LocaleAliasesV1);
        singleton!(LocaleLikelySubtagsExtendedV1);
        singleton!(LocaleLikelySubtagsLanguageV1);
        singleton!(LocaleLikelySubtagsScriptRegionV1);
        singleton!(LocaleParentsV1);
        exporter.close()?;
    }
    if bytes.is_empty() {
        return Err("ICU Locale export produced no payload".into());
    }
    Ok(bytes)
}
