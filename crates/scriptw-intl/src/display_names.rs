//! Pure DisplayNames data service. JavaScript option and ToString observations
//! belong to the emitted Wasm shell, before these primitive requests exist.

use crate::number_format::options::LocaleMatcher;
use crate::{CanonicalLocaleId, CanonicalizeLocale, IntlOperationProvider};
use core::fmt;
use std::sync::Arc;

mod code;
mod kernel_identity;
pub(crate) use kernel_identity::DISPLAY_NAMES_KERNEL_SHA256;
mod profiles;
mod raw;
#[cfg(test)]
mod tests;
pub use code::DisplayNameCode;
pub(crate) use profiles::DISPLAY_NAMES_PROFILE;
pub use profiles::{DisplayNamesProfiles, DISPLAY_NAMES_DATA_SHA256};

macro_rules! display_domain {
    ($name:ident { $($variant:ident = $code:literal => $text:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name { $($variant),+ }
        impl $name {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];
            pub const OPTIONS: &'static [(&'static str, i64)] = &[$(($text, $code)),+];
            pub const fn name(self) -> &'static str { match self { $(Self::$variant => $text),+ } }
            pub const fn wire_code(self) -> u64 { match self { $(Self::$variant => $code),+ } }
            pub const fn from_wire_code(code: u64) -> Option<Self> { match code { $($code => Some(Self::$variant),)+ _ => None } }
        }
    }
}
display_domain!(DisplayNamesType { Language = 1 => "language", Region = 2 => "region", Script = 3 => "script", Currency = 4 => "currency", Calendar = 5 => "calendar", DateTimeField = 6 => "dateTimeField" });
display_domain!(DisplayNamesStyle { Long = 1 => "long", Short = 2 => "short", Narrow = 3 => "narrow" });
display_domain!(DisplayNamesFallback { Code = 1 => "code", None = 2 => "none" });
display_domain!(DisplayNamesLanguageDisplay { Dialect = 1 => "dialect", Standard = 2 => "standard" });
display_domain!(DisplayNamesDateTimeField { Era = 1 => "era", Year = 2 => "year", Quarter = 3 => "quarter", Month = 4 => "month", WeekOfYear = 5 => "weekOfYear", Weekday = 6 => "weekday", Day = 7 => "day", DayPeriod = 8 => "dayPeriod", Hour = 9 => "hour", Minute = 10 => "minute", Second = 11 => "second", TimeZoneName = 12 => "timeZoneName" });

