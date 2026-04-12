use crate::{
    Context, JsNativeError, JsResult, JsValue,
    builtins::{
        Array,
        intl::{
            Service,
            options::{IntlOptions, LocaleMatcher, coerce_options_to_object},
        },
        options::get_option,
    },
    context::icu::IntlProvider,
    js_string,
    object::JsObject,
};

use icu_locale::{LanguageIdentifier, Locale, LocaleCanonicalizer};
use icu_provider::{
    DataIdentifierBorrowed, DataLocale, DataMarker, DataMarkerAttributes, DataRequest,
    DataRequestMetadata, DryDataProvider,
};
use indexmap::IndexSet;

use tap::TapOptional;

const SUPPORTED_CALENDARS: &[&str] = &[
    "buddhist",
    "chinese",
    "coptic",
    "dangi",
    "ethioaa",
    "ethiopic",
    "gregory",
    "hebrew",
    "indian",
    "islamic-civil",
    "islamic-tbla",
    "islamic-umalqura",
    "iso8601",
    "japanese",
    "persian",
    "roc",
];

const SUPPORTED_COLLATIONS: &[&str] = &[
    "compat",
    "dict",
    "emoji",
    "eor",
    "phonebk",
    "pinyin",
    "searchjl",
    "stroke",
    "trad",
    "unihan",
    "zhuyin",
];

const SUPPORTED_HOUR_CYCLES: &[&str] = &["h11", "h12", "h23", "h24"];

const SUPPORTED_NUMBERING_SYSTEMS: &[&str] = &[
    "adlm",
    "ahom",
    "arab",
    "arabext",
    "bali",
    "beng",
    "bhks",
    "brah",
    "cakm",
    "cham",
    "deva",
    "diak",
    "fullwide",
    "gara",
    "gong",
    "gonm",
    "gujr",
    "gukh",
    "guru",
    "hanidec",
    "hmng",
    "hmnp",
    "java",
    "kali",
    "kawi",
    "khmr",
    "knda",
    "krai",
    "lana",
    "lanatham",
    "laoo",
    "latn",
    "lepc",
    "limb",
    "mathbold",
    "mathdbl",
    "mathmono",
    "mathsanb",
    "mathsans",
    "mlym",
    "modi",
    "mong",
    "mroo",
    "mtei",
    "mymr",
    "mymrepka",
    "mymrpao",
    "mymrshan",
    "mymrtlng",
    "nagm",
    "newa",
    "nkoo",
    "olck",
    "onao",
    "orya",
    "osma",
    "outlined",
    "rohg",
    "saur",
    "segment",
    "shrd",
    "sind",
    "sinh",
    "sora",
    "sund",
    "sunu",
    "takr",
    "talu",
    "tamldec",
    "tnsa",
    "telu",
    "thai",
    "tirh",
    "tibt",
    "tols",
    "vaii",
    "wara",
    "wcho",
];

const SUPPORTED_TIME_ZONES: &[&str] = &[
    "Etc/GMT+1",
    "Etc/GMT+10",
    "Etc/GMT+11",
    "Etc/GMT+12",
    "Etc/GMT+2",
    "Etc/GMT+3",
    "Etc/GMT+4",
    "Etc/GMT+5",
    "Etc/GMT+6",
    "Etc/GMT+7",
    "Etc/GMT+8",
    "Etc/GMT+9",
    "Etc/GMT-1",
    "Etc/GMT-10",
    "Etc/GMT-11",
    "Etc/GMT-12",
    "Etc/GMT-13",
    "Etc/GMT-14",
    "Etc/GMT-2",
    "Etc/GMT-3",
    "Etc/GMT-4",
    "Etc/GMT-5",
    "Etc/GMT-6",
    "Etc/GMT-7",
    "Etc/GMT-8",
    "Etc/GMT-9",
    "UTC",
];

