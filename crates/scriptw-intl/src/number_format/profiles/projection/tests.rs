use super::*;
use crate::number_format::numeric::{
    normalize_numeric_input, round_decimal, NumericLimits, ObservedNumericInput, RoundingSettings,
};
use crate::number_format::options::*;
use crate::number_format::{
    resolve_number_locale, NumberFormatConfiguration, NumberLocaleRequest, NumberingSystemOption,
    PartitionLimits,
};
use crate::number_operation::{format_number_parts_operation, NumberFormatRequest};
use crate::plural_rules::PluralType;
use crate::CanonicalLocaleId;

fn names(values: &[&str]) -> Vec<Box<str>> {
    values.iter().map(|value| (*value).into()).collect()
}

fn options(style: NumberStyle, notation: Notation) -> NumberFormatOptions {
    NumberFormatOptions {
        style,
        notation,
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
    }
}

fn format(
    owner: &Arc<NumberProfiles>,
    locale: &str,
    system: &str,
    options: NumberFormatOptions,
) -> String {
    let locale = resolve_number_locale(
        &NumberLocaleRequest {
            requested: vec![CanonicalLocaleId::from_data(locale).unwrap()].into_boxed_slice(),
            matcher: LocaleMatcher::Lookup,
            numbering_system: Some(NumberingSystemOption::parse(system).unwrap()),
        },
        owner,
    )
    .unwrap();
    format_number_parts_operation(
        NumberFormatRequest {
            configuration: NumberFormatConfiguration { locale, options },
            input: ObservedNumericInput::StringNumericLiteral(
                "-1234567.5".encode_utf16().collect(),
            ),
        },
        owner,
        &PartitionLimits::HOST_ABI,
    )
    .unwrap()
    .to_text()
}

#[test]
fn complete_typed_encoder_reproduces_the_exact_pinned_binary() {
    let source = crate::number_image::pinned_number_source().unwrap();
    let encoded = encode_full(&source).unwrap();
    assert_eq!(
        encoded.as_slice(),
        include_bytes!("../../../../data/number-cldr-47/profiles.bin")
    );
    let decoded = read::decode(&encoded, &source.profiles().locale_data).unwrap();
    assert_eq!(decoded.locales.len(), 1082);
    assert_eq!(decoded.systems.len(), 78);
}

