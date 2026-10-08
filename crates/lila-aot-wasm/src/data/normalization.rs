//! Immutable normalization rows shared by every emitted runtime image.

use icu_normalizer::{
    properties::{CanonicalCombiningClassMapBorrowed, CanonicalCompositionBorrowed},
    properties::{CanonicalDecompositionBorrowed, Decomposed},
    DecomposingNormalizerBorrowed,
};
use std::sync::OnceLock;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct NormalizationMapping {
    pub(super) codepoint: u32,
    pub(super) sequence_index: u32,
    pub(super) sequence_len: u32,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct NormalizationTables {
    pub(super) canonical_mappings: Vec<NormalizationMapping>,
    pub(super) canonical_sequences: Vec<u32>,
    pub(super) compatibility_mappings: Vec<NormalizationMapping>,
    pub(super) compatibility_sequences: Vec<u32>,
    pub(super) combining_classes: Vec<(u32, u8)>,
    pub(super) compositions: Vec<(u32, u32, u32)>,
}

pub(super) fn tables() -> &'static NormalizationTables {
    static TABLES: OnceLock<NormalizationTables> = OnceLock::new();
    TABLES.get_or_init(build)
}

fn build() -> NormalizationTables {
    let nfd = DecomposingNormalizerBorrowed::new_nfd();
    let nfkd = DecomposingNormalizerBorrowed::new_nfkd();
    let combining_classes = CanonicalCombiningClassMapBorrowed::new();
    let compositions = CanonicalCompositionBorrowed::new();
    let canonical_decomposition = CanonicalDecompositionBorrowed::new();
    let mut tables = NormalizationTables::default();
    let mut utf8 = [0; 4];

    for codepoint in (0..=char::MAX as u32).filter_map(char::from_u32) {
        let source: &str = codepoint.encode_utf8(&mut utf8);
        // ICU borrows the original UTF-8 for identity mappings. Keep those
        // scalars allocation-free; only changed sequences need stored rows.
        let canonical = nfd.normalize(source);
        if canonical.as_ref() != source {
            let sequence_index = tables.canonical_sequences.len();
            tables
                .canonical_sequences
                .extend(canonical.chars().map(u32::from));
            tables.canonical_mappings.push(NormalizationMapping {
                codepoint: u32::from(codepoint),
                sequence_index: sequence_index as u32,
                sequence_len: (tables.canonical_sequences.len() - sequence_index) as u32,
            });
        }

        let compatibility = nfkd.normalize(source);
        if compatibility.as_ref() != source {
            let sequence_index = tables.compatibility_sequences.len();
            tables
                .compatibility_sequences
                .extend(compatibility.chars().map(u32::from));
            tables.compatibility_mappings.push(NormalizationMapping {
                codepoint: u32::from(codepoint),
                sequence_index: sequence_index as u32,
                sequence_len: (tables.compatibility_sequences.len() - sequence_index) as u32,
            });
        }

        let combining_class = combining_classes.get_u8(codepoint);
        if combining_class != 0 {
            tables
                .combining_classes
                .push((u32::from(codepoint), combining_class));
        }

        if let Decomposed::Expansion(first, second) = canonical_decomposition.decompose(codepoint) {
            if let Some(composed) = compositions.compose(first, second) {
                tables.compositions.push((
                    u32::from(first),
                    u32::from(second),
                    u32::from(composed),
                ));
            }
        }
    }
    tables.compositions.sort_unstable();
    tables.compositions.dedup();
    tables
}

#[cfg(test)]
mod tests {
    use super::*;

    // Retain the previous scalar algorithm as an independent inventory oracle.
    // Comparing all fields also checks sequence offsets and row ordering used
    // by the unchanged Wasm data serializer.
    fn full_scalar_reference() -> NormalizationTables {
        let nfd = DecomposingNormalizerBorrowed::new_nfd();
        let nfkd = DecomposingNormalizerBorrowed::new_nfkd();
        let combining_classes = CanonicalCombiningClassMapBorrowed::new();
        let compositions = CanonicalCompositionBorrowed::new();
        let canonical_decomposition = CanonicalDecompositionBorrowed::new();
        let mut tables = NormalizationTables::default();

        for codepoint in (0..=char::MAX as u32).filter_map(char::from_u32) {
            let source = codepoint.to_string();
            let canonical: Vec<u32> = nfd.normalize(&source).chars().map(u32::from).collect();
            if canonical.as_slice() != [u32::from(codepoint)] {
                tables.canonical_mappings.push(NormalizationMapping {
                    codepoint: u32::from(codepoint),
                    sequence_index: tables.canonical_sequences.len() as u32,
                    sequence_len: canonical.len() as u32,
                });
                tables.canonical_sequences.extend(canonical);
            }

            let compatibility: Vec<u32> = nfkd.normalize(&source).chars().map(u32::from).collect();
            if compatibility.as_slice() != [u32::from(codepoint)] {
                tables.compatibility_mappings.push(NormalizationMapping {
                    codepoint: u32::from(codepoint),
                    sequence_index: tables.compatibility_sequences.len() as u32,
                    sequence_len: compatibility.len() as u32,
                });
                tables.compatibility_sequences.extend(compatibility);
            }

            let combining_class = combining_classes.get_u8(codepoint);
            if combining_class != 0 {
                tables
                    .combining_classes
                    .push((u32::from(codepoint), combining_class));
            }
            if let Decomposed::Expansion(first, second) =
                canonical_decomposition.decompose(codepoint)
            {
                if let Some(composed) = compositions.compose(first, second) {
                    tables.compositions.push((
                        u32::from(first),
                        u32::from(second),
                        u32::from(composed),
                    ));
                }
            }
        }
        tables.compositions.sort_unstable();
        tables.compositions.dedup();
        tables
    }

    #[test]
    fn normalization_tables_match_full_scalar_reference() {
        assert_eq!(tables(), &full_scalar_reference());
    }
}