const SUPPORTED_UNITS: &[&str] = &[
    "acre",
    "bit",
    "byte",
    "celsius",
    "centimeter",
    "day",
    "degree",
    "fahrenheit",
    "fluid-ounce",
    "foot",
    "gallon",
    "gigabit",
    "gigabyte",
    "gram",
    "hectare",
    "hour",
    "inch",
    "kilobit",
    "kilobyte",
    "kilogram",
    "kilometer",
    "liter",
    "megabit",
    "megabyte",
    "meter",
    "microsecond",
    "mile",
    "mile-scandinavian",
    "milliliter",
    "millimeter",
    "millisecond",
    "minute",
    "month",
    "nanosecond",
    "ounce",
    "percent",
    "petabyte",
    "pound",
    "second",
    "stone",
    "terabit",
    "terabyte",
    "week",
    "yard",
    "year",
];

/// Abstract operation `DefaultLocale ( )`
///
/// Returns a String value representing the structurally valid and canonicalized
/// Unicode BCP 47 locale identifier for the host environment's current locale.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-defaultlocale
pub(crate) fn default_locale(canonicalizer: &LocaleCanonicalizer) -> Locale {
    sys_locale::get_locale()
        .and_then(|loc| loc.parse::<Locale>().ok())
        .tap_some_mut(|loc| {
            canonicalizer.canonicalize(loc);
        })
        .unwrap_or(Locale::UNKNOWN)
}

/// Gets the `Locale` struct from a `JsValue`.
pub(crate) fn locale_from_value(tag: &JsValue, context: &mut Context) -> JsResult<Locale> {
    // ii. If Type(kValue) is not String or Object, throw a TypeError exception.
    if !(tag.is_object() || tag.is_string()) {
        return Err(JsNativeError::typ()
            .with_message("locale should be a String or Object")
            .into());
    }
    // iii. If Type(kValue) is Object and kValue has an [[InitializedLocale]] internal slot, then
    let object = tag.as_object();
    if let Some(tag) = object.as_ref().and_then(|obj| obj.downcast_ref::<Locale>()) {
        // 1. Let tag be kValue.[[Locale]].
        // No need to canonicalize since all `Locale` objects should already be canonicalized.
        return Ok(tag.clone());
    }

    // iv. Else,
    // 1. Let tag be ? ToString(kValue).
    let tag = tag.to_string(context)?.to_std_string_escaped();
    if tag.contains('_') {
        return Err(JsNativeError::range()
            .with_message("locale is not a structurally valid language tag")
            .into());
    }

    let tag = if tag.eq_ignore_ascii_case("posix") {
        "und-posix".to_owned()
    } else {
        tag
    };

    let mut tag = tag
        .parse()
        // v. If IsStructurallyValidLanguageTag(tag) is false, throw a RangeError exception.
        .map_err(|_| {
            JsNativeError::range().with_message("locale is not a structurally valid language tag")
        })?;

    // All callers of `locale_from_value` canonicalize the result immediately after, so
    // we canonicalize here instead.
    context
        .intl_provider()
        .locale_canonicalizer()?
        .canonicalize(&mut tag);

    Ok(tag)
}

/// Abstract operation `CanonicalizeLocaleList ( locales )`
///
/// Converts an array of [`JsValue`]s containing structurally valid
/// [Unicode BCP 47 locale identifiers][bcp-47] into their [canonical form][canon].
///
/// For efficiency, this returns [`Locale`]s instead of [`String`]s, since
/// `Locale` allows us to modify individual parts of the locale without scanning
/// the whole string again.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-canonicalizelocalelist
/// [bcp-47]: https://unicode.org/reports/tr35/#Unicode_locale_identifier
/// [canon]: https://unicode.org/reports/tr35/#LocaleId_Canonicalization
pub(crate) fn canonicalize_locale_list(
    locales: &JsValue,
    context: &mut Context,
) -> JsResult<Vec<Locale>> {
    // 1. If locales is undefined, then
    if locales.is_undefined() {
        // a. Return a new empty List.
        return Ok(Vec::default());
    }

    // 2. Let seen be a new empty List.
    let mut seen = IndexSet::new();
    let mut result = Vec::new();

    // 3. If Type(locales) is String or Type(locales) is Object and locales has an [[InitializedLocale]] internal slot, then
    let o = if locales.is_string() || locales.as_object().is_some_and(|o| o.is::<Locale>()) {
        // a. Let O be CreateArrayFromList(« locales »).
        Array::create_array_from_list([locales.clone()], context)
    } else {
        // 4. Else,
        // a. Let O be ? ToObject(locales).
        locales.to_object(context)?
    };

    // 5. Let len be ? ToLength(? Get(O, "length")).
    let len = o.length_of_array_like(context)?;

    // 6 Let k be 0.
    // 7. Repeat, while k < len,
    for k in 0..len {
        // a. Let Pk be ToString(k).
        // b. Let kPresent be ? HasProperty(O, Pk).
        // c. If kPresent is true, then
        // c.i. Let kValue be ? Get(O, Pk).
        if let Some(k_value) = o.try_get(k, context)? {
            let tag = locale_from_value(&k_value, context)?;
            let canonical = locale_to_canonical_string(&tag);

            // vii. If canonicalizedTag is not an element of seen, append canonicalizedTag as the last element of seen.
            if seen.insert(canonical) {
                result.push(tag);
            }
        }
        // d. Increase k by 1.
    }

    // 8. Return seen.
    Ok(result)
}

