//! Unicode's reserved 5–8-letter language domain alongside ICU's 2–3 letters.
//!
//! `und` is only a structural parser placeholder for reserved language fields.
//! Such identifiers never enter ICU's canonicalizer or likely-subtag lookup:
//! a nonempty language absent from the pinned lookup data has different
//! territory-alias semantics from the empty language represented by `und`.

use core::ops::Range;

use icu_locale::extensions::unicode::key;
use icu_locale::provider::{Aliases, Baked, LanguageStrStrPair};
use icu_locale::subtags::{Variant, Variants};
use icu_locale::{LanguageIdentifier, Locale, LocaleCanonicalizer};

use crate::{CanonicalLocaleId, CanonicalizeLocaleError, LocaleId, UnsupportedLocale};

#[derive(Debug)]
struct ReservedLanguage(Box<str>);

impl ReservedLanguage {
    fn parse(subtag: &str) -> Option<Self> {
        ((5..=8).contains(&subtag.len()) && subtag.bytes().all(|byte| byte.is_ascii_alphabetic()))
            .then(|| Self(subtag.to_ascii_lowercase().into_boxed_str()))
    }
}

#[derive(Debug)]
struct ReservedVariantAlias {
    source: Vec<Variant>,
    replacement: LanguageIdentifier,
}

/// Validated evidence that the reduced reserved-language operation is complete
/// for the pinned alias data. Unknown future data fails provider setup.
#[derive(Debug)]
pub(super) struct ReservedLanguageAliasRules {
    aliases: &'static Aliases<'static>,
    wildcard_variants: Vec<ReservedVariantAlias>,
}

impl ReservedLanguageAliasRules {
    pub(super) fn from_pinned_data() -> Result<Self, &'static str> {
        let aliases = Baked::SINGLETON_LOCALE_ALIASES_V1;
        // Every other language-keyed alias/likely-subtag map uses a 2- or
        // 3-byte key in ICU's authoritative schema. Only this unconstrained
        // string-keyed list could contain a reserved language rule.
        if !aliases.language.is_empty() {
            return Err("general language alias list requires reserved-language review");
        }
        let mut wildcard_variants = Vec::new();
        for rule in aliases.language_variants.iter() {
            let LanguageStrStrPair(language, variants, replacement) = rule.into();
            if !language.is_unknown() {
                continue;
            }
            let replacement: LanguageIdentifier = replacement
                .parse()
                .map_err(|_| "wildcard variant replacement is not a language identifier")?;
            if !replacement.language.is_unknown() {
                return Err("wildcard variant alias would replace an absent language");
            }
            let source = variants
                .split('-')
                .map(str::parse::<Variant>)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "wildcard variant source is not a variant sequence")?;
            wildcard_variants.push(ReservedVariantAlias {
                source,
                replacement,
            });
        }
        Ok(Self {
            aliases,
            wildcard_variants,
        })
    }

    fn canonicalize_reserved_identifier(&self, identifier: &mut LanguageIdentifier) {
        // The placeholder is never used as a semantic language value. The
        // actual reserved language is retained by ParsedLocale and restored
        // after serialization of the canonical structural components.
        assert!(identifier.language.is_unknown());
        loop {
            if let Some(rule) = self.wildcard_variants.iter().find(|rule| {
                rule.source
                    .iter()
                    .all(|source| identifier.variants.iter().any(|variant| variant == source))
            }) {
                let mut variants: Vec<_> = identifier
                    .variants
                    .iter()
                    .filter(|variant| !rule.source.contains(variant))
                    .copied()
                    .collect();
                variants.extend(rule.replacement.variants.iter().copied());
                variants.sort_unstable();
                variants.dedup();
                identifier.variants = Variants::from_vec_unchecked(variants);
                if identifier.script.is_none() {
                    identifier.script = rule.replacement.script;
                }
                if identifier.region.is_none() {
                    identifier.region = rule.replacement.region;
                }
                continue;
            }
            if let Some(script) = identifier.script {
                if let Some(&replacement) = self
                    .aliases
                    .script
                    .get(&script.to_tinystr().to_unvalidated())
                {
                    identifier.script = Some(replacement);
                    continue;
                }
            }
            if let Some(region) = identifier.region {
                let replacement = if region.is_alphabetic() {
                    self.aliases
                        .region_alpha
                        .get(&region.to_tinystr().resize().to_unvalidated())
                } else {
                    self.aliases
                        .region_num
                        .get(&region.to_tinystr().to_unvalidated())
                };
                if let Some(&replacement) = replacement {
                    identifier.region = Some(replacement);
                    continue;
                }
                if let Some(replacements) = self
                    .aliases
                    .complex_region
                    .get(&region.to_tinystr().to_unvalidated())
                {
                    if let Some(first) = replacements.get(0) {
                        // UTS35 Annex C: no likely-subtag key can match a
                        // 5–8-letter language in these pinned 3-byte maps.
                        // Failed lookup selects the first territory alias;
                        // script inference for `und` would be incorrect.
                        identifier.region = Some(first);
                        continue;
                    }
                }
            }
            let mut variants = identifier.variants.to_vec();
            let mut changed = false;
            for variant in &mut variants {
                if let Some(&replacement) = self
                    .aliases
                    .variant
                    .get(&variant.to_tinystr().to_unvalidated())
                {
                    *variant = replacement;
                    changed = true;
                }
            }
            if changed {
                variants.sort_unstable();
                variants.dedup();
                identifier.variants = Variants::from_vec_unchecked(variants);
                continue;
            }
            break;
        }
    }

    fn canonicalize_subdivision_keywords(&self, locale: &mut Locale) {
        for key in [key!("rg"), key!("sd")] {
            if let Some(value) = locale.extensions.unicode.keywords.get_mut(&key) {
                if let Some(subtag) = value.as_single_subtag() {
                    if let Some(replacement) = self
                        .aliases
                        .subdivision
                        .get(&subtag.to_tinystr().resize().to_unvalidated())
                    {
                        *value = replacement
                            .as_str()
                            .parse()
                            .expect("pinned subdivision alias is a Unicode value");
                    }
                }
            }
        }
    }
}

