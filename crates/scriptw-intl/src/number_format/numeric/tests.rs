use core::cmp::Ordering;
use core::num::{NonZeroU32, NonZeroU64};

use super::super::options::*;
use super::*;

mod vectors;

#[test]
fn configuration_domains_validate_bounds_and_active_style_fields() {
    assert!(IntegerDigitCount::new(0).is_err());
    assert!(IntegerDigitCount::new(22).is_err());
    assert!(FractionDigitCount::new(101).is_err());
    assert!(SignificantDigitCount::new(0).is_err());
    assert!(SignificantDigitCount::new(22).is_err());
    assert!(FractionDigitRange::new(
        FractionDigitCount::new(2).unwrap(),
        FractionDigitCount::new(1).unwrap()
    )
    .is_err());
    assert!(SignificantDigitRange::new(
        SignificantDigitCount::new(2).unwrap(),
        SignificantDigitCount::new(1).unwrap()
    )
    .is_err());
    assert_eq!(CurrencyCode::parse("zzz").unwrap().ascii(), *b"ZZZ");
    for code in ["US", "USDD", "12X", "\u{212a}RW"] {
        assert!(CurrencyCode::parse(code).is_err());
    }
    assert_eq!(
        UnitIdentifier::parse("meter-per-second").unwrap(),
        UnitIdentifier::Per {
            numerator: SingleUnit::Meter,
            denominator: SingleUnit::Second
        }
    );
    for unit in ["Meter", "unknown", "meter-per-second-per-hour", "per-meter"] {
        assert!(UnitIdentifier::parse(unit).is_err());
    }
}