pub(crate) fn is_variant_only_posix(locale: &Locale) -> bool {
    locale.id.language == LanguageIdentifier::UNKNOWN.language
        && locale.id.script.is_none()
        && locale.id.region.is_none()
        && locale.id.variants.to_string() == "posix"
        && locale.extensions.unicode.is_empty()
        && locale.extensions.transform.is_empty()
        && locale.extensions.private.is_empty()
}

pub(crate) fn locale_to_canonical_string(locale: &Locale) -> String {
    let mut locale = locale.to_string();

    if locale == "und-posix" {
        locale = "posix".to_owned();
    }

    for (from, to) in [
        ("-ca-ethiopic-amete-alem", "-ca-ethioaa"),
        ("-ca-islamicc", "-ca-islamic-civil"),
        ("-ks-primary", "-ks-level1"),
        ("-ks-tertiary", "-ks-level3"),
        ("-ms-imperial", "-ms-uksystem"),
        ("-tz-cnckg", "-tz-cnsha"),
        ("-tz-eire", "-tz-iedub"),
        ("-tz-est", "-tz-papty"),
        ("-tz-gmt0", "-tz-gmt"),
        ("-tz-uct", "-tz-utc"),
        ("-tz-zulu", "-tz-utc"),
        ("-kb-yes", "-kb"),
        ("-kc-yes", "-kc"),
        ("-kh-yes", "-kh"),
        ("-kk-yes", "-kk"),
        ("-kn-yes", "-kn"),
        ("-m0-names", "-m0-prprname"),
    ] {
        locale = locale.replace(from, to);
    }

    locale
}

pub(crate) fn supported_calendars() -> &'static [&'static str] {
    SUPPORTED_CALENDARS
}

pub(crate) fn supported_collations() -> &'static [&'static str] {
    SUPPORTED_COLLATIONS
}

pub(crate) fn supported_hour_cycles() -> &'static [&'static str] {
    SUPPORTED_HOUR_CYCLES
}

pub(crate) fn supported_numbering_systems() -> &'static [&'static str] {
    SUPPORTED_NUMBERING_SYSTEMS
}

pub(crate) fn supported_time_zones() -> &'static [&'static str] {
    SUPPORTED_TIME_ZONES
}

pub(crate) fn supported_units() -> &'static [&'static str] {
    SUPPORTED_UNITS
}

pub(crate) fn text_info_direction(locale: &Locale) -> &'static str {
    match locale.id.language.as_str() {
        "ar" | "fa" | "he" | "ur" => "rtl",
        _ => "ltr",
    }
}

pub(crate) fn week_info_first_day(locale: &Locale) -> u8 {
    if let Some(first_day) = locale
        .extensions
        .unicode
        .keywords
        .get(&icu_locale::extensions_unicode_key!("fw"))
        .and_then(|value| weekday_to_number(&value.to_string()))
    {
        return first_day;
    }

    if locale.id.region.as_ref().is_some_and(|region| region.as_str() == "US") {
        7
    } else {
        1
    }
}

pub(crate) fn weekday_to_string(value: &str) -> Option<&'static str> {
    match value {
        "0" | "7" | "sun" => Some("sun"),
        "1" | "mon" => Some("mon"),
        "2" | "tue" => Some("tue"),
        "3" | "wed" => Some("wed"),
        "4" | "thu" => Some("thu"),
        "5" | "fri" => Some("fri"),
        "6" | "sat" => Some("sat"),
        _ => None,
    }
}

