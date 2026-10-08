//! Whole-value aliases from CLDR's BCP47 domain, without lossy ICU value parsing.

use core::ops::Range;
use std::collections::BTreeMap;

use icu_locale::extensions::{transform, unicode};
use icu_locale::subtags::Subtag;
use icu_locale::Locale;

#[cfg(test)]
mod generated;
pub(super) const PROVIDER_DATA_SHA256: [u8; 32] = [
    0xae, 0x86, 0x2a, 0x65, 0x8b, 0x99, 0x0b, 0xe3, 0x81, 0x7e, 0x29, 0xc4, 0x51, 0x43, 0x94, 0x90,
    0xbb, 0xa1, 0x75, 0xf0, 0x12, 0xfa, 0x71, 0xbf, 0x26, 0x39, 0x8a, 0x54, 0x5a, 0x69, 0xdf, 0xbb,
];
const PINNED_KEYWORDS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/keyword-data.json"));

#[cfg(test)]
struct UnicodeKeywordAliases {
    key: unicode::Key,
    replacements: &'static [(&'static str, &'static str)],
}
#[cfg(test)]
struct TransformKeywordAliases {
    key: transform::Key,
    replacements: &'static [(&'static str, &'static str)],
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAliasData {
    schema: u32,
    algorithm: String,
    cldr_release: String,
    cldr_commit: String,
    provider_data_sha256: [u8; 32],
    unicode: Vec<RawAliases>,
    transform: Vec<RawAliases>,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAliases {
    key: String,
    replacements: Vec<(String, String)>,
}
#[derive(Debug)]
struct UnicodeReplacement {
    source: Box<str>,
    target: unicode::Value,
}
#[derive(Debug)]
struct AdmittedUnicodeAliases {
    key: unicode::Key,
    replacements: Box<[UnicodeReplacement]>,
}
#[derive(Debug)]
struct AdmittedTransformAliases {
    key: transform::Key,
    replacements: Box<[(Box<str>, Box<str>)]>,
}

/// Complete typed keyword authority admitted with the selected Locale image.
/// Syntax parsing remains independent; only this owner can rewrite data aliases.
#[derive(Debug)]
pub(crate) struct KeywordAliasData {
    unicode: Box<[AdmittedUnicodeAliases]>,
    transform: Box<[AdmittedTransformAliases]>,
}

fn checked_rows(rows: &[RawAliases], count: usize) -> Result<(), crate::IntlDataImageError> {
    let fail = || crate::IntlDataImageError::consumer("invalid pinned keyword alias rows");
    if rows.iter().map(|row| row.replacements.len()).sum::<usize>() != count
        || !rows.windows(2).all(|pair| pair[0].key < pair[1].key)
    {
        return Err(fail());
    }
    for row in rows {
        if row.replacements.is_empty()
            || !row
                .replacements
                .windows(2)
                .all(|pair| pair[0].0 < pair[1].0)
        {
            return Err(fail());
        }
        for (source, target) in &row.replacements {
            let value = |value: &str| {
                !value.is_empty()
                    && value.split('-').all(|subtag| {
                        (3..=8).contains(&subtag.len())
                            && subtag
                                .bytes()
                                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
                    })
            };
            if !value(source)
                || !value(target)
                || source == target
                || row
                    .replacements
                    .binary_search_by(|(alias, _)| alias.cmp(target))
                    .is_ok()
            {
                return Err(fail());
            }
        }
    }
    Ok(())
}

impl KeywordAliasData {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, crate::IntlDataImageError> {
        if bytes != PINNED_KEYWORDS {
            return Err(crate::IntlDataImageError::consumer(
                "keyword aliases differ from exact locked producer bytes",
            ));
        }
        let raw: RawAliasData =
            serde_json::from_slice(bytes).map_err(crate::IntlDataImageError::consumer)?;
        if raw.schema != 1
            || raw.algorithm != "keyword-aliases-cldr47-v1"
            || raw.cldr_release != "47.0.0"
            || raw.cldr_commit != "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c"
            || raw.provider_data_sha256 != PROVIDER_DATA_SHA256
        {
            return Err(crate::IntlDataImageError::consumer(
                "keyword alias source descriptor",
            ));
        }
        checked_rows(&raw.unicode, 60)?;
        checked_rows(&raw.transform, 5)?;
        let unicode = raw
            .unicode
            .into_iter()
            .map(|row| {
                let key = row
                    .key
                    .parse()
                    .map_err(crate::IntlDataImageError::consumer)?;
                let replacements = row
                    .replacements
                    .into_iter()
                    .map(|(source, target)| {
                        let value: unicode::Value = target
                            .parse()
                            .map_err(crate::IntlDataImageError::consumer)?;
                        if value.to_string() != target && !(target == "true" && value.is_empty()) {
                            return Err(crate::IntlDataImageError::consumer(
                                "lossy keyword alias target",
                            ));
                        }
                        Ok(UnicodeReplacement {
                            source: source.into_boxed_str(),
                            target: value,
                        })
                    })
                    .collect::<Result<Vec<_>, crate::IntlDataImageError>>()?
                    .into_boxed_slice();
                Ok(AdmittedUnicodeAliases { key, replacements })
            })
            .collect::<Result<Vec<_>, crate::IntlDataImageError>>()?
            .into_boxed_slice();
        let transform = raw
            .transform
            .into_iter()
            .map(|row| {
                let key = row
                    .key
                    .parse()
                    .map_err(crate::IntlDataImageError::consumer)?;
                Ok(AdmittedTransformAliases {
                    key,
                    replacements: row
                        .replacements
                        .into_iter()
                        .map(|(source, target)| (source.into_boxed_str(), target.into_boxed_str()))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                })
            })
            .collect::<Result<Vec<_>, crate::IntlDataImageError>>()?
            .into_boxed_slice();
        Ok(Self { unicode, transform })
    }

    pub(crate) fn canonicalize_unicode_keywords(&self, locale: &mut Locale) {
        for aliases in &self.unicode {
            if let Some(value) = locale.extensions.unicode.keywords.get_mut(&aliases.key) {
                let source = value.to_string();
                if let Ok(index) = aliases
                    .replacements
                    .binary_search_by(|alias| alias.source.as_ref().cmp(&source))
                {
                    *value = aliases.replacements[index].target.clone();
                }
            }
        }
    }
}

/// The source has already passed ICU's complete locale parser. Reconstruct
/// Unicode values from subtags because ICU2 removes every "true" subtag, even
/// inside a compound value whose complete spelling has no canonical alias.
pub(super) fn lossless_unicode_keywords(source: &str) -> unicode::Keywords {
    let mut keywords = unicode::Keywords::new();
    let Some(range) = extension_range(source, b'u') else {
        return keywords;
    };
    let mut subtags = source[range]
        .split('-')
        .skip_while(|subtag| subtag.len() > 2)
        .peekable();
    while let Some(key) = subtags.next() {
        let key: unicode::Key = key.parse().expect("ICU-validated Unicode key");
        let values: Vec<Subtag> = std::iter::from_fn(|| {
            subtags
                .next_if(|subtag| subtag.len() > 2)
                .map(|subtag| subtag.parse().expect("ICU-validated Unicode value subtag"))
        })
        .collect();
        if !keywords.contains_key(&key) {
            let value = if values.len() == 1 && values[0].as_str() == "true" {
                unicode::Value::new_empty()
            } else {
                values.into_iter().collect()
            };
            keywords.set(key, value);
        }
    }
    keywords
}

/// ICU2's opaque transform Value cannot represent compound "true" losslessly.
/// These private records retain the complete validated values while ICU owns
/// structural parsing, language aliases, extension order and attribute order.
pub(super) struct TransformKeywordValues(BTreeMap<transform::Key, Box<str>>);

impl TransformKeywordValues {
    pub(super) fn from_validated_locale(source: &str) -> Self {
        let mut values = BTreeMap::new();
        if let Some(range) = extension_range(source, b't') {
            let mut subtags = source[range]
                .split('-')
                .skip_while(|subtag| !is_transform_key(subtag))
                .peekable();
            while let Some(key) = subtags.next() {
                let key = key.parse().expect("ICU-validated transform key");
                let value = std::iter::from_fn(|| subtags.next_if(|subtag| subtag.len() > 2))
                    .collect::<Vec<_>>()
                    .join("-")
                    .to_ascii_lowercase();
                assert!(
                    !value.is_empty(),
                    "ICU-validated transform value is nonempty"
                );
                values.entry(key).or_insert_with(|| value.into_boxed_str());
            }
        }
        Self(values)
    }

    pub(super) fn canonicalize(&mut self, data: &KeywordAliasData) {
        for aliases in &data.transform {
            if let Some(value) = self.0.get_mut(&aliases.key) {
                if let Ok(index) = aliases
                    .replacements
                    .binary_search_by(|(source, _)| source.as_ref().cmp(value))
                {
                    *value = aliases.replacements[index].1.clone();
                }
            }
        }
    }

    pub(super) fn write_canonical_fields(&self, locale: &mut String) {
        if self.0.is_empty() {
            return;
        }
        let range = extension_range(locale, b't')
            .expect("a parsed transform extension survives canonicalization");
        let mut start = range.start;
        for subtag in locale[range.clone()].split('-') {
            if is_transform_key(subtag) {
                break;
            }
            start += subtag.len() + 1;
        }
        assert!(
            start < range.end,
            "parsed transform fields survive serialization"
        );
        let mut fields = String::new();
        for (key, value) in &self.0 {
            fields.push('-');
            fields.push_str(key.as_str());
            fields.push('-');
            fields.push_str(value);
        }
        // Keep ICU's independently canonicalized tlang and the following
        // extension/private-use suffix. Field keys are already ordered.
        locale.replace_range(start - 1..range.end, &fields);
    }
}

fn is_transform_key(subtag: &str) -> bool {
    subtag.len() == 2 && subtag.as_bytes()[1].is_ascii_digit()
}

fn extension_range(source: &str, singleton: u8) -> Option<Range<usize>> {
    let mut start = None;
    let mut offset = 0;
    for subtag in source.split('-') {
        if subtag.len() == 1 {
            if let Some(start) = start {
                return Some(start..offset - 1);
            }
            let byte = subtag.as_bytes()[0].to_ascii_lowercase();
            if byte == b'x' {
                return None;
            }
            if byte == singleton {
                start = Some(offset + 2);
            }
        }
        offset += subtag.len() + 1;
    }
    start.map(|start| start..source.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CanonicalizeLocale, EmbeddedIntlProvider, IntlDataVersions, IntlKernel, IntlProvider,
        LocaleId, LocaleTransformRequest,
    };

    fn canonical(source: &str) -> String {
        let provider = EmbeddedIntlProvider::new().unwrap();
        let kernel = IntlKernel::new(provider.identity().clone(), provider).unwrap();
        kernel
            .operation::<CanonicalizeLocale>()
            .unwrap()
            .execute(LocaleTransformRequest::new(
                LocaleId::parse(source).unwrap(),
            ))
            .unwrap()
            .locale()
            .as_str()
            .to_owned()
    }

    #[test]
    fn complete_generated_alias_tables_match_the_pinned_data_domain() {
        assert_eq!(IntlDataVersions::PINNED.cldr, "47.0.0");
        assert_eq!(PROVIDER_DATA_SHA256, generated::PROVIDER_DATA_SHA256);
        assert_eq!(
            generated::UNICODE_ALIASES
                .iter()
                .map(|entry| entry.replacements.len())
                .sum::<usize>(),
            60
        );
        assert_eq!(
            generated::TRANSFORM_ALIASES
                .iter()
                .map(|entry| entry.replacements.len())
                .sum::<usize>(),
            5
        );
        for aliases in generated::UNICODE_ALIASES {
            for &(source, target) in aliases.replacements {
                let input = format!("en-u-{}-{source}", aliases.key);
                let expected = if target == "true" {
                    format!("en-u-{}", aliases.key)
                } else {
                    format!("en-u-{}-{target}", aliases.key)
                };
                assert_eq!(canonical(&input), expected, "{input}");
                assert_eq!(canonical(&input.to_ascii_uppercase()), expected, "{input}");
                assert_eq!(canonical(&expected), expected, "{expected}");
            }
            assert!(aliases
                .replacements
                .windows(2)
                .all(|pair| pair[0].0 < pair[1].0));
        }
        for aliases in generated::TRANSFORM_ALIASES {
            for &(source, target) in aliases.replacements {
                let input = format!("en-t-iw-{}-{source}", aliases.key);
                let expected = format!("en-t-he-{}-{target}", aliases.key);
                assert_eq!(canonical(&input), expected, "{input}");
                assert_eq!(canonical(&input.to_ascii_uppercase()), expected, "{input}");
                assert_eq!(canonical(&expected), expected, "{expected}");
            }
            assert!(aliases
                .replacements
                .windows(2)
                .all(|pair| pair[0].0 < pair[1].0));
        }
    }

    #[test]
    fn aliases_are_exact_values_scoped_by_extension_and_key() {
        for input in [
            "en-u-ca-islamicc-foo",
            "en-u-ca-foo-islamicc",
            "en-u-co-islamicc",
            "en-u-ka-yes",
            "en-u-kf-yes",
            "en-u-ks-yes",
            "en-u-ms-imperial-foo",
            "en-t-m0-names-foo",
            "en-t-h0-names",
            "en-u-hc-names",
            "en-x-u-ca-islamicc",
            "en-x-t-m0-names",
        ] {
            assert_eq!(canonical(input), input, "{input}");
        }
        assert_eq!(
            canonical("en-u-ca-islamicc-ca-ethiopic-amete-alem"),
            "en-u-ca-islamic-civil"
        );
    }

    #[test]
    fn compound_true_subtags_survive_in_both_extension_domains() {
        for input in [
            "en-u-ca-islamicc-true",
            "en-u-ca-true-islamicc",
            "en-u-ca-true-foo",
            "en-u-ca-foo-true",
            "en-u-ca-true-true",
            "en-u-kn-yes-true",
            "en-u-zz-true-foo",
            "en-t-m0-true",
            "en-t-en-h0-true-hybrid",
            "en-t-m0-names-true",
            "en-t-m0-true-names",
            "en-t-x0-true-true",
        ] {
            assert_eq!(canonical(input), input, "{input}");
            assert_eq!(canonical(&input.to_ascii_uppercase()), input, "{input}");
        }
        assert_eq!(canonical("en-u-ca-true"), "en-u-ca");
        assert_eq!(canonical("en-u-kn-yes"), "en-u-kn");
    }

    #[test]
    fn aliases_preserve_languages_unrelated_fields_and_private_use() {
        assert_eq!(
            canonical(
                "ABCDE-Armn-SU-t-abcdef-Qaai-DD-m0-names-d0-name-u-attr-ms-imperial-ca-islamicc-rg-cn11-sd-cn11-x-islamicc-true"
            ),
            "abcde-Armn-RU-t-abcdef-zinh-de-d0-charname-m0-prprname-u-attr-ca-islamic-civil-ms-uksystem-rg-cnbj-sd-cnbj-x-islamicc-true"
        );
        let private = "-abcdefgh".repeat(100);
        assert_eq!(
            canonical(&format!("en-t-m0-names-u-ca-islamicc-x{private}")),
            format!("en-t-m0-prprname-u-ca-islamic-civil-x{private}")
        );
    }

    #[test]
    fn admitted_transform_rows_remain_owned_after_the_locale_image_is_dropped() {
        let image = crate::LocaleDataImage::from_bytes(
            crate::embedded_locale_data_image().unwrap().bytes(),
        )
        .unwrap();
        let authority = image.keyword_aliases();
        drop(image);
        for (source, expected) in [
            ("en-t-m0-names-d0-name", "en-t-d0-charname-m0-prprname"),
            ("en-t-m0-names-true", "en-t-m0-names-true"),
        ] {
            let mut fields = TransformKeywordValues::from_validated_locale(source);
            fields.canonicalize(&authority);
            let mut locale = source.parse::<Locale>().unwrap().to_string();
            fields.write_canonical_fields(&mut locale);
            assert_eq!(locale, expected);
        }
    }
}
