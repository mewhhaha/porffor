use super::*;
use crate::intl_host_probe::IntlHostProbe;
use lila_intl::{LocalTimeCoordinate, NamedTimeZoneTransitionDirection, TimeZoneInstant};

fn query(operation: IntlHostOp, bytes: &[u8], expected_size: usize) -> Vec<u8> {
    let mut probe = IntlHostProbe::new();
    let response = probe
        .invoke(operation, bytes)
        .unwrap()
        .expect("accepted time-zone request");
    assert_eq!(response.len(), expected_size);
    response
}
fn word(bytes: &[u8], index: usize) -> i64 {
    i64::from_le_bytes(bytes[index * 8..index * 8 + 8].try_into().unwrap())
}

#[test]
fn exact_offset_and_strict_transition_dispatch_use_the_typed_kernel() {
    let id = TimeZoneId::parse("America/New_York").unwrap();
    let t = 1_615_705_200_000_000_000_i128;
    for (ns, offset) in [(t - 1, -18_000), (t, -14_400), (t + 1, -14_400)] {
        let request = NamedTimeZoneOffsetRequest::new(
            id.clone(),
            TimeZoneInstant::from_nanoseconds(ns).unwrap(),
        )
        .unwrap();
        assert_eq!(
            word(
                &query(IntlHostOp::NamedTimeZoneOffset, &request.encode(), 8),
                0
            ),
            offset
        );
    }
    let request = FindNamedTimeZoneTransitionRequest::new(
        id,
        TimeZoneInstant::from_nanoseconds(t + 1).unwrap(),
        NamedTimeZoneTransitionDirection::Previous,
    )
    .unwrap();
    let result = query(
        IntlHostOp::FindNamedTimeZoneTransition,
        &request.encode(),
        16,
    );
    assert_eq!((word(&result, 0), word(&result, 1)), (1, 1_615_705_200));
}

#[test]
fn inverse_gc_response_preserves_fractional_candidates_and_gap_units() {
    let id = TimeZoneId::parse("America/New_York").unwrap();
    let request = PossibleNamedTimeZoneEpochsRequest::new(
        id,
        LocalTimeCoordinate::new(1_636_248_600, 123_456_789).unwrap(),
    )
    .unwrap();
    let result = query(
        IntlHostOp::PossibleNamedTimeZoneEpochs,
        &request.encode(),
        64,
    );
    assert_eq!((word(&result, 0), word(&result, 1)), (1, 2));
    assert_eq!(
        (word(&result, 2), word(&result, 3), word(&result, 4)),
        (1_636_263_000, 123_456_789, -14_400)
    );
    assert_eq!(
        (word(&result, 5), word(&result, 6), word(&result, 7)),
        (1_636_266_600, 123_456_789, -18_000)
    );
    let request = PossibleNamedTimeZoneEpochsRequest::new(
        TimeZoneId::parse("Pacific/Apia").unwrap(),
        LocalTimeCoordinate::new(1_325_203_200, 0).unwrap(),
    )
    .unwrap();
    let result = query(
        IntlHostOp::PossibleNamedTimeZoneEpochs,
        &request.encode(),
        40,
    );
    assert_eq!(
        (
            word(&result, 0),
            word(&result, 1),
            word(&result, 2),
            word(&result, 3),
            word(&result, 4)
        ),
        (0, 0, 1_325_239_200, -36_000, 50_400)
    );
}

#[test]
fn invalid_exact_request_and_unresolved_identifier_trap_at_the_gc_boundary() {
    let valid = NamedTimeZoneOffsetRequest::new(
        TimeZoneId::parse("UTC").unwrap(),
        TimeZoneInstant::from_nanoseconds(-1).unwrap(),
    )
    .unwrap();
    let mut probe = IntlHostProbe::new();
    let mut invalid_nano = valid.encode();
    invalid_nano[8..16].copy_from_slice(&1_000_000_000_i64.to_le_bytes());
    let unknown = NamedTimeZoneOffsetRequest::new(
        TimeZoneId::parse("Missing/Zone").unwrap(),
        valid.instant(),
    )
    .unwrap();
    for bytes in [invalid_nano, unknown.encode()] {
        assert!(probe
            .invoke(IntlHostOp::NamedTimeZoneOffset, &bytes)
            .is_err());
    }
}