pub(super) struct ParsedLocale {
    locale: Locale,
    reserved_base_language: Option<ReservedLanguage>,
    reserved_transform_language: Option<ReservedLanguage>,
}

impl ParsedLocale {
    pub(super) fn parse(input: &LocaleId) -> Result<Self, UnsupportedLocale> {
        let source = input.as_str();
        let base_end = source.find('-').unwrap_or(source.len());
        let reserved_base_language = ReservedLanguage::parse(&source[..base_end]);
        let transform_range = transform_language_range(source);
        let reserved_transform_language = transform_range
            .as_ref()
            .and_then(|range| ReservedLanguage::parse(&source[range.clone()]));
        let mut structural = source.to_string();
        // Replace the later span first so the base-language range is stable.
        if reserved_transform_language.is_some() {
            structural.replace_range(transform_range.expect("reserved transform span"), "und");
        }
        if reserved_base_language.is_some() {
            structural.replace_range(..base_end, "und");
        }
        let locale = structural
            .parse()
            .map_err(|_| UnsupportedLocale::new(input.clone()))?;
        Ok(Self {
            locale,
            reserved_base_language,
            reserved_transform_language,
        })
    }

    pub(super) fn canonicalize(
        mut self,
        canonicalizer: &LocaleCanonicalizer,
        rules: &ReservedLanguageAliasRules,
    ) -> Result<CanonicalLocaleId, CanonicalizeLocaleError> {
        if self.reserved_base_language.is_some() {
            rules.canonicalize_reserved_identifier(&mut self.locale.id);
        } else {
            self.locale.id = canonicalize_native_identifier(
                core::mem::replace(&mut self.locale.id, LanguageIdentifier::UNKNOWN),
                canonicalizer,
            );
        }
        if let Some(mut transform) = self.locale.extensions.transform.lang.take() {
            if self.reserved_transform_language.is_some() {
                rules.canonicalize_reserved_identifier(&mut transform);
            } else {
                transform = canonicalize_native_identifier(transform, canonicalizer);
            }
            self.locale.extensions.transform.lang = Some(transform);
        }
        rules.canonicalize_subdivision_keywords(&mut self.locale);
        let mut canonical = self.locale.to_string();
        if let Some(language) = self.reserved_transform_language {
            let range = transform_language_range(&canonical)
                .expect("parsed transform language survives canonicalization");
            assert_eq!(&canonical[range.clone()], "und");
            canonical.replace_range(range, &language.0);
        }
        if let Some(language) = self.reserved_base_language {
            assert!(self.locale.id.language.is_unknown());
            canonical.replace_range(..3, &language.0);
        }
        Ok(CanonicalLocaleId::from_data(canonical.into_boxed_str())?)
    }
}

fn canonicalize_native_identifier(
    identifier: LanguageIdentifier,
    canonicalizer: &LocaleCanonicalizer,
) -> LanguageIdentifier {
    let mut locale = Locale::from(identifier);
    canonicalizer.canonicalize(&mut locale);
    locale.id
}

