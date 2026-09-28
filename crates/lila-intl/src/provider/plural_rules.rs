use super::*;
use std::sync::OnceLock;

use crate::intl_services::{
    execute_plural_category_with_profiles, filter_service_locales, match_service_locale,
    IntlServiceKind, PluralRulesLocaleQuery, ResolvedPluralRules,
};
use crate::{
    PluralCategoryRequest, PluralRulesLocaleRequest, PluralRulesLocaleResult,
    ResolvePluralRulesLocale, SelectPluralCategory,
};
use icu_locale::LocaleFallbacker;
use icu_plurals::provider::{PluralsCardinalV1, PluralsOrdinalV1, PluralsRangesV1};
use icu_provider::buf::{AsDeserializingBufferProvider, DeserializingBufferProvider};
use icu_provider::{DataError, DataMarker, DataProvider, DataRequest, DataResponse};
use icu_provider_adapters::fallback::LocaleFallbackProvider;
use icu_provider_blob::BlobDataProvider;

/// All three plural markers come from the full pinned CLDR 47 export. ICU4X's
/// compiled subset omits valid rules for locales such as Manx and Cornish.
pub(crate) struct PluralRulesDataProvider;

type PluralDeserializingProvider = DeserializingBufferProvider<'static, BlobDataProvider>;

static PLURAL_DATA_PROVIDER: OnceLock<Result<BlobDataProvider, DataError>> = OnceLock::new();

fn plural_data_provider() -> Result<&'static BlobDataProvider, DataError> {
    match PLURAL_DATA_PROVIDER.get_or_init(|| {
        BlobDataProvider::try_new_from_static_blob(include_bytes!(
            "plural_rules/generated/plurals.postcard"
        ))
    }) {
        Ok(provider) => Ok(provider),
        Err(error) => Err(*error),
    }
}

fn load_plural_data<M>(request: DataRequest) -> Result<DataResponse<M>, DataError>
where
    M: DataMarker,
    PluralDeserializingProvider: DataProvider<M>,
{
    let blob_provider = plural_data_provider()?;
    let provider = LocaleFallbackProvider::new(
        blob_provider.as_deserializing(),
        LocaleFallbacker::new().static_to_owned(),
    );
    DataProvider::<M>::load(&provider, request)
}

macro_rules! delegate_plural_data {
    ($marker:ty) => {
        impl DataProvider<$marker> for PluralRulesDataProvider {
            fn load(&self, request: DataRequest) -> Result<DataResponse<$marker>, DataError> {
                load_plural_data(request)
            }
        }
    };
}

delegate_plural_data!(PluralsCardinalV1);
delegate_plural_data!(PluralsOrdinalV1);
delegate_plural_data!(PluralsRangesV1);

impl IntlOperationProvider<ResolvePluralRulesLocale> for EmbeddedIntlProvider {
    fn execute(
        &self,
        request: PluralRulesLocaleRequest,
    ) -> Result<PluralRulesLocaleResult, IntlServiceError> {
        let available = self.numbers.available_locales();
        match request.query() {
            PluralRulesLocaleQuery::SupportedLocales => {
                Ok(PluralRulesLocaleResult::SupportedLocales(
                    filter_service_locales(request.requested(), available, request.matcher())?,
                ))
            }
            PluralRulesLocaleQuery::Resolve(options) => {
                let (locale, data_locale) = match_service_locale(
                    request.requested(),
                    available,
                    self.identity.default_locale().as_str(),
                    request.matcher(),
                    IntlServiceKind::PluralRules,
                )?;
                Ok(PluralRulesLocaleResult::Resolved(
                    ResolvedPluralRules::from_provider(locale, data_locale, options.clone())?,
                ))
            }
        }
    }
}