#[test]
fn selected_numbering_records_keep_original_defaults_and_global_digits_on_the_same_owner() {
    let id = crate::CustomProfileId::parse("numbering-closure").unwrap();
    let locale =
        crate::LocaleDataImage::for_profile(crate::IntlDataProfile::Custom(id.clone())).unwrap();
    let lists = crate::ListDataImage::for_profile(locale.profile().clone(), &locale).unwrap();
    let locales = [
        crate::LocaleId::parse("fr").unwrap(),
        crate::LocaleId::parse("ar-EG").unwrap(),
    ];
    let systems = [NumberingSystemOption::parse("deva").unwrap()];
    let currencies = [CurrencyCode::parse("EUR").unwrap()];
    let image = crate::NumberProfilesDataImage::for_custom_numbering_projection(
        &id,
        Some(&locales),
        Some(&currencies),
        &systems,
        None,
        None,
        &locale,
        &lists,
    )
    .unwrap();
    let projected = image.profiles();
    let baseline = crate::NumberProfilesDataImage::for_custom_data_projection(
        &id,
        Some(&locales),
        &currencies,
        None,
        None,
        &locale,
        &lists,
    )
    .unwrap();
    let old = baseline.profiles();
    assert_eq!(projected.system_names, old.system_names);
    assert_eq!(projected.systems.len(), 78);
    for (actual, source) in projected.systems.iter().zip(old.systems.iter()) {
        assert_eq!(actual.name, source.name);
        assert_eq!(actual.digits, source.digits);
    }
    for row in projected.profiles.iter() {
        for (index, record) in row.numbering.iter().enumerate() {
            assert_eq!(
                record.is_some(),
                index == usize::from(row.default_numbering)
                    || projected.system_names[index].as_ref() == "deva"
            );
        }
    }
    assert!(projected.numbering_profiles.len() < old.numbering_profiles.len());
    assert!(projected.symbols.len() < old.symbols.len());
    assert!(image.bytes().len() < baseline.bytes().len());
    for (name, default) in [("fr", "latn"), ("en-US", "latn"), ("ar-EG", "arab")] {
        let request = NumberLocaleRequest {
            requested: vec![CanonicalLocaleId::from_data(name).unwrap()].into_boxed_slice(),
            matcher: LocaleMatcher::Lookup,
            numbering_system: Some(NumberingSystemOption::parse("beng").unwrap()),
        };
        let selected = resolve_number_locale(&request, &projected).unwrap();
        assert_eq!(selected.numbering_system().name(), default);
        assert_eq!(selected.resolved().as_str(), name);
        assert!(crate::number_format::ResolvedNumberLocale::from_resolved(
            CanonicalLocaleId::from_data(name).unwrap(),
            CanonicalLocaleId::from_data(name).unwrap(),
            "beng",
            &projected
        )
        .is_err());
        for system in ["deva", default] {
            for style in [
                NumberStyle::Decimal,
                NumberStyle::Currency {
                    code: currencies[0].clone(),
                    display: CurrencyDisplay::Name,
                    sign: CurrencySign::Standard,
                },
            ] {
                for notation in [Notation::Standard, Notation::Compact(CompactDisplay::Long)] {
                    let config = options(style.clone(), notation);
                    assert_eq!(
                        format(&projected, name, system, config.clone()),
                        format(&old, name, system, config)
                    );
                }
            }
        }
    }
    let fields = format(
        &projected,
        "fr",
        "deva",
        options(NumberStyle::Decimal, Notation::Standard),
    );
    assert!(fields
        .chars()
        .any(|value| ('\u{0966}'..='\u{096f}').contains(&value)));
    let default = crate::number_format::locale_default_numbering_system(
        crate::LocaleNumberingSystemsRequest::new(CanonicalLocaleId::from_data("bn-BD").unwrap())
            .unwrap(),
        &projected,
    )
    .unwrap();
    assert_eq!(default.name(), "beng");
    assert_eq!(
        projected.required_relative_time_locales().unwrap().len(),
        14
    );
    assert_eq!(projected.required_duration_locales().unwrap().len(), 15);
    let reminted =
        crate::NumberProfilesDataImage::from_bytes(image.bytes(), &locale, &lists).unwrap();
    assert_eq!(
        reminted.numbering_system_selection(),
        Some(systems.as_slice())
    );
    drop(image);
    drop(locale);
    drop(lists);
    assert_eq!(
        format(
            &reminted.profiles(),
            "fr",
            "deva",
            options(NumberStyle::Decimal, Notation::Standard)
        ),
        fields
    );
}

