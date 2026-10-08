use super::*;
use crate::datetime_image::{payload_with_native, DateTimeDataImage, DATETIME_IMAGE_MARKERS};
use crate::image::{DataImageComponent, DataImageEnvelope};

#[test]
fn selected_named_zone_descriptor_and_localized_records_require_the_original_exact_closure() {
    let id = CustomProfileId::parse("date-zone-closure-damage").unwrap();
    let profile = IntlDataProfile::Custom(id.clone());
    let locale = LocaleDataImage::for_profile(profile.clone()).unwrap();
    let named = NamedTimeZoneDataImage::for_custom_projection(
        &id,
        &[crate::TimeZoneId::parse("America/New_York").unwrap()],
    )
    .unwrap();
    let payload = produce_named(&id, &locale, &named).unwrap();
    let changes: &[fn(&mut Descriptor, &mut serde_json::Value)] = &[
        |descriptor, _| descriptor.schema = 1,
        |descriptor, _| descriptor.named_time_zones = None,
        |descriptor, _| {
            descriptor
                .named_time_zones
                .as_mut()
                .unwrap()
                .push("Europe/Paris".into())
        },
        |descriptor, _| descriptor.named_time_zone_image_sha256[0] ^= 1,
        |_, raw| {
            raw["zone_name_pool"][0]["zones"]
                .as_array_mut()
                .unwrap()
                .clear()
        },
        |_, raw| {
            raw["zone_name_pool"][0]["metazones"]
                .as_array_mut()
                .unwrap()
                .clear()
        },
        |_, raw| raw["zone_geography"]["zones"][0]["identifier"] = "Changed/Zone".into(),
    ];
    for change in changes {
        let (mut descriptor, native) = split(&payload).unwrap();
        let mut raw: serde_json::Value = serde_json::from_slice(native).unwrap();
        change(&mut descriptor, &mut raw);
        let modified = frame(
            &descriptor,
            &serde_json::to_vec(&canonical_objects(raw)).unwrap(),
        )
        .unwrap();
        let bytes = DataImageEnvelope::encode(
            DataImageComponent::DateTime,
            &profile,
            DATETIME_IMAGE_MARKERS,
            &payload_with_native(&modified).unwrap(),
        )
        .unwrap();
        assert!(DateTimeDataImage::from_bytes(bytes, &locale, &named).is_err());
    }
}

#[test]
fn numbering_projection_rejects_missing_defaults_foreign_digit_rows_and_changed_selection() {
    let id = CustomProfileId::parse("date-numbering-damage").unwrap();
    let profile = IntlDataProfile::Custom(id.clone());
    let locale = LocaleDataImage::for_profile(profile.clone()).unwrap();
    let named = NamedTimeZoneDataImage::for_profile(profile.clone()).unwrap();
    let systems = [
        crate::number_format::NumberingSystemOption::parse("deva").unwrap(),
        crate::number_format::NumberingSystemOption::parse("arab").unwrap(),
    ];
    let locales = [
        LocaleId::parse("fr").unwrap(),
        LocaleId::parse("ar-EG").unwrap(),
    ];
    let payload = produce_numbering(&id, Some(&locales), None, &systems, &locale, &named).unwrap();
    let image = DateTimeDataImage::for_custom_numbering_projection(
        &id,
        Some(&locales),
        None,
        &systems,
        &locale,
        &named,
    )
    .unwrap();
    let reordered = DateTimeDataImage::for_custom_numbering_projection(
        &id,
        Some(&[locales[1].clone(), locales[0].clone()]),
        None,
        &[systems[1].clone(), systems[0].clone()],
        &locale,
        &named,
    )
    .unwrap();
    assert_eq!(image.bytes(), reordered.bytes());
    let changes: &[fn(&mut Descriptor, &mut serde_json::Value)] = &[
        |descriptor, _| descriptor.schema = 1,
        |descriptor, _| descriptor.numbering_systems = None,
        |descriptor, _| descriptor.numbering_systems.as_mut().unwrap().reverse(),
        |descriptor, _| descriptor.numbering_systems.as_mut().unwrap()[0] = "roman".into(),
        |descriptor, _| descriptor.named_time_zone_image_sha256[0] ^= 1,
        |_, raw| raw["numbering_systems"][0]["digits"] = "0123456789".into(),
        |_, raw| {
            let row = raw["locales"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|row| row["locale"] == "en-US")
                .unwrap();
            row["decimal_separators"]
                .as_array_mut()
                .unwrap()
                .retain(|pair| pair[0] != "latn");
        },
        |_, raw| {
            let row = raw["locales"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|row| row["locale"] == "en-US")
                .unwrap();
            row["minus_signs"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!(["beng", "-"]));
        },
    ];
    for change in changes {
        let (mut descriptor, native) = split(&payload).unwrap();
        let mut raw: serde_json::Value = serde_json::from_slice(native).unwrap();
        change(&mut descriptor, &mut raw);
        let changed = frame(
            &descriptor,
            &serde_json::to_vec(&canonical_objects(raw)).unwrap(),
        )
        .unwrap();
        let bytes = DataImageEnvelope::encode(
            DataImageComponent::DateTime,
            &profile,
            DATETIME_IMAGE_MARKERS,
            &payload_with_native(&changed).unwrap(),
        )
        .unwrap();
        assert!(DateTimeDataImage::from_bytes(bytes, &locale, &named).is_err());
    }
    for values in [
        vec![],
        vec![systems[0].clone(), systems[0].clone()],
        vec![crate::number_format::NumberingSystemOption::parse("roman").unwrap()],
    ] {
        assert!(DateTimeDataImage::for_custom_numbering_projection(
            &id, None, None, &values, &locale, &named
        )
        .is_err());
    }
}

