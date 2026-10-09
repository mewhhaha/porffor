//! The original ordered ICU walk, run while building the compiler.

use super::{NormalizationMapping, NormalizationTables};
use icu_normalizer::{
    properties::{CanonicalCombiningClassMapBorrowed, CanonicalCompositionBorrowed},
    properties::{CanonicalDecompositionBorrowed, Decomposed},
    DecomposingNormalizerBorrowed,
};

pub(super) fn build() -> NormalizationTables {
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
