//! Typed service algorithms shared by the Intl host operations.
//!
//! JavaScript owns option observation and receiver branding. This module owns
//! locale matching and the provider-backed Collator, PluralRules, and
//! RelativeTimeFormat work after those observations have completed.

use core::{cmp::Ordering, fmt};
use std::{collections::BTreeMap, sync::OnceLock};

use icu_collator::{
    options::{
        AlternateHandling, CaseLevel, CollatorOptions as IcuCollatorOptions, MaxVariable, Strength,
    },
    preferences::{CollationCaseFirst, CollationNumericOrdering},
    Collator, CollatorPreferences,
};
use icu_locale::Locale;
use icu_plurals::{
    PluralCategory as IcuPluralCategory, PluralRuleType, PluralRules,
    PluralRulesOptions as IcuPluralRulesOptions, PluralRulesWithRanges,
};
use serde::Deserialize;

use crate::{
    provider::{CollatorDataProvider, PluralRulesDataProvider},
    CanonicalLocaleId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceLocaleMatcher {
    Lookup,
    BestFit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntlServiceKind {
    Collator,
    PluralRules,
    RelativeTimeFormat,
}

impl IntlServiceKind {
    const fn relevant_unicode_keys(self) -> &'static [&'static str] {
        match self {
            Self::Collator => &["co", "kf", "kn"],
            Self::PluralRules => &[],
            Self::RelativeTimeFormat => &["nu"],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollatorUsage {
    Sort,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollatorSensitivity {
    Base,
    Accent,
    Case,
    Variant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollatorCaseFirst {
    Upper,
    Lower,
    False,
}

/// A syntactically valid Unicode collation type after ECMA-402 option
/// observation. Unsupported values are represented here too; locale data
/// matching decides whether one can be retained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollationOption(Box<str>);

impl CollationOption {
    pub fn parse(source: &str) -> Result<Self, IntlServiceError> {
        if source.is_empty()
            || !source.split('-').all(|part| {
                (3..=8).contains(&part.len())
                    && part.bytes().all(|byte| byte.is_ascii_alphanumeric())
            })
        {
            return Err(IntlServiceError::InvalidOption);
        }
        Ok(Self(source.to_ascii_lowercase().into_boxed_str()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollatorOptions {
    usage: CollatorUsage,
    collation: Option<CollationOption>,
    numeric: Option<bool>,
    case_first: Option<CollatorCaseFirst>,
    sensitivity: Option<CollatorSensitivity>,
    ignore_punctuation: Option<bool>,
}

impl Default for CollatorOptions {
    fn default() -> Self {
        Self {
            usage: CollatorUsage::Sort,
            collation: None,
            numeric: None,
            case_first: None,
            sensitivity: None,
            ignore_punctuation: None,
        }
    }
}

impl CollatorOptions {
    pub fn new(
        usage: CollatorUsage,
        collation: Option<CollationOption>,
        numeric: Option<bool>,
        case_first: Option<CollatorCaseFirst>,
        sensitivity: Option<CollatorSensitivity>,
        ignore_punctuation: Option<bool>,
    ) -> Self {
        Self {
            usage,
            collation,
            numeric,
            case_first,
            sensitivity,
            ignore_punctuation,
        }
    }

    pub const fn usage(&self) -> CollatorUsage {
        self.usage
    }
    pub fn collation(&self) -> Option<&CollationOption> {
        self.collation.as_ref()
    }
    pub const fn numeric(&self) -> Option<bool> {
        self.numeric
    }
    pub const fn case_first(&self) -> Option<CollatorCaseFirst> {
        self.case_first
    }
    pub const fn sensitivity(&self) -> Option<CollatorSensitivity> {
        self.sensitivity
    }
    pub const fn ignore_punctuation(&self) -> Option<bool> {
        self.ignore_punctuation
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCollator {
    locale: CanonicalLocaleId,
    data_locale: CanonicalLocaleId,
    options: ResolvedCollatorOptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCollatorOptions {
    usage: CollatorUsage,
    collation: Box<str>,
    numeric: bool,
    case_first: CollatorCaseFirst,
    sensitivity: CollatorSensitivity,
    ignore_punctuation: bool,
}

impl ResolvedCollator {
    pub fn from_provider(
        locale: CanonicalLocaleId,
        data_locale: CanonicalLocaleId,
        options: ResolvedCollatorOptions,
    ) -> Result<Self, IntlServiceError> {
        let base = base_locale(locale.as_str());
        if base != data_locale.as_str()
            && !base
                .strip_prefix(data_locale.as_str())
                .is_some_and(|suffix| suffix.starts_with('-'))
        {
            return Err(IntlServiceError::InvalidLocale);
        }
        Ok(Self {
            locale,
            data_locale,
            options,
        })
    }

    pub fn locale(&self) -> &CanonicalLocaleId {
        &self.locale
    }
    pub fn data_locale(&self) -> &CanonicalLocaleId {
        &self.data_locale
    }
    pub fn options(&self) -> &ResolvedCollatorOptions {
        &self.options
    }
}

impl ResolvedCollatorOptions {
    pub const fn usage(&self) -> CollatorUsage {
        self.usage
    }
    pub fn collation(&self) -> &str {
        &self.collation
    }
    pub const fn numeric(&self) -> bool {
        self.numeric
    }
    pub const fn case_first(&self) -> CollatorCaseFirst {
        self.case_first
    }
    pub const fn sensitivity(&self) -> CollatorSensitivity {
        self.sensitivity
    }
    pub const fn ignore_punctuation(&self) -> bool {
        self.ignore_punctuation
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollatorLocaleQuery {
    Resolve(CollatorOptions),
    SupportedLocales,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollatorLocaleRequest {
    requested: Box<[CanonicalLocaleId]>,
    matcher: ServiceLocaleMatcher,
    query: CollatorLocaleQuery,
}

impl CollatorLocaleRequest {
    pub fn resolve(
        requested: Box<[CanonicalLocaleId]>,
        matcher: ServiceLocaleMatcher,
        options: CollatorOptions,
    ) -> Self {
        Self {
            requested,
            matcher,
            query: CollatorLocaleQuery::Resolve(options),
        }
    }

    pub fn supported_locales(
        requested: Box<[CanonicalLocaleId]>,
        matcher: ServiceLocaleMatcher,
    ) -> Self {
        Self {
            requested,
            matcher,
            query: CollatorLocaleQuery::SupportedLocales,
        }
    }

    pub fn requested(&self) -> &[CanonicalLocaleId] {
        &self.requested
    }

    pub const fn matcher(&self) -> ServiceLocaleMatcher {
        self.matcher
    }

    pub fn query(&self) -> &CollatorLocaleQuery {
        &self.query
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, IntlServiceError> {
        let mut reader = ServiceReader::new(bytes);
        reader.header(IntlHostOpTag::ResolveCollatorLocale)?;
        let mode = reader.word()?;
        let matcher = match reader.word()? {
            0 => ServiceLocaleMatcher::BestFit,
            1 => ServiceLocaleMatcher::Lookup,
            _ => return Err(IntlServiceError::InvalidWire),
        };
        let requested = reader.locales()?;
        let query = match mode {
            0 => {
                let usage = match reader.word()? {
                    0 => CollatorUsage::Sort,
                    1 => CollatorUsage::Search,
                    _ => return Err(IntlServiceError::InvalidWire),
                };
                let collation = reader.utf8()?;
                let collation = (!collation.is_empty())
                    .then(|| CollationOption::parse(&collation))
                    .transpose()?;
                let numeric = reader.optional_bool()?;
                let case_first = match reader.word()? {
                    0 => None,
                    1 => Some(CollatorCaseFirst::False),
                    2 => Some(CollatorCaseFirst::Upper),
                    3 => Some(CollatorCaseFirst::Lower),
                    _ => return Err(IntlServiceError::InvalidWire),
                };
                let sensitivity = match reader.word()? {
                    0 => None,
                    1 => Some(CollatorSensitivity::Base),
                    2 => Some(CollatorSensitivity::Accent),
                    3 => Some(CollatorSensitivity::Case),
                    4 => Some(CollatorSensitivity::Variant),
                    _ => return Err(IntlServiceError::InvalidWire),
                };
                let ignore_punctuation = reader.optional_bool()?;
                CollatorLocaleQuery::Resolve(CollatorOptions::new(
                    usage,
                    collation,
                    numeric,
                    case_first,
                    sensitivity,
                    ignore_punctuation,
                ))
            }
            1 => CollatorLocaleQuery::SupportedLocales,
            _ => return Err(IntlServiceError::InvalidWire),
        };
        reader.finish()?;
        Ok(Self {
            requested,
            matcher,
            query,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollatorLocaleResult {
    Resolved(ResolvedCollator),
    SupportedLocales(Box<[CanonicalLocaleId]>),
}

impl CollatorLocaleResult {
    pub fn encode(&self) -> Result<Vec<u8>, IntlServiceError> {
        let mut writer = ServiceWriter::new(IntlHostOpTag::ResolveCollatorLocale);
        match self {
            Self::Resolved(configuration) => {
                writer.word(0);
                writer.utf8(configuration.locale().as_str())?;
                writer.utf8(configuration.data_locale().as_str())?;
                writer.word(match configuration.options().usage() {
                    CollatorUsage::Sort => 0,
                    CollatorUsage::Search => 1,
                });
                writer.utf8(configuration.options().collation())?;
                writer.word(u64::from(configuration.options().numeric()));
                writer.word(match configuration.options().case_first() {
                    CollatorCaseFirst::False => 1,
                    CollatorCaseFirst::Upper => 2,
                    CollatorCaseFirst::Lower => 3,
                });
                writer.word(match configuration.options().sensitivity() {
                    CollatorSensitivity::Base => 1,
                    CollatorSensitivity::Accent => 2,
                    CollatorSensitivity::Case => 3,
                    CollatorSensitivity::Variant => 4,
                });
                writer.word(u64::from(configuration.options().ignore_punctuation()));
            }
            Self::SupportedLocales(locales) => {
                writer.word(1);
                writer.word(locales.len() as u64);
                for locale in locales.iter() {
                    writer.utf8(locale.as_str())?;
                }
            }
        }
        Ok(writer.finish())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollatorCompareRequest {
    configuration: ResolvedCollator,
    left: Box<[u16]>,
    right: Box<[u16]>,
}

impl CollatorCompareRequest {
    pub fn new(configuration: ResolvedCollator, left: Box<[u16]>, right: Box<[u16]>) -> Self {
        Self {
            configuration,
            left,
            right,
        }
    }

    pub fn configuration(&self) -> &ResolvedCollator {
        &self.configuration
    }

    pub fn left(&self) -> &[u16] {
        &self.left
    }

    pub fn right(&self) -> &[u16] {
        &self.right
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, IntlServiceError> {
        let mut reader = ServiceReader::new(bytes);
        reader.header(IntlHostOpTag::CompareCollator)?;
        let locale = reader.locale()?;
        let data_locale = reader.locale()?;
        let usage = match reader.word()? {
            0 => CollatorUsage::Sort,
            1 => CollatorUsage::Search,
            _ => return Err(IntlServiceError::InvalidWire),
        };
        let collation = reader.utf8()?.into_boxed_str();
        if collation.is_empty() {
            return Err(IntlServiceError::InvalidWire);
        }
        let numeric = reader.bool()?;
        let case_first = match reader.word()? {
            1 => CollatorCaseFirst::False,
            2 => CollatorCaseFirst::Upper,
            3 => CollatorCaseFirst::Lower,
            _ => return Err(IntlServiceError::InvalidWire),
        };
        let sensitivity = match reader.word()? {
            1 => CollatorSensitivity::Base,
            2 => CollatorSensitivity::Accent,
            3 => CollatorSensitivity::Case,
            4 => CollatorSensitivity::Variant,
            _ => return Err(IntlServiceError::InvalidWire),
        };
        let ignore_punctuation = reader.bool()?;
        let left = reader.utf16()?;
        let right = reader.utf16()?;
        reader.finish()?;
        let options = ResolvedCollatorOptions {
            usage,
            collation,
            numeric,
            case_first,
            sensitivity,
            ignore_punctuation,
        };
        let configuration = ResolvedCollator::from_provider(locale, data_locale, options)?;
        Ok(Self::new(configuration, left, right))
    }
}

/// Versioned service wire shared by the AOT emitter and engine host.
pub const COLLATOR_WIRE_VERSION: u64 = 1;
pub const COLLATOR_WIRE_HEADER_BYTES: u64 = 16;

#[derive(Clone, Copy)]
enum IntlHostOpTag {
    ResolveCollatorLocale,
    CompareCollator,
}

impl IntlHostOpTag {
    const fn request_tag(self) -> u64 {
        match self {
            Self::ResolveCollatorLocale => {
                crate::IntlHostOp::ResolveCollatorLocale.code() as u64 * 2
            }
            Self::CompareCollator => crate::IntlHostOp::CompareCollator.code() as u64 * 2,
        }
    }
    const fn response_tag(self) -> u64 {
        self.request_tag() + 1
    }
}

struct ServiceReader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> ServiceReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }
    fn header(&mut self, operation: IntlHostOpTag) -> Result<(), IntlServiceError> {
        if self.word()? != COLLATOR_WIRE_VERSION || self.word()? != operation.request_tag() {
            return Err(IntlServiceError::InvalidWire);
        }
        Ok(())
    }
    fn word(&mut self) -> Result<u64, IntlServiceError> {
        let end = self
            .cursor
            .checked_add(8)
            .ok_or(IntlServiceError::InvalidWire)?;
        let word = self
            .bytes
            .get(self.cursor..end)
            .ok_or(IntlServiceError::InvalidWire)?;
        self.cursor = end;
        Ok(u64::from_le_bytes(word.try_into().expect("8-byte slice")))
    }
    fn byte_string(&mut self) -> Result<&'a [u8], IntlServiceError> {
        let length = usize::try_from(self.word()?).map_err(|_| IntlServiceError::InvalidWire)?;
        let end = self
            .cursor
            .checked_add(length)
            .ok_or(IntlServiceError::InvalidWire)?;
        let bytes = self
            .bytes
            .get(self.cursor..end)
            .ok_or(IntlServiceError::InvalidWire)?;
        self.cursor = end;
        Ok(bytes)
    }
    fn utf8(&mut self) -> Result<String, IntlServiceError> {
        core::str::from_utf8(self.byte_string()?)
            .map(str::to_owned)
            .map_err(|_| IntlServiceError::InvalidWire)
    }
    fn locale(&mut self) -> Result<CanonicalLocaleId, IntlServiceError> {
        CanonicalLocaleId::from_data(self.utf8()?.into_boxed_str())
            .map_err(|_| IntlServiceError::InvalidLocale)
    }
    fn locales(&mut self) -> Result<Box<[CanonicalLocaleId]>, IntlServiceError> {
        let count = usize::try_from(self.word()?).map_err(|_| IntlServiceError::InvalidWire)?;
        let mut locales = Vec::new();
        locales
            .try_reserve_exact(count)
            .map_err(|_| IntlServiceError::InvalidWire)?;
        for _ in 0..count {
            locales.push(self.locale()?);
        }
        Ok(locales.into_boxed_slice())
    }
    fn bool(&mut self) -> Result<bool, IntlServiceError> {
        match self.word()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(IntlServiceError::InvalidWire),
        }
    }
    fn optional_bool(&mut self) -> Result<Option<bool>, IntlServiceError> {
        match self.word()? {
            0 => Ok(None),
            1 => Ok(Some(false)),
            2 => Ok(Some(true)),
            _ => Err(IntlServiceError::InvalidWire),
        }
    }
    fn utf16(&mut self) -> Result<Box<[u16]>, IntlServiceError> {
        let count = usize::try_from(self.word()?).map_err(|_| IntlServiceError::InvalidWire)?;
        let byte_length = count.checked_mul(2).ok_or(IntlServiceError::InvalidWire)?;
        let end = self
            .cursor
            .checked_add(byte_length)
            .ok_or(IntlServiceError::InvalidWire)?;
        let bytes = self
            .bytes
            .get(self.cursor..end)
            .ok_or(IntlServiceError::InvalidWire)?;
        self.cursor = end;
        let mut units = Vec::new();
        units
            .try_reserve_exact(count)
            .map_err(|_| IntlServiceError::InvalidWire)?;
        units.extend(
            bytes
                .chunks_exact(2)
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]])),
        );
        Ok(units.into_boxed_slice())
    }
    fn finish(self) -> Result<(), IntlServiceError> {
        (self.cursor == self.bytes.len())
            .then_some(())
            .ok_or(IntlServiceError::InvalidWire)
    }
}

struct ServiceWriter(Vec<u8>);

impl ServiceWriter {
    fn new(operation: IntlHostOpTag) -> Self {
        let mut writer = Self(Vec::with_capacity(64));
        writer.word(COLLATOR_WIRE_VERSION);
        writer.word(operation.response_tag());
        writer
    }
    fn word(&mut self, value: u64) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }
    fn utf8(&mut self, value: &str) -> Result<(), IntlServiceError> {
        self.word(u64::try_from(value.len()).map_err(|_| IntlServiceError::InvalidWire)?);
        self.0.extend_from_slice(value.as_bytes());
        Ok(())
    }
    fn finish(self) -> Vec<u8> {
        self.0
    }
}

pub fn encode_collator_comparison(result: i32) -> Result<Vec<u8>, IntlServiceError> {
    if !(-1..=1).contains(&result) {
        return Err(IntlServiceError::InvalidWire);
    }
    let mut writer = ServiceWriter::new(IntlHostOpTag::CompareCollator);
    writer.word(result as i64 as u64);
    Ok(writer.finish())
}

pub fn resolve_collator(
    locale: CanonicalLocaleId,
    data_locale: CanonicalLocaleId,
    requested: CollatorOptions,
    supported_collations: &[Box<str>],
) -> Result<ResolvedCollator, IntlServiceError> {
    let original_locale = locale
        .as_str()
        .parse::<Locale>()
        .map_err(|_| IntlServiceError::InvalidLocale)?;
    // The internal provider record retains only the three Locale options keys
    // exposed by Intl.Collator. Candidate matching has already selected the
    // base locale, so all other extensions are irrelevant to this service.
    let mut public_icu_locale = base_locale(locale.as_str())
        .parse::<Locale>()
        .map_err(|_| IntlServiceError::InvalidLocale)?;
    let original_keywords = original_locale.extensions.unicode.keywords;
    let original_collation = original_keywords
        .get(&icu_locale::extensions::unicode::key!("co"))
        .map(ToString::to_string);
    let original_case_first = original_keywords
        .get(&icu_locale::extensions::unicode::key!("kf"))
        .map(ToString::to_string);
    let original_numeric = original_keywords
        .get(&icu_locale::extensions::unicode::key!("kn"))
        .map(ToString::to_string);
    let extension_collation = original_collation
        .as_deref()
        .filter(|value| supported_collation(value, supported_collations));
    let extension_case_first = original_case_first
        .as_deref()
        .and_then(parse_case_first_keyword);
    let extension_numeric = original_numeric.as_deref().and_then(parse_numeric_keyword);

    let option_collation = requested
        .collation
        .as_ref()
        .map(CollationOption::as_str)
        .filter(|value| supported_collation(value, supported_collations));
    let selected_collation = if requested.usage == CollatorUsage::Search {
        // ECMA-402 exposes the default collation for search usage. Locale data
        // may select a separate operational search tailoring below.
        "default"
    } else {
        // An unsupported option is ignored. ResolveLocale may still select a
        // supported `co` value from the requested locale extension.
        option_collation
            .or(extension_collation)
            .unwrap_or("default")
    };
    let selected_case_first = requested.case_first.or(extension_case_first);
    let selected_numeric = requested.numeric.or(extension_numeric).unwrap_or(false);
    let ignore_punctuation = requested
        .ignore_punctuation
        .unwrap_or_else(|| default_ignore_punctuation(data_locale.as_str()));

    // ResolveLocale retains only service keys whose requested extension value
    // is the value actually selected after applying explicit options and the
    // locale's supported data. An option that differs from an extension must
    // remove that extension from the public resolved locale.
    set_keyword(
        &mut public_icu_locale,
        "co",
        (requested.usage == CollatorUsage::Sort
            && selected_collation != "default"
            && original_collation.as_deref() == Some(selected_collation))
        .then_some(selected_collation),
    )?;
    set_keyword(
        &mut public_icu_locale,
        "kf",
        (original_case_first
            .as_deref()
            .and_then(parse_case_first_keyword)
            .zip(selected_case_first)
            .is_some_and(|(extension, selected)| extension == selected))
        .then_some(match selected_case_first {
            Some(CollatorCaseFirst::Upper) => "upper",
            Some(CollatorCaseFirst::Lower) => "lower",
            Some(CollatorCaseFirst::False) => "false",
            None => "false",
        }),
    )?;
    set_keyword(
        &mut public_icu_locale,
        "kn",
        (original_numeric.as_deref().and_then(parse_numeric_keyword) == Some(selected_numeric))
            .then_some(if selected_numeric { "true" } else { "false" }),
    )?;
    let public_locale =
        CanonicalLocaleId::from_data(public_icu_locale.to_string().into_boxed_str())
            .map_err(|_| IntlServiceError::InvalidLocale)?;

    let preferences_tag = if requested.usage == CollatorUsage::Search {
        base_locale(data_locale.as_str())
    } else {
        base_locale(locale.as_str())
    };
    let mut preferences_locale = preferences_tag
        .parse::<Locale>()
        .map_err(|_| IntlServiceError::InvalidLocale)?;
    if requested.usage == CollatorUsage::Search {
        set_keyword(&mut preferences_locale, "co", Some("search"))?;
    } else {
        set_keyword(
            &mut preferences_locale,
            "co",
            (selected_collation != "default").then_some(selected_collation),
        )?;
    }
    let mut preferences: CollatorPreferences = preferences_locale.into();
    if let Some(case_first) = selected_case_first {
        preferences.case_first = Some(match case_first {
            CollatorCaseFirst::Upper => CollationCaseFirst::Upper,
            CollatorCaseFirst::Lower => CollationCaseFirst::Lower,
            CollatorCaseFirst::False => CollationCaseFirst::False,
        });
    }
    preferences.numeric_ordering = Some(if selected_numeric {
        CollationNumericOrdering::True
    } else {
        CollationNumericOrdering::False
    });
    let mut icu_options = IcuCollatorOptions::default();
    if let Some(sensitivity) = requested.sensitivity {
        let (strength, case_level) = sensitivity_strength(sensitivity);
        icu_options.strength = Some(strength);
        icu_options.case_level = Some(case_level);
    } else if requested.usage == CollatorUsage::Search {
        icu_options.strength = Some(Strength::Primary);
        icu_options.case_level = Some(CaseLevel::Off);
    } else {
        // ECMA-402's default sort sensitivity is variant regardless of any
        // lower default strength in ICU data for a locale.
        icu_options.strength = Some(Strength::Tertiary);
        icu_options.case_level = Some(CaseLevel::Off);
    }
    icu_options.alternate_handling = Some(if ignore_punctuation {
        AlternateHandling::Shifted
    } else {
        AlternateHandling::NonIgnorable
    });
    if ignore_punctuation {
        icu_options.max_variable = Some(MaxVariable::Punctuation);
    }
    let collator = Collator::try_new_unstable(&CollatorDataProvider, preferences, icu_options)
        .map_err(|error| IntlServiceError::Provider(error.to_string()))?;
    let resolved = collator.as_borrowed().resolved_options();
    let options = ResolvedCollatorOptions {
        usage: requested.usage,
        collation: selected_collation.to_owned().into_boxed_str(),
        numeric: resolved.numeric == CollationNumericOrdering::True,
        case_first: match resolved.case_first {
            CollationCaseFirst::Upper => CollatorCaseFirst::Upper,
            CollationCaseFirst::Lower => CollatorCaseFirst::Lower,
            CollationCaseFirst::False => CollatorCaseFirst::False,
            _ => CollatorCaseFirst::False,
        },
        sensitivity: sensitivity_from_strength(resolved.strength, resolved.case_level),
        ignore_punctuation: resolved.alternate_handling == AlternateHandling::Shifted,
    };
    ResolvedCollator::from_provider(public_locale, data_locale, options)
}

fn supported_collation(value: &str, supported: &[Box<str>]) -> bool {
    supported
        .iter()
        .any(|candidate| candidate.as_ref() == value)
}

fn parse_case_first_keyword(value: &str) -> Option<CollatorCaseFirst> {
    match value {
        "upper" => Some(CollatorCaseFirst::Upper),
        "lower" => Some(CollatorCaseFirst::Lower),
        "false" => Some(CollatorCaseFirst::False),
        _ => None,
    }
}

fn parse_numeric_keyword(value: &str) -> Option<bool> {
    match value {
        "true" | "" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn default_ignore_punctuation(locale: &str) -> bool {
    // ECMA-402 LocaleData uses true for Thai's default collation and false
    // elsewhere in the pinned ICU/CLDR data surface.
    locale.split('-').next() == Some("th")
}

fn set_keyword(
    locale: &mut Locale,
    key: &str,
    value: Option<&str>,
) -> Result<(), IntlServiceError> {
    let parsed = value
        .map(icu_locale::extensions::unicode::Value::try_from_str)
        .transpose()
        .map_err(|_| IntlServiceError::InvalidLocale)?;
    let keywords = &mut locale.extensions.unicode.keywords;
    match key {
        "co" => match parsed {
            Some(value) => {
                keywords.set(icu_locale::extensions::unicode::key!("co"), value);
            }
            None => {
                keywords.remove(icu_locale::extensions::unicode::key!("co"));
            }
        },
        "kf" => match parsed {
            Some(value) => {
                keywords.set(icu_locale::extensions::unicode::key!("kf"), value);
            }
            None => {
                keywords.remove(icu_locale::extensions::unicode::key!("kf"));
            }
        },
        "kn" => match parsed {
            Some(value) => {
                keywords.set(icu_locale::extensions::unicode::key!("kn"), value);
            }
            None => {
                keywords.remove(icu_locale::extensions::unicode::key!("kn"));
            }
        },
        _ => return Err(IntlServiceError::InvalidWire),
    }
    Ok(())
}

fn sensitivity_strength(sensitivity: CollatorSensitivity) -> (Strength, CaseLevel) {
    match sensitivity {
        CollatorSensitivity::Base => (Strength::Primary, CaseLevel::Off),
        CollatorSensitivity::Accent => (Strength::Secondary, CaseLevel::Off),
        CollatorSensitivity::Case => (Strength::Primary, CaseLevel::On),
        CollatorSensitivity::Variant => (Strength::Tertiary, CaseLevel::Off),
    }
}

fn sensitivity_from_strength(strength: Strength, case_level: CaseLevel) -> CollatorSensitivity {
    match (strength, case_level) {
        (Strength::Primary, CaseLevel::Off) => CollatorSensitivity::Base,
        (Strength::Primary, CaseLevel::On) => CollatorSensitivity::Case,
        (Strength::Secondary, _) => CollatorSensitivity::Accent,
        _ => CollatorSensitivity::Variant,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluralRulesType {
    Cardinal,
    Ordinal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluralCategory {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

impl PluralCategory {
    pub const ALL: [Self; 6] = [
        Self::Zero,
        Self::One,
        Self::Two,
        Self::Few,
        Self::Many,
        Self::Other,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Zero => "zero",
            Self::One => "one",
            Self::Two => "two",
            Self::Few => "few",
            Self::Many => "many",
            Self::Other => "other",
        }
    }

    const fn bit(self) -> u8 {
        match self {
            Self::Zero => 1 << 0,
            Self::One => 1 << 1,
            Self::Two => 1 << 2,
            Self::Few => 1 << 3,
            Self::Many => 1 << 4,
            Self::Other => 1 << 5,
        }
    }

    pub const fn wire_code(self) -> u64 {
        match self {
            Self::Zero => 1,
            Self::One => 2,
            Self::Two => 3,
            Self::Few => 4,
            Self::Many => 5,
            Self::Other => 6,
        }
    }

    pub const fn from_wire_code(code: u64) -> Option<Self> {
        match code {
            1 => Some(Self::Zero),
            2 => Some(Self::One),
            3 => Some(Self::Two),
            4 => Some(Self::Few),
            5 => Some(Self::Many),
            6 => Some(Self::Other),
            _ => None,
        }
    }

    fn from_icu(category: IcuPluralCategory) -> Self {
        match category {
            IcuPluralCategory::Zero => Self::Zero,
            IcuPluralCategory::One => Self::One,
            IcuPluralCategory::Two => Self::Two,
            IcuPluralCategory::Few => Self::Few,
            IcuPluralCategory::Many => Self::Many,
            IcuPluralCategory::Other => Self::Other,
        }
    }

    fn to_icu(self) -> IcuPluralCategory {
        match self {
            Self::Zero => IcuPluralCategory::Zero,
            Self::One => IcuPluralCategory::One,
            Self::Two => IcuPluralCategory::Two,
            Self::Few => IcuPluralCategory::Few,
            Self::Many => IcuPluralCategory::Many,
            Self::Other => IcuPluralCategory::Other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluralRulesOptions {
    rule_type: PluralRulesType,
    number: crate::number_format::options::NumberFormatOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluralRulesPrecision {
    Fraction(crate::number_format::options::FractionDigitRange),
    Significant(crate::number_format::options::SignificantDigitRange),
}

impl PluralRulesPrecision {
    pub fn fraction(minimum: u8, maximum: u8) -> Result<Self, IntlServiceError> {
        use crate::number_format::options::{FractionDigitCount, FractionDigitRange};
        let minimum =
            FractionDigitCount::new(minimum).map_err(|_| IntlServiceError::InvalidOption)?;
        let maximum =
            FractionDigitCount::new(maximum).map_err(|_| IntlServiceError::InvalidOption)?;
        Ok(Self::Fraction(
            FractionDigitRange::new(minimum, maximum)
                .map_err(|_| IntlServiceError::InvalidOption)?,
        ))
    }

    pub fn significant(minimum: u8, maximum: u8) -> Result<Self, IntlServiceError> {
        use crate::number_format::options::{SignificantDigitCount, SignificantDigitRange};
        let minimum =
            SignificantDigitCount::new(minimum).map_err(|_| IntlServiceError::InvalidOption)?;
        let maximum =
            SignificantDigitCount::new(maximum).map_err(|_| IntlServiceError::InvalidOption)?;
        Ok(Self::Significant(
            SignificantDigitRange::new(minimum, maximum)
                .map_err(|_| IntlServiceError::InvalidOption)?,
        ))
    }
}

impl PluralRulesOptions {
    pub fn new(
        rule_type: PluralRulesType,
        minimum_integer_digits: u8,
        precision: PluralRulesPrecision,
    ) -> Result<Self, IntlServiceError> {
        use crate::number_format::options::{
            FractionPrecision, Grouping, IntegerDigitCount, Notation, NumberFormatOptions,
            NumberStyle, RoundingMode, SignDisplay, TrailingZeroDisplay,
        };
        let minimum_integer_digits = IntegerDigitCount::new(minimum_integer_digits)
            .map_err(|_| IntlServiceError::InvalidOption)?;
        let precision = match precision {
            PluralRulesPrecision::Fraction(range) => {
                crate::number_format::options::Precision::Fraction(FractionPrecision::Range(range))
            }
            PluralRulesPrecision::Significant(range) => {
                crate::number_format::options::Precision::Significant(range)
            }
        };
        let number = NumberFormatOptions {
            style: NumberStyle::Decimal,
            notation: Notation::Standard,
            minimum_integer_digits,
            precision,
            rounding_mode: RoundingMode::HalfExpand,
            trailing_zero_display: TrailingZeroDisplay::Auto,
            grouping: Grouping::Never,
            sign_display: SignDisplay::Auto,
        };
        Ok(Self { rule_type, number })
    }

    pub fn from_number_format_options(
        rule_type: PluralRulesType,
        number: crate::number_format::options::NumberFormatOptions,
    ) -> Result<Self, IntlServiceError> {
        use crate::number_format::options::{NumberStyle, SignDisplay};
        if number.style != NumberStyle::Decimal || number.sign_display != SignDisplay::Auto {
            return Err(IntlServiceError::InvalidOption);
        }
        Ok(Self { rule_type, number })
    }

    pub fn wire_words(&self) -> [u64; crate::NUMBER_CONFIGURATION_WORDS] {
        self.number.wire_words()
    }

    pub(crate) fn from_wire_words(
        rule_type: PluralRulesType,
        words: [u64; crate::NUMBER_CONFIGURATION_WORDS],
    ) -> Result<Self, IntlServiceError> {
        let number = crate::number_protocol::decode_number_options_words(words, "")
            .map_err(|_| IntlServiceError::InvalidWire)?;
        Self::from_number_format_options(rule_type, number)
    }

    fn validate(&self) -> Result<(), IntlServiceError> {
        use crate::number_format::options::{NumberStyle, SignDisplay};
        if self.number.style != NumberStyle::Decimal
            || self.number.sign_display != SignDisplay::Auto
        {
            return Err(IntlServiceError::InvalidOption);
        }
        Ok(())
    }

    pub const fn rule_type(&self) -> PluralRulesType {
        self.rule_type
    }
    pub const fn minimum_integer_digits(&self) -> u8 {
        self.number.minimum_integer_digits.get()
    }
    pub const fn notation(&self) -> crate::number_format::options::Notation {
        self.number.notation
    }
    pub fn number_options(&self) -> &crate::number_format::options::NumberFormatOptions {
        &self.number
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPluralRules {
    locale: CanonicalLocaleId,
    data_locale: CanonicalLocaleId,
    options: PluralRulesOptions,
    categories: Box<[PluralCategory]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluralRulesLocaleQuery {
    Resolve(PluralRulesOptions),
    SupportedLocales,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluralRulesLocaleRequest {
    requested: Box<[CanonicalLocaleId]>,
    matcher: ServiceLocaleMatcher,
    query: PluralRulesLocaleQuery,
}

impl PluralRulesLocaleRequest {
    pub fn resolve(
        requested: Box<[CanonicalLocaleId]>,
        matcher: ServiceLocaleMatcher,
        options: PluralRulesOptions,
    ) -> Self {
        Self {
            requested,
            matcher,
            query: PluralRulesLocaleQuery::Resolve(options),
        }
    }

    pub fn supported_locales(
        requested: Box<[CanonicalLocaleId]>,
        matcher: ServiceLocaleMatcher,
    ) -> Self {
        Self {
            requested,
            matcher,
            query: PluralRulesLocaleQuery::SupportedLocales,
        }
    }

    pub fn requested(&self) -> &[CanonicalLocaleId] {
        &self.requested
    }

    pub const fn matcher(&self) -> ServiceLocaleMatcher {
        self.matcher
    }

    pub fn query(&self) -> &PluralRulesLocaleQuery {
        &self.query
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluralRulesLocaleResult {
    Resolved(ResolvedPluralRules),
    SupportedLocales(Box<[CanonicalLocaleId]>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluralCategoryInput {
    Single(crate::number_format::numeric::ObservedNumericInput),
    Range {
        start: crate::number_format::numeric::ObservedNumericInput,
        end: crate::number_format::numeric::ObservedNumericInput,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluralCategoryRequest {
    configuration: ResolvedPluralRules,
    input: PluralCategoryInput,
}

impl PluralCategoryRequest {
    pub fn select(
        configuration: ResolvedPluralRules,
        number: crate::number_format::numeric::ObservedNumericInput,
    ) -> Self {
        Self {
            configuration,
            input: PluralCategoryInput::Single(number),
        }
    }

    pub fn select_range(
        configuration: ResolvedPluralRules,
        start: crate::number_format::numeric::ObservedNumericInput,
        end: crate::number_format::numeric::ObservedNumericInput,
    ) -> Self {
        Self {
            configuration,
            input: PluralCategoryInput::Range { start, end },
        }
    }

    pub fn configuration(&self) -> &ResolvedPluralRules {
        &self.configuration
    }

    pub fn input(&self) -> &PluralCategoryInput {
        &self.input
    }
}

/// Executes a PluralRules selection using the provider's pinned NumberFormat
/// locale profile for notation-sensitive rounding and compact operands.
pub fn execute_plural_category_with_profiles(
    request: PluralCategoryRequest,
    profiles: &crate::number_format::NumberProfiles,
) -> Result<PluralCategory, IntlServiceError> {
    match request.input {
        PluralCategoryInput::Single(number) => {
            plural_category_with_profiles(&request.configuration, number, profiles)
        }
        PluralCategoryInput::Range { start, end } => {
            plural_category_range_with_profiles(&request.configuration, start, end, profiles)
        }
    }
}

impl ResolvedPluralRules {
    pub fn from_provider(
        locale: CanonicalLocaleId,
        data_locale: CanonicalLocaleId,
        options: PluralRulesOptions,
    ) -> Result<Self, IntlServiceError> {
        options.validate()?;
        let base = base_locale(locale.as_str());
        if base != data_locale.as_str()
            && !base
                .strip_prefix(data_locale.as_str())
                .is_some_and(|suffix| suffix.starts_with('-'))
        {
            return Err(IntlServiceError::InvalidLocale);
        }
        let locale_for_rules = data_locale
            .as_str()
            .parse::<Locale>()
            .map_err(|_| IntlServiceError::InvalidLocale)?;
        let categories = plural_rules(locale_for_rules, options.rule_type)?
            .categories()
            .map(PluralCategory::from_icu)
            .collect();
        Ok(Self {
            locale,
            data_locale,
            options,
            categories,
        })
    }
    pub fn locale(&self) -> &CanonicalLocaleId {
        &self.locale
    }
    pub fn data_locale(&self) -> &CanonicalLocaleId {
        &self.data_locale
    }
    pub fn options(&self) -> &PluralRulesOptions {
        &self.options
    }
    pub fn categories(&self) -> &[PluralCategory] {
        &self.categories
    }
    pub fn category_mask(&self) -> u8 {
        self.categories
            .iter()
            .fold(0, |mask, category| mask | category.bit())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeTimeStyle {
    Long,
    Short,
    Narrow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeTimeNumeric {
    Always,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeTimeNumberKind {
    ShortestDecimal,
    NegativeZero,
}

impl RelativeTimeNumberKind {
    pub const fn wire_code(self) -> u64 {
        match self {
            Self::ShortestDecimal => 2,
            Self::NegativeZero => 3,
        }
    }

    pub const fn from_wire_code(code: u64) -> Option<Self> {
        match code {
            2 => Some(Self::ShortestDecimal),
            3 => Some(Self::NegativeZero),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativeTimeFormatOptions {
    style: RelativeTimeStyle,
    numeric: RelativeTimeNumeric,
    numbering_system: Option<crate::number_format::NumberingSystemOption>,
}

impl Default for RelativeTimeFormatOptions {
    fn default() -> Self {
        Self {
            style: RelativeTimeStyle::Long,
            numeric: RelativeTimeNumeric::Always,
            numbering_system: None,
        }
    }
}

impl RelativeTimeFormatOptions {
    pub const fn new(style: RelativeTimeStyle, numeric: RelativeTimeNumeric) -> Self {
        Self {
            style,
            numeric,
            numbering_system: None,
        }
    }

    pub fn with_numbering_system(
        mut self,
        numbering_system: Option<crate::number_format::NumberingSystemOption>,
    ) -> Self {
        self.numbering_system = numbering_system;
        self
    }

    pub const fn style(&self) -> RelativeTimeStyle {
        self.style
    }
    pub const fn numeric(&self) -> RelativeTimeNumeric {
        self.numeric
    }

    pub fn numbering_system(&self) -> Option<&crate::number_format::NumberingSystemOption> {
        self.numbering_system.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRelativeTimeFormat {
    locale: CanonicalLocaleId,
    data_locale: CanonicalLocaleId,
    number_locale: crate::number_format::ResolvedNumberLocale,
    options: RelativeTimeFormatOptions,
}

impl ResolvedRelativeTimeFormat {
    pub fn from_provider(
        locale: CanonicalLocaleId,
        data_locale: CanonicalLocaleId,
        options: RelativeTimeFormatOptions,
    ) -> Result<Self, IntlServiceError> {
        let profiles = crate::number_format::embedded_number_profiles()
            .map_err(|error| IntlServiceError::Provider(error.to_string()))?;
        let number_locale = crate::number_format::resolve_number_locale(
            &crate::number_format::NumberLocaleRequest {
                requested: vec![locale.clone()].into_boxed_slice(),
                matcher: crate::number_format::options::LocaleMatcher::Lookup,
                numbering_system: options.numbering_system.clone(),
            },
            profiles,
        )
        .map_err(|error| IntlServiceError::Provider(error.to_string()))?;
        Self::from_number_locale(
            number_locale.resolved().clone(),
            data_locale,
            options,
            number_locale,
        )
    }

    pub fn from_resolved(
        locale: CanonicalLocaleId,
        data_locale: CanonicalLocaleId,
        options: RelativeTimeFormatOptions,
        numbering_system: &str,
    ) -> Result<Self, IntlServiceError> {
        let numbering_system = crate::number_format::NumberingSystemOption::parse(numbering_system)
            .map_err(|_| IntlServiceError::InvalidOption)?;
        let profiles = crate::number_format::embedded_number_profiles()
            .map_err(|error| IntlServiceError::Provider(error.to_string()))?;
        let number_locale = crate::number_format::resolve_number_locale(
            &crate::number_format::NumberLocaleRequest {
                requested: vec![locale.clone()].into_boxed_slice(),
                matcher: crate::number_format::options::LocaleMatcher::Lookup,
                numbering_system: Some(numbering_system),
            },
            profiles,
        )
        .map_err(|error| IntlServiceError::Provider(error.to_string()))?;
        if number_locale.resolved() != &locale {
            return Err(IntlServiceError::InvalidWire);
        }
        Self::from_number_locale(locale, data_locale, options, number_locale)
    }

    fn from_number_locale(
        locale: CanonicalLocaleId,
        data_locale: CanonicalLocaleId,
        options: RelativeTimeFormatOptions,
        number_locale: crate::number_format::ResolvedNumberLocale,
    ) -> Result<Self, IntlServiceError> {
        let base = base_locale(locale.as_str());
        if base != data_locale.as_str()
            && !base
                .strip_prefix(data_locale.as_str())
                .is_some_and(|suffix| suffix.starts_with('-'))
        {
            return Err(IntlServiceError::InvalidLocale);
        }
        Ok(Self {
            locale,
            data_locale,
            number_locale,
            options,
        })
    }
    pub fn locale(&self) -> &CanonicalLocaleId {
        &self.locale
    }
    pub fn data_locale(&self) -> &CanonicalLocaleId {
        &self.data_locale
    }
    pub const fn options(&self) -> &RelativeTimeFormatOptions {
        &self.options
    }

    pub fn numbering_system(&self) -> &str {
        self.number_locale.numbering_system().name()
    }

    pub const fn number_locale(&self) -> &crate::number_format::ResolvedNumberLocale {
        &self.number_locale
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelativeTimeLocaleQuery {
    Resolve(RelativeTimeFormatOptions),
    SupportedLocales,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativeTimeLocaleRequest {
    requested: Box<[CanonicalLocaleId]>,
    matcher: ServiceLocaleMatcher,
    query: RelativeTimeLocaleQuery,
}

impl RelativeTimeLocaleRequest {
    pub fn resolve(
        requested: Box<[CanonicalLocaleId]>,
        matcher: ServiceLocaleMatcher,
        options: RelativeTimeFormatOptions,
    ) -> Self {
        Self {
            requested,
            matcher,
            query: RelativeTimeLocaleQuery::Resolve(options),
        }
    }

    pub fn supported_locales(
        requested: Box<[CanonicalLocaleId]>,
        matcher: ServiceLocaleMatcher,
    ) -> Self {
        Self {
            requested,
            matcher,
            query: RelativeTimeLocaleQuery::SupportedLocales,
        }
    }

    pub fn requested(&self) -> &[CanonicalLocaleId] {
        &self.requested
    }

    pub const fn matcher(&self) -> ServiceLocaleMatcher {
        self.matcher
    }

    pub const fn query(&self) -> &RelativeTimeLocaleQuery {
        &self.query
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelativeTimeLocaleResult {
    Resolved(ResolvedRelativeTimeFormat),
    SupportedLocales(Box<[CanonicalLocaleId]>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeTimeUnit {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

impl RelativeTimeUnit {
    pub const ALL: [Self; 8] = [
        Self::Second,
        Self::Minute,
        Self::Hour,
        Self::Day,
        Self::Week,
        Self::Month,
        Self::Quarter,
        Self::Year,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Second => "second",
            Self::Minute => "minute",
            Self::Hour => "hour",
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
            Self::Quarter => "quarter",
            Self::Year => "year",
        }
    }

    pub const fn wire_code(self) -> u64 {
        match self {
            Self::Second => 1,
            Self::Minute => 2,
            Self::Hour => 3,
            Self::Day => 4,
            Self::Week => 5,
            Self::Month => 6,
            Self::Quarter => 7,
            Self::Year => 8,
        }
    }

    pub const fn from_wire_code(code: u64) -> Option<Self> {
        match code {
            1 => Some(Self::Second),
            2 => Some(Self::Minute),
            3 => Some(Self::Hour),
            4 => Some(Self::Day),
            5 => Some(Self::Week),
            6 => Some(Self::Month),
            7 => Some(Self::Quarter),
            8 => Some(Self::Year),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativeTimePart {
    pub kind: RelativeTimePartKind,
    pub value: Box<str>,
    pub unit: RelativeTimeUnit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeTimePartKind {
    ApproximatelySign,
    Compact,
    Currency,
    Decimal,
    ExponentInteger,
    ExponentMinusSign,
    ExponentSeparator,
    Fraction,
    Group,
    Infinity,
    Integer,
    Literal,
    MinusSign,
    NaN,
    PercentSign,
    PlusSign,
    Unit,
}

impl RelativeTimePartKind {
    pub const ALL: [Self; 17] = [
        Self::ApproximatelySign,
        Self::Compact,
        Self::Currency,
        Self::Decimal,
        Self::ExponentInteger,
        Self::ExponentMinusSign,
        Self::ExponentSeparator,
        Self::Fraction,
        Self::Group,
        Self::Infinity,
        Self::Integer,
        Self::Literal,
        Self::MinusSign,
        Self::NaN,
        Self::PercentSign,
        Self::PlusSign,
        Self::Unit,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::ApproximatelySign => "approximatelySign",
            Self::Compact => "compact",
            Self::Currency => "currency",
            Self::Decimal => "decimal",
            Self::ExponentInteger => "exponentInteger",
            Self::ExponentMinusSign => "exponentMinusSign",
            Self::ExponentSeparator => "exponentSeparator",
            Self::Fraction => "fraction",
            Self::Group => "group",
            Self::Infinity => "infinity",
            Self::Integer => "integer",
            Self::Literal => "literal",
            Self::MinusSign => "minusSign",
            Self::NaN => "nan",
            Self::PercentSign => "percentSign",
            Self::PlusSign => "plusSign",
            Self::Unit => "unit",
        }
    }

    pub const fn wire_code(self) -> u64 {
        match self {
            Self::ApproximatelySign => 1,
            Self::Compact => 2,
            Self::Currency => 3,
            Self::Decimal => 4,
            Self::ExponentInteger => 5,
            Self::ExponentMinusSign => 6,
            Self::ExponentSeparator => 7,
            Self::Fraction => 8,
            Self::Group => 9,
            Self::Infinity => 10,
            Self::Integer => 11,
            Self::Literal => 12,
            Self::MinusSign => 13,
            Self::NaN => 14,
            Self::PercentSign => 15,
            Self::PlusSign => 16,
            Self::Unit => 17,
        }
    }

    pub const fn from_wire_code(code: u64) -> Option<Self> {
        match code {
            1 => Some(Self::ApproximatelySign),
            2 => Some(Self::Compact),
            3 => Some(Self::Currency),
            4 => Some(Self::Decimal),
            5 => Some(Self::ExponentInteger),
            6 => Some(Self::ExponentMinusSign),
            7 => Some(Self::ExponentSeparator),
            8 => Some(Self::Fraction),
            9 => Some(Self::Group),
            10 => Some(Self::Infinity),
            11 => Some(Self::Integer),
            12 => Some(Self::Literal),
            13 => Some(Self::MinusSign),
            14 => Some(Self::NaN),
            15 => Some(Self::PercentSign),
            16 => Some(Self::PlusSign),
            17 => Some(Self::Unit),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativeTimeParts(pub Box<[RelativeTimePart]>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativeTimeFormatRequest {
    configuration: ResolvedRelativeTimeFormat,
    unit: RelativeTimeUnit,
    input: crate::number_format::numeric::ObservedNumericInput,
}

impl RelativeTimeFormatRequest {
    pub fn new(
        configuration: ResolvedRelativeTimeFormat,
        unit: RelativeTimeUnit,
        input: crate::number_format::numeric::ObservedNumericInput,
    ) -> Self {
        Self {
            configuration,
            unit,
            input,
        }
    }

    pub const fn configuration(&self) -> &ResolvedRelativeTimeFormat {
        &self.configuration
    }

    pub const fn unit(&self) -> RelativeTimeUnit {
        self.unit
    }

    pub const fn input(&self) -> &crate::number_format::numeric::ObservedNumericInput {
        &self.input
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntlServiceError {
    UnsupportedLocale,
    InvalidLocale,
    InvalidNumber,
    InvalidOption,
    MissingRelativeTimePattern,
    InvalidRelativeTimeProfile(String),
    InvalidWire,
    Provider(String),
}

impl fmt::Display for IntlServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedLocale => formatter.write_str("no supported service locale matched"),
            Self::InvalidLocale => formatter.write_str("provider received an invalid locale"),
            Self::InvalidNumber => formatter.write_str("provider received an invalid number"),
            Self::InvalidOption => formatter.write_str("provider received an invalid option"),
            Self::MissingRelativeTimePattern => {
                formatter.write_str("the pinned relative-time profile lacks a requested pattern")
            }
            Self::InvalidRelativeTimeProfile(reason) => {
                write!(formatter, "invalid relative-time profile: {reason}")
            }
            Self::InvalidWire => formatter.write_str("invalid Intl service wire record"),
            Self::Provider(reason) => formatter.write_str(reason),
        }
    }
}

impl std::error::Error for IntlServiceError {}

/// Finds a best-available service locale after removing Unicode and private
/// extensions for matching. The public resolved locale keeps the selected
/// request spelling; the data locale is the key actually present in CLDR.
pub fn match_service_locale(
    requested: &[CanonicalLocaleId],
    available: &[&str],
    default_locale: &str,
    _matcher: ServiceLocaleMatcher,
    service: IntlServiceKind,
) -> Result<(CanonicalLocaleId, CanonicalLocaleId), IntlServiceError> {
    for candidate in requested {
        let base = base_locale(candidate.as_str());
        if let Some(data) = best_available(base, available) {
            let resolved = retain_service_extensions(candidate.as_str(), data, service)?;
            let data = CanonicalLocaleId::from_data(data.to_owned().into_boxed_str())
                .map_err(|_| IntlServiceError::InvalidLocale)?;
            return Ok((resolved, data));
        }
    }
    let data =
        best_available(default_locale, available).ok_or(IntlServiceError::UnsupportedLocale)?;
    let locale = retain_service_extensions(default_locale, data, service)?;
    let data = CanonicalLocaleId::from_data(data.to_owned().into_boxed_str())
        .map_err(|_| IntlServiceError::InvalidLocale)?;
    Ok((locale, data))
}

pub fn filter_service_locales(
    requested: &[CanonicalLocaleId],
    available: &[&str],
    _matcher: ServiceLocaleMatcher,
) -> Result<Box<[CanonicalLocaleId]>, IntlServiceError> {
    let mut selected = Vec::new();
    selected
        .try_reserve_exact(requested.len())
        .map_err(|error| IntlServiceError::Provider(error.to_string()))?;
    for candidate in requested {
        if best_available(base_locale(candidate.as_str()), available).is_some() {
            selected.push(candidate.clone());
        }
    }
    Ok(selected.into_boxed_slice())
}

fn retain_service_extensions(
    requested_locale: &str,
    matched_locale: &str,
    service: IntlServiceKind,
) -> Result<CanonicalLocaleId, IntlServiceError> {
    let subtags: Vec<&str> = requested_locale.split('-').collect();
    // Private-use data is opaque, so a singleton-looking subtag such as `u`
    // after `x` cannot introduce a Unicode locale extension.
    let private_use = subtags
        .iter()
        .position(|part| *part == "x")
        .unwrap_or(subtags.len());
    let Some(unicode) = subtags[..private_use].iter().position(|part| *part == "u") else {
        return CanonicalLocaleId::from_data(matched_locale.to_owned().into_boxed_str())
            .map_err(|_| IntlServiceError::InvalidLocale);
    };
    let mut keys = Vec::<(&str, Vec<&str>)>::new();
    let mut cursor = unicode + 1;
    while cursor < private_use && subtags[cursor].len() > 1 {
        if subtags[cursor].len() != 2 {
            cursor += 1;
            continue;
        }
        let key = subtags[cursor];
        cursor += 1;
        let start = cursor;
        while cursor < private_use && subtags[cursor].len() > 2 {
            cursor += 1;
        }
        if service.relevant_unicode_keys().contains(&key) {
            keys.push((key, subtags[start..cursor].to_vec()));
        }
    }
    keys.sort_by_key(|(key, _)| *key);
    let mut output = matched_locale.to_owned();
    if !keys.is_empty() {
        output.push_str("-u");
        for (key, values) in keys {
            output.push('-');
            output.push_str(key);
            for value in values {
                output.push('-');
                output.push_str(value);
            }
        }
    }
    CanonicalLocaleId::from_data(output.into_boxed_str())
        .map_err(|_| IntlServiceError::InvalidLocale)
}

fn base_locale(locale: &str) -> &str {
    ["-u-", "-t-", "-x-"]
        .into_iter()
        .filter_map(|extension| locale.find(extension))
        .min()
        .map_or(locale, |offset| &locale[..offset])
}

fn best_available<'a>(locale: &str, available: &'a [&str]) -> Option<&'a str> {
    let mut candidate = locale;
    loop {
        if let Ok(index) = available.binary_search(&candidate) {
            return available.get(index).copied();
        }
        let mut end = candidate.rfind('-')?;
        if end >= 2 && candidate.as_bytes().get(end - 2) == Some(&b'-') {
            end -= 2;
        }
        candidate = candidate.get(..end)?;
    }
}

#[cfg(test)]
pub fn compare_collator(
    configuration: &ResolvedCollator,
    left: &str,
    right: &str,
) -> Result<i32, IntlServiceError> {
    let left: Vec<u16> = left.encode_utf16().collect();
    let right: Vec<u16> = right.encode_utf16().collect();
    compare_collator_utf16(configuration, &left, &right)
}

pub fn compare_collator_utf16(
    configuration: &ResolvedCollator,
    left: &[u16],
    right: &[u16],
) -> Result<i32, IntlServiceError> {
    let comparison_tag = if configuration.options.usage() == CollatorUsage::Search {
        configuration.data_locale.as_str()
    } else {
        configuration.locale.as_str()
    };
    let mut locale = comparison_tag
        .parse::<Locale>()
        .map_err(|_| IntlServiceError::InvalidLocale)?;
    if configuration.options.usage() == CollatorUsage::Search {
        set_keyword(&mut locale, "co", Some("search"))?;
    } else if configuration.options.collation() != "default"
        && configuration.options.collation() != "search"
    {
        let collation = configuration.options.collation();
        let value = icu_locale::extensions::unicode::Value::try_from_str(collation)
            .map_err(|_| IntlServiceError::InvalidLocale)?;
        locale
            .extensions
            .unicode
            .keywords
            .set(icu_locale::extensions::unicode::key!("co"), value);
    }
    let mut preferences: CollatorPreferences = locale.into();
    preferences.case_first = Some(match configuration.options.case_first() {
        CollatorCaseFirst::Upper => CollationCaseFirst::Upper,
        CollatorCaseFirst::Lower => CollationCaseFirst::Lower,
        CollatorCaseFirst::False => CollationCaseFirst::False,
    });
    preferences.numeric_ordering = Some(if configuration.options.numeric() {
        CollationNumericOrdering::True
    } else {
        CollationNumericOrdering::False
    });
    let (strength, case_level) = sensitivity_strength(configuration.options.sensitivity());
    let mut options = IcuCollatorOptions::default();
    options.strength = Some(strength);
    options.case_level = Some(case_level);
    options.alternate_handling = Some(if configuration.options.ignore_punctuation() {
        AlternateHandling::Shifted
    } else {
        AlternateHandling::NonIgnorable
    });
    if configuration.options.ignore_punctuation() {
        options.max_variable = Some(MaxVariable::Punctuation);
    }
    let collator = Collator::try_new_unstable(&CollatorDataProvider, preferences, options)
        .map_err(|error| IntlServiceError::Provider(error.to_string()))?;
    Ok(match collator.as_borrowed().compare_utf16(left, right) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    })
}

pub fn plural_category(
    configuration: &ResolvedPluralRules,
    number: &str,
) -> Result<&'static str, IntlServiceError> {
    Ok(plural_category_from_input(
        configuration,
        crate::number_format::numeric::ObservedNumericInput::NumberShortestDecimal(
            number.to_owned().into_boxed_str(),
        ),
    )?
    .name())
}

pub fn plural_category_from_input(
    configuration: &ResolvedPluralRules,
    number: crate::number_format::numeric::ObservedNumericInput,
) -> Result<PluralCategory, IntlServiceError> {
    let profiles = crate::number_format::embedded_number_profiles()
        .map_err(|error| IntlServiceError::Provider(error.to_string()))?;
    plural_category_with_profiles(configuration, number, profiles)
}

#[cfg(test)]
pub fn plural_category_for_range(
    configuration: &ResolvedPluralRules,
    start: crate::number_format::numeric::ObservedNumericInput,
    end: crate::number_format::numeric::ObservedNumericInput,
) -> Result<PluralCategory, IntlServiceError> {
    let profiles = crate::number_format::embedded_number_profiles()
        .map_err(|error| IntlServiceError::Provider(error.to_string()))?;
    plural_category_range_with_profiles(configuration, start, end, profiles)
}

fn plural_category_with_profiles(
    configuration: &ResolvedPluralRules,
    number: crate::number_format::numeric::ObservedNumericInput,
    profiles: &crate::number_format::NumberProfiles,
) -> Result<PluralCategory, IntlServiceError> {
    let value = plural_numeric_value(number)?;
    Ok(plural_category_and_string_for_value(configuration, &value, profiles)?.0)
}

fn plural_numeric_value(
    input: crate::number_format::numeric::ObservedNumericInput,
) -> Result<crate::number_format::numeric::IntlMathematicalValue, IntlServiceError> {
    crate::number_format::numeric::normalize_numeric_input(
        input,
        &crate::number_format::numeric::NumericLimits::HOST_ABI,
    )
    .map_err(|_| IntlServiceError::InvalidNumber)
}

fn plural_category_and_string_for_value(
    configuration: &ResolvedPluralRules,
    value: &crate::number_format::numeric::IntlMathematicalValue,
    profiles: &crate::number_format::NumberProfiles,
) -> Result<(PluralCategory, Box<[u8]>), IntlServiceError> {
    use crate::number_format::numeric::IntlMathematicalValue;

    match value {
        IntlMathematicalValue::NaN => {
            return Ok((PluralCategory::Other, b"NaN".to_vec().into_boxed_slice()));
        }
        IntlMathematicalValue::Infinity(crate::number_format::numeric::NumberSign::Positive) => {
            return Ok((
                PluralCategory::Other,
                b"Infinity".to_vec().into_boxed_slice(),
            ));
        }
        IntlMathematicalValue::Infinity(crate::number_format::numeric::NumberSign::Negative) => {
            return Ok((
                PluralCategory::Other,
                b"-Infinity".to_vec().into_boxed_slice(),
            ));
        }
        IntlMathematicalValue::Finite(_) | IntlMathematicalValue::Zero(_) => {}
    }

    let system = crate::number_format::default_numbering_system_by_prefix(
        configuration.data_locale.as_str(),
        profiles,
    )
    .ok_or(IntlServiceError::InvalidLocale)?;
    let number_locale = crate::number_format::ResolvedNumberLocale::from_resolved(
        configuration.data_locale.clone(),
        configuration.data_locale.clone(),
        system,
        profiles,
    )
    .map_err(|error| IntlServiceError::Provider(error.to_string()))?;
    let selection = crate::number_format::plural_operands_for_selection(
        value,
        &number_locale,
        configuration.options.number_options(),
        profiles,
    )
    .map_err(|error| IntlServiceError::Provider(error.to_string()))?
    .ok_or(IntlServiceError::InvalidNumber)?;
    let locale = configuration
        .data_locale
        .as_str()
        .parse::<Locale>()
        .map_err(|_| IntlServiceError::InvalidLocale)?;
    let rules = plural_rules(locale, configuration.options.rule_type())?;
    Ok((
        PluralCategory::from_icu(rules.category_for(selection.operands)),
        selection.formatted_string,
    ))
}

fn plural_category_range_with_profiles(
    configuration: &ResolvedPluralRules,
    start: crate::number_format::numeric::ObservedNumericInput,
    end: crate::number_format::numeric::ObservedNumericInput,
    profiles: &crate::number_format::NumberProfiles,
) -> Result<PluralCategory, IntlServiceError> {
    use crate::number_format::numeric::IntlMathematicalValue;

    let start = plural_numeric_value(start)?;
    let end = plural_numeric_value(end)?;
    if matches!(&start, IntlMathematicalValue::NaN) || matches!(&end, IntlMathematicalValue::NaN) {
        return Err(IntlServiceError::InvalidOption);
    }
    let (start_category, start_formatted) =
        plural_category_and_string_for_value(configuration, &start, profiles)?;
    let (end_category, end_formatted) =
        plural_category_and_string_for_value(configuration, &end, profiles)?;
    if start_formatted == end_formatted {
        return Ok(start_category);
    }
    let locale = configuration
        .data_locale
        .as_str()
        .parse::<Locale>()
        .map_err(|_| IntlServiceError::InvalidLocale)?;
    let options = IcuPluralRulesOptions::default()
        .with_type(icu_rule_type(configuration.options.rule_type()));
    let rules =
        PluralRulesWithRanges::try_new_unstable(&PluralRulesDataProvider, locale.into(), options)
            .map_err(|error| IntlServiceError::Provider(error.to_string()))?;
    Ok(PluralCategory::from_icu(rules.resolve_range(
        start_category.to_icu(),
        end_category.to_icu(),
    )))
}

fn input_number_value(
    input: &crate::number_format::numeric::ObservedNumericInput,
) -> Result<f64, IntlServiceError> {
    use crate::number_format::numeric::ObservedNumericInput;
    match input {
        ObservedNumericInput::NumberShortestDecimal(number) => number
            .parse::<f64>()
            .map_err(|_| IntlServiceError::InvalidNumber),
        ObservedNumericInput::NegativeZero => Ok(-0.0),
        ObservedNumericInput::StringNumericLiteral(_) | ObservedNumericInput::BigIntDecimal(_) => {
            Err(IntlServiceError::InvalidNumber)
        }
    }
}

fn plural_rules(
    locale: Locale,
    rule_type: PluralRulesType,
) -> Result<PluralRules, IntlServiceError> {
    PluralRules::try_new_unstable(
        &PluralRulesDataProvider,
        locale.into(),
        IcuPluralRulesOptions::default().with_type(icu_rule_type(rule_type)),
    )
    .map_err(|error| IntlServiceError::Provider(error.to_string()))
}

const fn icu_rule_type(rule_type: PluralRulesType) -> PluralRuleType {
    match rule_type {
        PluralRulesType::Cardinal => PluralRuleType::Cardinal,
        PluralRulesType::Ordinal => PluralRuleType::Ordinal,
    }
}

#[derive(Debug, Deserialize)]
struct RelativeTimeData {
    locales: BTreeMap<String, BTreeMap<String, RelativeTimeEntry>>,
}

#[derive(Debug, Deserialize)]
struct RelativeTimeEntry {
    patterns: BTreeMap<String, BTreeMap<String, String>>,
    relative: BTreeMap<String, String>,
}

static RELATIVE_TIME_DATA: OnceLock<Result<RelativeTimeData, String>> = OnceLock::new();

fn relative_time_data() -> Result<&'static RelativeTimeData, IntlServiceError> {
    RELATIVE_TIME_DATA
        .get_or_init(|| {
            serde_json::from_str(include_str!("provider/relative_time/patterns.json"))
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|reason| IntlServiceError::InvalidRelativeTimeProfile(reason.clone()))
}

pub fn relative_time_available_locales() -> Result<Box<[&'static str]>, IntlServiceError> {
    Ok(relative_time_data()?
        .locales
        .keys()
        .filter(|locale| locale.as_str() != "root")
        .map(String::as_str)
        .collect())
}

/// Formats one finite relative value from the resolved CLDR 47 field patterns.
/// The numeric portion uses the NumberFormat decimal defaults and retains its
/// localized integer, grouping, decimal, and fraction parts.
pub fn format_relative_time_input(
    configuration: &ResolvedRelativeTimeFormat,
    unit: RelativeTimeUnit,
    input: crate::number_format::numeric::ObservedNumericInput,
) -> Result<RelativeTimeParts, IntlServiceError> {
    let data = relative_time_data()?;
    let locale = data
        .locales
        .get(configuration.data_locale.as_str())
        .ok_or(IntlServiceError::MissingRelativeTimePattern)?;
    let style = match configuration.options.style() {
        RelativeTimeStyle::Long => "long",
        RelativeTimeStyle::Short => "short",
        RelativeTimeStyle::Narrow => "narrow",
    };
    let field = locale
        .get(&format!("{}/{style}", unit.name()))
        .or_else(|| locale.get(&format!("{}/long", unit.name())))
        .ok_or(IntlServiceError::MissingRelativeTimePattern)?;
    let numeric = input_number_value(&input)?;
    if !numeric.is_finite() {
        return Err(IntlServiceError::InvalidNumber);
    }
    let number = match &input {
        crate::number_format::numeric::ObservedNumericInput::NumberShortestDecimal(number) => {
            number.as_ref()
        }
        crate::number_format::numeric::ObservedNumericInput::NegativeZero => "-0",
        crate::number_format::numeric::ObservedNumericInput::StringNumericLiteral(_)
        | crate::number_format::numeric::ObservedNumericInput::BigIntDecimal(_) => {
            return Err(IntlServiceError::InvalidNumber);
        }
    };
    if configuration.options.numeric() == RelativeTimeNumeric::Auto {
        if let Some(relative) = field.relative.get(&relative_offset_key(numeric)) {
            return Ok(RelativeTimeParts(
                vec![RelativeTimePart {
                    kind: RelativeTimePartKind::Literal,
                    value: relative.as_str().into(),
                    unit,
                }]
                .into_boxed_slice(),
            ));
        }
    }
    let direction = if numeric.is_sign_negative() {
        "past"
    } else {
        "future"
    };
    let magnitude = number.strip_prefix('-').unwrap_or(number);
    let plural = ResolvedPluralRules::from_provider(
        configuration.locale.clone(),
        configuration.data_locale.clone(),
        PluralRulesOptions::new(
            PluralRulesType::Cardinal,
            1,
            PluralRulesPrecision::fraction(0, 3)?,
        )?,
    )?;
    let category = plural_category(&plural, magnitude)?;
    let pattern = field
        .patterns
        .get(direction)
        .and_then(|counts| counts.get(category).or_else(|| counts.get("other")))
        .ok_or(IntlServiceError::MissingRelativeTimePattern)?;
    let Some(position) = pattern.find("{0}") else {
        return Ok(RelativeTimeParts(
            vec![RelativeTimePart {
                kind: RelativeTimePartKind::Literal,
                value: pattern.as_str().into(),
                unit,
            }]
            .into_boxed_slice(),
        ));
    };
    let profiles = crate::number_format::embedded_number_profiles()
        .map_err(|error| IntlServiceError::Provider(error.to_string()))?;
    let number_configuration = crate::number_format::NumberFormatConfiguration {
        locale: configuration.number_locale.clone(),
        options: crate::number_format::options::NumberFormatOptions {
            style: crate::number_format::options::NumberStyle::Decimal,
            notation: crate::number_format::options::Notation::Standard,
            minimum_integer_digits: crate::number_format::options::IntegerDigitCount::new(1)
                .map_err(|_| IntlServiceError::InvalidOption)?,
            precision: crate::number_format::options::Precision::Fraction(
                crate::number_format::options::FractionPrecision::Range(
                    crate::number_format::options::FractionDigitRange::new(
                        crate::number_format::options::FractionDigitCount::new(0)
                            .map_err(|_| IntlServiceError::InvalidOption)?,
                        crate::number_format::options::FractionDigitCount::new(3)
                            .map_err(|_| IntlServiceError::InvalidOption)?,
                    )
                    .map_err(|_| IntlServiceError::InvalidOption)?,
                ),
            ),
            rounding_mode: crate::number_format::options::RoundingMode::HalfExpand,
            trailing_zero_display: crate::number_format::options::TrailingZeroDisplay::Auto,
            grouping: crate::number_format::options::Grouping::Auto,
            sign_display: crate::number_format::options::SignDisplay::Auto,
        },
    };
    let number_partition = crate::number_operation::format_number_parts_operation(
        crate::number_operation::NumberFormatRequest {
            configuration: number_configuration,
            input: crate::number_format::numeric::ObservedNumericInput::NumberShortestDecimal(
                magnitude.to_owned().into_boxed_str(),
            ),
        },
        profiles,
        &crate::number_format::PartitionLimits::HOST_ABI,
    )
    .map_err(|error| IntlServiceError::Provider(error.to_string()))?;
    let mut parts = Vec::with_capacity(3);
    if position != 0 {
        parts.push(RelativeTimePart {
            kind: RelativeTimePartKind::Literal,
            value: pattern[..position].into(),
            unit,
        });
    }
    for part in number_partition.parts() {
        let kind = match part.kind() {
            crate::number_format::NumberPartKind::Integer => RelativeTimePartKind::Integer,
            crate::number_format::NumberPartKind::Group => RelativeTimePartKind::Group,
            crate::number_format::NumberPartKind::Decimal => RelativeTimePartKind::Decimal,
            crate::number_format::NumberPartKind::Fraction => RelativeTimePartKind::Fraction,
            crate::number_format::NumberPartKind::Literal => RelativeTimePartKind::Literal,
            crate::number_format::NumberPartKind::PlusSign => RelativeTimePartKind::PlusSign,
            crate::number_format::NumberPartKind::MinusSign => RelativeTimePartKind::MinusSign,
            crate::number_format::NumberPartKind::PercentSign => RelativeTimePartKind::PercentSign,
            crate::number_format::NumberPartKind::Currency => RelativeTimePartKind::Currency,
            crate::number_format::NumberPartKind::Unit => RelativeTimePartKind::Unit,
            crate::number_format::NumberPartKind::Compact => RelativeTimePartKind::Compact,
            crate::number_format::NumberPartKind::ExponentSeparator => {
                RelativeTimePartKind::ExponentSeparator
            }
            crate::number_format::NumberPartKind::ExponentMinusSign => {
                RelativeTimePartKind::ExponentMinusSign
            }
            crate::number_format::NumberPartKind::ExponentInteger => {
                RelativeTimePartKind::ExponentInteger
            }
            crate::number_format::NumberPartKind::Infinity => RelativeTimePartKind::Infinity,
            crate::number_format::NumberPartKind::NaN => RelativeTimePartKind::NaN,
        };
        parts.push(RelativeTimePart {
            kind,
            value: part.text().into(),
            unit,
        });
    }
    let suffix = &pattern[position + 3..];
    if !suffix.is_empty() {
        parts.push(RelativeTimePart {
            kind: RelativeTimePartKind::Literal,
            value: suffix.into(),
            unit,
        });
    }
    Ok(RelativeTimeParts(parts.into_boxed_slice()))
}

fn relative_offset_key(number: f64) -> String {
    if number == 0.0 {
        "0".to_owned()
    } else if number == -1.0 {
        "-1".to_owned()
    } else if number == 1.0 {
        "1".to_owned()
    } else if number == -2.0 {
        "-2".to_owned()
    } else if number == 2.0 {
        "2".to_owned()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod collator_tests {
    use super::*;

    fn locale(value: &str) -> CanonicalLocaleId {
        CanonicalLocaleId::from_data(value.to_owned().into_boxed_str()).unwrap()
    }

    fn collations(values: &[&str]) -> Box<[Box<str>]> {
        values
            .iter()
            .map(|value| (*value).into())
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    fn resolve(
        request_locale: &str,
        options: CollatorOptions,
        collations: &[Box<str>],
    ) -> ResolvedCollator {
        resolve_collator(
            locale(request_locale),
            locale(base_locale(request_locale)),
            options,
            collations,
        )
        .unwrap()
    }

    #[test]
    fn collator_resolves_supported_base_and_default_sort_options() {
        let (public, data) = match_service_locale(
            &[locale("en-ZZ")],
            &["en", "fr"],
            "en",
            ServiceLocaleMatcher::Lookup,
            IntlServiceKind::Collator,
        )
        .unwrap();
        assert_eq!(public.as_str(), "en");
        assert_eq!(data.as_str(), "en");

        let resolved = resolve("en", CollatorOptions::default(), &collations(&[]));
        assert_eq!(resolved.options().usage(), CollatorUsage::Sort);
        assert_eq!(resolved.options().collation(), "default");
        assert_eq!(
            resolved.options().sensitivity(),
            CollatorSensitivity::Variant
        );
        assert_eq!(resolved.options().case_first(), CollatorCaseFirst::False);
        assert!(!resolved.options().numeric());
        assert!(!resolved.options().ignore_punctuation());
    }

    #[test]
    fn collator_retains_only_extension_values_that_survive_option_resolution() {
        let supported = collations(&["emoji"]);
        let retained = resolve("en-u-kn", CollatorOptions::default(), &supported);
        assert!(retained.options().numeric());
        assert_eq!(retained.locale().as_str(), "en-u-kn");

        let overridden = resolve(
            "en-u-kn",
            CollatorOptions::new(CollatorUsage::Sort, None, Some(false), None, None, None),
            &supported,
        );
        assert!(!overridden.options().numeric());
        assert_eq!(overridden.locale().as_str(), "en");

        let unsupported = resolve("en-u-co-phonebk", CollatorOptions::default(), &supported);
        assert_eq!(unsupported.options().collation(), "default");
        assert_eq!(unsupported.locale().as_str(), "en");
    }

    #[test]
    fn collator_private_use_does_not_introduce_unicode_extensions() {
        let (public, data) = match_service_locale(
            &[locale("de-x-u-co-phonebk")],
            &["de"],
            "en",
            ServiceLocaleMatcher::Lookup,
            IntlServiceKind::Collator,
        )
        .unwrap();
        assert_eq!(public.as_str(), "de");
        assert_eq!(data.as_str(), "de");

        let resolved = resolve_collator(
            public,
            data,
            CollatorOptions::default(),
            &collations(&["phonebk"]),
        )
        .unwrap();
        assert_eq!(resolved.options().collation(), "default");
        assert_eq!(resolved.locale().as_str(), "de");
    }

    #[test]
    fn unsupported_collation_option_does_not_override_supported_locale_extension() {
        let requested = CollatorOptions::new(
            CollatorUsage::Sort,
            Some(CollationOption::parse("pinyin").unwrap()),
            None,
            None,
            None,
            None,
        );
        let retained = resolve_collator(
            locale("de-u-co-phonebk"),
            locale("de"),
            requested.clone(),
            &collations(&["phonebk"]),
        )
        .unwrap();
        assert_eq!(retained.options().collation(), "phonebk");
        assert_eq!(retained.locale().as_str(), "de-u-co-phonebk");

        let fallback = resolve_collator(
            locale("en-u-co-phonebk"),
            locale("en"),
            requested,
            &collations(&[]),
        )
        .unwrap();
        assert_eq!(fallback.options().collation(), "default");
        assert_eq!(fallback.locale().as_str(), "en");
    }

    #[test]
    fn collator_search_overrides_requested_collation_and_uses_primary_strength() {
        let search = resolve(
            "en-u-co-emoji",
            CollatorOptions::new(
                CollatorUsage::Search,
                Some(CollationOption::parse("emoji").unwrap()),
                None,
                None,
                None,
                None,
            ),
            &collations(&["emoji"]),
        );
        assert_eq!(search.options().collation(), "default");
        assert_eq!(search.locale().as_str(), "en");
        assert_eq!(search.options().sensitivity(), CollatorSensitivity::Base);
        assert_eq!(compare_collator(&search, "A", "a").unwrap(), 0);
    }

    #[test]
    fn collator_search_uses_german_search_tailoring_with_regional_fallback() {
        let base = resolve(
            "de",
            CollatorOptions::new(CollatorUsage::Search, None, None, None, None, None),
            &collations(&["phonebk"]),
        );
        assert_eq!(base.options().collation(), "default");
        assert_eq!(base.options().sensitivity(), CollatorSensitivity::Base);
        assert_eq!(compare_collator(&base, "AE", "Ä").unwrap(), 0);

        let regional = resolve(
            "de-AT",
            CollatorOptions::new(CollatorUsage::Search, None, None, None, None, None),
            &collations(&[]),
        );
        assert_eq!(regional.options().collation(), "default");
        assert_eq!(compare_collator(&regional, "AE", "Ä").unwrap(), 0);

        let accents = resolve(
            "de",
            CollatorOptions::new(
                CollatorUsage::Search,
                None,
                None,
                None,
                Some(CollatorSensitivity::Accent),
                None,
            ),
            &collations(&["phonebk"]),
        );
        assert_eq!(compare_collator(&accents, "AE", "Ä").unwrap(), -1);
    }

    #[test]
    fn collator_search_loads_root_arabic_and_korean_search_rules() {
        let root = resolve(
            "und",
            CollatorOptions::new(
                CollatorUsage::Search,
                None,
                None,
                None,
                Some(CollatorSensitivity::Base),
                None,
            ),
            &collations(&[]),
        );
        assert_eq!(compare_collator(&root, "=", "≠").unwrap(), -1);

        let arabic = resolve(
            "ar",
            CollatorOptions::new(
                CollatorUsage::Search,
                None,
                None,
                None,
                Some(CollatorSensitivity::Variant),
                None,
            ),
            &collations(&[]),
        );
        assert_eq!(compare_collator(&arabic, "ا", "ﺍ").unwrap(), -1);

        let korean = resolve(
            "ko",
            CollatorOptions::new(CollatorUsage::Search, None, None, None, None, None),
            &collations(&[]),
        );
        assert_eq!(compare_collator(&korean, "ᄀᄂ", "ᇺ").unwrap(), 0);
    }

    #[test]
    fn collator_search_matches_pinned_cldr_korean_jamo_rules() {
        // CLDR 47 root.xml search rules equate modern leading/trailing Jamo,
        // doubled leading Jamo, and modern complex final Jamo. They are search
        // equivalences, not ordinary Korean sort equivalences.
        let root_search = resolve(
            "en",
            CollatorOptions::new(CollatorUsage::Search, None, None, None, None, None),
            &collations(&[]),
        );
        for (leading, trailing) in [
            ("\u{1100}", "\u{11a8}"),         // KIYEOK L = KIYEOK T
            ("\u{1100}\u{1100}", "\u{1101}"), // KIYEOK L + KIYEOK L = SSANGKIYEOK L
            ("\u{1100}\u{1100}", "\u{11a9}"), // KIYEOK L + KIYEOK L = SSANGKIYEOK T
            ("\u{1100}\u{1109}", "\u{11aa}"), // KIYEOK L + SIOS L = KIYEOK-SIOS T
            ("\u{1169}\u{1161}", "\u{116a}"), // O + A = WA
        ] {
            assert_eq!(
                compare_collator(&root_search, leading, trailing).unwrap(),
                0,
                "{leading:?} versus {trailing:?}"
            );
        }

        let root_sort = resolve("en", CollatorOptions::default(), &collations(&[]));
        assert_ne!(
            compare_collator(&root_sort, "\u{1100}", "\u{11a8}").unwrap(),
            0
        );
        assert_ne!(
            compare_collator(&root_sort, "\u{1100}\u{1100}", "\u{1101}").unwrap(),
            0
        );

        // CLDR 47 ko.xml adds archaic consonant and vowel mappings after
        // importing root search. The same archaic mappings are not specified
        // for every locale, so exercise them with the Korean search profile.
        let korean_search = resolve(
            "ko",
            CollatorOptions::new(CollatorUsage::Search, None, None, None, None, None),
            &collations(&[]),
        );
        for (expanded, archaic) in [
            ("\u{1100}\u{1102}", "\u{11fa}"), // KIYEOK + NIEUN = KIYEOK-NIEUN T
            ("\u{1102}\u{1100}", "\u{1113}"), // NIEUN + KIYEOK = NIEUN-KIYEOK L
            ("\u{1102}\u{1100}", "\u{11c5}"), // NIEUN + KIYEOK = NIEUN-KIYEOK T
            ("\u{1161}\u{1169}", "\u{1176}"), // A + O = archaic A-O
        ] {
            assert_eq!(
                compare_collator(&korean_search, expanded, archaic).unwrap(),
                0,
                "{expanded:?} versus {archaic:?}"
            );
        }

        // Hangul syllables and their canonical algorithmic decomposition are
        // equivalent in both profiles; this is separate from search tailoring.
        let syllable = "\u{ac01}"; // 각
        let decomposed = "\u{1100}\u{1161}\u{11a8}"; // ᄀ + ᅡ + ᆨ
        assert_eq!(
            compare_collator(&root_search, syllable, decomposed).unwrap(),
            0
        );
        assert_eq!(
            compare_collator(&root_sort, syllable, decomposed).unwrap(),
            0
        );
    }

    #[test]
    fn collator_searchjl_preserves_hangul_prefixes_and_canonical_equivalence() {
        // CLDR 47 ko.xml searchjl makes a repeated leading consonant primary
        // ignorable in that prefix context. Precomposed Hangul must take the
        // same contextual path as its explicit L/V/T decomposition.
        for sensitivity in [
            CollatorSensitivity::Base,
            CollatorSensitivity::Accent,
            CollatorSensitivity::Variant,
        ] {
            let configuration = resolve(
                "ko-u-co-searchjl",
                CollatorOptions::new(
                    CollatorUsage::Sort,
                    None,
                    None,
                    None,
                    Some(sensitivity),
                    None,
                ),
                &collations(&["searchjl"]),
            );
            assert_eq!(configuration.options().collation(), "searchjl");
            for (composed, decomposed) in [
                ("\u{1100}각", "\u{1100}\u{1100}\u{1161}\u{11a8}"),
                ("\u{1100}가", "\u{1100}\u{1100}\u{1161}"),
                ("각\u{301}ᄀᄀ", "각\u{301}ᄀᄀ"),
                ("가\u{301}ᄀᄀ", "가\u{301}ᄀᄀ"),
            ] {
                assert_eq!(
                    compare_collator(&configuration, composed, decomposed).unwrap(),
                    0,
                    "{sensitivity:?}: {composed:?} versus {decomposed:?}"
                );
                assert_eq!(
                    compare_collator(&configuration, decomposed, composed).unwrap(),
                    0
                );
            }
            let common = "가".repeat(10_000);
            assert_eq!(
                compare_collator(
                    &configuration,
                    &format!("{common}ᄀ가"),
                    &format!("{common}ᄀ가"),
                )
                .unwrap(),
                0,
                "{sensitivity:?}: canonical equivalence after a long shared prefix"
            );
            if sensitivity == CollatorSensitivity::Base {
                assert_eq!(compare_collator(&configuration, "ᄀᄀ", "ᄀ").unwrap(), 0);
                assert_eq!(compare_collator(&configuration, "ᄀ", "ᄀᄀ").unwrap(), 0);
            }
        }
    }

    #[test]
    fn collator_compare_uses_numeric_case_utf16_and_embedded_nul_data() {
        let numeric = resolve(
            "en",
            CollatorOptions::new(CollatorUsage::Sort, None, Some(true), None, None, None),
            &collations(&[]),
        );
        assert_eq!(compare_collator(&numeric, "a2", "a10").unwrap(), -1);

        let case_first = resolve(
            "en",
            CollatorOptions::new(
                CollatorUsage::Sort,
                None,
                None,
                Some(CollatorCaseFirst::Upper),
                None,
                None,
            ),
            &collations(&[]),
        );
        assert_eq!(compare_collator(&case_first, "A", "a").unwrap(), -1);

        let default = resolve("en", CollatorOptions::default(), &collations(&[]));
        assert_eq!(
            compare_collator_utf16(
                &default,
                &[b'a' as u16, 0, b'c' as u16],
                &[b'a' as u16, 0, b'b' as u16]
            )
            .unwrap(),
            1
        );
        assert_eq!(
            compare_collator_utf16(&default, &[0xd800], &[0xd800]).unwrap(),
            0
        );
    }
}

#[cfg(test)]
mod plural_rules_tests {
    use super::*;
    use crate::number_format::numeric::ObservedNumericInput;
    use crate::number_format::options::{
        CompactDisplay, FractionDigitCount, FractionDigitRange, FractionPrecision, Grouping,
        IntegerDigitCount, Notation, NumberFormatOptions, NumberStyle, Precision, RoundingMode,
        SignDisplay, SignificantDigitCount, SignificantDigitRange, TrailingZeroDisplay,
    };

    fn locale(value: &str) -> CanonicalLocaleId {
        CanonicalLocaleId::from_data(value.to_owned().into_boxed_str()).unwrap()
    }

    fn options(
        rule_type: PluralRulesType,
        precision: Precision,
        mode: RoundingMode,
    ) -> PluralRulesOptions {
        PluralRulesOptions::from_number_format_options(
            rule_type,
            NumberFormatOptions {
                style: NumberStyle::Decimal,
                notation: Notation::Standard,
                minimum_integer_digits: IntegerDigitCount::new(1).unwrap(),
                precision,
                rounding_mode: mode,
                trailing_zero_display: TrailingZeroDisplay::Auto,
                grouping: Grouping::Never,
                sign_display: SignDisplay::Auto,
            },
        )
        .unwrap()
    }

    fn fractional_precision(minimum: u8, maximum: u8) -> Precision {
        Precision::Fraction(FractionPrecision::Range(
            FractionDigitRange::new(
                FractionDigitCount::new(minimum).unwrap(),
                FractionDigitCount::new(maximum).unwrap(),
            )
            .unwrap(),
        ))
    }

    fn resolved(
        rule_type: PluralRulesType,
        precision: Precision,
        mode: RoundingMode,
    ) -> ResolvedPluralRules {
        ResolvedPluralRules::from_provider(
            locale("en"),
            locale("en"),
            options(rule_type, precision, mode),
        )
        .unwrap()
    }

    fn number(value: &str) -> ObservedNumericInput {
        ObservedNumericInput::NumberShortestDecimal(value.to_owned().into_boxed_str())
    }

    #[test]
    fn plural_rules_round_operands_with_the_number_format_digit_domain() {
        let default = resolved(
            PluralRulesType::Cardinal,
            fractional_precision(0, 3),
            RoundingMode::HalfExpand,
        );
        assert_eq!(plural_category(&default, "1.01").unwrap(), "other");

        let integer = resolved(
            PluralRulesType::Cardinal,
            fractional_precision(0, 0),
            RoundingMode::HalfExpand,
        );
        assert_eq!(plural_category(&integer, "1.01").unwrap(), "one");

        let ceil = resolved(
            PluralRulesType::Cardinal,
            fractional_precision(0, 0),
            RoundingMode::Ceil,
        );
        assert_eq!(plural_category(&ceil, "1.01").unwrap(), "other");
        assert_eq!(
            plural_category_from_input(&integer, ObservedNumericInput::NegativeZero).unwrap(),
            PluralCategory::Other
        );
        assert_eq!(
            plural_category_from_input(&integer, number("Infinity")).unwrap(),
            PluralCategory::Other
        );
        assert_eq!(
            integer.categories(),
            &[PluralCategory::One, PluralCategory::Other]
        );
    }

    #[test]
    fn resolved_cardinal_categories_follow_pinned_cldr_rules_in_spec_order() {
        use PluralCategory::{Few, Many, One, Other, Two, Zero};

        // The same seven locales and category lists as pinned Test262's
        // plural-categories-order.js, including Manx (`gv`).
        for (tag, expected) in [
            ("ar", &[Zero, One, Two, Few, Many, Other][..]),
            ("en", &[One, Other][..]),
            ("fa", &[One, Other][..]),
            ("fr", &[One, Many, Other][..]),
            ("gv", &[One, Two, Few, Many, Other][..]),
            ("ko", &[Other][..]),
            ("sl", &[One, Two, Few, Other][..]),
        ] {
            let tag = locale(tag);
            let resolved = ResolvedPluralRules::from_provider(
                tag.clone(),
                tag,
                options(
                    PluralRulesType::Cardinal,
                    fractional_precision(0, 3),
                    RoundingMode::HalfExpand,
                ),
            )
            .unwrap();
            assert_eq!(
                resolved.categories(),
                expected,
                "{}",
                resolved.locale().as_str()
            );
        }
    }

    #[test]
    fn manx_selection_uses_full_pinned_cldr_cardinal_rules() {
        let manx = locale("gv");
        let resolved = ResolvedPluralRules::from_provider(
            manx.clone(),
            manx,
            options(
                PluralRulesType::Cardinal,
                fractional_precision(0, 3),
                RoundingMode::HalfExpand,
            ),
        )
        .unwrap();
        for (value, expected) in [
            ("1", PluralCategory::One),
            ("2", PluralCategory::Two),
            ("20", PluralCategory::Few),
            ("1.5", PluralCategory::Many),
            ("3", PluralCategory::Other),
        ] {
            assert_eq!(
                plural_category_from_input(&resolved, number(value)).unwrap(),
                expected,
                "gv {value}"
            );
        }
        assert_eq!(
            plural_category_for_range(&resolved, number("1"), number("1")).unwrap(),
            PluralCategory::One
        );
        assert_eq!(
            plural_category_for_range(&resolved, number("1"), number("2")).unwrap(),
            // CLDR has no Manx range pair here; ICU4X returns the end category.
            PluralCategory::Two
        );
    }

    #[test]
    fn ordinal_rules_cover_pinned_cldr_locales_absent_from_icu_compiled_subset() {
        use PluralCategory::{Many, One, Other};

        for (tag, categories, samples) in [
            ("bal", &[One, Other][..], &[("1", One), ("2", Other)][..]),
            (
                "kw",
                &[One, Many, Other][..],
                &[("1", One), ("24", One), ("105", Many), ("6", Other)][..],
            ),
            ("lld", &[Many, Other][..], &[("8", Many), ("9", Other)][..]),
            (
                "scn",
                &[Many, Other][..],
                &[("800", Many), ("9", Other)][..],
            ),
        ] {
            let tag = locale(tag);
            let resolved = ResolvedPluralRules::from_provider(
                tag.clone(),
                tag,
                options(
                    PluralRulesType::Ordinal,
                    fractional_precision(0, 3),
                    RoundingMode::HalfExpand,
                ),
            )
            .unwrap();
            assert_eq!(
                resolved.categories(),
                categories,
                "{}",
                resolved.locale().as_str()
            );
            for &(value, expected) in samples {
                assert_eq!(
                    plural_category_from_input(&resolved, number(value)).unwrap(),
                    expected,
                    "{} {value}",
                    resolved.locale().as_str()
                );
            }
        }
    }

    #[test]
    fn plural_rules_select_range_uses_pinned_range_categories_and_checks_order() {
        let configuration = resolved(
            PluralRulesType::Cardinal,
            fractional_precision(0, 3),
            RoundingMode::HalfExpand,
        );
        assert_eq!(
            plural_category_for_range(&configuration, number("1"), number("1")).unwrap(),
            PluralCategory::One
        );
        assert_eq!(
            plural_category_for_range(&configuration, number("1"), number("2")).unwrap(),
            PluralCategory::Other
        );
        assert!(plural_category_for_range(&configuration, number("2"), number("1")).is_ok());
        assert_eq!(
            plural_category_for_range(&configuration, number("NaN"), number("1")),
            Err(IntlServiceError::InvalidOption)
        );
    }

    #[test]
    fn plural_rules_compact_notation_uses_numberformat_rounding_and_compact_exponent() {
        let profiles = crate::number_format::embedded_number_profiles().unwrap();
        let french = locale("fr");
        let standard = ResolvedPluralRules::from_provider(
            french.clone(),
            french.clone(),
            options(
                PluralRulesType::Cardinal,
                fractional_precision(0, 3),
                RoundingMode::HalfExpand,
            ),
        )
        .unwrap();
        let compact = ResolvedPluralRules::from_provider(
            french.clone(),
            french,
            PluralRulesOptions::from_number_format_options(
                PluralRulesType::Cardinal,
                NumberFormatOptions {
                    style: NumberStyle::Decimal,
                    notation: Notation::Compact(CompactDisplay::Short),
                    minimum_integer_digits: IntegerDigitCount::new(1).unwrap(),
                    precision: Precision::Significant(
                        SignificantDigitRange::new(
                            SignificantDigitCount::new(1).unwrap(),
                            SignificantDigitCount::new(2).unwrap(),
                        )
                        .unwrap(),
                    ),
                    rounding_mode: RoundingMode::HalfExpand,
                    trailing_zero_display: TrailingZeroDisplay::Auto,
                    grouping: Grouping::Auto,
                    sign_display: SignDisplay::Auto,
                },
            )
            .unwrap(),
        )
        .unwrap();
        let value = number("1500000");

        assert_eq!(
            plural_category_with_profiles(&standard, value.clone(), &profiles).unwrap(),
            PluralCategory::Other
        );
        assert_eq!(
            plural_category_with_profiles(&compact, value, &profiles).unwrap(),
            PluralCategory::Many
        );
    }

    #[test]
    fn scientific_and_engineering_use_rounded_source_operands() {
        let profiles = crate::number_format::embedded_number_profiles().unwrap();
        let en = locale("en");
        for notation in [Notation::Scientific, Notation::Engineering] {
            let mut rules_options = options(
                PluralRulesType::Cardinal,
                fractional_precision(0, 3),
                RoundingMode::HalfExpand,
            );
            rules_options.number.notation = notation;
            let configuration =
                ResolvedPluralRules::from_provider(en.clone(), en.clone(), rules_options).unwrap();

            // Both scientific mantissas are 1; plural selection still uses
            // the rounded source number 1,000,000, whose English category is
            // `other`.
            assert_eq!(
                plural_category_with_profiles(&configuration, number("1000000"), &profiles)
                    .unwrap(),
                PluralCategory::Other
            );
        }
    }
}
