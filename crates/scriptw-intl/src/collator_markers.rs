//! One raw ICU marker census shared by admission, projection and export.
//! The locale-inventory envelope member is deliberately outside this census.
macro_rules! for_each {
    ($consumer:ident) => {
        $consumer! {
            Root, CollationRootV1, "collation/root/v1", Singleton;
            Tailoring, CollationTailoringV1, "collation/tailoring/v1", Profile;
            Metadata, CollationMetadataV1, "collation/metadata/v1", Profile;
            Diacritics, CollationDiacriticsV1, "collation/diacritics/v1", Diacritics;
            Jamo, CollationJamoV1, "collation/jamo/v1", Singleton;
            Reordering, CollationReorderingV1, "collation/reordering/v1", Profile;
            SpecialPrimaries, CollationSpecialPrimariesV1, "collation/special/primaries/v1", Singleton;
            NfdData, NormalizerNfdDataV1, "normalizer/nfd/data/v1", Singleton;
            NfdTables, NormalizerNfdTablesV1, "normalizer/nfd/tables/v1", Singleton;
        }
    };
}
pub(crate) use for_each;
