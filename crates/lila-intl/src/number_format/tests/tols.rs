use super::*;

#[test]
fn tols_exact_numbers_and_currency_keep_supplement_digits_and_measurement_text() {
    assert_eq!(
        scalar(
            "en-u-nu-tols",
            "1234567890",
            NumberFormatOptions {
                grouping: Grouping::Never,
                ..options()
            },
        )
        .to_text(),
        "𑷡𑷢𑷣𑷤𑷥𑷦𑷧𑷨𑷩𑷠"
    );
    let result = scalar(
        "en-u-nu-tols",
        "123.45",
        currency("USD", CurrencyDisplay::Code, CurrencySign::Standard),
    );
    assert_eq!(result.to_text(), "USD\u{a0}𑷡𑷢𑷣.𑷤𑷥");
    assert!(result
        .parts()
        .iter()
        .any(|part| part.kind() == NumberPartKind::Currency && part.text() == "USD"));
    assert_eq!(
        scalar("en-u-nu-tols", "-0.125", options()).to_text(),
        "-𑷠.𑷡𑷢𑷥"
    );
}

#[test]
fn tols_selected_profile_preserves_explicit_override_and_unicode_spacing_category() {
    let mut request = NumberLocaleRequest {
        requested: vec![canonical("en-US-u-nu-tols")].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
        numbering_system: None,
    };
    let resolved = resolve_number_locale(&request, profiles()).unwrap();
    assert_eq!(resolved.resolved().as_str(), "en-US-u-nu-tols");
    assert_eq!(resolved.numbering_system().name(), "tols");
    request.numbering_system = Some(NumberingSystemOption::parse("arab").unwrap());
    let resolved = resolve_number_locale(&request, profiles()).unwrap();
    assert_eq!(resolved.resolved().as_str(), "en-US");
    assert_eq!(resolved.numbering_system().name(), "arab");
    for digit in "𑷠𑷡𑷢𑷣𑷤𑷥𑷦𑷧𑷨𑷩".chars() {
        assert!(profiles().is_digit(digit));
        assert!(!profiles().is_letter(digit));
    }
    assert!(!profiles().is_digit('一'));
    assert!(profiles().is_digit('𝟏'));
}