#[test]
fn currency_projection_removes_real_labels_overrides_and_unreachable_tables_without_pruning_globals(
) {
    let id = crate::CustomProfileId::parse("currency-closure").unwrap();
    let locale =
        crate::LocaleDataImage::for_profile(crate::IntlDataProfile::Custom(id.clone())).unwrap();
    let lists = crate::ListDataImage::for_profile(locale.profile().clone(), &locale).unwrap();
    let currencies = [
        CurrencyCode::parse("EUR").unwrap(),
        CurrencyCode::parse("JPY").unwrap(),
    ];
    let locales = [
        crate::LocaleId::parse("de").unwrap(),
        crate::LocaleId::parse("fr").unwrap(),
    ];
    let image = crate::NumberProfilesDataImage::for_custom_data_projection(
        &id,
        Some(&locales),
        &currencies,
        None,
        None,
        &locale,
        &lists,
    )
    .unwrap();
    let projected = image.profiles();
    let source = crate::number_image::pinned_number_source().unwrap();
    let old = source.profiles();
    for set in &projected.currency_sets {
        assert!(set
            .0
            .iter()
            .all(|row| [*b"EUR", *b"JPY"].contains(&row.code)));
    }
    for numbering in &projected.numbering_profiles {
        assert!(numbering
            .currency_overrides
            .iter()
            .all(|row| [*b"EUR", *b"JPY"].contains(&row.code)));
    }
    assert_eq!(
        projected
            .reachable_currency_codes()
            .collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from([*b"EUR", *b"JPY"])
    );
    assert!(projected.texts.len() < old.texts.len());
    assert!(projected.string_choices.len() < old.string_choices.len());
    assert_eq!(projected.system_names, old.system_names);
    assert_eq!(projected.unit_names, old.unit_names);
    for code in ["JPY", "USD", "BHD", "ZZZ"] {
        let code = CurrencyCode::parse(code).unwrap();
        assert_eq!(
            projected.currency_fractions().digits(&code),
            old.currency_fractions().digits(&code)
        );
    }
    for name in ["de", "fr", "en-US"] {
        for currency in ["EUR", "JPY"] {
            for display in [
                CurrencyDisplay::Symbol,
                CurrencyDisplay::NarrowSymbol,
                CurrencyDisplay::Code,
                CurrencyDisplay::Name,
            ] {
                let config = options(
                    NumberStyle::Currency {
                        code: CurrencyCode::parse(currency).unwrap(),
                        display,
                        sign: CurrencySign::Accounting,
                    },
                    Notation::Standard,
                );
                assert_eq!(
                    format(&projected, name, "latn", config.clone()),
                    format(old, name, "latn", config)
                );
            }
        }
        let config = options(
            NumberStyle::Currency {
                code: CurrencyCode::parse("USD").unwrap(),
                display: CurrencyDisplay::Symbol,
                sign: CurrencySign::Standard,
            },
            Notation::Standard,
        );
        assert!(format(&projected, name, "latn", config).contains("USD"));
        for style in [
            NumberStyle::Decimal,
            NumberStyle::Unit {
                identifier: UnitIdentifier::parse("meter").unwrap(),
                display: UnitDisplay::Long,
            },
        ] {
            let config = options(style, Notation::Standard);
            assert_eq!(
                format(&projected, name, "latn", config.clone()),
                format(old, name, "latn", config)
            );
        }
    }
    let reminted =
        crate::NumberProfilesDataImage::from_bytes(image.bytes(), &locale, &lists).unwrap();
    assert_eq!(reminted.bytes(), image.bytes());
    assert_eq!(reminted.currency_codes(), Some(currencies.as_slice()));
    for codes in [
        vec![],
        vec![currencies[0].clone(), currencies[0].clone()],
        vec![CurrencyCode::parse("ZZZ").unwrap()],
    ] {
        assert!(crate::NumberProfilesDataImage::for_custom_data_projection(
            &id, None, &codes, None, None, &locale, &lists
        )
        .is_err());
    }
}

#[test]
fn selected_tables_keep_currency_units_compact_and_all_global_numeric_data() {
    let source = crate::number_image::pinned_number_source().unwrap();
    let locales = names(&["de", "en-US", "fr"]);
    let encoded = encode_selected(&source, &locales).unwrap();
    assert!(encoded.len() < include_bytes!("../../../../data/number-cldr-47/profiles.bin").len());
    let projected = Arc::new(read::decode(&encoded, &source.profiles().locale_data).unwrap());
    assert_eq!(projected.locales.as_ref(), locales.as_slice());
    assert!(projected.profile("ja").is_none());
    assert_eq!(projected.system_names, source.profiles().system_names);
    for (actual, original) in projected
        .systems
        .iter()
        .zip(source.profiles().systems.iter())
    {
        assert_eq!(actual.name, original.name);
        assert_eq!(actual.digits, original.digits);
    }
    assert_eq!(projected.unit_names, source.profiles().unit_names);
    assert_eq!(
        projected.fractions.default_digits(),
        source.profiles().fractions.default_digits()
    );
    assert_eq!(
        projected.fractions.overrides(),
        source.profiles().fractions.overrides()
    );
    for character in ['A', '一', '𝟏', '١', '\u{a0}', '\u{202f}', '\u{200f}'] {
        assert_eq!(
            projected.is_letter(character),
            source.profiles().is_letter(character)
        );
        assert_eq!(
            projected.is_digit(character),
            source.profiles().is_digit(character)
        );
        assert_eq!(
            projected.is_whitespace(character),
            source.profiles().is_whitespace(character)
        );
    }
    for locale in ["de", "en-US", "fr"] {
        for system in ["latn", "arab", "tols"] {
            let cases = [
                options(NumberStyle::Decimal, Notation::Standard),
                options(NumberStyle::Percent, Notation::Standard),
                options(
                    NumberStyle::Decimal,
                    Notation::Compact(CompactDisplay::Short),
                ),
                options(
                    NumberStyle::Decimal,
                    Notation::Compact(CompactDisplay::Long),
                ),
                options(
                    NumberStyle::Currency {
                        code: CurrencyCode::parse("EUR").unwrap(),
                        display: CurrencyDisplay::Name,
                        sign: CurrencySign::Standard,
                    },
                    Notation::Standard,
                ),
                options(
                    NumberStyle::Currency {
                        code: CurrencyCode::parse("USD").unwrap(),
                        display: CurrencyDisplay::Symbol,
                        sign: CurrencySign::Accounting,
                    },
                    Notation::Compact(CompactDisplay::Short),
                ),
                options(
                    NumberStyle::Unit {
                        identifier: UnitIdentifier::parse("meter-per-second").unwrap(),
                        display: UnitDisplay::Long,
                    },
                    Notation::Standard,
                ),
            ];
            for options in cases {
                assert_eq!(
                    format(&projected, locale, system, options.clone()),
                    format(source.profiles(), locale, system, options),
                    "{locale} {system}"
                );
            }
        }
    }
}

