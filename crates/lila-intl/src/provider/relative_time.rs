use super::*;
use crate::intl_services::{
    filter_service_locales, format_relative_time_input, match_service_locale,
    relative_time_available_locales, IntlServiceKind, RelativeTimeLocaleQuery,
};
use crate::{
    FormatRelativeTime, IntlServiceError, RelativeTimeFormatRequest, RelativeTimeLocaleRequest,
    RelativeTimeLocaleResult, RelativeTimeParts, ResolveRelativeTimeLocale,
    ResolvedRelativeTimeFormat,
};

impl IntlOperationProvider<ResolveRelativeTimeLocale> for EmbeddedIntlProvider {
    fn execute(
        &self,
        request: RelativeTimeLocaleRequest,
    ) -> Result<RelativeTimeLocaleResult, IntlServiceError> {
        let available = relative_time_available_locales()?;
        match request.query() {
            RelativeTimeLocaleQuery::SupportedLocales => {
                Ok(RelativeTimeLocaleResult::SupportedLocales(
                    filter_service_locales(request.requested(), &available, request.matcher())?,
                ))
            }
            RelativeTimeLocaleQuery::Resolve(options) => {
                let (locale, data_locale) = match_service_locale(
                    request.requested(),
                    &available,
                    self.identity.default_locale().as_str(),
                    request.matcher(),
                    IntlServiceKind::RelativeTimeFormat,
                )?;
                Ok(RelativeTimeLocaleResult::Resolved(
                    ResolvedRelativeTimeFormat::from_provider(
                        locale,
                        data_locale,
                        options.clone(),
                    )?,
                ))
            }
        }
    }
}