pub(crate) fn weekday_to_number(value: &str) -> Option<u8> {
    match value {
        "mon" => Some(1),
        "tue" => Some(2),
        "wed" => Some(3),
        "thu" => Some(4),
        "fri" => Some(5),
        "sat" => Some(6),
        "sun" => Some(7),
        _ => None,
    }
}

/// Abstract operation [`LookupMatchingLocaleByPrefix ( availableLocales, requestedLocales )`][prefix]
/// and [`LookupMatchingLocaleByBestFit ( availableLocales, requestedLocales )`][best]
///
/// Compares `requestedLocales`, which must be a `List` as returned by `CanonicalizeLocaleList`,
/// against the locales in `availableLocales` and determines the best available language to
/// meet the request.
///
/// # Notes
///
/// - This differs a bit from the spec, since we don't have an `[[AvailableLocales]]`
///   list to compare with. However, we can do data requests to a [`DryDataProvider`]
///   in order to see if a certain [`Locale`] is supported.
///
/// - Calling this function with a singleton [`DataMarker`] will always return `None`.
///
/// [prefix]: https://tc39.es/ecma402/#sec-lookupmatchinglocalebyprefix
/// [best]: https://tc39.es/ecma402/#sec-lookupmatchinglocalebybestfit
fn lookup_matching_locale_by_prefix<S: Service>(
    requested_locales: impl IntoIterator<Item = Locale>,
    provider: &IntlProvider,
) -> Option<Locale>
where
    IntlProvider: DryDataProvider<S::LangMarker>,
{
    // 1. For each element locale of requestedLocales, do
    for locale in requested_locales {
        // a. Let extension be empty.
        // b. If locale contains a Unicode locale extension sequence, then
        //     i. Set extension to the Unicode locale extension sequence of locale.
        //     ii. Set locale to the String value that is locale with any Unicode locale extension sequences removed.
        let mut locale = locale.clone();
        let id = std::mem::replace(&mut locale.id, LanguageIdentifier::UNKNOWN);
        locale.extensions.transform.clear();
        locale.extensions.private.clear();

        // c. Let prefix be locale.
        let mut prefix = id.into();

        // d. Repeat, while prefix is not the empty String,
        // We don't use a `while !prefix.is_und()` because it could be that prefix is und at the start,
        // so we need to make the request at least once.
        loop {
            // i. If availableLocales contains prefix, return the Record { [[locale]]: prefix, [[extension]]: extension }.
            // ICU4X requires doing data requests in order to check if a locale
            // is part of the set of supported locales.
            let response = DryDataProvider::dry_load(
                provider,
                DataRequest {
                    id: DataIdentifierBorrowed::for_marker_attributes_and_locale(
                        S::ATTRIBUTES,
                        &prefix,
                    ),
                    metadata: {
                        let mut metadata = DataRequestMetadata::default();
                        metadata.silent = true;
                        metadata
                    },
                },
            );

            if let Ok(metadata) = response {
                // `metadata.locale` returns None when the provider doesn't have a fallback mechanism,
                // but supports the required locale. However, if the provider has a fallback mechanism,
                // this will return `Some(locale)`, where the locale is the used locale after applying
                // the fallback algorithm, even if the used locale is exactly the same as the required
                // locale.
                match metadata.locale {
                    Some(loc) if loc.into_locale().id == prefix.into_locale().id => {
                        locale.id = prefix.into_locale().id;
                        return Some(locale);
                    }
                    None => {
                        locale.id = prefix.into_locale().id;
                        return Some(locale);
                    }
                    _ => {}
                }
            }

            // ii. If prefix contains "-" (code unit 0x002D HYPHEN-MINUS), let pos be the index into prefix of the last occurrence of "-"; else let pos be 0.
            // iii. Repeat, while pos ≥ 2 and the substring of prefix from pos - 2 to pos - 1 is "-",
            //     1. Set pos to pos - 2.
            // iv. Set prefix to the substring of prefix from 0 to pos.
            // Since the definition of `LanguageIdentifier` allows us to manipulate it
            // without using strings, we can replace these steps by a simpler
            // algorithm.
            if prefix.variant.take().is_none()
                && prefix.region.take().is_none()
                && prefix.script.take().is_none()
            {
                break;
            }
        }
    }

    // 2. Return undefined.
    None
}