fn transform_language_range(source: &str) -> Option<Range<usize>> {
    let mut subtags = source.split('-');
    let mut offset = 0;
    while let Some(subtag) = subtags.next() {
        let start = offset;
        offset += subtag.len() + 1;
        if subtag.eq_ignore_ascii_case("x") {
            return None;
        }
        if start != 0 && subtag.eq_ignore_ascii_case("t") {
            let language = subtags.next()?;
            return ((2..=3).contains(&language.len()) || (5..=8).contains(&language.len()))
                .then_some(offset..offset + language.len());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use icu_locale::subtags::Region;

    fn canonical(source: &str) -> String {
        ParsedLocale::parse(&LocaleId::parse(source).unwrap())
            .unwrap()
            .canonicalize(
                &LocaleCanonicalizer::new_extended(),
                &ReservedLanguageAliasRules::from_pinned_data().unwrap(),
            )
            .unwrap()
            .as_str()
            .to_string()
    }

    #[test]
    fn pinned_alias_rules_admit_the_reserved_language_domain() {
        let rules = ReservedLanguageAliasRules::from_pinned_data().unwrap();
        assert!(rules.aliases.language.is_empty());
        assert_eq!(rules.aliases.language_variants.len(), 19);
        assert_eq!(rules.wildcard_variants.len(), 10);
        assert!(rules
            .wildcard_variants
            .iter()
            .all(|rule| rule.replacement.language.is_unknown()));
        // The actual lookup schemas cannot contain 5–8-letter language keys.
        let common = Baked::SINGLETON_LOCALE_LIKELY_SUBTAGS_LANGUAGE_V1;
        let extended = Baked::SINGLETON_LOCALE_LIKELY_SUBTAGS_EXTENDED_V1;
        assert!(common
            .language
            .iter_keys()
            .all(|language| core::mem::size_of_val(language) == 3));
        assert!(extended
            .language
            .iter_keys()
            .all(|language| core::mem::size_of_val(language) == 3));
        assert!(common
            .language_script
            .iter_keys()
            .all(|key| core::mem::size_of_val(key) == 7));
        assert!(extended
            .language_script
            .iter_keys()
            .all(|key| core::mem::size_of_val(key) == 7));
        assert!(common
            .language_region
            .iter_keys()
            .all(|key| core::mem::size_of_val(key) == 6));
        assert!(extended
            .language_region
            .iter_keys()
            .all(|key| core::mem::size_of_val(key) == 6));
    }

    #[test]
    fn reserved_languages_keep_their_language_while_resolving_other_aliases() {
        for (source, expected) in [
            ("ABCDE", "abcde"),
            ("abcdef", "abcdef"),
            ("abcdefg", "abcdefg"),
            ("ABCDEFGH-Latn-US", "abcdefgh-Latn-US"),
            ("abcde-Qaai-DD", "abcde-Zinh-DE"),
            ("abcde-hepburn-heploc", "abcde-alalc97"),
            ("abcde-aaland", "abcde-AX"),
            ("abcde-SU-aaland", "abcde-RU"),
            ("abcde-polytoni", "abcde-polyton"),
            ("abcde-Armn-SU", "abcde-Armn-RU"),
            ("und-Armn-SU", "und-Armn-AM"),
        ] {
            assert_eq!(canonical(source), expected, "{source}");
        }
    }

    #[test]
    fn every_complex_territory_uses_its_first_replacement_for_reserved_languages() {
        let aliases = Baked::SINGLETON_LOCALE_ALIASES_V1;
        let alphabetic = (b'A'..=b'Z')
            .flat_map(|a| (b'A'..=b'Z').map(move |b| format!("{}{}", a as char, b as char)));
        let numeric = (0..=999).map(|number| format!("{number:03}"));
        let mut checked = 0;
        for spelling in alphabetic.chain(numeric) {
            let region: Region = spelling.parse().unwrap();
            let Some(replacements) = aliases
                .complex_region
                .get(&region.to_tinystr().to_unvalidated())
            else {
                continue;
            };
            let first = replacements
                .get(0)
                .expect("pinned complex territory has a replacement");
            for script in ["Armn", "Latn"] {
                let source = format!("abcde-{script}-{region}");
                assert_eq!(
                    canonical(&source),
                    format!("abcde-{script}-{first}"),
                    "{source}"
                );
            }
            checked += 1;
        }
        assert_eq!(checked, aliases.complex_region.len());
    }

    #[test]
    fn transform_languages_have_an_independent_domain_and_alias_pass() {
        for (source, expected) in [
            ("en-t-ABCDE", "en-t-abcde"),
            ("abcde-t-iw-il", "abcde-t-he-il"),
            ("iw-t-ABCDE-armn-su", "he-t-abcde-armn-ru"),
            (
                "abcde-Qaai-DD-t-abcdef-Qaai-DD-h0-hybrid-u-ca-gregory-x-t-private",
                "abcde-Zinh-DE-t-abcdef-zinh-de-h0-hybrid-u-ca-gregory-x-t-private",
            ),
            ("en-x-t-ABCDE", "en-x-t-abcde"),
        ] {
            assert_eq!(canonical(source), expected, "{source}");
        }
    }

    #[test]
    fn locale_domain_has_no_fixed_private_use_length_limit() {
        let private = "-abcdefgh".repeat(100);
        assert_eq!(
            canonical(&format!("ABCDE-x{private}")),
            format!("abcde-x{private}")
        );
        assert_eq!(
            canonical(&format!("iw-IL-u-ca-gregory-x{private}")),
            format!("he-IL-u-ca-gregory-x{private}")
        );
    }
}
