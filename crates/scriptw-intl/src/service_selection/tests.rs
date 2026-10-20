use super::*;
use crate::*;

#[test]
fn sparse_number_admission_rejects_an_equal_digest_with_a_different_locale_owner() {
    let profile = IntlDataProfile::Custom(CustomProfileId::parse("number-locale-owner").unwrap());
    let locale = LocaleDataImage::for_profile(profile.clone()).unwrap();
    let foreign = LocaleDataImage::from_bytes(locale.bytes()).unwrap();
    assert_eq!(locale.digest(), foreign.digest());
    let lists = ListDataImage::for_profile(profile.clone(), &locale).unwrap();
    let numbers = NumberProfilesDataImage::for_profile(profile, &foreign).unwrap();
    assert!(!numbers.uses_locale(&locale));
    let selection =
        CheckedIntlServiceSelection::new(IntlServiceSet::EMPTY.with(IntlService::NumberFormat))
            .unwrap();
    assert!(EmbeddedIntlProvider::with_selected_data_images(
        selection,
        locale,
        Some(lists),
        None,
        Some(numbers),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .is_err());
}

#[test]
fn service_owner_derives_actual_frame_dependencies_and_rejects_empty_or_foreign_wire_bits() {
    let list =
        CheckedIntlServiceSelection::new(IntlServiceSet::EMPTY.with(IntlService::ListFormat))
            .unwrap();
    assert_eq!(
        list.components().iter().collect::<Vec<_>>(),
        [IntlDataComponent::Locale, IntlDataComponent::List]
    );
    let relative = CheckedIntlServiceSelection::new(
        IntlServiceSet::EMPTY.with(IntlService::RelativeTimeFormat),
    )
    .unwrap();
    assert_eq!(
        relative.components().iter().collect::<Vec<_>>(),
        [
            IntlDataComponent::Locale,
            IntlDataComponent::List,
            IntlDataComponent::Number,
            IntlDataComponent::RelativeTime
        ]
    );
    assert!(!relative.permits(IntlService::NumberFormat));
    assert!(!relative.permits(IntlService::PluralRules));
    for &service in IntlService::ALL {
        let selection =
            CheckedIntlServiceSelection::new(IntlServiceSet::EMPTY.with(service)).unwrap();
        assert_eq!(
            CheckedIntlServiceSelection::from_wire(selection.wire()).unwrap(),
            selection
        );
        assert!(selection.components().contains(IntlDataComponent::Locale));
    }
    assert!(CheckedIntlServiceSelection::new(IntlServiceSet::EMPTY).is_err());
    for bits in [0, 1 << 10, u16::MAX] {
        assert!(CheckedIntlServiceSelection::from_wire(bits).is_err());
    }
    let all = CheckedIntlServiceSelection::new(IntlServiceSet::ALL).unwrap();
    assert_eq!(
        all.components().iter().collect::<Vec<_>>(),
        IntlDataComponent::ALL
    );
    for component in IntlDataComponent::ALL {
        assert_eq!(
            IntlDataComponent::from_code(component.code()),
            Some(component)
        );
        assert!(!component.section_name().is_empty());
    }
    assert_eq!(IntlDataComponent::from_code(12), None);
}

#[test]
fn retained_relative_number_foundation_cannot_authorize_public_number_plural_or_wire_reminting() {
    let profile = IntlDataProfile::Custom(CustomProfileId::parse("relative-only-service").unwrap());
    let locale = LocaleDataImage::for_profile(profile.clone()).unwrap();
    let lists = ListDataImage::for_profile(profile.clone(), &locale).unwrap();
    let numbers = NumberProfilesDataImage::for_profile(profile.clone(), &locale).unwrap();
    let relative = RelativeTimeDataImage::for_profile(profile, &locale, &numbers).unwrap();
    let selection = CheckedIntlServiceSelection::new(
        IntlServiceSet::EMPTY.with(IntlService::RelativeTimeFormat),
    )
    .unwrap();
    let provider = EmbeddedIntlProvider::with_selected_data_images(
        selection,
        locale,
        Some(lists),
        None,
        Some(numbers),
        None,
        None,
        Some(relative),
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert!(provider.number_profiles().is_some());
    assert!(provider.display_names_profiles().is_none());
    let request = number_format::NumberLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data("en-US").unwrap()].into_boxed_slice(),
        matcher: number_format::options::LocaleMatcher::Lookup,
        numbering_system: None,
    };
    let resolved =
        <EmbeddedIntlProvider as IntlOperationProvider<ResolveRelativeTimeLocale>>::execute(
            &provider,
            request.clone(),
        )
        .unwrap();
    assert_eq!(resolved.resolved().as_str(), "en-US");
    assert!(matches!(
        <EmbeddedIntlProvider as IntlOperationProvider<ResolveNumberLocale>>::execute(
            &provider, request
        ),
        Err(NumberFormatOperationError::UnavailableService(
            IntlService::NumberFormat
        ))
    ));
    let lookup = LookupNamedTimeZoneRequest::new(TimeZoneId::parse("UTC").unwrap());
    assert!(matches!(
        <EmbeddedIntlProvider as IntlOperationProvider<LookupNamedTimeZone>>::execute(
            &provider, lookup
        ),
        Err(NamedTimeZoneLookupError::UnavailableService(_))
    ));
    let kernel = IntlKernel::new(provider.identity().clone(), provider).unwrap();
    assert!(kernel.operation::<ResolveRelativeTimeLocale>().is_ok());
    assert!(kernel.operation::<CanonicalizeLocale>().is_ok());
    assert_eq!(
        kernel
            .operation::<ResolveNumberLocale>()
            .unwrap_err()
            .unavailable_service(),
        Some(IntlService::NumberFormat)
    );
    assert_eq!(
        kernel
            .operation::<ResolvePluralLocale>()
            .unwrap_err()
            .unavailable_service(),
        Some(IntlService::PluralRules)
    );
    assert!(matches!(
        kernel.decode_number_format_request(&[]),
        Err(NumberWireError::UnavailableService(
            IntlService::NumberFormat
        ))
    ));
    assert!(matches!(
        kernel.decode_plural_select_request(&[]),
        Err(PluralWireError::Operation(
            PluralRulesOperationError::UnavailableService(IntlService::PluralRules)
        ))
    ));
}

