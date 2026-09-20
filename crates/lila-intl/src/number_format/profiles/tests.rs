use super::*;

#[test]
fn profile_reader_rejects_truncation_and_unknown_header_before_publication() {
    let bytes = include_bytes!("../../../data/number-cldr-47/profiles.bin");
    for length in [0, 1, 7, 8, 11, bytes.len() - 1] {
        assert!(read::decode(&bytes[..length]).is_err(), "length {length}");
    }
    assert_eq!(
        read::decode(b"LNF48\0\x01\0").unwrap_err().reason,
        NumberProfileError::Version
    );
    assert_eq!(
        read::decode(b"LNF47\0\x01\0\xff\xff\xff\xff")
            .unwrap_err()
            .reason,
        NumberProfileError::Truncated
    );
}

#[test]
fn typed_profile_validation_rejects_cross_table_and_pattern_role_forgery() {
    let mut profiles =
        read::decode(include_bytes!("../../../data/number-cldr-47/profiles.bin")).unwrap();
    let profile = profiles.locale_profiles[0].0;
    let original = profiles.profiles[profile].default_numbering;
    profiles.profiles[profile].default_numbering = u8::MAX;
    assert_eq!(
        profiles.validate().unwrap_err().table,
        NumberProfileTable::Profiles
    );
    profiles.profiles[profile].default_numbering = original;
    let numbering = profiles.profiles[profile].numbering[0].0;
    let original = profiles.numbering_profiles[numbering].range_pattern;
    profiles.numbering_profiles[numbering].range_pattern =
        profiles.signed_patterns[profiles.numbering_profiles[numbering].decimal.0].positive;
    assert_eq!(
        profiles.validate().unwrap_err().reason,
        NumberProfileError::PatternRole
    );
    profiles.numbering_profiles[numbering].range_pattern = original;
    profiles.unicode_sets[profiles.letter_set.0].0[0] = (10, 9);
    assert_eq!(
        profiles.validate().unwrap_err().reason,
        NumberProfileError::InvalidRange
    );
}

#[test]
fn spacing_properties_use_unicode_categories_including_non_decimal_han_digits() {
    let profiles = embedded_number_profiles().unwrap();
    assert!(profiles.is_letter('A'));
    assert!(profiles.is_letter('一'));
    assert!(!profiles.is_digit('一'));
    assert!(profiles.is_digit('𝟏'));
    assert!(profiles.is_digit('١'));
    assert!(profiles.is_whitespace('\u{a0}'));
    assert!(profiles.is_whitespace('\u{202f}'));
    assert!(!profiles.is_whitespace('\u{200f}'));
}