/// Abstract operation [`LookupMatchingLocaleByBestFit ( availableLocales, requestedLocales )`][spec]
///
/// Compares `requestedLocales`, which must be a `List` as returned by `CanonicalizeLocaleList`,
/// against the locales in `availableLocales` and determines the best available language to
/// meet the request. The algorithm is implementation dependent, but should produce results
/// that a typical user of the requested locales would perceive as at least as good as those
/// produced by the `LookupMatcher` abstract operation.
///
/// [spec]: https://tc39.es/ecma402/#sec-bestfitmatcher
fn lookup_matching_locale_by_best_fit<S: Service>(
    requested_locales: impl IntoIterator<Item = Locale>,
    provider: &IntlProvider,
) -> Option<Locale>
where
    IntlProvider: DryDataProvider<S::LangMarker>,
{
    for mut locale in requested_locales {
        let id = std::mem::replace(&mut locale.id, LanguageIdentifier::UNKNOWN);

        // Only leave unicode extensions when returning the locale.
        locale.extensions.transform.clear();
        locale.extensions.private.clear();

        let dl = &DataLocale::from(&id);

        let Ok(response) = DryDataProvider::dry_load(
            provider,
            DataRequest {
                id: DataIdentifierBorrowed::for_marker_attributes_and_locale(S::ATTRIBUTES, dl),
                metadata: {
                    let mut md = DataRequestMetadata::default();
                    md.silent = true;
                    md
                },
            },
        ) else {
            continue;
        };

        if id == LanguageIdentifier::UNKNOWN {
            return Some(locale);
        }

        if let Some(id) = response
            .locale
            .map(|dl| dl.into_locale().id)
            .or(Some(id))
            .filter(|loc| loc != &LanguageIdentifier::UNKNOWN)
        {
            locale.id = id;
            return Some(locale);
        }
    }
    None
}