fn exact(sign: NumberSign, coefficient: &str, exponent: i64) -> ExactDecimal {
    ExactDecimal::new(
        sign,
        DecimalCoefficient::from_digits(
            coefficient
                .bytes()
                .map(|byte| byte - b'0')
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .unwrap(),
        exponent,
    )
}

fn string_input(source: &str) -> IntlMathematicalValue {
    normalize_numeric_input(
        ObservedNumericInput::StringNumericLiteral(source.encode_utf16().collect()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap()
}

fn bigint_input(source: &str) -> IntlMathematicalValue {
    normalize_numeric_input(
        ObservedNumericInput::BigIntDecimal(source.into()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap()
}

fn fraction(minimum: u8, maximum: u8) -> Precision {
    Precision::Fraction(FractionPrecision::Range(
        FractionDigitRange::new(
            FractionDigitCount::new(minimum).unwrap(),
            FractionDigitCount::new(maximum).unwrap(),
        )
        .unwrap(),
    ))
}

fn significant(minimum: u8, maximum: u8) -> Precision {
    Precision::Significant(
        SignificantDigitRange::new(
            SignificantDigitCount::new(minimum).unwrap(),
            SignificantDigitCount::new(maximum).unwrap(),
        )
        .unwrap(),
    )
}

fn settings(precision: Precision, mode: RoundingMode) -> RoundingSettings {
    RoundingSettings {
        precision,
        mode,
        minimum_integer_digits: IntegerDigitCount::new(1).unwrap(),
        trailing_zero_display: TrailingZeroDisplay::Auto,
    }
}

fn rounded(source: &str, precision: Precision, mode: RoundingMode) -> RoundedDecimal {
    let value = string_input(source);
    round_decimal(
        value.finite_value().unwrap(),
        &settings(precision, mode),
        &NumericLimits::HOST_ABI,
    )
    .unwrap()
}

fn text(rounded: &RoundedDecimal) -> String {
    let mut text = String::new();
    if rounded.sign() == NumberSign::Negative {
        text.push('-');
    }
    text.extend(
        rounded
            .integer_digits()
            .iter()
            .map(|digit| char::from(b'0' + digit)),
    );
    if !rounded.fraction_digits().is_empty() {
        text.push('.');
        text.extend(
            rounded
                .fraction_digits()
                .iter()
                .map(|digit| char::from(b'0' + digit)),
        );
    }
    text
}

#[test]
fn all_rounding_modes_preserve_direction_and_signed_ties() {
    let cases = [
        (RoundingMode::Ceil, "1.3", "-1.2"),
        (RoundingMode::Floor, "1.2", "-1.3"),
        (RoundingMode::Expand, "1.3", "-1.3"),
        (RoundingMode::Trunc, "1.2", "-1.2"),
        (RoundingMode::HalfCeil, "1.3", "-1.2"),
        (RoundingMode::HalfFloor, "1.2", "-1.3"),
        (RoundingMode::HalfExpand, "1.3", "-1.3"),
        (RoundingMode::HalfTrunc, "1.2", "-1.2"),
        (RoundingMode::HalfEven, "1.2", "-1.2"),
    ];
    for (mode, positive, negative) in cases {
        assert_eq!(
            text(&rounded("1.25", fraction(1, 1), mode)),
            positive,
            "{mode:?}"
        );
        assert_eq!(
            text(&rounded("-1.25", fraction(1, 1), mode)),
            negative,
            "{mode:?}"
        );
    }
    assert_eq!(
        text(&rounded("1.35", fraction(1, 1), RoundingMode::HalfEven)),
        "1.4"
    );
    assert_eq!(
        text(&rounded("-1.35", fraction(1, 1), RoundingMode::HalfEven)),
        "-1.4"
    );
    for mode in [
        RoundingMode::HalfCeil,
        RoundingMode::HalfFloor,
        RoundingMode::HalfExpand,
        RoundingMode::HalfTrunc,
        RoundingMode::HalfEven,
    ] {
        assert_eq!(
            text(&rounded("1.2499999999999999", fraction(1, 1), mode)),
            "1.2"
        );
        assert_eq!(
            text(&rounded("1.2500000000000001", fraction(1, 1), mode)),
            "1.3"
        );
    }
}

fn two_fraction_digits(value: u64) -> String {
    format!("{}.{:02}", value / 100, value % 100)
}

#[test]
fn every_admitted_increment_rounds_on_its_own_grid() {
    let increments = [
        None,
        Some(NonUnitRoundingIncrement::Two),
        Some(NonUnitRoundingIncrement::Five),
        Some(NonUnitRoundingIncrement::Ten),
        Some(NonUnitRoundingIncrement::Twenty),
        Some(NonUnitRoundingIncrement::TwentyFive),
        Some(NonUnitRoundingIncrement::Fifty),
        Some(NonUnitRoundingIncrement::Hundred),
        Some(NonUnitRoundingIncrement::TwoHundred),
        Some(NonUnitRoundingIncrement::TwoHundredFifty),
        Some(NonUnitRoundingIncrement::FiveHundred),
        Some(NonUnitRoundingIncrement::Thousand),
        Some(NonUnitRoundingIncrement::TwoThousand),
        Some(NonUnitRoundingIncrement::TwoThousandFiveHundred),
        Some(NonUnitRoundingIncrement::FiveThousand),
    ];
    for increment in increments {
        let value = u64::from(increment.map_or(1, NonUnitRoundingIncrement::value));
        let precision = match increment {
            Some(increment) => Precision::Fraction(FractionPrecision::Increment {
                digits: FractionDigitCount::new(2).unwrap(),
                increment,
            }),
            None => fraction(2, 2),
        };
        let even_tie = format!("{}e-3", value * 25);
        let odd_tie = format!("{}e-3", value * 35);
        assert_eq!(
            text(&rounded(&even_tie, precision, RoundingMode::HalfEven)),
            two_fraction_digits(value * 2)
        );
        assert_eq!(
            text(&rounded(&odd_tie, precision, RoundingMode::HalfEven)),
            two_fraction_digits(value * 4)
        );
        assert_eq!(
            text(&rounded(&even_tie, precision, RoundingMode::HalfExpand)),
            two_fraction_digits(value * 3)
        );
        assert_eq!(
            text(&rounded(
                &format!("-{}", even_tie),
                precision,
                RoundingMode::HalfEven
            )),
            format!("-{}", two_fraction_digits(value * 2))
        );
        assert_eq!(
            text(&rounded(
                &format!("{}e-3", value * 25 + 1),
                precision,
                RoundingMode::HalfEven
            )),
            two_fraction_digits(value * 3)
        );
        assert_eq!(
            text(&rounded(
                &format!("{}e-3", value * 25 - 1),
                precision,
                RoundingMode::HalfEven
            )),
            two_fraction_digits(value * 2)
        );
    }
}

#[test]
fn significant_precision_carries_update_the_rounding_magnitude() {
    for (source, expected, magnitude) in [
        ("9.99", "10", 0),
        ("99.9", "100", 1),
        ("0.00999", "0.01", -3),
        ("0.0999", "0.1", -2),
    ] {
        let result = rounded(source, significant(1, 2), RoundingMode::HalfExpand);
        assert_eq!(text(&result), expected);
        assert_eq!(result.rounding_magnitude(), magnitude);
    }
    for (source, expected) in [
        ("0", "0.00"),
        ("1", "1.00"),
        ("10", "10.0"),
        ("100", "100"),
        ("0.001", "0.00100"),
    ] {
        assert_eq!(
            text(&rounded(
                source,
                significant(3, 3),
                RoundingMode::HalfExpand
            )),
            expected
        );
    }
}

#[test]
fn precision_priority_uses_final_magnitudes_and_specified_tie_choice() {
    let fractional = FractionDigitRange::new(
        FractionDigitCount::new(2).unwrap(),
        FractionDigitCount::new(2).unwrap(),
    )
    .unwrap();
    let significant = SignificantDigitRange::new(
        SignificantDigitCount::new(1).unwrap(),
        SignificantDigitCount::new(3).unwrap(),
    )
    .unwrap();
    assert_eq!(
        text(&rounded(
            "1",
            Precision::More {
                fraction: fractional,
                significant
            },
            RoundingMode::HalfExpand
        )),
        "1"
    );
    assert_eq!(
        text(&rounded(
            "1",
            Precision::Less {
                fraction: fractional,
                significant
            },
            RoundingMode::HalfExpand
        )),
        "1.00"
    );
    let fractional = FractionDigitRange::new(
        FractionDigitCount::new(1).unwrap(),
        FractionDigitCount::new(1).unwrap(),
    )
    .unwrap();
    let significant = SignificantDigitRange::new(
        SignificantDigitCount::new(1).unwrap(),
        SignificantDigitCount::new(2).unwrap(),
    )
    .unwrap();
    assert_eq!(
        text(&rounded(
            "9.99",
            Precision::More {
                fraction: fractional,
                significant
            },
            RoundingMode::HalfExpand
        )),
        "10.0"
    );
    assert_eq!(
        text(&rounded(
            "9.99",
            Precision::Less {
                fraction: fractional,
                significant
            },
            RoundingMode::HalfExpand
        )),
        "10"
    );
}

#[test]
fn trailing_zero_policy_integer_padding_and_negative_zero_are_independent() {
    let mut controls = settings(fraction(3, 3), RoundingMode::HalfExpand);
    controls.minimum_integer_digits = IntegerDigitCount::new(4).unwrap();
    controls.trailing_zero_display = TrailingZeroDisplay::StripIfInteger;
    for (source, expected) in [
        ("1", "0001"),
        ("1.5", "0001.500"),
        ("-0", "-0000"),
        ("-0.0001", "-0000"),
    ] {
        let value = string_input(source);
        let result = round_decimal(
            value.finite_value().unwrap(),
            &controls,
            &NumericLimits::HOST_ABI,
        )
        .unwrap();
        assert_eq!(text(&result), expected);
    }
    let zero = rounded("-0.49", fraction(0, 0), RoundingMode::HalfExpand);
    assert_eq!(
        zero.rounded_value(),
        FiniteValue::Zero(NumberSign::Negative)
    );
    assert_eq!(
        text(&rounded("-0.01", fraction(0, 0), RoundingMode::Floor)),
        "-1"
    );
    assert_eq!(
        text(&rounded("-0.01", fraction(0, 0), RoundingMode::Ceil)),
        "-0"
    );
}

#[test]
fn one_hundred_fraction_digits_and_twenty_one_significant_digits_are_exact() {
    assert_eq!(
        text(&rounded(
            "1e-100",
            fraction(100, 100),
            RoundingMode::HalfExpand
        )),
        format!("0.{}1", "0".repeat(99))
    );
    assert_eq!(
        text(&rounded(
            "1e-101",
            fraction(0, 100),
            RoundingMode::HalfExpand
        )),
        "0"
    );
    let value = bigint_input(&format!("1{}", "2".repeat(999)));
    let result = round_decimal(
        value.finite_value().unwrap(),
        &settings(significant(21, 21), RoundingMode::HalfExpand),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(
        text(&result),
        format!("1{}{}", "2".repeat(20), "0".repeat(979))
    );
    assert_eq!(result.rounding_magnitude(), 979);
    assert_eq!(
        text(&rounded(
            "1.0000000000000001",
            fraction(16, 16),
            RoundingMode::HalfExpand
        )),
        "1.0000000000000001"
    );
}

fn notation_result(
    source: &str,
    notation: NotationScaling<'_>,
    precision: Precision,
) -> RoundedDecimal {
    let value = string_input(source);
    let controls = DecimalFormatSettings {
        rounding: settings(precision, RoundingMode::HalfExpand),
        scale: DecimalScale::Unit,
        notation,
    };
    format_decimal(
        value.finite_value().unwrap(),
        &controls,
        &NumericLimits::HOST_ABI,
    )
    .unwrap()
}

#[test]
fn scientific_and_engineering_scale_and_reselect_after_carry() {
    for (source, notation, expected, exponent) in [
        ("1234", NotationScaling::Scientific, "1.23", 3),
        ("-1234", NotationScaling::Scientific, "-1.23", 3),
        ("9.999", NotationScaling::Scientific, "1.00", 1),
        ("10000", NotationScaling::Engineering, "10.00", 3),
        ("0.001234", NotationScaling::Engineering, "1.23", -3),
        ("0.0001234", NotationScaling::Engineering, "123.40", -6),
        ("999.999", NotationScaling::Engineering, "1.00", 3),
        ("-0", NotationScaling::Scientific, "-0.00", 0),
    ] {
        let result = notation_result(source, notation, fraction(2, 2));
        assert_eq!(text(&result), expected);
        assert_eq!(result.notation().exponent(), exponent);
    }
}

#[test]
fn percent_scaling_precedes_notation_and_preserves_exactness() {
    let controls = DecimalFormatSettings {
        rounding: settings(fraction(2, 2), RoundingMode::HalfExpand),
        scale: DecimalScale::Percent,
        notation: NotationScaling::Standard,
    };
    let value = string_input("0.0123");
    assert_eq!(
        text(
            &format_decimal(
                value.finite_value().unwrap(),
                &controls,
                &NumericLimits::HOST_ABI
            )
            .unwrap()
        ),
        "1.23"
    );
    let controls = DecimalFormatSettings {
        notation: NotationScaling::Scientific,
        ..controls
    };
    let value = string_input("1234");
    let result = format_decimal(
        value.finite_value().unwrap(),
        &controls,
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(text(&result), "1.23");
    assert_eq!(result.notation().exponent(), 5);
}

#[test]
fn exponent_selection_uses_the_positive_probe_before_signed_rounding() {
    let value = string_input("-9.91");
    let controls = DecimalFormatSettings {
        rounding: settings(fraction(0, 0), RoundingMode::Ceil),
        scale: DecimalScale::Unit,
        notation: NotationScaling::Scientific,
    };
    let result = format_decimal(
        value.finite_value().unwrap(),
        &controls,
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(result.notation().exponent(), 1);
    assert_eq!(
        result.rounded_value(),
        FiniteValue::Zero(NumberSign::Negative)
    );
}

fn compact_rows(rows: &[(u32, u32)]) -> CompactExponentTable {
    CompactExponentTable::new(
        rows.iter()
            .map(|(magnitude, exponent)| CompactExponentRow::new(*magnitude, *exponent).unwrap())
            .collect(),
    )
    .unwrap()
}

#[test]
fn compact_selection_retains_the_pattern_key_through_carry_and_extent_fallback() {
    let table = compact_rows(&[(3, 3), (4, 3), (5, 3), (6, 6), (7, 6), (8, 6), (9, 9)]);
    for (source, expected, exponent, pattern) in [
        ("12", "12", 0, None),
        ("999.95", "1", 3, Some(3)),
        ("9999.95", "10", 3, Some(4)),
        ("999999.95", "1", 6, Some(6)),
        ("1e20", "100000000000", 9, Some(9)),
        ("-0", "-0", 0, None),
    ] {
        let result = notation_result(source, NotationScaling::Compact(&table), fraction(0, 1));
        assert_eq!(text(&result), expected);
        assert_eq!(result.notation().exponent(), exponent);
        assert_eq!(result.notation().compact_pattern_magnitude(), pattern);
    }
    let empty = CompactExponentTable::empty();
    let result = notation_result("1200", NotationScaling::Compact(&empty), fraction(0, 1));
    assert_eq!(text(&result), "1200");
    assert_eq!(result.notation().compact_pattern_magnitude(), None);
}

#[test]
fn compact_table_rejects_invalid_rows_and_preserves_nonmonotone_valid_exponents() {
    assert_eq!(
        CompactExponentRow::new(3, 4),
        Err(InvalidCompactExponentTable::ExponentExceedsMagnitude)
    );
    let row = CompactExponentRow::new(3, 3).unwrap();
    assert_eq!(
        CompactExponentTable::new(Box::new([row, row])),
        Err(InvalidCompactExponentTable::NonIncreasingMagnitude)
    );
    assert_eq!(compact_rows(&[(3, 3), (4, 0)]).rows().len(), 2);
}

#[test]
fn compact_default_precision_and_options_mapping_share_the_numeric_core() {
    let table = compact_rows(&[(3, 3), (4, 3), (5, 3), (6, 6)]);
    let options = NumberFormatOptions {
        style: NumberStyle::Decimal,
        notation: Notation::Compact(CompactDisplay::Short),
        minimum_integer_digits: IntegerDigitCount::new(1).unwrap(),
        precision: Precision::More {
            fraction: FractionDigitRange::new(
                FractionDigitCount::new(0).unwrap(),
                FractionDigitCount::new(0).unwrap(),
            )
            .unwrap(),
            significant: SignificantDigitRange::new(
                SignificantDigitCount::new(1).unwrap(),
                SignificantDigitCount::new(2).unwrap(),
            )
            .unwrap(),
        },
        rounding_mode: RoundingMode::HalfExpand,
        trailing_zero_display: TrailingZeroDisplay::Auto,
        grouping: Grouping::MinTwo,
        sign_display: SignDisplay::Auto,
    };
    let controls = DecimalFormatSettings::from_options(&options, &table);
    for (source, expected, exponent) in [("123", "123", 0), ("12345", "12", 3), ("999.9", "1", 3)] {
        let value = string_input(source);
        let rounded = format_decimal(
            value.finite_value().unwrap(),
            &controls,
            &NumericLimits::HOST_ABI,
        )
        .unwrap();
        assert_eq!(text(&rounded), expected);
        assert_eq!(rounded.notation().exponent(), exponent);
    }
    assert_eq!(
        options.precision.computed_rounding_priority(),
        RoundingPriority::MorePrecision
    );
}

fn assert_plural(rounded: &RoundedDecimal, values: &[(PluralOperand, u64)]) {
    for (operand, expected) in values {
        let value = rounded.plural_operands().operand(*operand);
        assert!(value.is_integer(), "{operand:?}");
        assert_eq!(
            value.compare_integer(*expected),
            Ordering::Equal,
            "{operand:?}"
        );
    }
}

#[test]
fn plural_operands_keep_visible_zeroes_and_exact_fractional_remainders() {
    let result = rounded("1.23", fraction(4, 4), RoundingMode::HalfExpand);
    assert_plural(
        &result,
        &[
            (PluralOperand::I, 1),
            (PluralOperand::V, 4),
            (PluralOperand::W, 2),
            (PluralOperand::F, 2300),
            (PluralOperand::T, 23),
            (PluralOperand::C, 0),
            (PluralOperand::E, 0),
        ],
    );
    let number = result.plural_operands().operand(PluralOperand::N);
    assert!(!number.is_integer());
    assert_eq!(number.compare_integer(1), Ordering::Greater);
    assert_eq!(number.compare_integer(2), Ordering::Less);
    let remainder = number.modulo(NonZeroU64::new(1).unwrap());
    assert!(!remainder.is_integer());
    assert_eq!(remainder.compare_integer(0), Ordering::Greater);
    assert_eq!(remainder.compare_integer(1), Ordering::Less);
    let result = rounded("1.03", fraction(3, 3), RoundingMode::HalfExpand);
    assert_plural(
        &result,
        &[
            (PluralOperand::V, 3),
            (PluralOperand::W, 2),
            (PluralOperand::F, 30),
            (PluralOperand::T, 3),
        ],
    );
}

#[test]
fn compact_plural_operands_expand_the_visible_mantissa_by_the_compact_exponent() {
    let table = compact_rows(&[(3, 3)]);
    let result = notation_result("1200.5", NotationScaling::Compact(&table), fraction(5, 5));
    assert_eq!(text(&result), "1.20050");
    assert_plural(
        &result,
        &[
            (PluralOperand::I, 1200),
            (PluralOperand::V, 2),
            (PluralOperand::W, 1),
            (PluralOperand::F, 50),
            (PluralOperand::T, 5),
            (PluralOperand::C, 3),
            (PluralOperand::E, 3),
        ],
    );
    let number = result.plural_operands().operand(PluralOperand::N);
    assert_eq!(number.compare_integer(1200), Ordering::Greater);
    assert_eq!(number.compare_integer(1201), Ordering::Less);
    let scientific = notation_result("1200.5", NotationScaling::Scientific, fraction(5, 5));
    assert_plural(&scientific, &[(PluralOperand::C, 0), (PluralOperand::E, 0)]);
}

#[test]
fn plural_integer_padding_and_zero_do_not_invent_numeric_magnitude() {
    let value = string_input("1");
    let mut controls = settings(fraction(2, 2), RoundingMode::HalfEven);
    controls.minimum_integer_digits = IntegerDigitCount::new(21).unwrap();
    let result = round_decimal(
        value.finite_value().unwrap(),
        &controls,
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_plural(
        &result,
        &[
            (PluralOperand::N, 1),
            (PluralOperand::I, 1),
            (PluralOperand::V, 2),
            (PluralOperand::W, 0),
            (PluralOperand::F, 0),
        ],
    );
    let zero = rounded("-0", fraction(3, 3), RoundingMode::HalfEven);
    assert_plural(
        &zero,
        &[
            (PluralOperand::N, 0),
            (PluralOperand::I, 0),
            (PluralOperand::V, 3),
            (PluralOperand::W, 0),
            (PluralOperand::F, 0),
        ],
    );
}

#[test]
fn explicit_resource_limits_do_not_turn_values_into_nan_or_zero() {
    let limit = NumericLimits::new(NonZeroU32::new(5).unwrap());
    let result =
        normalize_numeric_input(ObservedNumericInput::BigIntDecimal("123456".into()), &limit);
    assert!(matches!(
        result,
        Err(NumericNormalizationError::Resource(
            NumberFormatResourceError::DigitExtent {
                requested: 6,
                maximum: 5
            }
        ))
    ));
    assert_eq!(
        normalize_numeric_input(
            ObservedNumericInput::StringNumericLiteral(
                "1e99999999999999999999999999999".encode_utf16().collect()
            ),
            &limit
        )
        .unwrap(),
        IntlMathematicalValue::Infinity(NumberSign::Positive)
    );
    let value = bigint_input("100000");
    assert!(matches!(
        round_decimal(
            value.finite_value().unwrap(),
            &settings(fraction(0, 0), RoundingMode::HalfEven),
            &limit
        ),
        Err(NumberFormatResourceError::DigitExtent { .. })
    ));
}

#[test]
fn directly_constructed_extreme_exponents_are_checked_before_arithmetic() {
    let low = exact(NumberSign::Positive, "1", i64::MIN);
    let zero = round_decimal(
        FiniteValue::Nonzero(&low),
        &settings(fraction(0, 0), RoundingMode::HalfEven),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(
        zero.rounded_value(),
        FiniteValue::Zero(NumberSign::Positive)
    );
    let controls = DecimalFormatSettings {
        rounding: settings(fraction(2, 2), RoundingMode::HalfExpand),
        scale: DecimalScale::Unit,
        notation: NotationScaling::Scientific,
    };
    let result = format_decimal(
        FiniteValue::Nonzero(&low),
        &controls,
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(text(&result), "1.00");
    assert_eq!(result.notation().exponent(), i64::MIN);
    let engineering = DecimalFormatSettings {
        notation: NotationScaling::Engineering,
        ..controls
    };
    assert_eq!(
        format_decimal(
            FiniteValue::Nonzero(&low),
            &engineering,
            &NumericLimits::HOST_ABI
        ),
        Err(NumberFormatResourceError::ExponentOverflow)
    );
    assert_eq!(
        round_decimal(
            FiniteValue::Nonzero(&low),
            &settings(significant(21, 21), RoundingMode::HalfEven),
            &NumericLimits::HOST_ABI
        ),
        Err(NumberFormatResourceError::ExponentOverflow)
    );
    let high = exact(NumberSign::Positive, "11", i64::MAX);
    assert_eq!(
        format_decimal(
            FiniteValue::Nonzero(&high),
            &controls,
            &NumericLimits::HOST_ABI
        ),
        Err(NumberFormatResourceError::ExponentOverflow)
    );
    let high = exact(NumberSign::Positive, "1", i64::MAX);
    let percent = DecimalFormatSettings {
        scale: DecimalScale::Percent,
        ..controls
    };
    assert_eq!(
        format_decimal(
            FiniteValue::Nonzero(&high),
            &percent,
            &NumericLimits::HOST_ABI
        ),
        Err(NumberFormatResourceError::ExponentOverflow)
    );
    let carry = exact(NumberSign::Positive, "9999", i64::MAX - 3);
    assert_eq!(
        format_decimal(
            FiniteValue::Nonzero(&carry),
            &controls,
            &NumericLimits::HOST_ABI
        ),
        Err(NumberFormatResourceError::ExponentOverflow)
    );
}

#[test]
fn numeric_grammar_covers_whitespace_decimal_radix_and_invalid_tails() {
    for source in ["", "\t\n\r ", "0", "00", "0e999999999999999999999999999999"] {
        assert_eq!(
            string_input(source),
            IntlMathematicalValue::Zero(NumberSign::Positive)
        );
    }
    for source in [".5", "0.5", "+00.500", "5e-1", "50.e-2"] {
        assert_eq!(
            string_input(source),
            IntlMathematicalValue::Finite(exact(NumberSign::Positive, "5", -1))
        );
    }
    for source in ["0b10000", "0B10000", "0o20", "0O20", "0x10", "0X10"] {
        assert_eq!(
            string_input(source),
            IntlMathematicalValue::Finite(exact(NumberSign::Positive, "16", 0))
        );
    }
    for unit in [
        0x0009, 0x000a, 0x000b, 0x000c, 0x000d, 0x0020, 0x00a0, 0x1680, 0x2000, 0x200a, 0x2028,
        0x2029, 0x202f, 0x205f, 0x3000, 0xfeff,
    ] {
        let source = vec![unit, u16::from(b'1'), unit].into_boxed_slice();
        assert_eq!(
            normalize_numeric_input(
                ObservedNumericInput::StringNumericLiteral(source),
                &NumericLimits::HOST_ABI
            )
            .unwrap(),
            IntlMathematicalValue::Finite(exact(NumberSign::Positive, "1", 0))
        );
    }
    for source in [
        "+0x10",
        "-0b1",
        "+0o7",
        "0x",
        "0b2",
        "1e",
        "+",
        ".",
        "1e+",
        "Infinityx",
        "1e999999x",
        "\u{85}",
        "\u{180e}",
        "\u{200b}",
        "\u{ffff}",
    ] {
        assert_eq!(
            string_input(source),
            IntlMathematicalValue::NaN,
            "{source:?}"
        );
    }
    assert_eq!(
        string_input(&format!("0x{}", "f".repeat(300))),
        IntlMathematicalValue::Infinity(NumberSign::Positive)
    );
    assert_eq!(
        string_input(&format!("0x{}z", "f".repeat(300))),
        IntlMathematicalValue::NaN
    );
    assert_eq!(
        string_input("-1e-999999999999999999999999999"),
        IntlMathematicalValue::Zero(NumberSign::Negative)
    );
}

#[test]
fn malformed_trusted_wire_is_distinct_from_user_numeric_syntax() {
    for spelling in ["", "+1", "00", "-0", "1.0", "0x10", " 1", "2e3", "1n"] {
        assert_eq!(
            normalize_numeric_input(
                ObservedNumericInput::BigIntDecimal(spelling.into()),
                &NumericLimits::HOST_ABI
            ),
            Err(NumericNormalizationError::InvalidWire(
                NumericWireError::BigIntSpelling
            ))
        );
    }
    for spelling in [
        "",
        "+1",
        "00",
        "-0",
        "1.0",
        "0x10",
        " 1",
        "1.",
        "1.e+21",
        "1E+21",
        "1e21",
        "1e+01",
        "1e+20",
        "0.0000001",
        "1e400",
        "foo",
    ] {
        assert_eq!(
            normalize_numeric_input(
                ObservedNumericInput::NumberShortestDecimal(spelling.into()),
                &NumericLimits::HOST_ABI
            ),
            Err(NumericNormalizationError::InvalidWire(
                NumericWireError::NumberSpelling
            )),
            "{spelling:?}"
        );
    }
    for spelling in [
        "NaN",
        "Infinity",
        "-Infinity",
        "0",
        "1e+21",
        "1e-7",
        "0.000001",
        "1.7976931348623157e+308",
        "5e-324",
        "1000000000000000100",
    ] {
        assert!(
            normalize_numeric_input(
                ObservedNumericInput::NumberShortestDecimal(spelling.into()),
                &NumericLimits::HOST_ABI
            )
            .is_ok(),
            "{spelling}"
        );
    }
}

#[test]
fn range_domain_preserves_descending_values_and_both_zero_signs() {
    assert!(NumberRange::new(string_input("2"), string_input("1")).is_ok());
    let range = NumberRange::new(string_input("-0"), string_input("0")).unwrap();
    assert_eq!(
        range.start(),
        &NumberRangeEndpoint::Zero(NumberSign::Negative)
    );
    assert_eq!(
        range.end(),
        &NumberRangeEndpoint::Zero(NumberSign::Positive)
    );
    assert_eq!(
        NumberRange::new(IntlMathematicalValue::NaN, string_input("0")),
        Err(NaNRangeEndpoint)
    );
}
