use super::*;

fn numbers() -> Arc<NumberProfiles> {
    crate::number_format::embedded_number_profiles_arc().unwrap()
}
fn profiles() -> &'static RelativeProfiles {
    embedded_relative_profiles().unwrap()
}
fn locale(value: &str) -> CanonicalLocaleId {
    CanonicalLocaleId::from_data(value).unwrap()
}
fn request(value: f64) -> RelativeRequest {
    let number = NumberLocaleRequest {
        requested: vec![locale("en-US-u-nu-arab")].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
        numbering_system: None,
    };
    let resolved = profiles()
        .resolve_locale(&number, &numbers(), &PartitionLimits::HOST_ABI)
        .unwrap();
    RelativeRequest::Parts {
        configuration: RelativeTimeConfiguration::new(
            resolved,
            RelativeStyle::Long,
            RelativeNumeric::Always,
            &numbers(),
        )
        .unwrap(),
        value: FiniteRelativeNumber::new(value).unwrap(),
        unit: RelativeUnit::Day,
    }
}
fn decode(operation: RelativeHostOp, bytes: &[u8]) -> Result<RelativeRequest, RelativeWireError> {
    decode_relative_request(
        operation,
        bytes,
        profiles(),
        &numbers(),
        &PartitionLimits::HOST_ABI,
    )
}
fn response(
    operation: RelativeHostOp,
    bytes: &[u8],
) -> Result<RelativeResponse, RelativeWireError> {
    decode_relative_response(
        operation,
        bytes,
        profiles(),
        &numbers(),
        &PartitionLimits::HOST_ABI,
    )
}

