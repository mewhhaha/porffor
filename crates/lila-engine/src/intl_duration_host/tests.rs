use super::*;
use crate::intl_host_probe::IntlHostProbe;
use lila_intl::number_format::options::LocaleMatcher;
use lila_intl::number_format::{
    embedded_number_profiles_arc, NumberLocaleRequest, PartitionLimits,
};
use lila_intl::*;

fn locale() -> NumberLocaleRequest {
    NumberLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data("sr").unwrap()].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
        numbering_system: None,
    }
}
fn request() -> DurationWireRequest {
    let profiles = embedded_duration_profiles().unwrap();
    let locale = profiles
        .resolve_locale(&locale(), &embedded_number_profiles_arc().unwrap())
        .unwrap();
    DurationWireRequest::Parts(
        DurationPartitionRequest::from_completed_number_fields(
            CheckedDurationConfiguration::new(
                locale,
                DurationOptions {
                    style: DurationStyle::Digital,
                    ..Default::default()
                },
            )
            .unwrap(),
            [0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 0.0, 0.0, 0.0],
        )
        .unwrap(),
    )
}
fn gc_response(probe: &mut IntlHostProbe, operation: IntlHostOp, bytes: &[u8]) -> Vec<u8> {
    probe
        .invoke(operation, bytes)
        .unwrap()
        .expect("accepted Intl request")
}

#[test]
fn all_duration_operations_return_owned_gc_responses() {
    let mut probe = IntlHostProbe::new();
    let profiles = embedded_duration_profiles().unwrap();
    let numbers = &embedded_number_profiles_arc().unwrap();
    let requests = [
        DurationWireRequest::Resolve(locale()),
        DurationWireRequest::SupportedLocales(DurationSupportedLocalesRequest {
            requested: locale().requested,
            matcher: LocaleMatcher::Lookup,
        }),
        request(),
    ];
    for request in requests {
        let bytes = encode_duration_request(&request).unwrap();
        let response = gc_response(&mut probe, request.operation().global_operation(), &bytes);
        match decode_duration_response(
            &request,
            &response,
            profiles,
            numbers,
            &PartitionLimits::HOST_ABI,
        )
        .unwrap()
        {
            DurationWireResponse::Resolved(locale) => {
                assert_eq!(locale.resolved().as_str(), "sr");
                assert!(!locale.two_digit_hours());
            }
            DurationWireResponse::SupportedLocales(locales) => {
                assert_eq!(locales[0].as_str(), "sr")
            }
            DurationWireResponse::Parts(parts) => {
                assert_eq!(parts.to_text().unwrap(), "1.02.03");
                assert_eq!(
                    parts
                        .parts()
                        .iter()
                        .filter(|part| part.unit().is_none())
                        .map(DurationWirePart::text)
                        .collect::<Vec<_>>(),
                    [".", "."]
                );
            }
        }
    }
}
#[test]
fn malformed_duration_frames_fault_at_the_gc_boundary() {
    let mut probe = IntlHostProbe::new();
    for request in [
        DurationWireRequest::Resolve(locale()),
        DurationWireRequest::SupportedLocales(DurationSupportedLocalesRequest {
            requested: locale().requested,
            matcher: LocaleMatcher::Lookup,
        }),
        request(),
    ] {
        let original = encode_duration_request(&request).unwrap();
        let mut version = original.clone();
        version[0] = 2;
        let mut operation = original.clone();
        operation[8] = 70;
        let mut trailing = original.clone();
        trailing.push(0);
        for bytes in [version, operation, trailing, original[..15].to_vec()] {
            assert!(probe
                .invoke(request.operation().global_operation(), &bytes)
                .is_err());
        }
    }
}
#[test]
fn mixed_sign_and_normative_bounds_reject_as_a_null_gc_response() {
    let mut probe = IntlHostProbe::new();
    let original = encode_duration_request(&request()).unwrap();
    for fields in [
        [1.0_f64, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        [4_294_967_296.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        [
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            9_007_199_254_740_992.0,
            0.0,
            0.0,
            0.0,
        ],
    ] {
        let mut bytes = original.clone();
        let start = bytes.len() - 80;
        for (index, value) in fields.iter().enumerate() {
            bytes[start + index * 8..start + index * 8 + 8]
                .copy_from_slice(&value.to_bits().to_le_bytes());
        }

        assert_eq!(
            probe
                .invoke(IntlHostOp::PartitionDurationFormat, &bytes)
                .unwrap(),
            None
        );
    }
}
#[test]
fn signed_zero_and_labels_survive_the_real_duration_host_boundary() {
    let mut probe = IntlHostProbe::new();
    let profiles = embedded_duration_profiles().unwrap();
    let numbers = &embedded_number_profiles_arc().unwrap();
    let locale = profiles
        .resolve_locale(
            &NumberLocaleRequest {
                requested: vec![CanonicalLocaleId::from_data("en").unwrap()].into_boxed_slice(),
                matcher: LocaleMatcher::Lookup,
                numbering_system: None,
            },
            numbers,
        )
        .unwrap();
    let request = DurationWireRequest::Parts(
        DurationPartitionRequest::from_completed_number_fields(
            CheckedDurationConfiguration::new(
                locale,
                DurationOptions {
                    style: DurationStyle::Digital,
                    ..Default::default()
                },
            )
            .unwrap(),
            [-0.0, -0.0, -0.0, -0.0, -0.0, -0.0, -1.0, -0.0, -0.0, -0.0],
        )
        .unwrap(),
    );
    let response = gc_response(
        &mut probe,
        IntlHostOp::PartitionDurationFormat,
        &encode_duration_request(&request).unwrap(),
    );
    let DurationWireResponse::Parts(parts) = decode_duration_response(
        &request,
        &response,
        profiles,
        numbers,
        &PartitionLimits::HOST_ABI,
    )
    .unwrap() else {
        panic!("parts")
    };
    assert_eq!(parts.to_text().unwrap(), "-0:00:01");
    assert_eq!(
        parts
            .parts()
            .iter()
            .filter(|part| part.kind() == lila_intl::number_format::NumberPartKind::MinusSign)
            .count(),
        1
    );
    assert!(parts
        .parts()
        .iter()
        .filter(|part| part.kind() != lila_intl::number_format::NumberPartKind::Literal)
        .all(|part| part.unit().is_some()));
}