#[test]
fn projected_cardinal_ordinal_range_and_compact_associations_match_the_original_owner() {
    let source = crate::number_image::pinned_number_source().unwrap();
    let locales = names(&["ar", "ca", "de", "en-IN", "en-US", "es", "fr"]);
    let encoded = encode_selected(&source, &locales).unwrap();
    let projected = read::decode(&encoded, &source.profiles().locale_data).unwrap();
    let settings = RoundingSettings::from(&options(NumberStyle::Decimal, Notation::Standard));
    for locale in &locales {
        for kind in PluralType::ALL {
            assert_eq!(
                projected.plural_categories(locale, kind),
                source.profiles().plural_categories(locale, kind)
            );
            for start in CardinalCategory::ALL {
                for end in CardinalCategory::ALL {
                    assert_eq!(
                        projected.plural_range_category(locale, kind, start, end),
                        source
                            .profiles()
                            .plural_range_category(locale, kind, start, end)
                    );
                }
            }
            for value in [
                "0",
                "1",
                "2",
                "11",
                "101",
                "1.25",
                "1000000",
                "9007199254740993000001",
            ] {
                let normalized = normalize_numeric_input(
                    ObservedNumericInput::StringNumericLiteral(value.encode_utf16().collect()),
                    &NumericLimits::HOST_ABI,
                )
                .unwrap();
                let rounded = round_decimal(
                    normalized.finite_value().unwrap(),
                    &settings,
                    &NumericLimits::HOST_ABI,
                )
                .unwrap();
                for notation in [
                    Notation::Standard,
                    Notation::Compact(CompactDisplay::Short),
                    Notation::Compact(CompactDisplay::Long),
                ] {
                    assert_eq!(
                        projected
                            .select_plural_category(locale, kind, notation, &rounded)
                            .unwrap(),
                        source
                            .profiles()
                            .select_plural_category(locale, kind, notation, &rounded)
                            .unwrap(),
                        "{locale} {kind:?} {value} {notation:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn physical_projection_refuses_missing_default_duplicate_disordered_and_foreign_rows() {
    let source = crate::number_image::pinned_number_source().unwrap();
    for (locales, reason) in [
        (names(&[]), NumberProfileError::Order),
        (names(&["fr"]), NumberProfileError::MissingDefaultLocale),
        (names(&["en-US", "en-US"]), NumberProfileError::Order),
        (names(&["fr", "en-US"]), NumberProfileError::Order),
        (
            names(&["en-US", "fr-XX"]),
            NumberProfileError::InvalidLocale,
        ),
    ] {
        let error = encode_selected(&source, &locales).unwrap_err();
        assert_eq!(error.table, NumberProfileTable::Locales);
        assert_eq!(error.reason, reason);
    }
}