#[test]
fn calendar_projection_requires_the_exact_canonical_selection_and_original_default_closure() {
    let id = CustomProfileId::parse("calendar-projection-damage").unwrap();
    let profile = IntlDataProfile::Custom(id.clone());
    let locale = LocaleDataImage::for_profile(profile.clone()).unwrap();
    let named = NamedTimeZoneDataImage::for_profile(profile.clone()).unwrap();
    let calendars = [DateTimeCalendar::Chinese, DateTimeCalendar::Japanese];
    for types in [
        &[][..],
        &[DateTimeCalendar::Chinese, DateTimeCalendar::Chinese][..],
    ] {
        assert!(
            DateTimeDataImage::for_custom_data_projection(&id, None, types, &locale, &named)
                .is_err()
        );
    }
    let payload = produce_data(&id, None, &calendars, &locale, &named).unwrap();
    let changes: &[fn(&mut Descriptor, &mut serde_json::Value)] = &[
        |descriptor, _| descriptor.schema = 1,
        |descriptor, _| descriptor.calendar_types = None,
        |descriptor, _| descriptor.calendar_types.as_mut().unwrap().reverse(),
        |descriptor, _| {
            descriptor
                .calendar_types
                .as_mut()
                .unwrap()
                .push("chinese".into())
        },
        |descriptor, _| descriptor.calendar_types.as_mut().unwrap()[0] = "gregorian".into(),
        |descriptor, _| descriptor.locale_image_sha256[0] ^= 1,
        |descriptor, _| descriptor.named_time_zone_image_sha256[0] ^= 1,
        |descriptor, _| descriptor.source_calendar_pool_ids.reverse(),
        |_, raw| {
            let row = raw["locales"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|row| row["locale"] == "en-US")
                .unwrap();
            row["calendar_refs"]
                .as_array_mut()
                .unwrap()
                .retain(|reference| reference[0] != "gregory");
        },
        |_, raw| raw["calendar_pool"][0]["names"][0]["value"] = "forged month name".into(),
        |_, raw| {
            let duplicate = raw["locales"][0]["calendar_refs"][0].clone();
            raw["locales"][0]["calendar_refs"]
                .as_array_mut()
                .unwrap()
                .push(duplicate);
        },
        |_, raw| raw["selector"]["calendar_identifiers"]["chinese"] = "gregorian".into(),
    ];
    for change in changes {
        let (mut descriptor, native) = split(&payload).unwrap();
        let mut raw: serde_json::Value = serde_json::from_slice(native).unwrap();
        change(&mut descriptor, &mut raw);
        let modified = frame(
            &descriptor,
            &serde_json::to_vec(&canonical_objects(raw)).unwrap(),
        )
        .unwrap();
        let bytes = DataImageEnvelope::encode(
            DataImageComponent::DateTime,
            &profile,
            DATETIME_IMAGE_MARKERS,
            &payload_with_native(&modified).unwrap(),
        )
        .unwrap();
        assert!(DateTimeDataImage::from_bytes(bytes, &locale, &named).is_err());
    }
    let other = LocaleDataImage::for_profile(IntlDataProfile::Custom(
        CustomProfileId::parse("different-calendar-foundation").unwrap(),
    ))
    .unwrap();
    assert!(
        DateTimeDataImage::for_custom_data_projection(&id, None, &calendars, &other, &named)
            .is_err()
    );
}
