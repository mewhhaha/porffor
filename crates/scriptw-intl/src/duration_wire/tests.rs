use super::*;
use crate::number_format::{embedded_number_profiles_arc, numeric::NumericLimits};
use core::num::NonZeroU32;

fn profiles() -> &'static DurationProfiles {
    crate::embedded_duration_profiles().unwrap()
}
fn locale_request(locale: &str) -> NumberLocaleRequest {
    NumberLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data(locale).unwrap()].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
        numbering_system: None,
    }
}
fn partition_request(
    locale: &str,
    options: DurationOptions,
    fields: [f64; 10],
) -> DurationWireRequest {
    let locale = profiles()
        .resolve_locale(
            &locale_request(locale),
            &embedded_number_profiles_arc().unwrap(),
        )
        .unwrap();
    DurationWireRequest::Parts(
        DurationPartitionRequest::from_completed_number_fields(
            CheckedDurationConfiguration::new(locale, options).unwrap(),
            fields,
        )
        .unwrap(),
    )
}
fn decode(
    request: &DurationWireRequest,
    bytes: &[u8],
) -> Result<DurationWireRequest, DurationWireError> {
    decode_duration_request(
        request.operation(),
        bytes,
        profiles(),
        &embedded_number_profiles_arc().unwrap(),
        &PartitionLimits::HOST_ABI,
    )
}
fn run(request: DurationWireRequest) -> DurationResponse {
    match request.into_native() {
        DurationRequest::Resolve(request) => DurationResponse::Resolved(
            profiles()
                .resolve_locale(&request, &embedded_number_profiles_arc().unwrap())
                .unwrap(),
        ),
        DurationRequest::SupportedLocales(request) => {
            DurationResponse::SupportedLocales(profiles().supported_locales(request))
        }
        DurationRequest::Parts {
            configuration,
            record,
        } => DurationResponse::Parts(
            crate::duration_format::format_duration_parts(
                &configuration,
                &record,
                profiles(),
                &PartitionLimits::HOST_ABI,
            )
            .unwrap(),
        ),
    }
}

