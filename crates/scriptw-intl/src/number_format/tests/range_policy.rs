use super::*;

// Test262 aa55200d: intl402/NumberFormat/prototype/formatRange/en-US.js.
#[test]
fn pinned_en_us_format_range_literals() {
    let rounded_currency = NumberFormatOptions {
        precision: fraction(0, 0),
        ..currency("USD", CurrencyDisplay::Symbol, CurrencySign::Standard)
    };
    assert_eq!(
        range("en-US", "3", "5", rounded_currency.clone()).to_text(),
        "$3 – $5"
    );
    assert_eq!(
        range("en-US", "2.9", "3.1", rounded_currency).to_text(),
        "~$3"
    );
    let signed_currency = NumberFormatOptions {
        sign_display: SignDisplay::Always,
        ..currency("USD", CurrencyDisplay::Symbol, CurrencySign::Standard)
    };
    assert_eq!(
        range("en-US", "2.9", "3.1", signed_currency).to_text(),
        "+$2.90–3.10"
    );
    assert_eq!(
        range(
            "en-US",
            "987654321987654321",
            "987654321987654322",
            options()
        )
        .to_text(),
        "987,654,321,987,654,321–987,654,321,987,654,322"
    );
}

// Test262 aa55200d: intl402/NumberFormat/prototype/formatRange/pt-PT.js.
#[test]
fn pinned_pt_pt_format_range_literals() {
    let rounded_currency = NumberFormatOptions {
        precision: fraction(0, 0),
        ..currency("EUR", CurrencyDisplay::Symbol, CurrencySign::Standard)
    };
    assert_eq!(
        range("pt-PT", "3", "5", rounded_currency.clone()).to_text(),
        "3 - 5\u{00a0}€"
    );
    assert_eq!(
        range("pt-PT", "2.9", "3.1", rounded_currency).to_text(),
        "~3\u{00a0}€"
    );
    let signed_currency = NumberFormatOptions {
        sign_display: SignDisplay::Always,
        ..currency("EUR", CurrencyDisplay::Symbol, CurrencySign::Standard)
    };
    assert_eq!(
        range("pt-PT", "2.9", "3.1", signed_currency).to_text(),
        "+2,90 - 3,10\u{00a0}€"
    );
    assert_eq!(
        range("pt-PT", "987654321987654321", "987654321987654322", options()).to_text(),
        "987\u{00a0}654\u{00a0}321\u{00a0}987\u{00a0}654\u{00a0}321 - 987\u{00a0}654\u{00a0}321\u{00a0}987\u{00a0}654\u{00a0}322"
    );
}

// Test262 aa55200d: intl402/NumberFormat/prototype/formatRangeToParts/en-US.js.
#[test]
fn pinned_en_us_format_range_parts_literals() {
    let options = NumberFormatOptions {
        precision: fraction(0, 0),
        ..currency("USD", CurrencyDisplay::Symbol, CurrencySign::Standard)
    };
    let rendered = range("en-US", "3", "5", options.clone());
    let actual: Vec<_> = rendered
        .parts()
        .iter()
        .map(|part| (part.kind_name(), part.text(), part.source()))
        .collect();
    assert_eq!(
        actual,
        [
            ("currency", "$", RangePartSource::Start),
            ("integer", "3", RangePartSource::Start),
            ("literal", " – ", RangePartSource::Shared),
            ("currency", "$", RangePartSource::End),
            ("integer", "5", RangePartSource::End),
        ]
    );
    for (start, end, integer) in [("1", "1", "1"), ("2.999", "3.001", "3")] {
        let rendered = range("en-US", start, end, options.clone());
        let actual: Vec<_> = rendered
            .parts()
            .iter()
            .map(|part| (part.kind_name(), part.text(), part.source()))
            .collect();
        assert_eq!(
            actual,
            [
                ("approximatelySign", "~", RangePartSource::Shared),
                ("currency", "$", RangePartSource::Shared),
                ("integer", integer, RangePartSource::Shared),
            ]
        );
    }
}
