use super::*;
use crate::intl_host_probe::IntlHostProbe;
use lila_intl::number_format::numeric::{ObservedNumericInput, RoundingSettings};
use lila_intl::number_format::options::*;
use lila_intl::number_format::PartitionLimits;
use lila_intl::plural_rules::resolve_plural_locale;
use lila_intl::{CanonicalLocaleId, CheckedPluralConfiguration, PluralType};
fn locale() -> PluralLocaleRequest {
    PluralLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data("en-US").unwrap()].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
    }
}
fn configuration() -> CheckedPluralConfiguration {
    CheckedPluralConfiguration::new(
        resolve_plural_locale(
            &locale(),
            &embedded_number_profiles_arc().unwrap(),
            &PartitionLimits::HOST_ABI,
        )
        .unwrap(),
        PluralType::Ordinal,
        Notation::Standard,
        RoundingSettings {
            precision: Precision::Fraction(FractionPrecision::Range(
                FractionDigitRange::new(
                    FractionDigitCount::new(0).unwrap(),
                    FractionDigitCount::new(3).unwrap(),
                )
                .unwrap(),
            )),
            minimum_integer_digits: IntegerDigitCount::new(1).unwrap(),
            mode: RoundingMode::HalfExpand,
            trailing_zero_display: TrailingZeroDisplay::Auto,
        },
    )
}
fn string(text: &str) -> ObservedNumericInput {
    ObservedNumericInput::StringNumericLiteral(text.encode_utf16().collect())
}
fn gc_response<O: PluralHostOperation>(probe: &mut IntlHostProbe, bytes: &[u8]) -> Vec<u8> {
    probe
        .invoke(O::HOST_OP, bytes)
        .unwrap()
        .expect("accepted Intl request")
}

#[test]
fn all_plural_operations_return_owned_gc_responses() {
    let mut probe = IntlHostProbe::new();
    let response = gc_response::<ResolvePluralLocale>(&mut probe, &locale().encode().unwrap());
    let resolved =
        ResolvedPluralLocale::decode(&response, &embedded_number_profiles_arc().unwrap()).unwrap();
    assert_eq!(resolved.data().as_str(), "en-US");
    let request = PluralSupportedLocalesRequest {
        requested: locale().requested,
        matcher: LocaleMatcher::Lookup,
    };
    let response = gc_response::<SupportedPluralLocales>(&mut probe, &request.encode().unwrap());
    assert_eq!(
        PluralSupportedLocalesResult::decode(&response)
            .unwrap()
            .locales[0]
            .as_str(),
        "en-US"
    );
    let request = SelectPluralRequest::new(
        configuration(),
        ObservedNumericInput::BigIntDecimal("100000000000000003".into()),
    );
    let response = gc_response::<SelectPlural>(&mut probe, &request.encode().unwrap());
    assert_eq!(
        PluralCategory::decode_scalar(&response).unwrap(),
        PluralCategory::Few
    );
    let request = SelectPluralRangeRequest::new(configuration(), string("4"), string("1"));
    let response = gc_response::<SelectPluralRange>(&mut probe, &request.encode().unwrap());
    assert_eq!(
        PluralCategory::decode_range(&response).unwrap(),
        PluralCategory::One
    );
}
#[test]
fn only_semantic_nan_range_rejection_returns_a_null_gc_response() {
    let mut probe = IntlHostProbe::new();
    let request = SelectPluralRangeRequest::new(configuration(), string("NaN"), string("1"))
        .encode()
        .unwrap();

    assert_eq!(
        probe
            .invoke(IntlHostOp::SelectPluralRange, &request)
            .unwrap(),
        None
    );

    let request = SelectPluralRequest::new(configuration(), string("NaN"))
        .encode()
        .unwrap();
    let response = gc_response::<SelectPlural>(&mut probe, &request);
    assert_eq!(
        PluralCategory::decode_scalar(&response).unwrap(),
        PluralCategory::Other
    );
}
#[test]
fn malformed_plural_wire_is_a_native_fault() {
    let mut probe = IntlHostProbe::new();
    let mut request = locale().encode().unwrap();
    request[0] = 2;

    assert!(probe
        .invoke(IntlHostOp::ResolvePluralLocale, &request)
        .is_err());
}
