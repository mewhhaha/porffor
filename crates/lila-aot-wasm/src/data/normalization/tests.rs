use super::*;
use icu_normalizer::{
    properties::{CanonicalCombiningClassMapBorrowed, CanonicalCompositionBorrowed},
    properties::{CanonicalDecompositionBorrowed, Decomposed},
    DecomposingNormalizerBorrowed,
};

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

#[test]
fn normalization_tables_match_full_scalar_reference() {
    let reference = full_scalar_reference();
    assert_eq!(tables(), &reference);
    assert_eq!(construction::build(), reference);
    assert_eq!(reference.to_image(), EMBEDDED_IMAGE);
    for prefix in 0..8 {
        assert_pool_matches_reference(&reference, prefix);
    }
}

fn append_words(bytes: &mut Vec<u8>, words: impl IntoIterator<Item = u32>) -> u32 {
    bytes.resize(bytes.len().next_multiple_of(8), 0);
    let pointer = super::super::STATIC_DATA_OFFSET + bytes.len() as u32;
    for word in words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    pointer
}

fn assert_pool_matches_reference(reference: &NormalizationTables, prefix: usize) {
    use super::super::StringPool;
    let mut expected = vec![0xa5; prefix];
    let mut pool = StringPool {
        bytes: expected.clone(),
        ..StringPool::default()
    };
    let canonical_sequences =
        append_words(&mut expected, reference.canonical_sequences.iter().copied());
    let canonical_mappings = append_words(
        &mut expected,
        reference.canonical_mappings.iter().flat_map(|mapping| {
            [
                mapping.codepoint,
                canonical_sequences + mapping.sequence_index * 4,
                mapping.sequence_len,
                0,
            ]
        }),
    );
    let compatibility_sequences = append_words(
        &mut expected,
        reference.compatibility_sequences.iter().copied(),
    );
    let compatibility_mappings = append_words(
        &mut expected,
        reference.compatibility_mappings.iter().flat_map(|mapping| {
            [
                mapping.codepoint,
                compatibility_sequences + mapping.sequence_index * 4,
                mapping.sequence_len,
                0,
            ]
        }),
    );
    let combining_classes = append_words(
        &mut expected,
        reference
            .combining_classes
            .iter()
            .flat_map(|&(point, class)| [point, u32::from(class)]),
    );
    let compositions = append_words(
        &mut expected,
        reference
            .compositions
            .iter()
            .flat_map(|&(first, second, composed)| [first, second, composed, 0]),
    );
    pool.append_normalization_tables();
    assert_eq!(
        pool.bytes, expected,
        "normalization pool bytes at prefix {prefix}"
    );
    assert_eq!(
        [
            pool.canonical_decomposition_table_ptr,
            pool.canonical_decomposition_count,
            pool.compatibility_decomposition_table_ptr,
            pool.compatibility_decomposition_count,
            pool.combining_class_table_ptr,
            pool.combining_class_count,
            pool.composition_table_ptr,
            pool.composition_count
        ],
        [
            canonical_mappings,
            reference.canonical_mappings.len() as u32,
            compatibility_mappings,
            reference.compatibility_mappings.len() as u32,
            combining_classes,
            reference.combining_classes.len() as u32,
            compositions,
            reference.compositions.len() as u32
        ],
        "normalization pool addresses and counts at prefix {prefix}",
    );
}

#[test]
fn normalization_image_rejects_invalid_extent_and_rows() {
    let reference = NormalizationTables {
        canonical_mappings: vec![NormalizationMapping {
            codepoint: 0xc5,
            sequence_index: 0,
            sequence_len: 2,
        }],
        canonical_sequences: vec![0x41, 0x30a],
        compatibility_mappings: vec![NormalizationMapping {
            codepoint: 0xfb03,
            sequence_index: 0,
            sequence_len: 3,
        }],
        compatibility_sequences: vec![0x66, 0x66, 0x69],
        combining_classes: vec![(0x30a, 230)],
        compositions: vec![(0x41, 0x30a, 0xc5)],
    };
    let image = reference.to_image();
    assert_eq!(NormalizationTables::from_image(&image), Some(reference));
    for cut in 0..image.len() {
        assert!(
            NormalizationTables::from_image(&image[..cut]).is_none(),
            "truncation {cut}"
        );
    }
    let mut extra = image.clone();
    extra.push(0);
    assert!(NormalizationTables::from_image(&extra).is_none());
    for (offset, value) in [
        (0, 0),         // format version/magic
        (8, u32::MAX),  // counted rows cannot exceed the admitted extent
        (32, 0xd800),   // mapping keys are scalar values
        (36, 1),        // the first sequence starts at index zero
        (40, 0),        // every retained mapping has a nonempty sequence
        (40, 3),        // mappings cannot claim words beyond their sequence image
        (44, 0x110000), // sequence payloads are scalar values
        (56, 1),        // compatibility offsets use their own sequence image
        (80, 0),        // zero combining classes are absent from this sparse table
        (80, 256),      // combining classes fit their actual u8 domain
        (84, 0xd800),   // composition inputs are scalar values
    ] {
        let mut invalid = image.clone();
        invalid[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(
            NormalizationTables::from_image(&invalid).is_none(),
            "word at {offset}: {value}"
        );
    }
    // These images have complete extents and valid scalar domains, but the
    // sparse lookup tables must still reject repeated keys and compositions.
    let mut repeated_class = image.clone();
    repeated_class.splice(84..84, image[76..84].iter().copied());
    repeated_class[24..28].copy_from_slice(&2_u32.to_le_bytes());
    assert!(NormalizationTables::from_image(&repeated_class).is_none());
    let mut repeated_composition = image.clone();
    repeated_composition.extend_from_slice(&image[84..96]);
    repeated_composition[28..32].copy_from_slice(&2_u32.to_le_bytes());
    assert!(NormalizationTables::from_image(&repeated_composition).is_none());
    repeated_composition[104..108].copy_from_slice(&0x212b_u32.to_le_bytes());
    assert!(NormalizationTables::from_image(&repeated_composition).is_none());
}
