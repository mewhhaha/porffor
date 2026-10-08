use super::*;
use crate::{LocaleId, LocaleTransformError, LocaleTransformRequest};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LanguageCode {
    pub canonical: Box<str>,
    pub language: Box<str>,
    pub script: Option<Box<str>>,
    pub region: Option<Box<str>>,
    pub variants: Box<[Box<str>]>,
}
impl LanguageCode {
    /// This is the Unicode language-id grammar, excluding extensions, root,
    /// private use, bare scripts, and repeated variants. The general LocaleId
    /// transport grammar alone is deliberately insufficient here.
    pub(super) fn structurally_checked(text: &str) -> Result<Self, DisplayNamesError> {
        if !text.is_ascii() {
            return Err(DisplayNamesError::InvalidCode);
        }
        let mut fields = text.split('-').peekable();
        let language = fields.next().ok_or(DisplayNamesError::InvalidCode)?;
        if !matches!(language.len(), 2 | 3 | 5..=8)
            || !language.bytes().all(|b| b.is_ascii_alphabetic())
        {
            return Err(DisplayNamesError::InvalidCode);
        }
        let script = fields
            .peek()
            .copied()
            .filter(|s| s.len() == 4 && s.bytes().all(|b| b.is_ascii_alphabetic()));
        if script.is_some() {
            fields.next();
        }
        let region = fields.peek().copied().filter(|s| valid_region(s));
        if region.is_some() {
            fields.next();
        }
        let mut variants = Vec::new();
        let mut seen = BTreeSet::new();
        for field in fields {
            if !valid_variant(field) || !seen.insert(field.to_ascii_lowercase()) {
                return Err(DisplayNamesError::InvalidCode);
            }
            variants.push(field.to_ascii_lowercase().into_boxed_str());
        }
        Ok(Self {
            canonical: text.into(),
            language: language.to_ascii_lowercase().into(),
            script: script.map(title_case),
            region: region.map(|r| r.to_ascii_uppercase().into()),
            variants: variants.into_boxed_slice(),
        })
    }
    pub(super) fn subtag_count(&self) -> usize {
        1 + usize::from(self.script.is_some())
            + usize::from(self.region.is_some())
            + self.variants.len()
    }
    pub(super) fn is_subset_of(&self, other: &Self) -> bool {
        self.language == other.language
            && self
                .script
                .as_ref()
                .is_none_or(|s| other.script.as_ref() == Some(s))
            && self
                .region
                .as_ref()
                .is_none_or(|r| other.region.as_ref() == Some(r))
            && self.variants.iter().all(|v| other.variants.contains(v))
    }
}
pub(super) fn valid_variant(value: &str) -> bool {
    ((5..=8).contains(&value.len()) || (value.len() == 4 && value.as_bytes()[0].is_ascii_digit()))
        && value.bytes().all(|b| b.is_ascii_alphanumeric())
}
fn valid_region(value: &str) -> bool {
    (value.len() == 2 && value.bytes().all(|b| b.is_ascii_alphabetic()))
        || (value.len() == 3 && value.bytes().all(|b| b.is_ascii_digit()))
}
fn title_case(value: &str) -> Box<str> {
    let mut result = value.to_ascii_lowercase();
    result[..1].make_ascii_uppercase();
    result.into_boxed_str()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Code {
    Language(LanguageCode),
    Region(Box<str>),
    Script(Box<str>),
    Currency(Box<str>),
    Calendar(Box<str>),
    DateTimeField(DisplayNamesDateTimeField),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayNameCode(Code);
impl DisplayNameCode {
    pub fn parse<P: IntlOperationProvider<CanonicalizeLocale>>(
        kind: DisplayNamesType,
        units: &[u16],
        canonicalizer: &P,
    ) -> Result<Self, DisplayNamesError> {
        Self::parse_with(kind, units, &|locale| {
            canonicalizer
                .execute(LocaleTransformRequest::new(locale))
                .map(|result| result.into_locale())
        })
    }
    pub(super) fn parse_with(
        kind: DisplayNamesType,
        units: &[u16],
        canonicalize: &impl Fn(LocaleId) -> Result<CanonicalLocaleId, LocaleTransformError>,
    ) -> Result<Self, DisplayNamesError> {
        if !units.iter().all(|&unit| unit <= 0x7f) {
            return Err(DisplayNamesError::InvalidCode);
        }
        let text: String = units.iter().map(|&u| char::from(u as u8)).collect();
        Self::from_text_with(kind, &text, canonicalize)
    }
    #[cfg(test)]
    pub(super) fn from_text<P: IntlOperationProvider<CanonicalizeLocale>>(
        kind: DisplayNamesType,
        text: &str,
        canonicalizer: &P,
    ) -> Result<Self, DisplayNamesError> {
        Self::from_text_with(kind, text, &|locale| {
            canonicalizer
                .execute(LocaleTransformRequest::new(locale))
                .map(|result| result.into_locale())
        })
    }
    pub(super) fn from_text_with(
        kind: DisplayNamesType,
        text: &str,
        canonicalize: &impl Fn(LocaleId) -> Result<CanonicalLocaleId, LocaleTransformError>,
    ) -> Result<Self, DisplayNamesError> {
        if !text.is_ascii() {
            return Err(DisplayNamesError::InvalidCode);
        }
        let code = match kind {
            DisplayNamesType::Language => {
                LanguageCode::structurally_checked(text)?;
                let locale = LocaleId::parse(text).map_err(|_| DisplayNamesError::InvalidCode)?;
                let canonical = canonicalize(locale).map_err(|_| DisplayNamesError::InvalidCode)?;
                Code::Language(LanguageCode::structurally_checked(canonical.as_str())?)
            }
            DisplayNamesType::Region if valid_region(text) => {
                Code::Region(text.to_ascii_uppercase().into())
            }
            DisplayNamesType::Script
                if text.len() == 4 && text.bytes().all(|b| b.is_ascii_alphabetic()) =>
            {
                Code::Script(title_case(text))
            }
            DisplayNamesType::Currency
                if text.len() == 3 && text.bytes().all(|b| b.is_ascii_alphabetic()) =>
            {
                Code::Currency(text.to_ascii_uppercase().into())
            }
            DisplayNamesType::Calendar
                if !text.is_empty()
                    && text.split('-').all(|s| {
                        (3..=8).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_alphanumeric())
                    }) =>
            {
                Code::Calendar(text.to_ascii_lowercase().into())
            }
            DisplayNamesType::DateTimeField => Code::DateTimeField(
                *DisplayNamesDateTimeField::ALL
                    .iter()
                    .find(|field| field.name() == text)
                    .ok_or(DisplayNamesError::InvalidCode)?,
            ),
            DisplayNamesType::Region
            | DisplayNamesType::Script
            | DisplayNamesType::Currency
            | DisplayNamesType::Calendar => return Err(DisplayNamesError::InvalidCode),
        };
        Ok(Self(code))
    }
    pub const fn kind(&self) -> DisplayNamesType {
        match self.0 {
            Code::Language(_) => DisplayNamesType::Language,
            Code::Region(_) => DisplayNamesType::Region,
            Code::Script(_) => DisplayNamesType::Script,
            Code::Currency(_) => DisplayNamesType::Currency,
            Code::Calendar(_) => DisplayNamesType::Calendar,
            Code::DateTimeField(_) => DisplayNamesType::DateTimeField,
        }
    }
    pub fn as_str(&self) -> &str {
        match &self.0 {
            Code::Language(language) => &language.canonical,
            Code::Region(text)
            | Code::Script(text)
            | Code::Currency(text)
            | Code::Calendar(text) => text,
            Code::DateTimeField(field) => field.name(),
        }
    }
    pub(super) fn language(&self) -> Option<&LanguageCode> {
        match &self.0 {
            Code::Language(value) => Some(value),
            _ => None,
        }
    }
}
