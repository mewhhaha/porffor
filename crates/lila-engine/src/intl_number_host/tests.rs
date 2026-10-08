use super::*;
use crate::intl_host_probe::IntlHostProbe;
use lila_intl::number_format::numeric::ObservedNumericInput;
use lila_intl::number_format::options::*;
use lila_intl::number_format::{resolve_number_locale, NumberFormatConfiguration, PartitionLimits};
use lila_intl::CanonicalLocaleId;

fn request_locale() -> NumberLocaleRequest {
    NumberLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data("en-US").unwrap()].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
        numbering_system: None,
    }
}
fn configuration() -> NumberFormatConfiguration {
    NumberFormatConfiguration {
        locale: resolve_number_locale(&request_locale(), &embedded_number_profiles_arc().unwrap())
            .unwrap(),
        options: NumberFormatOptions {
            style: NumberStyle::Decimal,
            notation: Notation::Standard,
            minimum_integer_digits: IntegerDigitCount::new(1).unwrap(),
            precision: Precision::Fraction(FractionPrecision::Range(
                FractionDigitRange::new(
                    FractionDigitCount::new(0).unwrap(),
                    FractionDigitCount::new(3).unwrap(),
                )
                .unwrap(),
            )),
            rounding_mode: RoundingMode::HalfExpand,
            trailing_zero_display: TrailingZeroDisplay::Auto,
            grouping: Grouping::Auto,
            sign_display: SignDisplay::Auto,
        },
    }
}
fn gc_response<O: NumberHostOperation>(probe: &mut IntlHostProbe, bytes: &[u8]) -> Vec<u8> {
    probe
        .invoke(O::HOST_OP, bytes)
        .unwrap()
        .expect("accepted Intl request")
}

#[test]
fn every_number_operation_returns_an_owned_gc_response() {
    let mut probe = IntlHostProbe::new();
    let response =
        gc_response::<ResolveNumberLocale>(&mut probe, &request_locale().encode().unwrap());
    assert_eq!(
        ResolvedNumberLocale::decode(&response, &embedded_number_profiles_arc().unwrap())
            .unwrap()
            .resolved()
            .as_str(),
        "en-US"
    );
    let response = gc_response::<SupportedNumberLocales>(
        &mut probe,
        &NumberSupportedLocalesRequest {
            requested: vec![
                CanonicalLocaleId::from_data("en-US-u-nu-arab").unwrap(),
                CanonicalLocaleId::from_data("zxx").unwrap(),
            ]
            .into_boxed_slice(),
            matcher: LocaleMatcher::Lookup,
        }
        .encode()
        .unwrap(),
    );
    assert_eq!(
        NumberSupportedLocalesResult::decode(&response)
            .unwrap()
            .locales
            .iter()
            .map(CanonicalLocaleId::as_str)
            .collect::<Vec<_>>(),
        ["en-US-u-nu-arab"]
    );
    let request = NumberFormatRequest {
        configuration: configuration(),
        input: ObservedNumericInput::BigIntDecimal("100000000000000001".into()),
    };
    let response = gc_response::<FormatNumberParts>(&mut probe, &request.encode().unwrap());
    assert_eq!(
        ScalarNumberPartition::decode(&response).unwrap().to_text(),
        "100,000,000,000,000,001"
    );
    let request = NumberRangeFormatRequest {
        configuration: configuration(),
        start: ObservedNumericInput::NumberShortestDecimal("1".into()),
        end: ObservedNumericInput::NumberShortestDecimal("1".into()),
    };
    let response = gc_response::<FormatNumberRangeParts>(&mut probe, &request.encode().unwrap());
    let range = RangeNumberPartition::decode(&response).unwrap();
    let direct = lila_intl::number_format::partition_number_range(
        &request.configuration,
        &lila_intl::number_format::numeric::NumberRange::new(
            lila_intl::number_format::numeric::normalize_numeric_input(
                request.start,
                &PartitionLimits::HOST_ABI.numeric(),
            )
            .unwrap(),
            lila_intl::number_format::numeric::normalize_numeric_input(
                request.end,
                &PartitionLimits::HOST_ABI.numeric(),
            )
            .unwrap(),
        )
        .unwrap(),
        &embedded_number_profiles_arc().unwrap(),
        &PartitionLimits::HOST_ABI,
    )
    .unwrap();
    assert_eq!(range, direct);
}

#[test]
fn malformed_number_messages_and_invalid_intrinsic_numeric_text_trap_at_the_gc_boundary() {
    let mut probe = IntlHostProbe::new();
    let request = NumberFormatRequest {
        configuration: configuration(),
        input: ObservedNumericInput::NegativeZero,
    };
    let valid = request.encode().unwrap();
    for bytes in [&valid[..valid.len() - 1], &[]] {
        assert!(probe.invoke(IntlHostOp::FormatNumberParts, &bytes).is_err());
    }
    let request = NumberFormatRequest {
        configuration: configuration(),
        input: ObservedNumericInput::NumberShortestDecimal("invalid-intrinsic-text".into()),
    };
    let bytes = request.encode().unwrap();

    assert!(probe.invoke(IntlHostOp::FormatNumberParts, &bytes).is_err());
}

#[test]
fn only_nan_range_endpoints_return_a_null_gc_response() {
    let mut probe = IntlHostProbe::new();
    let request = NumberRangeFormatRequest {
        configuration: configuration(),
        start: ObservedNumericInput::StringNumericLiteral(
            "bad numeric syntax".encode_utf16().collect(),
        ),
        end: ObservedNumericInput::NumberShortestDecimal("2".into()),
    };
    let bytes = request.encode().unwrap();

    assert_eq!(
        probe
            .invoke(IntlHostOp::FormatNumberRangeParts, &bytes)
            .unwrap(),
        None
    );

    let scalar = NumberFormatRequest {
        configuration: request.configuration,
        input: request.start,
    };
    let response = gc_response::<FormatNumberParts>(&mut probe, &scalar.encode().unwrap());
    assert_eq!(
        ScalarNumberPartition::decode(&response).unwrap().to_text(),
        "NaN"
    );
}
