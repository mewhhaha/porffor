use crate::builtins::intl::number_format::RoundingIncrement;
use crate::builtins::intl::number_format::{
    CompactDisplay, compact_format_pattern, compact_format_pattern_for_display,
};
use fixed_decimal::RoundingIncrement::*;
use icu_locale::Locale;

#[test]
fn u16_to_rounding_increment_sunny_day() {
    #[rustfmt::skip]
    let valid_cases: [(u16, RoundingIncrement); 15] = [
        // Singles
        (1, RoundingIncrement::from_parts(MultiplesOf1, 0).unwrap()),
        (2, RoundingIncrement::from_parts(MultiplesOf2, 0).unwrap()),
        (5, RoundingIncrement::from_parts(MultiplesOf5, 0).unwrap()),
        // Tens
        (10, RoundingIncrement::from_parts(MultiplesOf1, 1).unwrap()),
        (20, RoundingIncrement::from_parts(MultiplesOf2, 1).unwrap()),
        (25, RoundingIncrement::from_parts(MultiplesOf25, 0).unwrap()),
        (50, RoundingIncrement::from_parts(MultiplesOf5, 1).unwrap()),
        // Hundreds
        (100, RoundingIncrement::from_parts(MultiplesOf1, 2).unwrap()),
        (200, RoundingIncrement::from_parts(MultiplesOf2, 2).unwrap()),
        (250, RoundingIncrement::from_parts(MultiplesOf25, 1).unwrap()),
        (500, RoundingIncrement::from_parts(MultiplesOf5, 2).unwrap()),
        // Thousands
        (1000, RoundingIncrement::from_parts(MultiplesOf1, 3).unwrap()),
        (2000, RoundingIncrement::from_parts(MultiplesOf2, 3).unwrap()),
        (2500, RoundingIncrement::from_parts(MultiplesOf25, 2).unwrap()),
        (5000, RoundingIncrement::from_parts(MultiplesOf5, 3).unwrap()),
    ];

    for (num, increment) in valid_cases {
        assert_eq!(RoundingIncrement::from_u16(num), Some(increment));
    }
}

#[test]
fn u16_to_rounding_increment_rainy_day() {
    const INVALID_CASES: [u16; 9] = [0, 4, 6, 24, 10000, 65535, 7373, 140, 1500];

    for num in INVALID_CASES {
        assert!(RoundingIncrement::from_u16(num).is_none());
    }
}

#[test]
fn de_compact_patterns_match_short_and_long_thresholds() {
    let locale: Locale = "de-DE".parse().expect("valid locale");

    assert_eq!(compact_format_pattern(&locale, 9_876.0), (1.0, ""));
    assert_eq!(
        compact_format_pattern(&locale, 98_765_432.0),
        (1_000_000.0, "\u{a0}Mio.")
    );

    assert_eq!(
        compact_format_pattern_for_display(&locale, 9_876.0, CompactDisplay::Long),
        (1_000.0, " Tausend")
    );
    assert_eq!(
        compact_format_pattern_for_display(&locale, 98_765_432.0, CompactDisplay::Long),
        (1_000_000.0, " Millionen")
    );
}
