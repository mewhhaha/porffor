use super::*;
use crate::{CanonicalLocaleId, CustomProfileId, DateTimeHourCycle};

fn foundations() -> (LocaleDataImage, DateTimeDataImage) {
    (
        crate::embedded_locale_data_image().unwrap(),
        crate::embedded_date_time_data_image().unwrap(),
    )
}

#[test]
fn all_four_actual_tables_and_selected_calendar_kernels_enter_the_image() {
    let (locale, date_time) = foundations();
    let image = NativeLocaleInformationDataImage::for_profile(
        IntlDataProfile::Minimal,
        &locale,
        &date_time,
    )
    .unwrap();
    let envelope = DataImageEnvelope::decode(
        image.bytes(),
        DataImageComponent::NativeLocaleInformation,
        &MARKERS,
    )
    .unwrap();
    assert_eq!(admit_tables(envelope.blob()).unwrap(), TABLES);
    assert_eq!(
        TABLES.iter().map(|table| table.len()).sum::<usize>(),
        122_201
    );
    assert!(image.uses_foundations(&locale, &date_time.provider()));
    let calendars = image
        .resolve_calendars(
            LocaleCalendarsRequest::new(CanonicalLocaleId::from_data("th-TH").unwrap()).unwrap(),
        )
        .unwrap();
    assert_eq!(
        calendars
            .names()
            .iter()
            .map(|name| name.as_ref())
            .collect::<Vec<_>>(),
        ["buddhist", "gregory"]
    );
    let cycles = image
        .resolve_hour_cycles(
            LocaleHourCyclesRequest::new(CanonicalLocaleId::from_data("en-US").unwrap()).unwrap(),
        )
        .unwrap();
    assert!(cycles.cycles().contains(&DateTimeHourCycle::H12));
    assert_eq!(
        image
            .resolve_text(LocaleTextInfoRequest::new(
                CanonicalLocaleId::from_data("ar").unwrap()
            ))
            .unwrap(),
        Some(LocaleTextDirection::RightToLeft)
    );
    assert_eq!(
        image
            .resolve_week(LocaleWeekRequest::new(
                CanonicalLocaleId::from_data("en-US").unwrap()
            ))
            .unwrap()
            .first_day()
            .iso_number(),
        7
    );
}

#[test]
fn self_consistent_foreign_table_bytes_and_mixed_profiles_cannot_claim_pinned_data() {
    let (locale, date_time) = foundations();
    let mut payload = native_payload().unwrap();
    let last = payload.len() - 1;
    payload[last] ^= 1;
    let bytes = DataImageEnvelope::encode(
        DataImageComponent::NativeLocaleInformation,
        &IntlDataProfile::Minimal,
        &MARKERS,
        &payload,
    )
    .unwrap();
    assert!(matches!(
        NativeLocaleInformationDataImage::from_bytes(bytes, &locale, &date_time),
        Err(IntlDataImageError::Consumer(_))
    ));
    let custom = LocaleDataImage::for_profile(IntlDataProfile::Custom(
        CustomProfileId::parse("locale-info-control").unwrap(),
    ))
    .unwrap();
    assert!(matches!(
        NativeLocaleInformationDataImage::for_profile(
            IntlDataProfile::Minimal,
            &custom,
            &date_time
        ),
        Err(IntlDataImageError::Consumer(_))
    ));
    assert!(matches!(
        NativeLocaleInformationDataImage::for_profile(
            IntlDataProfile::Conformance,
            &locale,
            &date_time
        ),
        Err(IntlDataImageError::Consumer(_))
    ));
}

#[test]
fn identical_digests_do_not_substitute_foreign_locale_or_datetime_owners() {
    let (locale, date_time) = foundations();
    let foreign_locale = LocaleDataImage::from_bytes(locale.bytes()).unwrap();
    assert_eq!(foreign_locale.digest(), locale.digest());
    assert!(!foreign_locale.same_owner(&locale));
    assert!(matches!(
        NativeLocaleInformationDataImage::for_profile(
            IntlDataProfile::Minimal,
            &foreign_locale,
            &date_time
        ),
        Err(IntlDataImageError::Consumer(_))
    ));
    let named = crate::embedded_named_time_zone_data_image().unwrap();
    let foreign_date_time =
        DateTimeDataImage::from_bytes(date_time.bytes(), &locale, &named).unwrap();
    assert_eq!(foreign_date_time.digest(), date_time.digest());
    let image = NativeLocaleInformationDataImage::for_profile(
        IntlDataProfile::Minimal,
        &locale,
        &date_time,
    )
    .unwrap();
    assert!(!image.uses_foundations(&locale, &foreign_date_time.provider()));
    assert!(!image.uses_foundations(&foreign_locale, &date_time.provider()));
}

#[test]
fn dynamically_admitted_tables_and_foundations_survive_original_sources() {
    let locale =
        LocaleDataImage::from_bytes(crate::embedded_locale_data_image().unwrap().bytes()).unwrap();
    let named = crate::embedded_named_time_zone_data_image().unwrap();
    let date_time = DateTimeDataImage::from_bytes(
        crate::embedded_date_time_data_image().unwrap().bytes(),
        &locale,
        &named,
    )
    .unwrap();
    let source = NativeLocaleInformationDataImage::for_profile(
        IntlDataProfile::Minimal,
        &locale,
        &date_time,
    )
    .unwrap();
    let bytes = source.bytes();
    let retained =
        NativeLocaleInformationDataImage::from_bytes(bytes.clone(), &locale, &date_time).unwrap();
    drop((source, bytes, date_time, named, locale));
    assert_eq!(
        retained
            .resolve_week(LocaleWeekRequest::new(
                CanonicalLocaleId::from_data("en-u-rg-gbzzzz").unwrap()
            ))
            .unwrap()
            .first_day()
            .iso_number(),
        1
    );
    assert_eq!(
        retained
            .resolve_text(LocaleTextInfoRequest::new(
                CanonicalLocaleId::from_data("en-Arab").unwrap()
            ))
            .unwrap(),
        Some(LocaleTextDirection::RightToLeft)
    );
    assert_eq!(
        retained
            .resolve_calendars(
                LocaleCalendarsRequest::new(CanonicalLocaleId::from_data("en-US").unwrap())
                    .unwrap()
            )
            .unwrap()
            .names()
            .iter()
            .map(|name| name.as_ref())
            .collect::<Vec<_>>(),
        ["gregory"]
    );
}
