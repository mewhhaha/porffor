use super::*;

fn profile(service: &str) -> IntlCompilationProfile {
    IntlCompilationProfile::CustomProjection(
        CustomIntlProfile::for_services(
            CustomProfileId::parse("cache-same-custom-id").unwrap(),
            &[service],
        )
        .unwrap(),
    )
}

#[test]
fn identical_profile_reuses_one_admitted_owner_and_complete_actual_bytes() {
    let cache = SelectedProfileCache::new();
    let profile = profile("ListFormat");
    let first = cache
        .select(&profile, SELECTED_PROFILE_CACHE_BYTES)
        .unwrap();
    let second = cache
        .select(&profile.clone(), SELECTED_PROFILE_CACHE_BYTES)
        .unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    let owner = IntlDataSelection::new(profile.clone());
    let repeated_owner = IntlDataSelection::new(profile.clone());
    // Other callers may evict the process-global one-entry cache between
    // these calls. Independent owners must preserve data, not pointer identity.
    assert_eq!(
        owner.selected().unwrap().identity(),
        repeated_owner.selected().unwrap().identity()
    );
    assert_eq!(
        owner.selected().unwrap().export_bytes().unwrap(),
        repeated_owner.selected().unwrap().export_bytes().unwrap()
    );
    let uncached = SelectedIntlDataBundle::build(&profile).unwrap();
    assert_eq!(first.identity(), uncached.identity());
    assert_eq!(
        first.export_bytes().unwrap(),
        uncached.export_bytes().unwrap()
    );
    let admitted =
        SelectedIntlDataBundle::from_export_bytes(&first.export_bytes().unwrap()).unwrap();
    assert_eq!(
        admitted.export_bytes().unwrap(),
        first.export_bytes().unwrap()
    );
}

#[test]
fn same_custom_name_cannot_reuse_different_public_services_or_frames() {
    let cache = SelectedProfileCache::new();
    let first_profile = profile("ListFormat");
    let second_profile = profile("Collator");
    let first = cache
        .select(&first_profile, SELECTED_PROFILE_CACHE_BYTES)
        .unwrap();
    let second = cache
        .select(&second_profile, SELECTED_PROFILE_CACHE_BYTES)
        .unwrap();
    assert!(!Arc::ptr_eq(&first, &second));
    assert_ne!(first.identity(), second.identity());
    assert_eq!(first.component_sections().len(), 2);
    assert_eq!(second.component_sections().len(), 2);
    assert_eq!(
        first.service_selection().unwrap().requested(),
        IntlServiceSet::EMPTY.with(IntlService::ListFormat)
    );
    assert_eq!(
        second.service_selection().unwrap().requested(),
        IntlServiceSet::EMPTY.with(IntlService::Collator)
    );
    assert!(cache
        .entry
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|(key, value)| key == &second_profile && Arc::ptr_eq(value, &second)));
    // Eviction cannot mutate the old caller's admitted graph.
    assert_eq!(
        first.export_bytes().unwrap(),
        SelectedIntlDataBundle::build(&first_profile)
            .unwrap()
            .export_bytes()
            .unwrap()
    );
}

#[test]
fn over_budget_selection_still_works_without_retaining_global_frames() {
    let cache = SelectedProfileCache::new();
    let profile = profile("ListFormat");
    let first = cache.select(&profile, 0).unwrap();
    assert!(cache.entry.lock().unwrap().is_none());
    let second = cache.select(&profile, 0).unwrap();
    assert!(!Arc::ptr_eq(&first, &second));
    assert_eq!(
        first.export_bytes().unwrap(),
        second.export_bytes().unwrap()
    );
}

#[test]
fn concurrent_identical_selections_keep_the_same_checked_owner() {
    let cache = SelectedProfileCache::new();
    let profile = profile("ListFormat");
    let values = std::thread::scope(|scope| {
        let left = scope.spawn(|| {
            cache
                .select(&profile, SELECTED_PROFILE_CACHE_BYTES)
                .unwrap()
        });
        let right = scope.spawn(|| {
            cache
                .select(&profile, SELECTED_PROFILE_CACHE_BYTES)
                .unwrap()
        });
        (left.join().unwrap(), right.join().unwrap())
    });
    assert!(Arc::ptr_eq(&values.0, &values.1));
    assert_eq!(values.0.identity(), values.1.identity());
}

fn list_projection(locale: &str) -> IntlCompilationProfile {
    IntlCompilationProfile::CustomProjection(
        CustomIntlProfile::new(
            CustomProfileId::parse("cache-same-custom-id").unwrap(),
            Some(&[locale]),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap()
        .with_services(&["ListFormat"])
        .unwrap(),
    )
}

#[test]
fn same_name_and_service_keep_distinct_actual_locale_projections() {
    let cache = SelectedProfileCache::new();
    let first = cache
        .select(&list_projection("fr"), SELECTED_PROFILE_CACHE_BYTES)
        .unwrap();
    let second = cache
        .select(&list_projection("ja"), SELECTED_PROFILE_CACHE_BYTES)
        .unwrap();
    assert!(!Arc::ptr_eq(&first, &second));
    assert_ne!(first.identity(), second.identity());
    assert_ne!(
        first.export_bytes().unwrap(),
        second.export_bytes().unwrap()
    );
    let locales = |bundle: &SelectedIntlDataBundle| {
        bundle
            .lists
            .as_ref()
            .unwrap()
            .profiles()
            .available_locales()
            .map(|locale| locale.as_str().to_owned())
            .collect::<Vec<_>>()
    };
    assert_eq!(locales(&first), ["en-US", "fr"]);
    assert_eq!(locales(&second), ["en-US", "ja"]);
}

#[test]
fn failed_selection_never_replaces_the_previous_checked_owner() {
    let cache = SelectedProfileCache::new();
    let valid = profile("ListFormat");
    let first = cache.select(&valid, SELECTED_PROFILE_CACHE_BYTES).unwrap();
    assert!(cache
        .select(&list_projection("zz-ZZ"), SELECTED_PROFILE_CACHE_BYTES)
        .is_err());
    assert!(Arc::ptr_eq(
        &first,
        &cache.select(&valid, SELECTED_PROFILE_CACHE_BYTES).unwrap()
    ));
}
