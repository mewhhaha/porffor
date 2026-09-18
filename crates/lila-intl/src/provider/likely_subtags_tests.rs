use super::*;
use crate::{IntlKernel, IntlOperation, LocaleId};

fn transform<O>(source: &str) -> String
where
    O: IntlOperation<
        Request = LocaleTransformRequest,
        Response = LocaleTransformResult,
        Error = LocaleTransformError,
    >,
    EmbeddedLocaleProvider: IntlOperationProvider<O>,
{
    let provider = EmbeddedLocaleProvider::new().unwrap();
    let kernel = IntlKernel::new(provider.identity().clone(), provider).unwrap();
    kernel
        .operation::<O>()
        .unwrap()
        .execute(LocaleTransformRequest::new(
            LocaleId::parse(source).unwrap(),
        ))
        .unwrap()
        .locale()
        .as_str()
        .to_string()
}

#[test]
fn likely_subtags_use_extended_pinned_data_and_region_favoring_minimization() {
    for (source, maximal, minimal) in [
        ("en", "en-Latn-US", "en"),
        ("en-Shaw", "en-Shaw-GB", "en-Shaw"),
        ("en-Arab", "en-Arab-US", "en-Arab"),
        ("en-GB", "en-Latn-GB", "en-GB"),
        ("it-Kana-CA", "it-Kana-CA", "it-Kana-CA"),
        ("und", "en-Latn-US", "en"),
        ("und-Thai", "th-Thai-TH", "th"),
        ("und-419", "es-Latn-419", "es-419"),
        ("und-150", "en-Latn-150", "en-150"),
        ("und-AT", "de-Latn-AT", "de-AT"),
        ("und-Cyrl-RO", "bg-Cyrl-RO", "bg-RO"),
        ("und-AQ", "en-Latn-AQ", "en-AQ"),
        ("zh-Hant-TW", "zh-Hant-TW", "zh-TW"),
        ("ccp", "ccp-Cakm-BD", "ccp"),
    ] {
        assert_eq!(transform::<MaximizeLocale>(source), maximal, "{source}");
        assert_eq!(transform::<MinimizeLocale>(source), minimal, "{source}");
        assert_eq!(transform::<MaximizeLocale>(minimal), maximal, "{source}");
        assert_eq!(transform::<MinimizeLocale>(maximal), minimal, "{source}");
    }
}

#[test]
fn likely_subtags_preserve_variants_extensions_and_keyword_aliases() {
    for suffix in [
        "-fonipa",
        "-a-not-assigned",
        "-u-attr",
        "-u-co",
        "-u-co-phonebk",
        "-x-private",
        "-t-de-latn-h0-hybrid-u-kn-true-x-keep",
    ] {
        let canonical = transform::<CanonicalizeLocale>(&format!("en{suffix}"));
        let suffix = canonical.strip_prefix("en").unwrap();
        assert_eq!(
            transform::<MaximizeLocale>(&canonical),
            format!("en-Latn-US{suffix}")
        );
        assert_eq!(
            transform::<MinimizeLocale>(&format!("en-Latn-US{suffix}")),
            canonical
        );
    }
    assert_eq!(
        transform::<MaximizeLocale>("en-u-ca-islamicc-kf-upper-kn-true"),
        "en-Latn-US-u-ca-islamic-civil-kf-upper-kn",
    );
}

#[test]
fn missing_language_data_and_reserved_languages_do_not_become_und() {
    for source in [
        "xtg",
        "xtg-Latn-FR",
        "mul",
        "abcdefg",
        "abcde-Cyrl-RU",
        "abcdefgh-Latn-US-fonipa-u-co-phonebk-x-keep",
        "abcde-t-abcde-cyrl-ru-h0-hybrid-u-ca-islamicc",
    ] {
        let canonical = transform::<CanonicalizeLocale>(source);
        assert_eq!(transform::<MaximizeLocale>(source), canonical, "{source}");
        assert_eq!(transform::<MinimizeLocale>(source), canonical, "{source}");
    }
}

#[test]
fn aliases_are_resolved_before_likely_subtag_lookup() {
    for (source, maximal, minimal) in [
        ("mo", "ro-Latn-RO", "ro"),
        ("aar-x-private", "aa-Latn-ET-x-private", "aa-x-private"),
        ("heb-x-private", "he-Hebr-IL-x-private", "he-x-private"),
        ("hy-arevela", "hy-Armn-AM", "hy"),
        ("hy-arevmda", "hyw-Armn-AM", "hyw"),
        ("art-lojban", "jbo-Latn-001", "jbo"),
        ("cel-gaulish", "xtg", "xtg"),
        ("zh-hakka", "hak-Hans-CN", "hak"),
        ("zh-xiang", "hsn-Hans-CN", "hsn"),
    ] {
        assert_eq!(transform::<MaximizeLocale>(source), maximal, "{source}");
        assert_eq!(transform::<MinimizeLocale>(source), minimal, "{source}");
    }
}

#[test]
fn unknown_base_fields_are_removed_only_when_likely_lookup_succeeds() {
    for (source, maximal, minimal) in [
        ("en-Zzzz-ZZ", "en-Latn-US", "en"),
        ("en-Zzzz", "en-Latn-US", "en"),
        ("en-ZZ", "en-Latn-US", "en"),
        ("en-Zzzz-GB", "en-Latn-GB", "en-GB"),
        ("en-Latn-ZZ", "en-Latn-US", "en"),
        ("und-Zzzz-ZZ", "en-Latn-US", "en"),
        ("und-Zzzz-419", "es-Latn-419", "es-419"),
        ("zh-Hant-ZZ", "zh-Hant-TW", "zh-TW"),
        ("zh-Zzzz-SG", "zh-Hans-SG", "zh-SG"),
    ] {
        assert_eq!(transform::<MaximizeLocale>(source), maximal, "{source}");
        assert_eq!(transform::<MinimizeLocale>(source), minimal, "{source}");
    }
    for source in [
        "xtg-Zzzz-ZZ",
        "xtg-Zzzz-FR",
        "xtg-Latn-ZZ",
        "mul-Zzzz-ZZ",
        "abcde-Zzzz-ZZ",
        "abcdefg-Zzzz-FR",
        "abcdefgh-Latn-ZZ",
    ] {
        let canonical = transform::<CanonicalizeLocale>(source);
        assert_eq!(transform::<MaximizeLocale>(source), canonical, "{source}");
        assert_eq!(transform::<MinimizeLocale>(source), canonical, "{source}");
    }
    let suffix = "-fonipa-t-en-zzzz-zz-u-rg-zzzzzz-x-zzzz-zz";
    for (source, maximal, minimal) in [
        ("en-Zzzz-ZZ", "en-Latn-US", "en"),
        ("xtg-Zzzz-ZZ", "xtg-Zzzz-ZZ", "xtg-Zzzz-ZZ"),
        ("abcde-Zzzz-ZZ", "abcde-Zzzz-ZZ", "abcde-Zzzz-ZZ"),
    ] {
        assert_eq!(
            transform::<MaximizeLocale>(&format!("{source}{suffix}")),
            format!("{maximal}{suffix}")
        );
        assert_eq!(
            transform::<MinimizeLocale>(&format!("{source}{suffix}")),
            format!("{minimal}{suffix}")
        );
    }
}
