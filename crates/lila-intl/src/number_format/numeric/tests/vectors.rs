// Generated from the frozen 23 exact normalization vectors.
use super::*;

#[test]
fn normalization_vector_01() {
    let actual = normalize_numeric_input(
        ObservedNumericInput::StringNumericLiteral("1.0000000000000001".encode_utf16().collect()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(
        actual,
        IntlMathematicalValue::Finite(exact(NumberSign::Positive, "10000000000000001", -16))
    );
}

#[test]
fn normalization_vector_02() {
    let actual = normalize_numeric_input(
        ObservedNumericInput::StringNumericLiteral("1.500".encode_utf16().collect()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(
        actual,
        IntlMathematicalValue::Finite(exact(NumberSign::Positive, "15", -1))
    );
}

#[test]
fn normalization_vector_03() {
    let actual = normalize_numeric_input(
        ObservedNumericInput::StringNumericLiteral("0x20000000000001".encode_utf16().collect()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(
        actual,
        IntlMathematicalValue::Finite(exact(NumberSign::Positive, "9007199254740993", 0))
    );
}

#[test]
fn normalization_vector_04() {
    let actual = normalize_numeric_input(
        ObservedNumericInput::NumberShortestDecimal("1.005".into()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(
        actual,
        IntlMathematicalValue::Finite(exact(NumberSign::Positive, "1005", -3))
    );
}

#[test]
fn normalization_vector_05() {
    let actual =
        normalize_numeric_input(ObservedNumericInput::NegativeZero, &NumericLimits::HOST_ABI)
            .unwrap();
    assert_eq!(actual, IntlMathematicalValue::Zero(NumberSign::Negative));
}

#[test]
fn normalization_vector_06() {
    let actual = normalize_numeric_input(
        ObservedNumericInput::StringNumericLiteral("\t-0\n".encode_utf16().collect()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(actual, IntlMathematicalValue::Zero(NumberSign::Negative));
}

#[test]
fn normalization_vector_07() {
    let actual = normalize_numeric_input(
        ObservedNumericInput::StringNumericLiteral("﻿ ".encode_utf16().collect()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(actual, IntlMathematicalValue::Zero(NumberSign::Positive));
}

#[test]
fn normalization_vector_08() {
    let actual = normalize_numeric_input(
        ObservedNumericInput::StringNumericLiteral("-0x10".encode_utf16().collect()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(actual, IntlMathematicalValue::NaN);
}

#[test]
fn normalization_vector_09() {
    let actual = normalize_numeric_input(
        ObservedNumericInput::StringNumericLiteral("1_000".encode_utf16().collect()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(actual, IntlMathematicalValue::NaN);
}

#[test]
fn normalization_vector_10() {
    let actual = normalize_numeric_input(
        ObservedNumericInput::StringNumericLiteral("1n".encode_utf16().collect()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(actual, IntlMathematicalValue::NaN);
}

#[test]
fn normalization_vector_11() {
    let actual = normalize_numeric_input(
        ObservedNumericInput::StringNumericLiteral("1,2".encode_utf16().collect()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(actual, IntlMathematicalValue::NaN);
}

#[test]
fn normalization_vector_12() {
    let actual = normalize_numeric_input(
        ObservedNumericInput::StringNumericLiteral("infinity".encode_utf16().collect()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(actual, IntlMathematicalValue::NaN);
}

#[test]
fn normalization_vector_13() {
    let actual = normalize_numeric_input(
        ObservedNumericInput::StringNumericLiteral(vec![55296].into_boxed_slice()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(actual, IntlMathematicalValue::NaN);
}

#[test]
fn normalization_vector_14() {
    let actual = normalize_numeric_input(
        ObservedNumericInput::StringNumericLiteral("​".encode_utf16().collect()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(actual, IntlMathematicalValue::NaN);
}

#[test]
fn normalization_vector_15() {
    let actual = normalize_numeric_input(
        ObservedNumericInput::StringNumericLiteral("-Infinity".encode_utf16().collect()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(
        actual,
        IntlMathematicalValue::Infinity(NumberSign::Negative)
    );
}

#[test]
fn normalization_vector_16() {
    let actual = normalize_numeric_input(
        ObservedNumericInput::StringNumericLiteral("1e400".encode_utf16().collect()),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(
        actual,
        IntlMathematicalValue::Infinity(NumberSign::Positive)
    );
}

#[test]
fn normalization_vector_17() {
    let actual = normalize_numeric_input(ObservedNumericInput::BigIntDecimal("10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".into()), &NumericLimits::HOST_ABI).unwrap();
    assert_eq!(
        actual,
        IntlMathematicalValue::Finite(exact(NumberSign::Positive, "1", 400))
    );
}

#[test]
fn normalization_vector_18() {
    let actual = normalize_numeric_input(ObservedNumericInput::StringNumericLiteral("24703282292062327208828439643411068618252990130716238221279284125033775363510437593264991818081799618989828234772285886546332835517796989819938739800539093906315035659515570226392290858392449105184435931802849936536152500319370457678249219365623669863658480757001585769269903706311928279558551332927834338409351978015531246597263579574622766465272827220056374006485499977096599470454020828166226237857393450736339007967761930577506740176324673600968951340535537458516661134223766678604162159680461914467291840300530057530849048765391711386591646239524912623653881879636239373280423891018672348497668235089863388587925628302755995657524455507255189313690836254779186948667994968324049705821028513185451396213837722826145437693412532098591327667236328125e-1075".encode_utf16().collect()), &NumericLimits::HOST_ABI).unwrap();
    assert_eq!(actual, IntlMathematicalValue::Zero(NumberSign::Positive));
}

#[test]
fn normalization_vector_19() {
    let actual = normalize_numeric_input(ObservedNumericInput::StringNumericLiteral("-24703282292062327208828439643411068618252990130716238221279284125033775363510437593264991818081799618989828234772285886546332835517796989819938739800539093906315035659515570226392290858392449105184435931802849936536152500319370457678249219365623669863658480757001585769269903706311928279558551332927834338409351978015531246597263579574622766465272827220056374006485499977096599470454020828166226237857393450736339007967761930577506740176324673600968951340535537458516661134223766678604162159680461914467291840300530057530849048765391711386591646239524912623653881879636239373280423891018672348497668235089863388587925628302755995657524455507255189313690836254779186948667994968324049705821028513185451396213837722826145437693412532098591327667236328125e-1075".encode_utf16().collect()), &NumericLimits::HOST_ABI).unwrap();
    assert_eq!(actual, IntlMathematicalValue::Zero(NumberSign::Negative));
}

#[test]
fn normalization_vector_20() {
    let actual = normalize_numeric_input(ObservedNumericInput::StringNumericLiteral("24703282292062327208828439643411068618252990130716238221279284125033775363510437593264991818081799618989828234772285886546332835517796989819938739800539093906315035659515570226392290858392449105184435931802849936536152500319370457678249219365623669863658480757001585769269903706311928279558551332927834338409351978015531246597263579574622766465272827220056374006485499977096599470454020828166226237857393450736339007967761930577506740176324673600968951340535537458516661134223766678604162159680461914467291840300530057530849048765391711386591646239524912623653881879636239373280423891018672348497668235089863388587925628302755995657524455507255189313690836254779186948667994968324049705821028513185451396213837722826145437693412532098591327667236328126e-1075".encode_utf16().collect()), &NumericLimits::HOST_ABI).unwrap();
    assert_eq!(actual, IntlMathematicalValue::Finite(exact(NumberSign::Positive, "24703282292062327208828439643411068618252990130716238221279284125033775363510437593264991818081799618989828234772285886546332835517796989819938739800539093906315035659515570226392290858392449105184435931802849936536152500319370457678249219365623669863658480757001585769269903706311928279558551332927834338409351978015531246597263579574622766465272827220056374006485499977096599470454020828166226237857393450736339007967761930577506740176324673600968951340535537458516661134223766678604162159680461914467291840300530057530849048765391711386591646239524912623653881879636239373280423891018672348497668235089863388587925628302755995657524455507255189313690836254779186948667994968324049705821028513185451396213837722826145437693412532098591327667236328126", -1075)));
}

#[test]
fn normalization_vector_21() {
    let actual = normalize_numeric_input(ObservedNumericInput::StringNumericLiteral("179769313486231580793728971405303415079934132710037826936173778980444968292764750946649017977587207096330286416692887910946555547851940402630657488671505820681908902000708383676273854845817711531764475730270069855571366959622842914819860834936475292719074168444365510704342711559699508093042880177904174497792".encode_utf16().collect()), &NumericLimits::HOST_ABI).unwrap();
    assert_eq!(
        actual,
        IntlMathematicalValue::Infinity(NumberSign::Positive)
    );
}

#[test]
fn normalization_vector_22() {
    let actual = normalize_numeric_input(ObservedNumericInput::StringNumericLiteral("179769313486231580793728971405303415079934132710037826936173778980444968292764750946649017977587207096330286416692887910946555547851940402630657488671505820681908902000708383676273854845817711531764475730270069855571366959622842914819860834936475292719074168444365510704342711559699508093042880177904174497791".encode_utf16().collect()), &NumericLimits::HOST_ABI).unwrap();
    assert_eq!(actual, IntlMathematicalValue::Finite(exact(NumberSign::Positive, "179769313486231580793728971405303415079934132710037826936173778980444968292764750946649017977587207096330286416692887910946555547851940402630657488671505820681908902000708383676273854845817711531764475730270069855571366959622842914819860834936475292719074168444365510704342711559699508093042880177904174497791", 0)));
}

#[test]
fn normalization_vector_23() {
    let actual = normalize_numeric_input(
        ObservedNumericInput::StringNumericLiteral(
            "-0e999999999999999999999999999".encode_utf16().collect(),
        ),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(actual, IntlMathematicalValue::Zero(NumberSign::Negative));
}

#[test]
fn plural_modulo_of_a_thousand_digit_integer_remains_exact() {
    let value = bigint_input(&"1234567890".repeat(100));
    let result = round_decimal(
        value.finite_value().unwrap(),
        &settings(fraction(0, 0), RoundingMode::HalfEven),
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    let remainder = result
        .plural_operands()
        .operand(PluralOperand::N)
        .modulo(NonZeroU64::new(97).unwrap());
    assert!(remainder.is_integer());
    assert_eq!(remainder.compare_integer(28), Ordering::Equal);
}

#[test]
fn compact_plural_modulo_keeps_a_four_billion_digit_zero_tail_virtual() {
    let value = exact(NumberSign::Positive, "1", i64::from(u32::MAX));
    let table = compact_rows(&[(u32::MAX, u32::MAX)]);
    let controls = DecimalFormatSettings {
        rounding: settings(fraction(0, 0), RoundingMode::HalfEven),
        scale: DecimalScale::Unit,
        notation: NotationScaling::Compact(&table),
    };
    let result = format_decimal(
        FiniteValue::Nonzero(&value),
        &controls,
        &NumericLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(text(&result), "1");
    let number = result.plural_operands().operand(PluralOperand::N);
    assert_eq!(number.compare_integer(u64::MAX), Ordering::Greater);
    let remainder = number.modulo(NonZeroU64::new(97).unwrap());
    assert_eq!(remainder.compare_integer(52), Ordering::Equal);
}
