use super::*;
use crate::display_names::{
    CheckedDisplayNamesConfiguration, DisplayNamesFallback, DisplayNamesLanguageDisplay,
    DisplayNamesSelection, DisplayNamesStyle, DisplayNamesType,
};
use crate::{
    decode_display_name_request, encode_display_name_request, CanonicalLocaleId, CustomProfileId,
};

fn request(
    image: &DisplayNamesDataImage,
    locale: &str,
    kind: DisplayNamesType,
    style: DisplayNamesStyle,
    fallback: DisplayNamesFallback,
    language: DisplayNamesLanguageDisplay,
    code: &[u16],
) -> DisplayNameRequest {
    DisplayNameRequest::new(
        CheckedDisplayNamesConfiguration::new(
            image
                .profiles()
                .admit(CanonicalLocaleId::from_data(locale).unwrap())
                .unwrap(),
            DisplayNamesSelection::from_options(kind, language),
            style,
            fallback,
        ),
        code.into(),
    )
    .unwrap()
}

#[test]
fn actual_native_image_consumes_every_locale_type_and_width_with_owned_data() {
    let locale = embedded_locale_data_image().unwrap();
    let original = embedded_display_names_data_image().unwrap();
    let bytes: Arc<[u8]> = original.bytes().as_ref().to_vec().into();
    let image = DisplayNamesDataImage::from_bytes(bytes.clone(), &locale).unwrap();
    drop(bytes);
    drop(locale);
    let profiles = image.profiles();
    let locales: Vec<_> = profiles
        .available_locales()
        .map(|locale| locale.as_str())
        .collect();
    assert_eq!(
        locales,
        [
            "ar",
            "ar-EG",
            "de",
            "en",
            "en-US",
            "fr",
            "hi",
            "it",
            "ja",
            "ko",
            "zh",
            "zh-Hans",
            "zh-Hans-CN"
        ]
    );
    for locale in locales {
        for &style in DisplayNamesStyle::ALL {
            for (kind, code) in [
                (DisplayNamesType::Language, "fr"),
                (DisplayNamesType::Region, "US"),
                (DisplayNamesType::Script, "Hans"),
                (DisplayNamesType::Currency, "USD"),
                (DisplayNamesType::Calendar, "gregory"),
                (DisplayNamesType::DateTimeField, "year"),
            ] {
                let request = request(
                    &image,
                    locale,
                    kind,
                    style,
                    DisplayNamesFallback::None,
                    DisplayNamesLanguageDisplay::Dialect,
                    &code.encode_utf16().collect::<Vec<_>>(),
                );
                assert!(!image
                    .display_name(&request)
                    .unwrap()
                    .name()
                    .unwrap()
                    .is_empty());
            }
        }
    }
    let request = request(
        &image,
        "en",
        DisplayNamesType::Language,
        DisplayNamesStyle::Long,
        DisplayNamesFallback::None,
        DisplayNamesLanguageDisplay::Standard,
        &"es-Cyrl-MX".encode_utf16().collect::<Vec<_>>(),
    );
    assert_eq!(
        image.display_name(&request).unwrap().name(),
        Some("Spanish (Cyrillic, Mexico)")
    );
}

#[test]
fn foreign_template_owner_is_rejected_before_lossless_code_validation() {
    let locale = embedded_locale_data_image().unwrap();
    let first = embedded_display_names_data_image().unwrap();
    let second = DisplayNamesDataImage::from_bytes(first.bytes(), &locale).unwrap();
    let request = request(
        &first,
        "en",
        DisplayNamesType::Language,
        DisplayNamesStyle::Long,
        DisplayNamesFallback::Code,
        DisplayNamesLanguageDisplay::Dialect,
        &[0xd800],
    );
    assert_eq!(
        first.display_name(&request),
        Err(DisplayNamesError::InvalidCode)
    );
    assert_eq!(
        second.display_name(&request),
        Err(DisplayNamesError::InvalidResolvedLocale)
    );
}

#[test]
fn selected_wire_admission_rebinds_templates_and_retains_real_locale_aliases() {
    let profile = IntlDataProfile::Custom(CustomProfileId::parse("display-owner-proof").unwrap());
    let locale = LocaleDataImage::for_profile(profile.clone()).unwrap();
    let first = DisplayNamesDataImage::for_profile(profile, &locale).unwrap();
    let second = DisplayNamesDataImage::from_bytes(first.bytes(), &locale).unwrap();
    let foreign = request(
        &first,
        "en",
        DisplayNamesType::Language,
        DisplayNamesStyle::Long,
        DisplayNamesFallback::Code,
        DisplayNamesLanguageDisplay::Dialect,
        &"IW".encode_utf16().collect::<Vec<_>>(),
    );
    assert_eq!(
        second.display_name(&foreign),
        Err(DisplayNamesError::InvalidResolvedLocale)
    );
    let encoded = encode_display_name_request(&foreign).unwrap();
    let selected = decode_display_name_request(&encoded, second.profiles_ref()).unwrap();
    assert_eq!(
        second.display_name(&selected).unwrap().name(),
        Some("Hebrew")
    );
    let reserved = request(
        &second,
        "en",
        DisplayNamesType::Language,
        DisplayNamesStyle::Long,
        DisplayNamesFallback::Code,
        DisplayNamesLanguageDisplay::Dialect,
        &"ZZZZZZZZ-aBcD-aB".encode_utf16().collect::<Vec<_>>(),
    );
    // The selected alias owner preserves a reserved 5–8-letter language rather
    // than treating its ICU parser placeholder as the semantic language.
    assert_eq!(
        second.display_name(&reserved).unwrap().name(),
        Some("zzzzzzzz-Abcd-AB")
    );
    let unknown = request(
        &second,
        "en",
        DisplayNamesType::Calendar,
        DisplayNamesStyle::Long,
        DisplayNamesFallback::None,
        DisplayNamesLanguageDisplay::Dialect,
        &"123-ABC".encode_utf16().collect::<Vec<_>>(),
    );
    assert_eq!(second.display_name(&unknown).unwrap().name(), None);
}

#[test]
fn self_consistent_altered_native_data_and_unavailable_profiles_are_rejected() {
    let locale = embedded_locale_data_image().unwrap();
    let image = embedded_display_names_data_image().unwrap();
    // A valid JSON whitespace change and a newly computed outer digest still
    // cannot claim the exact captured CLDR native payload.
    let mut changed = DISPLAY_NAMES_PROFILE.to_vec();
    changed.push(b' ');
    let bytes = DataImageEnvelope::encode(
        DataImageComponent::DisplayNames,
        &IntlDataProfile::Minimal,
        DISPLAY_NAMES_IMAGE_MARKERS,
        &changed,
    )
    .unwrap();
    assert!(matches!(
        DisplayNamesDataImage::from_bytes(bytes, &locale),
        Err(IntlDataImageError::Consumer(_))
    ));
    let named = LocaleDataImage::for_profile(IntlDataProfile::Custom(
        CustomProfileId::parse("foreign-display-foundation").unwrap(),
    ))
    .unwrap();
    assert!(matches!(
        DisplayNamesDataImage::from_bytes(image.bytes(), &named),
        Err(IntlDataImageError::Consumer(_))
    ));
    assert!(matches!(
        DisplayNamesDataImage::for_profile(IntlDataProfile::Conformance, &locale),
        Err(IntlDataImageError::Consumer(_))
    ));
}
