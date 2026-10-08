use super::*;
use crate::collator::embedded_collator_profiles;

fn names(tag: &str) -> Vec<String> {
    resolve_locale_collations(
        LocaleCollationsRequest::new(CanonicalLocaleId::from_data(tag).unwrap()).unwrap(),
        embedded_collator_profiles().unwrap(),
    )
    .unwrap()
    .names()
    .iter()
    .map(ToString::to_string)
    .collect()
}

#[test]
fn every_present_co_is_rejected_before_native_profile_selection() {
    for tag in [
        "en-u-co",
        "en-u-co-true",
        "en-u-co-foobar",
        "de-u-co-phonebk",
        "en-u-co-search",
        "en-u-co-standard",
        "en-u-attr-ca-gregory-co",
        "en-u-co-ca-gregory",
    ] {
        assert!(matches!(
            LocaleCollationsRequest::new(CanonicalLocaleId::from_data(tag).unwrap()),
            Err(LocaleCollationsError::ExplicitCollation)
        ));
    }
}

#[test]
fn private_co_text_does_not_become_a_unicode_keyword() {
    assert_eq!(names("de-x-co-search"), names("de"));
}

#[test]
fn checked_request_clone_retains_the_original_canonical_owner() {
    let locale = CanonicalLocaleId::from_data("de-DE-u-ca-gregory-rg-uszzzz-x-co-foobar").unwrap();
    let request = LocaleCollationsRequest::new(locale.clone()).unwrap();
    assert_eq!(request.clone().into_locale(), locale);
    assert_eq!(request.into_locale(), locale);
}

#[test]
fn unmatched_language_has_only_locale_independent_sort_preferences() {
    assert_eq!(names("qaa"), ["emoji", "eor"]);
    assert_eq!(names("qaa-US-u-rg-dezzzz"), ["emoji", "eor"]);
}

#[test]
fn lookup_prefix_uses_the_matched_german_profile() {
    let actual = names("de-DE-fonipa");
    assert_eq!(actual, names("de"));
    assert!(actual.iter().any(|name| name == "phonebk"));
}

#[test]
fn matched_locale_does_not_receive_the_global_sort_union() {
    let profiles = embedded_collator_profiles().unwrap();
    assert!(profiles
        .available_collations()
        .iter()
        .any(|name| name.as_ref() == "phonebk"));
    assert!(names("de").iter().any(|name| name == "phonebk"));
    assert!(!names("en").iter().any(|name| name == "phonebk"));
}

#[test]
fn unrelated_unicode_preferences_do_not_change_lookup_matching() {
    assert_eq!(names("de-u-rg-uszzzz-sd-usca"), names("de"));
    assert_eq!(names("de-u-ca-buddhist"), names("de"));
}

#[test]
fn all_admitted_locale_results_are_unique_sorted_sort_purpose_names() {
    let profiles = embedded_collator_profiles().unwrap();
    let union_before = profiles.available_collations().to_vec();
    for locale in profiles.available_locales() {
        let result = names(locale.as_str());
        assert!(result.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(result
            .iter()
            .all(|name| !matches!(name.as_str(), "standard" | "search" | "searchjl")));
        assert!(result.iter().all(|name| union_before
            .iter()
            .any(|admitted| admitted.as_ref() == name.as_str())));
    }
    assert_eq!(profiles.available_collations(), union_before.as_slice());
}

#[test]
fn typed_provider_operation_uses_the_locale_profile_owner() {
    use crate::{EmbeddedIntlProvider, IntlOperationProvider, LocaleCollationsOperation};
    let provider = EmbeddedIntlProvider::new().unwrap();
    let request =
        LocaleCollationsRequest::new(CanonicalLocaleId::from_data("de").unwrap()).unwrap();
    let actual =
        <EmbeddedIntlProvider as IntlOperationProvider<LocaleCollationsOperation>>::execute(
            &provider, request,
        )
        .unwrap();
    assert!(actual.names().iter().any(|name| name.as_ref() == "phonebk"));
}

#[test]
fn locale_profile_requires_collation_data_without_granting_collator_service() {
    use crate::{IntlDataCapability, IntlProfilePlan, IntlService, IntlServiceSet};
    let profile =
        IntlProfilePlan::minimal(IntlServiceSet::EMPTY.with(IntlService::Locale)).unwrap();
    assert!(profile
        .capabilities()
        .contains(IntlDataCapability::Collation));
    assert!(!profile.services().contains(IntlService::Collator));
}