/// The language-display slot exists only for Language. A malformed nonlanguage
/// configuration with a language-display slot cannot enter the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayNamesSelection {
    Language(DisplayNamesLanguageDisplay),
    Region,
    Script,
    Currency,
    Calendar,
    DateTimeField,
}
impl DisplayNamesSelection {
    pub const fn kind(self) -> DisplayNamesType {
        match self {
            Self::Language(_) => DisplayNamesType::Language,
            Self::Region => DisplayNamesType::Region,
            Self::Script => DisplayNamesType::Script,
            Self::Currency => DisplayNamesType::Currency,
            Self::Calendar => DisplayNamesType::Calendar,
            Self::DateTimeField => DisplayNamesType::DateTimeField,
        }
    }
    pub const fn language_display(self) -> Option<DisplayNamesLanguageDisplay> {
        match self {
            Self::Language(display) => Some(display),
            Self::Region | Self::Script | Self::Currency | Self::Calendar | Self::DateTimeField => {
                None
            }
        }
    }
    pub fn from_options(kind: DisplayNamesType, language: DisplayNamesLanguageDisplay) -> Self {
        match kind {
            DisplayNamesType::Language => Self::Language(language),
            DisplayNamesType::Region => Self::Region,
            DisplayNamesType::Script => Self::Script,
            DisplayNamesType::Currency => Self::Currency,
            DisplayNamesType::Calendar => Self::Calendar,
            DisplayNamesType::DateTimeField => Self::DateTimeField,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayNamesLocaleRequest {
    pub requested: Box<[CanonicalLocaleId]>,
    pub matcher: LocaleMatcher,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayNamesSupportedLocalesResult {
    pub locales: Box<[CanonicalLocaleId]>,
}

/// Only complete checked data admission can mint this locale association.
#[derive(Debug, Clone)]
pub struct ResolvedDisplayNamesLocale {
    profile: Arc<profiles::LocaleProfile>,
}
impl ResolvedDisplayNamesLocale {
    pub fn resolved(&self) -> &CanonicalLocaleId {
        self.profile.locale()
    }
}

#[derive(Debug, Clone)]
pub struct CheckedDisplayNamesConfiguration {
    locale: ResolvedDisplayNamesLocale,
    selection: DisplayNamesSelection,
    style: DisplayNamesStyle,
    fallback: DisplayNamesFallback,
}
impl CheckedDisplayNamesConfiguration {
    pub const fn new(
        locale: ResolvedDisplayNamesLocale,
        selection: DisplayNamesSelection,
        style: DisplayNamesStyle,
        fallback: DisplayNamesFallback,
    ) -> Self {
        Self {
            locale,
            selection,
            style,
            fallback,
        }
    }
    pub fn locale(&self) -> &ResolvedDisplayNamesLocale {
        &self.locale
    }
    pub const fn selection(&self) -> DisplayNamesSelection {
        self.selection
    }
    pub const fn style(&self) -> DisplayNamesStyle {
        self.style
    }
    pub const fn fallback(&self) -> DisplayNamesFallback {
        self.fallback
    }
}

/// Lossless units observed by JS ToString. They remain uncanonicalized on the
/// wire; even isolated surrogates must reach code validation unchanged.
#[derive(Debug, Clone)]
pub struct DisplayNameRequest {
    configuration: CheckedDisplayNamesConfiguration,
    code: Box<[u16]>,
}
impl DisplayNameRequest {
    pub fn new(
        configuration: CheckedDisplayNamesConfiguration,
        code: Box<[u16]>,
    ) -> Result<Self, DisplayNamesError> {
        let extent = code
            .len()
            .checked_mul(2)
            .and_then(|n| n.checked_add(configuration.locale().resolved().as_str().len()))
            .and_then(|n| n.checked_add(72))
            .ok_or(DisplayNamesError::Resource("request extent"))?;
        u32::try_from(extent).map_err(|_| DisplayNamesError::Resource("request exceeds Wasm32"))?;
        Ok(Self {
            configuration,
            code,
        })
    }
    pub fn configuration(&self) -> &CheckedDisplayNamesConfiguration {
        &self.configuration
    }
    pub fn code(&self) -> &[u16] {
        &self.code
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayNameResult {
    name: Option<Box<str>>,
}
impl DisplayNameResult {
    pub fn new(name: Option<Box<str>>) -> Result<Self, DisplayNamesError> {
        if name
            .as_ref()
            .is_some_and(|text| u32::try_from(text.len()).is_err())
        {
            return Err(DisplayNamesError::Resource("result exceeds Wasm32"));
        }
        Ok(Self { name })
    }
    /// None becomes JS undefined, distinct from an empty string.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayNamesError {
    UnavailableService(crate::IntlService),
    InvalidCode,
    InvalidConfiguration,
    InvalidResolvedLocale,
    InvalidData(Box<str>),
    Resource(&'static str),
}
impl fmt::Display for DisplayNamesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableService(service) => crate::UnavailableIntlService(*service).fmt(f),
            Self::InvalidCode => f.write_str("invalid DisplayNames code"),
            Self::InvalidConfiguration => f.write_str("invalid DisplayNames configuration"),
            Self::InvalidResolvedLocale => f.write_str("unadmitted DisplayNames locale"),
            Self::InvalidData(reason) => write!(f, "invalid DisplayNames profile: {reason}"),
            Self::Resource(reason) => write!(f, "DisplayNames resource limit: {reason}"),
        }
    }
}
impl std::error::Error for DisplayNamesError {}
fn data_error(reason: impl fmt::Display) -> DisplayNamesError {
    DisplayNamesError::InvalidData(reason.to_string().into_boxed_str())
}

pub fn display_name<P: IntlOperationProvider<CanonicalizeLocale>>(
    request: &DisplayNameRequest,
    canonicalizer: &P,
) -> Result<DisplayNameResult, DisplayNamesError> {
    let code = DisplayNameCode::parse(
        request.configuration.selection().kind(),
        request.code(),
        canonicalizer,
    )?;
    profiles::display_name(request.configuration(), &code)
}
