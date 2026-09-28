use std::{env, fs, path::PathBuf};

use icu_collator::provider::{CollationMetadataV1, CollationTailoringV1};
use icu_provider::{DataProvider, DataRequest, IterableDataProvider};
use icu_provider_export::{
    blob_exporter::BlobExporter,
    prelude::{
        DataLocaleFamily, DataMarker, DeduplicationStrategy, ExportDriver, LocaleFallbacker,
    },
};
use icu_provider_source::{CollationRootHan, SourceDataProvider};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1).map(PathBuf::from);
    let output = args.next().ok_or("missing output directory")?;
    let cldr = args.next().ok_or("missing verified CLDR ZIP")?;
    let icu_export = args.next().ok_or("missing verified ICU export ZIP")?;
    let original_icu_export = args.next().ok_or("missing original ICU export ZIP")?;
    if args.next().is_some() {
        return Err("unexpected generator argument".into());
    }

    let source = SourceDataProvider::new_custom()
        .with_collation_root_han(CollationRootHan::Implicit)
        .with_cldr(&cldr)?
        .with_icuexport(&icu_export)?;
    let original = SourceDataProvider::new_custom()
        .with_collation_root_han(CollationRootHan::Implicit)
        .with_cldr(&cldr)?
        .with_icuexport(&original_icu_export)?;
    validate_jamo_overlay(&original, &source)?;
    let fallbacker = LocaleFallbacker::try_new_unstable(&source)?;

    let driver = ExportDriver::new(
        [DataLocaleFamily::FULL],
        DeduplicationStrategy::Maximal.into(),
        fallbacker,
    )
    .with_markers([CollationTailoringV1::INFO, CollationMetadataV1::INFO])
    .with_additional_collations(["search*".to_owned()])
    .with_marker_attributes_filter("collator", |attributes| {
        attributes.as_str().starts_with("search")
    });

    fs::create_dir_all(&output)?;
    let exporter =
        BlobExporter::new_with_sink(Box::new(fs::File::create(output.join("search.postcard"))?));
    driver.export(&source, exporter)?;
    Ok(())
}

/// The export repair may restore Jamo mappings, but may not change any other
/// code point or the CE/context tables that own those mappings. Check the whole
/// Unicode domain with ICU's own trie decoder rather than interpreting a
/// particular serialized trie layout in the Python wrapper.
fn validate_jamo_overlay(
    original: &SourceDataProvider,
    repaired: &SourceDataProvider,
) -> Result<(), Box<dyn std::error::Error>> {
    let search_ids = |provider: &SourceDataProvider| {
        <SourceDataProvider as IterableDataProvider<CollationTailoringV1>>::iter_ids(provider).map(
            |ids| {
                ids.into_iter()
                    .filter(|id| id.marker_attributes.as_str().starts_with("search"))
                    .map(|id| id.as_borrowed().into_owned())
                    .collect::<std::collections::BTreeSet<_>>()
            },
        )
    };
    let ids = search_ids(original)?;
    if ids.len() != 22 || ids != search_ids(repaired)? {
        return Err("the pinned search collation inventory must contain the same 22 IDs".into());
    }
    for id in ids {
        let request = DataRequest {
            id: id.as_borrowed(),
            metadata: Default::default(),
        };
        let before = DataProvider::<CollationTailoringV1>::load(original, request)?.payload;
        let after = DataProvider::<CollationTailoringV1>::load(repaired, request)?.payload;
        let (before, after) = (before.get(), after.get());
        if before.ces != after.ces
            || before.ce32s != after.ce32s
            || before.contexts != after.contexts
        {
            return Err(format!("search collation {id} changed CE or context data").into());
        }
        let mut restored = 0;
        for code_point in 0..=0x10ffff {
            if before.trie.get32(code_point) == after.trie.get32(code_point) {
                continue;
            }
            if !(0x1100..0x1200).contains(&code_point) {
                return Err(
                    format!("search collation {id} changed non-Jamo U+{code_point:04X}").into(),
                );
            }
            restored += 1;
        }
        if restored == 0 {
            return Err(format!("search collation {id} has no restored Jamo mappings").into());
        }
    }
    println!("Verified 22 search collations: only Jamo trie mappings changed");
    Ok(())
}