/// Abstract operation `ResolveLocale ( availableLocales, requestedLocales, options, relevantExtensionKeys, localeData )`
///
/// Compares a BCP 47 language priority list `requestedLocales` against the locales
/// in `availableLocales` and determines the best available language to meet the request.
/// `availableLocales`, `requestedLocales`, and `relevantExtensionKeys` must be provided as
/// `List` values, options and `localeData` as Records.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-resolvelocale
pub(in crate::builtins::intl) fn resolve_locale<S>(
    requested_locales: impl IntoIterator<Item = Locale>,
    options: &mut IntlOptions<S::LocaleOptions>,
    provider: &IntlProvider,
) -> JsResult<Locale>
where
    S: Service,
    IntlProvider: DryDataProvider<S::LangMarker>,
{
    // 1. Let matcher be options.[[localeMatcher]].
    // 2. If matcher is "lookup", then
    //     a. Let r be LookupMatchingLocaleByPrefix(availableLocales, requestedLocales).
    // 3. Else,
    //     a. Let r be LookupMatchingLocaleByBestFit(availableLocales, requestedLocales).
    // 4. If r is undefined, set r to the Record { [[locale]]: DefaultLocale(), [[extension]]: empty }.
    let found_locale = if options.matcher == LocaleMatcher::Lookup {
        lookup_matching_locale_by_prefix::<S>(requested_locales, provider)
    } else {
        lookup_matching_locale_by_best_fit::<S>(requested_locales, provider)
    };

    let mut found_locale = if let Some(loc) = found_locale {
        loc
    } else {
        let default = default_locale(provider.locale_canonicalizer()?);
        lookup_matching_locale_by_best_fit::<S>([default], provider).ok_or_else(|| {
            JsNativeError::typ().with_message("could not find i18n data for Intl service")
        })?
    };

    // From here, the spec differs significantly from the implementation,
    // since ICU4X allows us to skip some steps and modularize the
    // extension resolution algorithm. However, the original spec is left here
    // for completion purposes.

    // 5. Let result be a new Record.
    // 6. Set result.[[dataLocale]] to foundLocale.
    // 7. If r has an [[extension]] field, then
    //     a. Let components be ! UnicodeExtensionComponents(r.[[extension]]).
    //     b. Let keywords be components.[[Keywords]].
    // 9. For each element key of relevantExtensionKeys, do
    //     a. Let foundLocaleData be localeData.[[<foundLocale>]].
    //     b. Assert: Type(foundLocaleData) is Record.
    //     c. Let keyLocaleData be foundLocaleData.[[<key>]].
    //     d. Assert: Type(keyLocaleData) is List.
    //     e. Let value be keyLocaleData[0].
    //     f. Assert: Type(value) is either String or Null.
    //     g. Let supportedExtensionAddition be "".
    //     h. If r has an [[extension]] field, then
    //         i. If keywords contains an element whose [[Key]] is the same as key, then
    //             1. Let entry be the element of keywords whose [[Key]] is the same as key.
    //             2. Let requestedValue be entry.[[Value]].
    //             3. If requestedValue is not the empty String, then
    //                 a. If keyLocaleData contains requestedValue, then
    //                     i. Let value be requestedValue.
    //                     ii. Let supportedExtensionAddition be the string-concatenation of "-", key, "-", and value.
    //             4. Else if keyLocaleData contains "true", then
    //                 a. Let value be "true".
    //                 b. Let supportedExtensionAddition be the string-concatenation of "-" and key.
    //     i. If options has a field [[<key>]], then
    //         i. Let optionsValue be options.[[<key>]].
    //         ii. Assert: Type(optionsValue) is either String, Undefined, or Null.
    //         iii. If Type(optionsValue) is String, then
    //             1. Let optionsValue be the string optionsValue after performing the algorithm steps to transform
    //                Unicode extension values to canonical syntax per Unicode Technical Standard #35 LDML § 3.2.1
    //                Canonical Unicode Locale Identifiers, treating key as ukey and optionsValue as uvalue productions.
    //             2. Let optionsValue be the string optionsValue after performing the algorithm steps to replace
    //                Unicode extension values with their canonical form per Unicode Technical Standard #35 LDML § 3.2.1
    //                Canonical Unicode Locale Identifiers, treating key as ukey and optionsValue as uvalue productions.
    //             3. If optionsValue is the empty String, then
    //                 a. Let optionsValue be "true".
    //         iv. If keyLocaleData contains optionsValue, then
    //             1. If SameValue(optionsValue, value) is false, then
    //                 a. Let value be optionsValue.
    //                 b. Let supportedExtensionAddition be "".
    //     j. Set result.[[<key>]] to value.
    //     k. Append supportedExtensionAddition to supportedExtension.
    // 10. If the number of elements in supportedExtension is greater than 2, then
    //     a. Let foundLocale be InsertUnicodeExtensionAndCanonicalize(foundLocale, supportedExtension).
    // 11. Set result.[[locale]] to foundLocale.

    // 12. Return result.
    S::resolve(&mut found_locale, &mut options.service_options, provider);
    provider
        .locale_canonicalizer()?
        .canonicalize(&mut found_locale);
    Ok(found_locale)
}

/// Abstract operation [`FilterLocales ( availableLocales, requestedLocales, options )`][spec]
///
/// Returns the subset of the provided BCP 47 language priority list requestedLocales for which
/// availableLocales has a matching locale.
///
/// # Note
///
/// Calling this function with a Service that has a singleton `LangMarker` will always return `None`.
///
/// [spec]: https://tc39.es/ecma402/#sec-supportedlocales
pub(in crate::builtins::intl) fn filter_locales<S: Service>(
    requested_locales: Vec<Locale>,
    options: &JsValue,
    context: &mut Context,
) -> JsResult<JsObject>
where
    IntlProvider: DryDataProvider<S::LangMarker>,
{
    // 1. Set options to ? CoerceOptionsToObject(options).
    let options = coerce_options_to_object(options, context)?;

    // 2. Let matcher be ? GetOption(options, "localeMatcher", string, « "lookup", "best fit" », "best fit").
    let matcher = get_option(&options, js_string!("localeMatcher"), context)?.unwrap_or_default();

    // 3. Let subset be a new empty List.
    let mut subset = Vec::with_capacity(requested_locales.len());

    // 4. For each element locale of requestedLocales, do
    for locale in requested_locales {
        // a. Let noExtensionsLocale be the String value that is locale with any Unicode locale extension sequences removed.
        let mut no_ext_loc = locale.clone();
        no_ext_loc.extensions.unicode.clear();
        let loc_match = match matcher {
            // b. If matcher is "lookup", then
            //     i. Let match be LookupMatchingLocaleByPrefix(availableLocales, noExtensionsLocale).
            LocaleMatcher::Lookup => {
                lookup_matching_locale_by_prefix::<S>([no_ext_loc], context.intl_provider())
            }
            // c. Else,
            //     i. Let match be LookupMatchingLocaleByBestFit(availableLocales, noExtensionsLocale).
            LocaleMatcher::BestFit => {
                lookup_matching_locale_by_best_fit::<S>([no_ext_loc], context.intl_provider())
            }
        };

        // d. If match is not undefined, append locale to subset.
        if loc_match.is_some() {
            subset.push(locale);
        }
    }

    // 5. Return CreateArrayFromList(subset).
    Ok(Array::create_array_from_list(
        subset
            .into_iter()
            .map(|loc| js_string!(loc.to_string()).into()),
        context,
    ))
}

