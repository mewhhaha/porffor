use super::*;

fn reference(lowercase: bool) -> Vec<u8> {
    let mut bytes = Vec::new();
    for point in (0..=char::MAX as u32).filter_map(char::from_u32) {
        let mapped: String = if lowercase {
            point.to_lowercase().collect()
        } else {
            point.to_uppercase().collect()
        };
        if mapped == point.to_string() {
            continue;
        }
        assert!(mapped.len() <= if lowercase { 4 } else { 8 });
        let mut row = [0; 16];
        row[..4].copy_from_slice(&(point as u32).to_le_bytes());
        row[4..8].copy_from_slice(&(mapped.len() as u32).to_le_bytes());
        row[8..8 + mapped.len()].copy_from_slice(mapped.as_bytes());
        bytes.extend_from_slice(&row);
    }
    bytes
}

fn reference_ranges<P: icu_properties::props::BinaryProperty>() -> Vec<u8> {
    let mut bytes = Vec::new();
    for range in CodePointSetData::new::<P>().iter_ranges() {
        bytes.extend_from_slice(&range.start().to_le_bytes());
        bytes.extend_from_slice(&range.end().to_le_bytes());
    }
    bytes
}

fn append_aligned(bytes: &mut Vec<u8>, image: &[u8]) -> u32 {
    bytes.resize(bytes.len().next_multiple_of(8), 0);
    let address = STATIC_DATA_OFFSET + bytes.len() as u32;
    bytes.extend_from_slice(image);
    address
}

#[test]
fn built_case_images_match_every_target_unicode_mapping_and_pool_alignment() {
    let lower = reference(true);
    let upper = reference(false);
    assert_eq!(LOWERCASE.bytes(), lower);
    assert_eq!(UPPERCASE.bytes(), upper);
    assert_eq!(LOWERCASE.count() as usize, lower.len() / 16);
    assert_eq!(UPPERCASE.count() as usize, upper.len() / 16);
    let cased = reference_ranges::<props::Cased>();
    let ignorable = reference_ranges::<props::CaseIgnorable>();
    for prefix in 0..8 {
        let mut expected = vec![0xA5; prefix];
        let mut pool = StringPool {
            bytes: expected.clone(),
            ..StringPool::default()
        };
        let lower_address = append_aligned(&mut expected, &lower);
        let cased_address = append_aligned(&mut expected, &cased);
        let ignorable_address = append_aligned(&mut expected, &ignorable);
        let upper_address = append_aligned(&mut expected, &upper);
        pool.append_lowercase_tables();
        pool.append_uppercase_tables();
        assert_eq!(pool.bytes, expected, "pool bytes at prefix {prefix}");
        assert_eq!(pool.lowercase_mapping_table_ptr, lower_address);
        assert_eq!(pool.lowercase_mapping_count, (lower.len() / 16) as u32);
        assert_eq!(pool.cased_range_table_ptr, cased_address);
        assert_eq!(pool.cased_range_count, (cased.len() / 8) as u32);
        assert_eq!(pool.case_ignorable_range_table_ptr, ignorable_address);
        assert_eq!(
            pool.case_ignorable_range_count,
            (ignorable.len() / 8) as u32
        );
        assert_eq!(pool.uppercase_mapping_table_ptr, upper_address);
        assert_eq!(pool.uppercase_mapping_count, (upper.len() / 16) as u32);
    }
}