impl IntlOperationProvider<SelectPluralCategory> for EmbeddedIntlProvider {
    fn execute(
        &self,
        request: PluralCategoryRequest,
    ) -> Result<crate::PluralCategory, IntlServiceError> {
        execute_plural_category_with_profiles(request, self.numbers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::number_format::numeric::ObservedNumericInput;
    use crate::number_format::options::{
        FractionDigitCount, FractionDigitRange, FractionPrecision, Grouping, IntegerDigitCount,
        Notation, NumberFormatOptions, NumberStyle, Precision, RoundingMode, SignDisplay,
        TrailingZeroDisplay,
    };
    use crate::{
        CanonicalLocaleId, IntlOperationProvider, IntlServiceError, PluralRulesOptions,
        PluralRulesPrecision, PluralRulesType, ResolvedPluralRules,
    };

    fn number(value: &str) -> ObservedNumericInput {
        ObservedNumericInput::NumberShortestDecimal(value.to_owned().into_boxed_str())
    }

    fn resolved(notation: Notation) -> ResolvedPluralRules {
        let locale = CanonicalLocaleId::from_data("en").unwrap();
        let options = PluralRulesOptions::from_number_format_options(
            PluralRulesType::Cardinal,
            NumberFormatOptions {
                style: NumberStyle::Decimal,
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
            },
        )
        .unwrap();
        ResolvedPluralRules::from_provider(locale.clone(), locale, options).unwrap()
    }

    fn resolved_fraction(
        rule_type: PluralRulesType,
        notation: Notation,
        maximum_fraction_digits: u8,
    ) -> ResolvedPluralRules {
        let locale = CanonicalLocaleId::from_data("en").unwrap();
        let options = PluralRulesOptions::from_number_format_options(
            rule_type,
            NumberFormatOptions {
                style: NumberStyle::Decimal,
                notation,
                minimum_integer_digits: IntegerDigitCount::new(1).unwrap(),
                precision: Precision::Fraction(FractionPrecision::Range(
                    FractionDigitRange::new(
                        FractionDigitCount::new(0).unwrap(),
                        FractionDigitCount::new(maximum_fraction_digits).unwrap(),
                    )
                    .unwrap(),
                )),
                rounding_mode: RoundingMode::HalfExpand,
                trailing_zero_display: TrailingZeroDisplay::Auto,
                grouping: Grouping::Auto,
                sign_display: SignDisplay::Auto,
            },
        )
        .unwrap();
        ResolvedPluralRules::from_provider(locale.clone(), locale, options).unwrap()
    }

    #[test]
    fn plural_rounding_uses_unscaled_source_for_scientific_and_engineering() {
        let provider = EmbeddedIntlProvider::new().unwrap();
        for notation in [Notation::Scientific, Notation::Engineering] {
            let result =
                <EmbeddedIntlProvider as IntlOperationProvider<SelectPluralCategory>>::execute(
                    &provider,
                    PluralCategoryRequest::select(
                        resolved_fraction(PluralRulesType::Ordinal, notation, 0),
                        number("1000001"),
                    ),
                )
                .unwrap();
            assert_eq!(result.name(), "one");
        }
    }

    #[test]
    fn equal_unsigned_formatted_range_endpoints_return_start_category() {
        let provider = EmbeddedIntlProvider::new().unwrap();
        let configuration = resolved_fraction(PluralRulesType::Cardinal, Notation::Standard, 0);
        for (start, end) in [("1", "1"), ("1.1", "1.2"), ("-1", "1")] {
            let result =
                <EmbeddedIntlProvider as IntlOperationProvider<SelectPluralCategory>>::execute(
                    &provider,
                    PluralCategoryRequest::select_range(
                        configuration.clone(),
                        number(start),
                        number(end),
                    ),
                )
                .unwrap();
            assert_eq!(result.name(), "one", "range {start}..{end}");
        }
    }

    #[test]
    fn provider_uses_source_number_for_scientific_and_engineering() {
        let provider = EmbeddedIntlProvider::new().unwrap();
        for notation in [Notation::Scientific, Notation::Engineering] {
            let result =
                <EmbeddedIntlProvider as IntlOperationProvider<SelectPluralCategory>>::execute(
                    &provider,
                    PluralCategoryRequest::select(resolved(notation), number("1000000")),
                )
                .unwrap();
            assert_eq!(result.name(), "other");
        }
    }

    fn resolved_significant(rule_type: PluralRulesType) -> ResolvedPluralRules {
        let locale = CanonicalLocaleId::from_data("en").unwrap();
        let options = PluralRulesOptions::new(
            rule_type,
            1,
            PluralRulesPrecision::significant(1, 21).unwrap(),
        )
        .unwrap();
        ResolvedPluralRules::from_provider(locale.clone(), locale, options).unwrap()
    }

    fn string_numeric(value: &str) -> ObservedNumericInput {
        ObservedNumericInput::StringNumericLiteral(value.encode_utf16().collect())
    }

    fn bigint(value: &str) -> ObservedNumericInput {
        ObservedNumericInput::BigIntDecimal(value.into())
    }

    #[test]
    fn provider_uses_exact_plural_rule_inputs_for_strings_and_bigints() {
        let provider = EmbeddedIntlProvider::new().unwrap();
        let cardinal = resolved_significant(PluralRulesType::Cardinal);
        for (input, expected) in [
            (string_numeric("1.0000000000000000001"), "other"),
            (bigint("1"), "one"),
            (bigint("100000000000000000001"), "other"),
        ] {
            let result =
                <EmbeddedIntlProvider as IntlOperationProvider<SelectPluralCategory>>::execute(
                    &provider,
                    PluralCategoryRequest::select(cardinal.clone(), input),
                )
                .unwrap();
            assert_eq!(result.name(), expected);
        }

        let ordinal = resolved_significant(PluralRulesType::Ordinal);
        for (input, expected) in [
            (bigint("100000000000000000001"), "one"),
            (bigint("100000000000000000011"), "other"),
        ] {
            let result =
                <EmbeddedIntlProvider as IntlOperationProvider<SelectPluralCategory>>::execute(
                    &provider,
                    PluralCategoryRequest::select(ordinal.clone(), input),
                )
                .unwrap();
            assert_eq!(result.name(), expected);
        }

        let latvian = CanonicalLocaleId::from_data("lv").unwrap();
        let options = PluralRulesOptions::new(
            PluralRulesType::Cardinal,
            1,
            PluralRulesPrecision::significant(1, 21).unwrap(),
        )
        .unwrap();
        let configuration =
            ResolvedPluralRules::from_provider(latvian.clone(), latvian, options).unwrap();
        for value in ["0.00000000000000000001", "0.12345678901234567891"] {
            let result =
                <EmbeddedIntlProvider as IntlOperationProvider<SelectPluralCategory>>::execute(
                    &provider,
                    PluralCategoryRequest::select(configuration.clone(), string_numeric(value)),
                )
                .unwrap();
            assert_eq!(result.name(), "one");
        }
    }

    #[test]
    fn provider_range_parses_both_exact_inputs_before_nan_rejection() {
        let provider = EmbeddedIntlProvider::new().unwrap();
        let result =
            <EmbeddedIntlProvider as IntlOperationProvider<SelectPluralCategory>>::execute(
                &provider,
                PluralCategoryRequest::select_range(
                    resolved_significant(PluralRulesType::Cardinal),
                    string_numeric("1"),
                    bigint("2"),
                ),
            )
            .unwrap();
        assert_eq!(result.name(), "other");

        let result = <EmbeddedIntlProvider as IntlOperationProvider<SelectPluralCategory>>::execute(
            &provider,
            PluralCategoryRequest::select_range(
                resolved_significant(PluralRulesType::Cardinal),
                string_numeric("not a number"),
                bigint("2"),
            ),
        );
        assert_eq!(result, Err(IntlServiceError::InvalidOption));
    }

    #[test]
    fn provider_resolves_descending_ranges_without_rejecting_them() {
        let provider = EmbeddedIntlProvider::new().unwrap();
        let result = <EmbeddedIntlProvider as IntlOperationProvider<SelectPluralCategory>>::execute(
            &provider,
            PluralCategoryRequest::select_range(
                resolved(Notation::Standard),
                number("2"),
                number("1"),
            ),
        );
        assert!(result.is_ok());
    }
}