#[test]
fn relative_frames_reserve_exact_three_codes_and_standard_direction_words() {
    let requests = [
        RelativeRequest::Resolve(NumberLocaleRequest {
            requested: Box::new([]),
            matcher: LocaleMatcher::Lookup,
            numbering_system: None,
        }),
        RelativeRequest::Supported {
            requested: Box::new([]),
            matcher: LocaleMatcher::Lookup,
        },
        request(0.0),
    ];
    for (index, operation) in RelativeHostOp::ALL.into_iter().enumerate() {
        assert_eq!(operation.code(), 30 + index as u64);
        assert_eq!(RelativeHostOp::from_code(operation.code()), Some(operation));
        let encoded = encode_relative_request(&requests[index]).unwrap();
        assert_eq!(&encoded[..8], &1u64.to_le_bytes());
        assert_eq!(&encoded[8..16], &(operation.code() * 2).to_le_bytes());
        let result = execute_relative_request(
            requests[index].clone(),
            profiles(),
            &numbers(),
            &PartitionLimits::HOST_ABI,
        )
        .unwrap();
        let encoded = encode_relative_response(&result).unwrap();
        assert_eq!(&encoded[8..16], &(operation.code() * 2 + 1).to_le_bytes());
    }
    for invalid in [0, 26, 27, 29, 33, u64::MAX] {
        assert_eq!(RelativeHostOp::from_code(invalid), None);
    }
}
#[test]
fn relative_requests_round_trip_canonical_locales_options_and_signed_zero() {
    let requests = [
        RelativeRequest::Resolve(NumberLocaleRequest {
            requested: vec![locale("fr-u-nu-latn"), locale("de")].into_boxed_slice(),
            matcher: LocaleMatcher::BestFit,
            numbering_system: Some(NumberingSystemOption::parse("arab").unwrap()),
        }),
        RelativeRequest::Supported {
            requested: vec![locale("en-GB"), locale("zz-ZZ")].into_boxed_slice(),
            matcher: LocaleMatcher::Lookup,
        },
        request(-0.0),
    ];
    for request in requests {
        let encoded = encode_relative_request(&request).unwrap();
        assert_eq!(decode(request.operation(), &encoded).unwrap(), request);
    }
}
#[test]
fn relative_responses_round_trip_unit_presence_and_rtl_number_parts() {
    for request in [request(-0.0), request(-12345.67)] {
        let operation = request.operation();
        let result =
            execute_relative_request(request, profiles(), &numbers(), &PartitionLimits::HOST_ABI)
                .unwrap();
        let encoded = encode_relative_response(&result).unwrap();
        assert_eq!(response(operation, &encoded).unwrap(), result);
        let RelativeResponse::Parts(parts) = result else {
            panic!("parts response")
        };
        assert!(parts
            .parts()
            .iter()
            .any(|part| part.unit() == Some(RelativeUnit::Day)));
        assert!(parts.parts().iter().any(|part| part.unit().is_none()));
        assert!(!parts.to_text().unwrap().contains('-'));
    }
    let resolve = RelativeRequest::Resolve(NumberLocaleRequest {
        requested: vec![locale("fr")].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
        numbering_system: None,
    });
    let result =
        execute_relative_request(resolve, profiles(), &numbers(), &PartitionLimits::HOST_ABI)
            .unwrap();
    assert_eq!(
        response(
            result.operation(),
            &encode_relative_response(&result).unwrap()
        )
        .unwrap(),
        result
    );
}
#[test]
fn truncated_trailing_and_cross_operation_relative_frames_are_rejected() {
    let request = request(1.5);
    let operation = request.operation();
    let encoded = encode_relative_request(&request).unwrap();
    for length in 0..encoded.len() {
        assert!(
            decode(operation, &encoded[..length]).is_err(),
            "accepted prefix {length}"
        );
    }
    let mut trailing = encoded.clone();
    trailing.push(0);
    assert!(decode(operation, &trailing).is_err());
    assert!(decode(RelativeHostOp::ResolveRelativeTimeLocale, &encoded).is_err());
    assert!(response(operation, &encoded).is_err());
    let result =
        execute_relative_request(request, profiles(), &numbers(), &PartitionLimits::HOST_ABI)
            .unwrap();
    let encoded = encode_relative_response(&result).unwrap();
    for length in 0..encoded.len() {
        assert!(
            response(operation, &encoded[..length]).is_err(),
            "accepted response prefix {length}"
        );
    }
}
#[test]
fn relative_decoder_rejects_nonfinite_bits_and_unknown_closed_options() {
    let config = match request(0.0) {
        RelativeRequest::Parts { configuration, .. } => configuration,
        _ => unreachable!(),
    };
    let frame = |style: u64, numeric: u64, bits: u64, unit: u64| {
        let mut writer = Writer::new(RelativeHostOp::FormatRelativeTimeParts, false).unwrap();
        write_resolved(&mut writer, config.locale()).unwrap();
        for value in [style, numeric, bits, unit] {
            writer.word(value).unwrap();
        }
        writer.finish()
    };
    for bits in [
        f64::NAN.to_bits(),
        f64::INFINITY.to_bits(),
        f64::NEG_INFINITY.to_bits(),
    ] {
        assert!(decode(
            RelativeHostOp::FormatRelativeTimeParts,
            &frame(1, 1, bits, 5)
        )
        .is_err());
    }
    for (style, numeric, unit) in [
        (0, 1, 5),
        (4, 1, 5),
        (1, 0, 5),
        (1, 3, 5),
        (1, 1, 0),
        (1, 1, 9),
    ] {
        assert!(decode(
            RelativeHostOp::FormatRelativeTimeParts,
            &frame(style, numeric, 0, unit)
        )
        .is_err());
    }
}
#[test]
fn relative_decoder_cannot_import_a_number_only_locale_into_service() {
    let mut writer = Writer::new(RelativeHostOp::FormatRelativeTimeParts, false).unwrap();
    for text in ["en-GB", "en-GB", "latn"] {
        writer.text(text).unwrap();
    }
    for value in [1, 1, 0, 5] {
        writer.word(value).unwrap();
    }
    assert!(decode(RelativeHostOp::FormatRelativeTimeParts, &writer.finish()).is_err());
    let mut writer = Writer::new(RelativeHostOp::SupportedRelativeTimeLocales, true).unwrap();
    write_locales(&mut writer, &[locale("zz-ZZ")]).unwrap();
    assert!(response(
        RelativeHostOp::SupportedRelativeTimeLocales,
        &writer.finish()
    )
    .is_err());
}
#[test]
fn relative_parts_decoder_rejects_unitless_numbers_nondecimal_kinds_and_mixed_units() {
    let frame = |rows: &[(NumberPartKind, u64, &str)]| {
        let mut writer = Writer::new(RelativeHostOp::FormatRelativeTimeParts, true).unwrap();
        writer.word(rows.len() as u64).unwrap();
        for (kind, unit, text) in rows {
            writer.word(kind.wire_code()).unwrap();
            writer.text(text).unwrap();
            writer.word(u64::from(*unit != 0)).unwrap();
            if *unit != 0 {
                writer.word(*unit).unwrap();
            }
        }
        writer.finish()
    };
    for rows in [
        vec![(NumberPartKind::Integer, 0, "1")],
        vec![(NumberPartKind::MinusSign, 5, "-")],
        vec![(NumberPartKind::Infinity, 5, "∞")],
        vec![(NumberPartKind::Integer, 9, "1")],
        vec![
            (NumberPartKind::Integer, 5, "1"),
            (NumberPartKind::Integer, 6, "2"),
        ],
        vec![(NumberPartKind::Literal, 0, "")],
        vec![(NumberPartKind::Literal, 0, "x\0y")],
        Vec::new(),
    ] {
        assert!(response(RelativeHostOp::FormatRelativeTimeParts, &frame(&rows)).is_err());
    }
    let good = frame(&[
        (NumberPartKind::Literal, 5, "\u{061c}"),
        (NumberPartKind::Integer, 5, "١"),
    ]);
    let RelativeResponse::Parts(parts) =
        response(RelativeHostOp::FormatRelativeTimeParts, &good).unwrap()
    else {
        panic!("parts")
    };
    assert!(parts
        .parts()
        .iter()
        .all(|part| part.unit() == Some(RelativeUnit::Day)));
}
#[test]
fn relative_frame_counts_utf8_and_numbering_presence_are_checked_before_allocation() {
    let mut writer = Writer::new(RelativeHostOp::FormatRelativeTimeParts, true).unwrap();
    writer.word(1).unwrap();
    writer.word(NumberPartKind::Literal.wire_code()).unwrap();
    writer.text("today").unwrap();
    writer.word(2).unwrap();
    assert!(response(RelativeHostOp::FormatRelativeTimeParts, &writer.finish()).is_err());
    let mut writer = Writer::new(RelativeHostOp::ResolveRelativeTimeLocale, false).unwrap();
    writer.word(u64::MAX).unwrap();
    assert!(decode(RelativeHostOp::ResolveRelativeTimeLocale, &writer.finish()).is_err());
    let mut writer = Writer::new(RelativeHostOp::ResolveRelativeTimeLocale, false).unwrap();
    write_locales(&mut writer, &[]).unwrap();
    writer.word(1).unwrap();
    writer.word(2).unwrap();
    assert!(decode(RelativeHostOp::ResolveRelativeTimeLocale, &writer.finish()).is_err());
    let mut writer = Writer::new(RelativeHostOp::SupportedRelativeTimeLocales, false).unwrap();
    writer.word(1).unwrap();
    writer.word(1).unwrap();
    let mut encoded = writer.finish();
    encoded.push(0xff);
    encoded.extend_from_slice(&1u64.to_le_bytes());
    assert!(decode(RelativeHostOp::SupportedRelativeTimeLocales, &encoded).is_err());
}
