use std::{collections::BTreeSet, env, fs, path::PathBuf};

use icu_plurals::provider::{PluralsCardinalV1, PluralsOrdinalV1, PluralsRangesV1};
use icu_provider::buf::AsDeserializingBufferProvider;
use icu_provider::{DataProvider, DataRequest, IterableDataProvider};
use icu_provider_blob::BlobDataProvider;
use icu_provider_export::{
    blob_exporter::BlobExporter,
    prelude::{DataLocaleFamily, DataMarker, DeduplicationStrategy, ExportDriver, LocaleFallbacker},
};
use icu_provider_source::SourceDataProvider;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1).map(PathBuf::from);
    let output = args.next().ok_or("missing output directory")?;
    let cldr = args.next().ok_or("missing verified CLDR ZIP")?;
    if args.next().is_some() {
        return Err("unexpected generator argument".into());
    }

    let source = SourceDataProvider::new_custom().with_cldr(&cldr)?;
    let cardinal = <SourceDataProvider as IterableDataProvider<PluralsCardinalV1>>::iter_ids(&source)?;
    let ordinal = <SourceDataProvider as IterableDataProvider<PluralsOrdinalV1>>::iter_ids(&source)?;
    let ranges = <SourceDataProvider as IterableDataProvider<PluralsRangesV1>>::iter_ids(&source)?;
    check_inventory("cardinal", &cardinal, 219, &["und", "gv", "bal", "kw", "lld", "scn"])?;
    check_inventory("ordinal", &ordinal, 104, &["und", "bal", "kw", "lld", "scn"])?;
    check_inventory("ranges", &ranges, 92, &["und", "scn"])?;

    let fallbacker = LocaleFallbacker::try_new_unstable(&source)?;
    let driver = ExportDriver::new(
        [DataLocaleFamily::FULL],
        DeduplicationStrategy::None.into(),
        fallbacker,
    )
    .with_markers([
        PluralsCardinalV1::INFO,
        PluralsOrdinalV1::INFO,
        PluralsRangesV1::INFO,
    ]);

    fs::create_dir_all(&output)?;
    let blob_path = output.join("plurals.postcard");
    let exporter = BlobExporter::new_with_sink(Box::new(fs::File::create(&blob_path)?));
    driver.export(&source, exporter)?;

    // The blob retains every source identifier. Confirm both the inventory and
    // each decoded payload before publishing it for runtime use.
    let blob = BlobDataProvider::try_new_from_blob(fs::read(blob_path)?.into_boxed_slice())?;
    let decoded = blob.as_deserializing();
    macro_rules! check_roundtrip {
        ($marker:ty, $ids:expr) => {
            for id in &$ids {
                let request = || DataRequest {
                    id: id.as_borrowed(),
                    metadata: Default::default(),
                };
                let expected = DataProvider::<$marker>::load(&source, request())?.payload;
                let actual = DataProvider::<$marker>::load(&decoded, request())?.payload;
                if expected.get() != actual.get() {
                    return Err(format!("plural blob changed {id} payload").into());
                }
            }
        };
    }
    check_roundtrip!(PluralsCardinalV1, cardinal);
    check_roundtrip!(PluralsOrdinalV1, ordinal);
    check_roundtrip!(PluralsRangesV1, ranges);
    println!("Verified plural blob: 219 cardinal, 104 ordinal, 92 range source identifiers");
    Ok(())
}

fn check_inventory(
    marker: &str,
    ids: &BTreeSet<icu_provider::DataIdentifierCow<'_>>,
    expected_count: usize,
    required: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let locales = ids
        .iter()
        .map(|id| id.locale.to_string())
        .collect::<BTreeSet<_>>();
    if locales.len() != expected_count || locales.len() != ids.len() {
        return Err(format!("{marker} source inventory has {} IDs, expected {expected_count}", ids.len()).into());
    }
    for locale in required {
        if !locales.contains(*locale) {
            return Err(format!("{marker} source inventory lacks {locale}").into());
        }
    }
    Ok(())
}