/// Validates that the unicode extension `key` with `value` is a valid extension value for the
/// `language`.
///
/// # Note
///
/// Calling this function with a singleton `DataMarker` will always return `None`.
pub(in crate::builtins::intl) fn validate_extension<M: DataMarker>(
    language: LanguageIdentifier,
    attributes: &DataMarkerAttributes,
    provider: &impl DryDataProvider<M>,
) -> bool {
    let locale = DataLocale::from(language);
    let req = DataRequest {
        id: DataIdentifierBorrowed::for_marker_attributes_and_locale(attributes, &locale),
        metadata: {
            let mut metadata = DataRequestMetadata::default();
            metadata.silent = true;
            metadata
        },
    };

    provider.dry_load(req).is_ok()
}

#[cfg(all(test, feature = "intl_bundled"))]
mod tests {
    use icu_locale::{Locale, langid, locale};
    use icu_plurals::provider::PluralsCardinalV1;

    struct TestService;

    impl Service for TestService {
        type LangMarker = PluralsCardinalV1;
        type LocaleOptions = ();
    }

    use crate::{
        builtins::intl::{
            Service,
            locale::utils::{lookup_matching_locale_by_best_fit, lookup_matching_locale_by_prefix},
        },
        context::icu::IntlProvider,
    };

    #[test]
    fn best_fit() {
        let icu = &IntlProvider::try_new_buffer(boa_icu_provider::buffer());

        assert_eq!(
            lookup_matching_locale_by_best_fit::<TestService>([locale!("en")], icu),
            Some(locale!("en"))
        );

        assert_eq!(
            lookup_matching_locale_by_best_fit::<TestService>([locale!("es-ES")], icu),
            Some(locale!("es"))
        );

        assert_eq!(
            lookup_matching_locale_by_best_fit::<TestService>([locale!("kr")], icu),
            None
        );
    }

    #[test]
    fn lookup_match() {
        let icu = &IntlProvider::try_new_buffer(boa_icu_provider::buffer());

        // requested: [fr-FR-u-hc-h12]
        let requested: Locale = "fr-FR-u-hc-h12".parse().unwrap();

        let result =
            lookup_matching_locale_by_prefix::<TestService>([requested.clone()], icu).unwrap();
        assert_eq!(result.id, langid!("fr"));
        assert_eq!(result.extensions, requested.extensions);

        // requested: [kr-KR-u-hc-h12, gr-GR-u-hc-h24-x-4a, es-ES-valencia-u-ca-gregory, uz-Cyrl]
        let kr = "kr-KR-u-hc-h12".parse().unwrap();
        let gr = "gr-GR-u-hc-h24-x-4a".parse().unwrap();
        let es: Locale = "es-ES-valencia-u-ca-gregory".parse().unwrap();
        let uz = locale!("uz-Cyrl");
        let requested = vec![kr, gr, es.clone(), uz];

        let res = lookup_matching_locale_by_best_fit::<TestService>(requested, icu).unwrap();
        assert_eq!(res.id, langid!("es"));
        assert_eq!(res.extensions, es.extensions);
    }
}
