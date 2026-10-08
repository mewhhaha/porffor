use super::*;
use crate::number_format::embedded_number_profiles;

fn resolved(tag: &str) -> DecimalNumberingSystem {
    resolve_locale_numbering_systems(
        LocaleNumberingSystemsRequest::new(CanonicalLocaleId::from_data(tag).unwrap()).unwrap(),
        embedded_number_profiles().unwrap(),
    )
    .unwrap()
}

#[test]
fn base_language_uses_its_default_not_likely_region() {
    assert_eq!(resolved("ar").name(), "latn");
}

#[test]
fn explicit_base_region_selects_exact_number_profile() {
    assert_eq!(resolved("ar-EG").name(), "arab");
}

#[test]
fn variant_falls_back_to_number_format_prefix() {
    assert_eq!(resolved("bn-BD-fonipa").name(), "beng");
}

#[test]
fn unrelated_unicode_calendar_does_not_select_number_system() {
    assert_eq!(resolved("ar-u-ca-islamic").name(), "latn");
}

#[test]
fn region_override_does_not_select_number_format_defaults() {
    assert_eq!(resolved("ar-u-rg-egzzzz").name(), "latn");
}

#[test]
fn subdivision_does_not_select_number_format_defaults() {
    assert_eq!(resolved("ar-u-sd-egc").name(), "latn");
}

#[test]
fn explicit_base_region_survives_unrelated_region_override() {
    assert_eq!(resolved("ar-EG-u-rg-uszzzz").name(), "arab");
}

#[test]
fn private_nu_text_remains_opaque() {
    assert_eq!(resolved("ar-x-nu-arab").name(), "latn");
}

#[test]
fn unmatched_locale_returns_latn_without_provider_default_locale() {
    assert_eq!(resolved("qaa").name(), "latn");
}

#[test]
fn every_present_open_numbering_keyword_is_rejected_before_defaults() {
    for tag in [
        "en-u-nu-latn",
        "en-u-nu-arab",
        "en-u-nu-foobar",
        "en-u-nu-arab-foobar",
        "en-u-nu",
        "en-u-nu-true",
        "en-u-nu-true-abc",
        "en-u-attr-ca-islamic-nu",
        "en-u-nu-ca-islamic",
    ] {
        assert!(matches!(
            LocaleNumberingSystemsRequest::new(CanonicalLocaleId::from_data(tag).unwrap()),
            Err(LocaleNumberingSystemsError::ExplicitNumberingSystem)
        ));
    }
}

#[test]
fn request_consumes_the_unchanged_canonical_owner_and_clone_preserves_admission() {
    let locale = CanonicalLocaleId::from_data("ar-EG-u-ca-islamic-rg-uszzzz-x-nu-arab").unwrap();
    let request = LocaleNumberingSystemsRequest::new(locale.clone()).unwrap();
    assert_eq!(request.clone().into_locale(), locale);
    assert_eq!(request.into_locale(), locale);
}

#[test]
fn every_profile_default_is_one_immutable_admitted_name() {
    let profiles = embedded_number_profiles().unwrap();
    let names = profiles.numbering_systems().to_vec();
    assert_eq!(profiles.available_locales().len(), 1082);
    assert_eq!(names.len(), 78);
    for tag in profiles.available_locales() {
        let first = resolved(tag);
        let retained = first.clone();
        assert!(names.iter().any(|name| name.as_ref() == first.name()));
        assert!((3..=8).contains(&first.name().len()));
        assert!(first
            .name()
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit()));
        assert_eq!(retained, resolved(tag));
    }
    assert_eq!(profiles.numbering_systems(), names.as_slice());
}

#[test]
fn typed_provider_operation_uses_the_shared_number_profile() {
    use crate::{EmbeddedIntlProvider, IntlOperationProvider, LocaleNumberingSystemsOperation};

    let provider = EmbeddedIntlProvider::new().unwrap();
    let request = LocaleNumberingSystemsRequest::new(
        CanonicalLocaleId::from_data("ar-EG-u-rg-uszzzz").unwrap(),
    )
    .unwrap();
    let result =
        <EmbeddedIntlProvider as IntlOperationProvider<LocaleNumberingSystemsOperation>>::execute(
            &provider, request,
        )
        .unwrap();
    assert_eq!(result.name(), "arab");
}

#[test]
fn locale_only_kernel_grants_numbering_default_without_number_format_service() {
    use crate::{
        EmbeddedIntlProvider, IntlDataIdentity, IntlDataPlacement, IntlKernel,
        IntlOperationProvider, IntlProfilePlan, IntlProvider, IntlService, IntlServiceSet,
        LocaleNumberingSystemsOperation,
    };

    struct LocaleOnlyProvider {
        identity: IntlDataIdentity,
        inner: EmbeddedIntlProvider,
    }
    impl IntlProvider for LocaleOnlyProvider {
        fn identity(&self) -> &IntlDataIdentity {
            &self.identity
        }
    }
    impl IntlOperationProvider<LocaleNumberingSystemsOperation> for LocaleOnlyProvider {
        fn execute(
            &self,
            request: LocaleNumberingSystemsRequest,
        ) -> Result<DecimalNumberingSystem, LocaleNumberingSystemsError> {
            <EmbeddedIntlProvider as IntlOperationProvider<LocaleNumberingSystemsOperation>>::execute(
                &self.inner,
                request,
            )
        }
    }

    let inner = EmbeddedIntlProvider::new().unwrap();
    let profile =
        IntlProfilePlan::minimal(IntlServiceSet::EMPTY.with(IntlService::Locale)).unwrap();
    assert!(!profile.services().contains(IntlService::NumberFormat));
    let identity = IntlDataIdentity::new(
        profile,
        CanonicalLocaleId::from_data("en-US").unwrap(),
        IntlDataPlacement::External,
        inner.identity().digest(),
    );
    let provider = LocaleOnlyProvider {
        identity: identity.clone(),
        inner,
    };
    let kernel = IntlKernel::new(identity, provider).unwrap();
    let operation = kernel
        .operation::<LocaleNumberingSystemsOperation>()
        .unwrap();
    let request =
        LocaleNumberingSystemsRequest::new(CanonicalLocaleId::from_data("ar-EG").unwrap()).unwrap();
    assert_eq!(operation.execute(request).unwrap().name(), "arab");
}
