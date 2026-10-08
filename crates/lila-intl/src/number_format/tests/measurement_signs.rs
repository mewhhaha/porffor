use super::*;

#[test]
fn measurement_prefixes_wrap_the_signed_number_placeholder() {
    // The pinned Japanese/Korean unit parts tests require the sign after the
    // localized prefix. Exercise negative zero, fractions and positive signs
    // through the same native partition consumed by emitted NumberFormat.
    for (locale, prefix, suffix) in [
        ("ja-JP", "時速", " キロメートル"),
        ("ko-KR", "시속", "킬로미터"),
    ] {
        for (source, magnitude) in [("-987", "987"), ("-0.001", "0.001"), ("-0", "0")] {
            let result = scalar(
                locale,
                source,
                unit("kilometer-per-hour", UnitDisplay::Long),
            );
            assert_eq!(result.to_text(), format!("{prefix} -{magnitude}{suffix}"));
            let parts = result.parts();
            assert_eq!(
                (parts[0].kind(), parts[0].text()),
                (NumberPartKind::Unit, prefix)
            );
            assert_eq!(
                (parts[1].kind(), parts[1].text()),
                (NumberPartKind::Literal, " ")
            );
            assert_eq!(
                (parts[2].kind(), parts[2].text()),
                (NumberPartKind::MinusSign, "-")
            );
            assert_eq!(parts[3].kind(), NumberPartKind::Integer);
        }
        let result = scalar(
            locale,
            "987",
            NumberFormatOptions {
                sign_display: SignDisplay::Always,
                ..unit("kilometer-per-hour", UnitDisplay::Long)
            },
        );
        assert_eq!(result.to_text(), format!("{prefix} +987{suffix}"));
        let result = scalar(
            locale,
            "-987",
            NumberFormatOptions {
                sign_display: SignDisplay::Never,
                ..unit("kilometer-per-hour", UnitDisplay::Long)
            },
        );
        assert_eq!(result.to_text(), format!("{prefix} 987{suffix}"));
    }
}

#[test]
fn signed_range_keeps_measurement_affixes_shared_and_approximation_at_the_number() {
    let options = NumberFormatOptions {
        precision: fraction(0, 0),
        ..unit("celsius", UnitDisplay::Long)
    };
    let result = range("ja", "-3", "-5", options.clone());
    let retained: Vec<_> = result
        .parts()
        .iter()
        .filter(|part| matches!(part.kind_name(), "unit" | "minusSign" | "integer"))
        .map(|part| (part.kind_name(), part.text(), part.source()))
        .collect();
    assert_eq!(
        retained,
        [
            ("unit", "摂氏", RangePartSource::Shared),
            ("minusSign", "-", RangePartSource::Start),
            ("integer", "3", RangePartSource::Start),
            ("minusSign", "-", RangePartSource::End),
            ("integer", "5", RangePartSource::End),
            ("unit", "度", RangePartSource::Shared),
        ]
    );
    let approximate = range("ja", "-2.9", "-3.1", options);
    let kinds: Vec<_> = approximate
        .parts()
        .iter()
        .map(NumberRangePart::kind_name)
        .collect();
    assert_eq!(
        kinds,
        [
            "unit",
            "literal",
            "approximatelySign",
            "minusSign",
            "integer",
            "literal",
            "unit"
        ]
    );
    assert!(approximate
        .parts()
        .iter()
        .all(|part| part.source() == RangePartSource::Shared));
}

#[test]
fn literal_only_measurement_forms_keep_the_sign_when_the_number_is_omitted() {
    for identifier in ["meter", "meter-per-byte"] {
        let options = unit(identifier, UnitDisplay::Short);
        let positive = scalar("ar-u-nu-latn", "2", options.clone());
        let negative = scalar("ar-u-nu-latn", "-2", options);
        assert!(!positive
            .parts()
            .iter()
            .any(|part| part.kind() == NumberPartKind::Integer));
        assert!(!negative
            .parts()
            .iter()
            .any(|part| part.kind() == NumberPartKind::Integer));
        let signs: Vec<_> = negative
            .parts()
            .iter()
            .filter(|part| part.kind() == NumberPartKind::MinusSign)
            .collect();
        assert_eq!(signs.len(), 1, "{identifier}");
        let labels = |result: &ScalarNumberPartition| {
            result
                .parts()
                .iter()
                .filter(|part| part.kind() == NumberPartKind::Unit)
                .map(|part| part.text().to_owned())
                .collect::<Vec<_>>()
        };
        assert_eq!(labels(&negative), labels(&positive));
    }
}