impl IntlOperationProvider<FormatRelativeTime> for EmbeddedIntlProvider {
    fn execute(
        &self,
        request: RelativeTimeFormatRequest,
    ) -> Result<RelativeTimeParts, IntlServiceError> {
        format_relative_time_input(
            request.configuration(),
            request.unit(),
            request.input().clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::number_format::numeric::ObservedNumericInput;
    use crate::number_format::NumberingSystemOption;
    use crate::{
        CanonicalLocaleId, IntlOperationProvider, RelativeTimeNumberKind, RelativeTimeUnit,
        ServiceLocaleMatcher,
    };
    use crate::{RelativeTimeFormatOptions, RelativeTimeNumeric, RelativeTimeStyle};

    fn locale(source: &str) -> CanonicalLocaleId {
        CanonicalLocaleId::from_data(source).unwrap()
    }

    fn resolve(
        provider: &EmbeddedIntlProvider,
        requested: &str,
        numbering_system: Option<&str>,
        numeric: RelativeTimeNumeric,
    ) -> ResolvedRelativeTimeFormat {
        resolve_with_style(
            provider,
            requested,
            numbering_system,
            numeric,
            RelativeTimeStyle::Long,
        )
    }

    fn resolve_with_style(
        provider: &EmbeddedIntlProvider,
        requested: &str,
        numbering_system: Option<&str>,
        numeric: RelativeTimeNumeric,
        style: RelativeTimeStyle,
    ) -> ResolvedRelativeTimeFormat {
        let numbering_system =
            numbering_system.map(|source| NumberingSystemOption::parse(source).unwrap());
        let options =
            RelativeTimeFormatOptions::new(style, numeric).with_numbering_system(numbering_system);
        match <EmbeddedIntlProvider as IntlOperationProvider<ResolveRelativeTimeLocale>>::execute(
            provider,
            RelativeTimeLocaleRequest::resolve(
                vec![locale(requested)].into_boxed_slice(),
                ServiceLocaleMatcher::Lookup,
                options,
            ),
        )
        .unwrap()
        {
            RelativeTimeLocaleResult::Resolved(configuration) => configuration,
            RelativeTimeLocaleResult::SupportedLocales(_) => unreachable!(),
        }
    }

    #[test]
    fn locale_inventory_is_the_generated_relative_time_profile() {
        let available = relative_time_available_locales().unwrap();
        assert_eq!(available.len(), 12);
        assert!(!available.contains(&"root"));
        assert!(available.contains(&"pl"));
        assert!(available.contains(&"pl-PL"));
        let provider = EmbeddedIntlProvider::new().unwrap();
        let result =
            <EmbeddedIntlProvider as IntlOperationProvider<ResolveRelativeTimeLocale>>::execute(
                &provider,
                RelativeTimeLocaleRequest::supported_locales(
                    vec![locale("ja-JP"), locale("zz")].into_boxed_slice(),
                    ServiceLocaleMatcher::Lookup,
                ),
            )
            .unwrap();
        assert_eq!(
            result,
            RelativeTimeLocaleResult::SupportedLocales(vec![locale("ja-JP")].into_boxed_slice())
        );
    }

    #[test]
    fn polish_relative_time_uses_pinned_patterns_and_number_grouping() {
        let provider = EmbeddedIntlProvider::new().unwrap();
        let cases = [
            (RelativeTimeStyle::Long, "za 2 godziny"),
            (RelativeTimeStyle::Short, "za 2 godz."),
            (RelativeTimeStyle::Narrow, "za 2 g."),
        ];
        for (style, expected) in cases {
            let result =
                <EmbeddedIntlProvider as IntlOperationProvider<FormatRelativeTime>>::execute(
                    &provider,
                    RelativeTimeFormatRequest::new(
                        resolve_with_style(
                            &provider,
                            "pl-PL",
                            None,
                            RelativeTimeNumeric::Always,
                            style,
                        ),
                        RelativeTimeUnit::Hour,
                        ObservedNumericInput::NumberShortestDecimal("2".into()),
                    ),
                )
                .unwrap();
            assert_eq!(
                result
                    .0
                    .iter()
                    .map(|part| part.value.as_ref())
                    .collect::<String>(),
                expected,
            );
        }

        let result = <EmbeddedIntlProvider as IntlOperationProvider<FormatRelativeTime>>::execute(
            &provider,
            RelativeTimeFormatRequest::new(
                resolve(&provider, "pl-PL", None, RelativeTimeNumeric::Always),
                RelativeTimeUnit::Day,
                ObservedNumericInput::NumberShortestDecimal("1000".into()),
            ),
        )
        .unwrap();
        assert_eq!(
            result
                .0
                .iter()
                .map(|part| part.value.as_ref())
                .collect::<String>(),
            "za 1000 dni",
        );
    }

    #[test]
    fn numbering_system_extension_and_option_precedence_are_resolved() {
        let provider = EmbeddedIntlProvider::new().unwrap();
        let extension = resolve(&provider, "en-u-nu-arab", None, RelativeTimeNumeric::Always);
        assert_eq!(extension.locale().as_str(), "en-u-nu-arab");
        assert_eq!(extension.numbering_system(), "arab");

        let override_value = resolve(
            &provider,
            "en-u-nu-latn",
            Some("arab"),
            RelativeTimeNumeric::Always,
        );
        assert_eq!(override_value.locale().as_str(), "en");
        assert_eq!(override_value.numbering_system(), "arab");
    }

    #[test]
    fn formatting_uses_resolved_digits_and_preserves_negative_zero_direction() {
        let provider = EmbeddedIntlProvider::new().unwrap();
        let configuration = resolve(&provider, "en", Some("arab"), RelativeTimeNumeric::Always);
        let parts = <EmbeddedIntlProvider as IntlOperationProvider<FormatRelativeTime>>::execute(
            &provider,
            RelativeTimeFormatRequest::new(
                configuration,
                RelativeTimeUnit::Day,
                ObservedNumericInput::NumberShortestDecimal("2".into()),
            ),
        )
        .unwrap();
        let output = parts
            .0
            .iter()
            .map(|part| part.value.as_ref())
            .collect::<String>();
        assert!(output.contains('٢'));
        assert!(output.contains("in "));

        let past = <EmbeddedIntlProvider as IntlOperationProvider<FormatRelativeTime>>::execute(
            &provider,
            RelativeTimeFormatRequest::new(
                resolve(&provider, "en", None, RelativeTimeNumeric::Always),
                RelativeTimeUnit::Day,
                ObservedNumericInput::NegativeZero,
            ),
        )
        .unwrap();
        let output = past
            .0
            .iter()
            .map(|part| part.value.as_ref())
            .collect::<String>();
        assert!(output.ends_with(" ago"));

        let automatic = resolve(&provider, "en", None, RelativeTimeNumeric::Auto);
        let today = <EmbeddedIntlProvider as IntlOperationProvider<FormatRelativeTime>>::execute(
            &provider,
            RelativeTimeFormatRequest::new(
                automatic,
                RelativeTimeUnit::Day,
                ObservedNumericInput::NegativeZero,
            ),
        )
        .unwrap();
        assert_eq!(today.0[0].value.as_ref(), "today");
        assert_eq!(RelativeTimeNumberKind::NegativeZero.wire_code(), 3);
    }

    #[test]
    fn fractional_plural_category_uses_the_rounded_displayed_number() {
        let provider = EmbeddedIntlProvider::new().unwrap();
        let result = <EmbeddedIntlProvider as IntlOperationProvider<FormatRelativeTime>>::execute(
            &provider,
            RelativeTimeFormatRequest::new(
                resolve(&provider, "en", None, RelativeTimeNumeric::Always),
                RelativeTimeUnit::Day,
                ObservedNumericInput::NumberShortestDecimal("0.9996".into()),
            ),
        )
        .unwrap();
        let output = result
            .0
            .iter()
            .map(|part| part.value.as_ref())
            .collect::<String>();
        assert_eq!(output, "in 1 day");
    }

    #[test]
    fn mismatched_pattern_data_locale_is_rejected_instead_of_using_root() {
        let resolved = ResolvedRelativeTimeFormat::from_resolved(
            locale("en"),
            locale("zz"),
            RelativeTimeFormatOptions::new(RelativeTimeStyle::Long, RelativeTimeNumeric::Always),
            "latn",
        );
        assert_eq!(resolved, Err(IntlServiceError::InvalidLocale));
    }
}
