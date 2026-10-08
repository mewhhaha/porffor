use super::*;

#[test]
fn profile_reader_rejects_truncation_and_unknown_header_before_publication() {
    let bytes = include_bytes!("../../../data/number-cldr-47/profiles.bin");
    let locale = crate::embedded_locale_data_image().unwrap();
    for length in [0, 1, 7, 8, 11, bytes.len() - 1] {
        assert!(
            read::decode(&bytes[..length], &locale).is_err(),
            "length {length}"
        );
    }
    assert_eq!(
        read::decode(b"LNF48\0\x01\0", &locale).unwrap_err().reason,
        NumberProfileError::Version
    );
    assert_eq!(
        read::decode(b"LNF47\0\x02\0\xff\xff\xff\xff", &locale)
            .unwrap_err()
            .reason,
        NumberProfileError::Truncated
    );
}

#[test]
fn typed_profile_validation_rejects_cross_table_and_pattern_role_forgery() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let mut profiles = read::decode(
        include_bytes!("../../../data/number-cldr-47/profiles.bin"),
        &locale,
    )
    .unwrap();
    let profile = profiles.locale_profiles[0].0;
    let original = profiles.profiles[profile].default_numbering;
    profiles.profiles[profile].default_numbering = u8::MAX;
    assert_eq!(
        profiles.validate().unwrap_err().table,
        NumberProfileTable::Profiles
    );
    profiles.profiles[profile].default_numbering = original;
    let numbering = profiles.profiles[profile].numbering[0].unwrap().0;
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

#[test]
fn plural_compact_policy_uses_default_numbering_and_distinct_short_long_exponents() {
    let profiles = embedded_number_profiles().unwrap();
    assert_eq!(profiles.profiles.len(), 535);
    assert_eq!(profiles.plural_rules.len(), 40);
    assert_eq!(profiles.ordinal_rules.len(), 26);
    assert_eq!(profiles.plural_ranges.len(), 10);
    for (locale, magnitude, short, long) in [
        ("de", 3, 0, 3),
        ("de", 4, 0, 3),
        ("ca", 9, 6, 9),
        ("es", 9, 6, 9),
        ("en-IN", 6, 5, 6),
    ] {
        let profile = profiles.profile(locale).unwrap();
        let numbering =
            profiles.numbering(profile.numbering[usize::from(profile.default_numbering)].unwrap());
        assert_eq!(
            profiles
                .compact(numbering.compact_short)
                .exponents
                .select(magnitude)
                .unwrap()
                .exponent(),
            short,
            "{locale} short"
        );
        assert_eq!(
            profiles
                .compact(numbering.compact_long)
                .exponents
                .select(magnitude)
                .unwrap()
                .exponent(),
            long,
            "{locale} long"
        );
    }
}
#[test]
fn every_admitted_plural_association_has_closed_categories_and_range_results() {
    use crate::plural_rules::{PluralCategory, PluralType};
    let profiles = embedded_number_profiles().unwrap();
    for locale in profiles.available_locales() {
        for kind in PluralType::ALL {
            let categories = profiles.plural_categories(locale, kind).unwrap();
            assert!(categories.contains(PluralCategory::Other));
            for start in PluralCategory::ALL {
                for end in PluralCategory::ALL {
                    if categories.contains(start) && categories.contains(end) {
                        assert!(
                            categories.contains(
                                profiles
                                    .plural_range_category(locale, kind, start, end)
                                    .unwrap()
                            ),
                            "{locale} {kind:?} {start:?} {end:?}"
                        );
                    }
                }
            }
        }
    }
}
