use super::*;

fn independent_image() -> crate::DateTimeDataImage {
    crate::DateTimeDataImage::from_bytes(
        crate::embedded_date_time_data_image().unwrap().bytes(),
        &crate::embedded_locale_data_image().unwrap(),
        &crate::embedded_named_time_zone_data_image().unwrap(),
    )
    .unwrap()
}

#[test]
fn portable_exact_plans_reuse_only_their_admitted_owner_and_parts_remain_fresh() {
    let image = independent_image();
    let cold = independent_image();
    let provider = image.provider_ref();
    let selected = image
        .select_plan(request(
            "en-US",
            DateTimeStyleSelection::Components(date_components()),
        ))
        .unwrap();
    assert!(provider.plans.get(&selected.plan).is_none());
    let first_request = DateTimeFormatRequest {
        plan: selected.plan.clone(),
        input: date(2020, 1, 25),
    };
    let first = image.format_parts(first_request.clone()).unwrap();
    assert_eq!(first, cold.format_parts(first_request).unwrap());
    let reused = provider.plans.get(&selected.plan).unwrap();
    for input in [date(1999, 12, 31), date(2021, 2, 28), instant(0, 0)] {
        let input = DateTimeFormatRequest {
            plan: selected.plan.clone(),
            input,
        };
        assert_eq!(
            image.format_parts(input.clone()).unwrap(),
            cold.format_parts(input).unwrap()
        );
        assert!(std::sync::Arc::ptr_eq(
            &reused,
            &provider.plans.get(&selected.plan).unwrap()
        ));
    }
    let mut poisoned_response = first;
    poisoned_response.parts[0].value.clear();
    assert_eq!(
        image
            .format_parts(DateTimeFormatRequest {
                plan: selected.plan,
                input: date(2020, 1, 25),
            })
            .unwrap()
            .to_formatted_string(),
        "1/25/2020"
    );
}

#[test]
fn warm_cache_rejects_corruption_and_keeps_decode_kind_and_selection_error_order() {
    let image = independent_image();
    let provider = image.provider_ref();
    let selected = image
        .select_plan(request(
            "en-US",
            DateTimeStyleSelection::Components(date_components()),
        ))
        .unwrap();
    image
        .format_parts(DateTimeFormatRequest {
            plan: selected.plan.clone(),
            input: date(2020, 1, 25),
        })
        .unwrap();
    assert!(provider.plans.get(&selected.plan).is_some());
    let mut forged = Vec::new();
    for offset in [0, 8, 40, 72] {
        let mut bytes = selected.plan.as_bytes().to_vec();
        bytes[offset] ^= 1;
        forged.push(bytes);
    }
    forged.push(selected.plan.as_bytes()[..103].to_vec());
    forged.push([selected.plan.as_bytes(), &[0]].concat());
    for bytes in forged {
        let plan = EncodedDateTimePlan::from_bytes(bytes);
        for _ in 0..2 {
            assert!(matches!(
                image.format_parts(DateTimeFormatRequest {
                    plan: plan.clone(),
                    input: date(2020, 1, 25),
                }),
                Err(DateTimeFormatError::InvalidPlan(_))
            ));
            assert!(matches!(
                image.format_range_parts(DateTimeRangeRequest {
                    plan: plan.clone(),
                    start: date(2020, 1, 25),
                    end: instant(0, 0),
                }),
                Err(DateTimeFormatError::InvalidPlan(_))
            ));
            assert!(provider.plans.get(&plan).is_none());
        }
    }
    let mut invalid = request(
        "en-US",
        DateTimeStyleSelection::Components(date_components()),
    );
    invalid.required = DateTimeRequired::Time;
    invalid.defaults = DateTimeDefaults::Date;
    let bytes = [
        &selected.plan.as_bytes()[..104],
        invalid.encode().unwrap().as_slice(),
    ]
    .concat();
    let invalid = EncodedDateTimePlan::from_bytes(bytes);
    assert_eq!(
        image.format_range_parts(DateTimeRangeRequest {
            plan: invalid.clone(),
            start: date(2020, 1, 25),
            end: instant(0, 0),
        }),
        Err(DateTimeFormatError::InputKindMismatch)
    );
    assert!(matches!(
        image.format_parts(DateTimeFormatRequest {
            plan: invalid.clone(),
            input: date(2020, 1, 25),
        }),
        Err(DateTimeFormatError::InvalidRequest(_))
    ));
    assert!(provider.plans.get(&invalid).is_none());
}

#[test]
fn identity_equal_portable_plan_and_foreign_profiles_keep_separate_cache_authorities() {
    let first = independent_image();
    let same = independent_image();
    let profile = crate::IntlDataProfile::Custom(
        crate::CustomProfileId::parse("plan-cache-foreign").unwrap(),
    );
    let locale = crate::LocaleDataImage::for_profile(profile.clone()).unwrap();
    let zones = crate::NamedTimeZoneDataImage::for_profile(profile.clone()).unwrap();
    let foreign = crate::DateTimeDataImage::for_profile(profile, &locale, &zones).unwrap();
    let encoded = first
        .select_plan(request(
            "en-US",
            DateTimeStyleSelection::Components(date_components()),
        ))
        .unwrap()
        .plan;
    let input = DateTimeFormatRequest {
        plan: encoded.clone(),
        input: date(2020, 1, 25),
    };
    let expected = first.format_parts(input.clone()).unwrap();
    assert!(first.provider_ref().plans.get(&encoded).is_some());
    assert!(same.provider_ref().plans.get(&encoded).is_none());
    assert_eq!(same.format_parts(input.clone()).unwrap(), expected);
    assert!(!std::sync::Arc::ptr_eq(
        &first.provider_ref().plans.get(&encoded).unwrap(),
        &same.provider_ref().plans.get(&encoded).unwrap(),
    ));
    let foreign_encoded = foreign
        .select_plan(request(
            "en-US",
            DateTimeStyleSelection::Components(date_components()),
        ))
        .unwrap()
        .plan;
    let foreign_admission = foreign
        .provider_ref()
        .decode_plan(&foreign_encoded)
        .unwrap()
        .select()
        .unwrap();
    first.provider_ref().plans.remember(foreign_admission);
    assert!(first.provider_ref().plans.get(&foreign_encoded).is_none());
    assert!(first.provider_ref().plans.get(&encoded).is_some());
    for _ in 0..2 {
        assert!(matches!(
            foreign.format_parts(input.clone()),
            Err(DateTimeFormatError::InvalidPlan(_))
        ));
        assert!(foreign.provider_ref().plans.get(&encoded).is_none());
    }
}