#[test]
fn duration_wire_admits_and_correlates_the_selected_template_owner() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let numbers = crate::embedded_number_profiles_data_image().unwrap();
    let lists = crate::embedded_list_data_image().unwrap();
    let selected = crate::DurationDataImage::from_bytes(
        crate::embedded_duration_data_image().unwrap().bytes(),
        &locale,
        &numbers,
        &lists,
    )
    .unwrap()
    .profiles();
    let request = partition_request("en", DurationOptions::default(), [0.0; 10]);
    let bytes = encode_duration_request(&request).unwrap();
    let decoded = decode_duration_request(
        request.operation(),
        &bytes,
        &selected,
        &numbers.profiles(),
        &PartitionLimits::HOST_ABI,
    )
    .unwrap();
    assert!(matches!(
        crate::execute_duration_request(
            request.clone().into_native(),
            &selected,
            &numbers.profiles(),
            &PartitionLimits::HOST_ABI,
        ),
        Err(DurationError::InvalidLocale)
    ));
    let response = crate::execute_duration_request(
        decoded.clone().into_native(),
        &selected,
        &numbers.profiles(),
        &PartitionLimits::HOST_ABI,
    )
    .unwrap();
    let bytes = encode_duration_response(&response).unwrap();
    assert!(matches!(
        decode_duration_response(
            &request,
            &bytes,
            &selected,
            &numbers.profiles(),
            &PartitionLimits::HOST_ABI,
        ),
        Err(DurationWireError::Rejected(DurationError::InvalidLocale))
    ));
    let response = decode_duration_response(
        &decoded,
        &bytes,
        &selected,
        &numbers.profiles(),
        &PartitionLimits::HOST_ABI,
    )
    .unwrap();
    assert!(matches!(response, DurationWireResponse::Parts(parts) if parts.parts().is_empty()));
}
fn word(bytes: &mut [u8], index: usize, value: u64) {
    bytes[index * 8..index * 8 + 8].copy_from_slice(&value.to_le_bytes());
}
fn parts_start(bytes: &[u8]) -> usize {
    let mut index = 16;
    for _ in 0..2 {
        let count = u64::from_le_bytes(bytes[index..index + 8].try_into().unwrap()) as usize;
        index += 8 + count;
    }
    index
}
#[test]
fn every_truncated_completed_record_and_wrong_header_is_malformed() {
    let request = partition_request("en", DurationOptions::default(), [0.0; 10]);
    let original = encode_duration_request(&request).unwrap();
    for end in 0..original.len() {
        assert!(matches!(
            decode(&request, &original[..end]),
            Err(DurationWireError::Malformed(_))
        ));
    }
    for (index, value) in [(0, 2), (1, 76 + 1), (1, 70)] {
        let mut bytes = original.clone();
        word(&mut bytes, index, value);
        assert!(matches!(
            decode(&request, &bytes),
            Err(DurationWireError::Malformed(_))
        ));
    }
}
#[test]
fn framing_faults_precede_completed_record_semantic_rejection() {
    let request = partition_request("en", DurationOptions::default(), [0.0; 10]);
    let mut bytes = encode_duration_request(&request).unwrap();
    let start = parts_start(&bytes) + DURATION_CONFIGURATION_WORDS * 8;
    bytes[start..start + 8].copy_from_slice(&f64::NAN.to_bits().to_le_bytes());
    assert!(matches!(
        decode(&request, &bytes),
        Err(DurationWireError::Rejected(DurationError::InvalidRecord))
    ));
    bytes.push(0);
    assert!(matches!(
        decode(&request, &bytes),
        Err(DurationWireError::Malformed("trailing bytes"))
    ));
}
#[test]
fn completed_field_bits_preserve_negative_zero_and_exact_large_subseconds() {
    for fields in [
        [-0.0; 10],
        [
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            2.0,
            3.0,
            9_007_199_254_740_991.0,
        ],
    ] {
        let request = partition_request(
            "en",
            DurationOptions {
                style: DurationStyle::Digital,
                ..Default::default()
            },
            fields,
        );
        let decoded = decode(&request, &encode_duration_request(&request).unwrap()).unwrap();
        let DurationWireRequest::Parts(decoded) = decoded else {
            panic!("partition request")
        };
        assert_eq!(decoded.number_bits(), &fields.map(f64::to_bits));
        if fields.iter().all(|field| *field == 0.0) {
            assert!(!decoded.record().negative());
        }
    }
}
#[test]
fn uniform_sign_and_normative_calendar_time_bounds_remain_semantic_errors() {
    let request = partition_request("en", DurationOptions::default(), [0.0; 10]);
    for (fields, expected) in [
        (
            [1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            DurationError::InvalidRecord,
        ),
        (
            [4_294_967_296.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            DurationError::InvalidBounds,
        ),
        (
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
            DurationError::InvalidBounds,
        ),
    ] {
        let mut bytes = encode_duration_request(&request).unwrap();
        let start = parts_start(&bytes) + DURATION_CONFIGURATION_WORDS * 8;
        for (index, value) in fields.iter().enumerate() {
            bytes[start + index * 8..start + index * 8 + 8]
                .copy_from_slice(&f64::to_bits(*value).to_le_bytes());
        }
        assert!(
            matches!(decode(&request, &bytes), Err(DurationWireError::Rejected(error)) if error == expected)
        );
    }
}
#[test]
fn effective_unit_domains_distinguish_reserved_bits_from_invalid_combinations() {
    let request = partition_request("en", DurationOptions::default(), [0.0; 10]);
    for (value, structural) in [(0x200_u64, true), (5, true), (3, false)] {
        let mut bytes = encode_duration_request(&request).unwrap();
        let start = parts_start(&bytes) + DurationConfigurationWord::Years.offset() as usize;
        bytes[start..start + 8].copy_from_slice(&value.to_le_bytes());
        let result = decode(&request, &bytes);
        if structural {
            assert!(matches!(result, Err(DurationWireError::Malformed(_))));
        } else {
            assert!(matches!(
                result,
                Err(DurationWireError::Rejected(DurationError::InvalidOptions))
            ));
        }
    }
}
#[test]
fn every_admitted_locale_style_has_one_canonical_checked_configuration_frame() {
    for locale in [
        "ar",
        "ar-EG",
        "de",
        "en",
        "en-US",
        "es",
        "fr",
        "hi",
        "it",
        "ja",
        "ko",
        "sr",
        "zh",
        "zh-Hans",
        "zh-Hans-CN",
    ] {
        for &style in DurationStyle::ALL {
            let request = partition_request(
                locale,
                DurationOptions {
                    style,
                    ..Default::default()
                },
                [0.0; 10],
            );
            let bytes = encode_duration_request(&request).unwrap();
            let decoded = decode(&request, &bytes).unwrap();
            assert_eq!(encode_duration_request(&decoded).unwrap(), bytes);
        }
    }
}
#[test]
fn resolved_numbering_proof_cannot_be_swapped_to_an_unsupported_owner() {
    let request = partition_request("en-u-nu-arab", DurationOptions::default(), [0.0; 10]);
    let bytes = encode_duration_request(&request).unwrap();
    assert!(decode(&request, &bytes).is_ok());
    let mut changed = bytes.clone();
    let position = changed
        .windows(4)
        .rposition(|slice| slice == b"arab")
        .unwrap();
    changed[position..position + 4].copy_from_slice(b"abcd");
    assert!(matches!(
        decode(&request, &changed),
        Err(DurationWireError::Malformed(
            "resolved locale/numbering association"
        ))
    ));
}
#[test]
fn supported_response_is_bound_to_requested_order_duplicates_and_parent_matching() {
    let request = DurationWireRequest::SupportedLocales(DurationSupportedLocalesRequest {
        requested: ["sr-Thai-RS", "es", "es", "zz"]
            .map(|locale| CanonicalLocaleId::from_data(locale).unwrap())
            .into(),
        matcher: LocaleMatcher::BestFit,
    });
    let response = run(request.clone());
    let bytes = encode_duration_response(&response).unwrap();
    let decoded = decode_duration_response(
        &request,
        &bytes,
        profiles(),
        &embedded_number_profiles_arc().unwrap(),
        &PartitionLimits::HOST_ABI,
    )
    .unwrap();
    let DurationWireResponse::SupportedLocales(locales) = decoded else {
        panic!("supported locales")
    };
    assert_eq!(
        locales
            .iter()
            .map(CanonicalLocaleId::as_str)
            .collect::<Vec<_>>(),
        ["sr-Thai-RS", "es", "es"]
    );
    let wrong = DurationResponse::SupportedLocales(
        vec![CanonicalLocaleId::from_data("es").unwrap()].into_boxed_slice(),
    );
    assert!(matches!(
        decode_duration_response(
            &request,
            &encode_duration_response(&wrong).unwrap(),
            profiles(),
            &embedded_number_profiles_arc().unwrap(),
            &PartitionLimits::HOST_ABI
        ),
        Err(DurationWireError::Malformed(
            "supported response/request association"
        ))
    ));
}
#[test]
fn genuine_serbian_literals_and_negative_zero_sign_survive_parts_and_normal_output() {
    for (locale, fields, expected) in [
        (
            "sr",
            [0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 0.0, 0.0, 0.0],
            "1.02.03",
        ),
        (
            "en",
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0],
            "-0:00:01",
        ),
    ] {
        let request = partition_request(
            locale,
            DurationOptions {
                style: DurationStyle::Digital,
                ..Default::default()
            },
            fields,
        );
        let response = run(request.clone());
        let bytes = encode_duration_response(&response).unwrap();
        let decoded = decode_duration_response(
            &request,
            &bytes,
            profiles(),
            &embedded_number_profiles_arc().unwrap(),
            &PartitionLimits::HOST_ABI,
        )
        .unwrap();
        let DurationWireResponse::Parts(parts) = decoded else {
            panic!("parts")
        };
        assert_eq!(parts.to_text().unwrap(), expected);
        assert_eq!(
            parts
                .parts()
                .iter()
                .filter(|part| part.unit().is_none())
                .map(DurationWirePart::text)
                .collect::<Vec<_>>(),
            if locale == "sr" {
                vec![".", "."]
            } else {
                vec![":", ":"]
            }
        );
        assert_eq!(
            parts
                .parts()
                .iter()
                .filter(|part| part.kind() == NumberPartKind::MinusSign)
                .count(),
            usize::from(locale == "en")
        );
    }
}
#[test]
fn decimal_and_unit_parts_require_labels_and_currency_parts_are_rejected() {
    let request = partition_request(
        "en",
        DurationOptions::default(),
        [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    );
    for (kind, unit) in [
        (NumberPartKind::Integer, None),
        (NumberPartKind::Currency, Some(DurationUnit::Year)),
    ] {
        let mut writer = Writer::new(DurationHostOp::Parts, true).unwrap();
        writer.word(1).unwrap();
        writer.word(kind.wire_code()).unwrap();
        writer.text("1").unwrap();
        writer.word(u64::from(unit.is_some())).unwrap();
        if let Some(unit) = unit {
            writer.word(unit.index() as u64).unwrap();
        }
        assert!(matches!(
            decode_duration_response(
                &request,
                &writer.finish(),
                profiles(),
                &embedded_number_profiles_arc().unwrap(),
                &PartitionLimits::HOST_ABI
            ),
            Err(DurationWireError::Malformed(_))
        ));
    }
}
#[test]
fn resource_limits_apply_to_cumulative_parts_and_locale_lists() {
    let limits = PartitionLimits::new(
        NumericLimits::HOST_ABI,
        NonZeroU32::new(3).unwrap(),
        NonZeroU32::new(1).unwrap(),
    );
    let request = DurationWireRequest::SupportedLocales(DurationSupportedLocalesRequest {
        requested: ["en", "es"]
            .map(|locale| CanonicalLocaleId::from_data(locale).unwrap())
            .into(),
        matcher: LocaleMatcher::Lookup,
    });
    assert!(matches!(
        decode_duration_request(
            request.operation(),
            &encode_duration_request(&request).unwrap(),
            profiles(),
            &embedded_number_profiles_arc().unwrap(),
            &limits
        ),
        Err(DurationWireError::Resource("locale count"))
    ));
    let request = partition_request(
        "en",
        DurationOptions {
            style: DurationStyle::Digital,
            ..Default::default()
        },
        [0.0; 10],
    );
    let mut writer = Writer::new(DurationHostOp::Parts, true).unwrap();
    writer.word(2).unwrap();
    for text in ["12", "34"] {
        writer.word(NumberPartKind::Integer.wire_code()).unwrap();
        writer.text(text).unwrap();
        writer.word(1).unwrap();
        writer.word(DurationUnit::Second.index() as u64).unwrap();
    }
    let limits = PartitionLimits::new(
        NumericLimits::HOST_ABI,
        NonZeroU32::new(3).unwrap(),
        NonZeroU32::new(2).unwrap(),
    );
    assert!(matches!(
        decode_duration_response(
            &request,
            &writer.finish(),
            profiles(),
            &embedded_number_profiles_arc().unwrap(),
            &limits
        ),
        Err(DurationWireError::Resource("part text extent"))
    ));
}

#[test]
fn resolve_response_binds_checked_two_digit_hours_boolean_to_genuine_profile() {
    let request = DurationWireRequest::Resolve(locale_request("sr"));
    let response = run(request.clone());
    let original = encode_duration_response(&response).unwrap();
    let DurationWireResponse::Resolved(locale) = decode_duration_response(
        &request,
        &original,
        profiles(),
        &embedded_number_profiles_arc().unwrap(),
        &PartitionLimits::HOST_ABI,
    )
    .unwrap() else {
        panic!("resolved")
    };
    assert!(!locale.two_digit_hours());
    for value in [1_u64, 2] {
        let mut changed = original.clone();
        let end = changed.len();
        changed[end - 8..].copy_from_slice(&value.to_le_bytes());
        assert!(matches!(
            decode_duration_response(
                &request,
                &changed,
                profiles(),
                &embedded_number_profiles_arc().unwrap(),
                &PartitionLimits::HOST_ABI
            ),
            Err(DurationWireError::Malformed(_))
        ));
    }
}