#[test]
fn sparse_native_provider_rejects_missing_extra_foreign_foundations_and_partial_noncustom_profiles()
{
    let services =
        CheckedIntlServiceSelection::new(IntlServiceSet::EMPTY.with(IntlService::ListFormat))
            .unwrap();
    let profile = IntlDataProfile::Custom(CustomProfileId::parse("list-only-native").unwrap());
    let locale = LocaleDataImage::for_profile(profile.clone()).unwrap();
    let lists = ListDataImage::for_profile(profile.clone(), &locale).unwrap();
    assert!(EmbeddedIntlProvider::with_selected_data_images(
        services.clone(),
        locale.clone(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None
    )
    .is_err());
    let extra = NumberProfilesDataImage::for_profile(profile, &locale).unwrap();
    assert!(EmbeddedIntlProvider::with_selected_data_images(
        services.clone(),
        locale.clone(),
        Some(lists.clone()),
        None,
        Some(extra),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None
    )
    .is_err());
    let other = LocaleDataImage::for_profile(IntlDataProfile::Custom(
        CustomProfileId::parse("foreign-list-only").unwrap(),
    ))
    .unwrap();
    let foreign = ListDataImage::for_profile(other.profile().clone(), &other).unwrap();
    assert!(EmbeddedIntlProvider::with_selected_data_images(
        services.clone(),
        locale.clone(),
        Some(foreign),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None
    )
    .is_err());
    for profile in [IntlDataProfile::Minimal, IntlDataProfile::Conformance] {
        let locale = LocaleDataImage::for_profile(profile.clone()).unwrap();
        let lists = ListDataImage::for_profile(profile, &locale).unwrap();
        assert!(EmbeddedIntlProvider::with_selected_data_images(
            services.clone(),
            locale,
            Some(lists),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None
        )
        .is_err());
    }
    let provider = EmbeddedIntlProvider::with_selected_data_images(
        services,
        locale,
        Some(lists),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    let request = ListLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data("fr").unwrap()].into_boxed_slice(),
        matcher: number_format::options::LocaleMatcher::Lookup,
    };
    assert!(
        <EmbeddedIntlProvider as IntlOperationProvider<ResolveListLocale>>::execute(
            &provider, request
        )
        .is_ok()
    );
}
