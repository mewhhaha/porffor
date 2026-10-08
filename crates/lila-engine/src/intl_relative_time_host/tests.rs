use super::*;
use crate::intl_host_probe::IntlHostProbe;
use lila_intl::number_format::{options::LocaleMatcher, NumberLocaleRequest, NumberPartKind};
use lila_intl::*;

fn locale(name: &str) -> NumberLocaleRequest {
    NumberLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data(name).unwrap()].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
        numbering_system: None,
    }
}
fn configuration(name: &str) -> RelativeTimeConfiguration {
    let profiles = embedded_relative_profiles().unwrap();
    let numbers = embedded_number_profiles_arc().unwrap();
    let resolved = profiles
        .resolve_locale(&locale(name), &numbers, &PartitionLimits::default())
        .unwrap();
    RelativeTimeConfiguration::new(
        resolved,
        RelativeStyle::Long,
        RelativeNumeric::Always,
        &numbers,
    )
    .unwrap()
}
fn gc_response(probe: &mut IntlHostProbe, request: &RelativeRequest) -> RelativeResponse {
    let bytes = encode_relative_request(request).unwrap();
    let response = probe
        .invoke(request.operation().global_operation(), &bytes)
        .unwrap()
        .expect("accepted Intl request");
    decode_relative_response(
        request.operation(),
        &response,
        embedded_relative_profiles().unwrap(),
        &embedded_number_profiles_arc().unwrap(),
        &PartitionLimits::default(),
    )
    .unwrap()
}

#[test]
fn all_relative_time_operations_preserve_signed_zero_parts() {
    let mut probe = IntlHostProbe::new();
    let RelativeResponse::Resolved(resolved) =
        gc_response(&mut probe, &RelativeRequest::Resolve(locale("en-US")))
    else {
        panic!("resolve response")
    };
    assert_eq!(resolved.resolved().as_str(), "en-US");
    assert_eq!(resolved.numbering_system(), "latn");
    let RelativeResponse::Supported(supported) = gc_response(
        &mut probe,
        &RelativeRequest::Supported {
            requested: vec![
                CanonicalLocaleId::from_data("en-US").unwrap(),
                CanonicalLocaleId::from_data("zz-ZZ").unwrap(),
            ]
            .into_boxed_slice(),
            matcher: LocaleMatcher::Lookup,
        },
    ) else {
        panic!("supported response")
    };
    assert_eq!(supported.len(), 1);
    assert_eq!(supported[0].as_str(), "en-US");
    let RelativeResponse::Parts(parts) = gc_response(
        &mut probe,
        &RelativeRequest::Parts {
            configuration: configuration("en-US"),
            value: FiniteRelativeNumber::new(-0.0).unwrap(),
            unit: RelativeUnit::Day,
        },
    ) else {
        panic!("parts response")
    };
    assert_eq!(parts.to_text().unwrap(), "0 days ago");
    assert!(parts.parts().iter().any(
        |part| part.kind() == NumberPartKind::Integer && part.unit() == Some(RelativeUnit::Day)
    ));
    assert!(parts
        .parts()
        .iter()
        .any(|part| part.kind() == NumberPartKind::Literal && part.unit().is_none()));
}

#[test]
fn arabic_literal_only_numeric_pattern_has_no_invented_unit_part() {
    let mut probe = IntlHostProbe::new();
    let RelativeResponse::Parts(parts) = gc_response(
        &mut probe,
        &RelativeRequest::Parts {
            configuration: configuration("ar"),
            value: FiniteRelativeNumber::new(1.0).unwrap(),
            unit: RelativeUnit::Day,
        },
    ) else {
        panic!("parts response")
    };
    assert_eq!(parts.to_text().unwrap(), "خلال يوم واحد");
    assert_eq!(parts.parts().len(), 1);
    assert_eq!(parts.parts()[0].kind(), NumberPartKind::Literal);
    assert_eq!(parts.parts()[0].unit(), None);
}

#[test]
fn malformed_relative_time_frames_fault_at_the_gc_boundary() {
    let mut probe = IntlHostProbe::new();
    let mut bytes = encode_relative_request(&RelativeRequest::Resolve(locale("en-US"))).unwrap();
    bytes[0] = 2;
    for operation in [
        IntlHostOp::ResolveRelativeTimeLocale,
        IntlHostOp::SupportedRelativeTimeLocales,
        IntlHostOp::FormatRelativeTimeParts,
    ] {
        assert!(probe.invoke(operation, &bytes).is_err());
    }
}
