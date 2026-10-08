//! Pinned list templates and lossless indexed element partitions.

use crate::number_format::options::LocaleMatcher;
use crate::CanonicalLocaleId;
use core::fmt;
use std::sync::Arc;

mod identity;
mod partition;
mod profiles;
#[cfg(test)]
mod tests;
pub(crate) use identity::LIST_FORMAT_DATA_SHA256;
pub use profiles::{embedded_list_profiles, ListProfiles};

macro_rules! closed_list_domain {
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
closed_list_domain!(ListType { Conjunction = 1 => "conjunction", Disjunction = 2 => "disjunction", Unit = 3 => "unit" });
closed_list_domain!(ListStyle { Long = 1 => "long", Short = 2 => "short", Narrow = 3 => "narrow" });
closed_list_domain!(ListPartKind { Literal = 0 => "literal", Element = 1 => "element" });

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListLocaleRequest {
    pub requested: Box<[CanonicalLocaleId]>,
    pub matcher: LocaleMatcher,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListSupportedLocalesRequest {
    pub requested: Box<[CanonicalLocaleId]>,
    pub matcher: LocaleMatcher,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListSupportedLocalesResult {
    pub locales: Box<[CanonicalLocaleId]>,
}

/// The only constructor is the catalogue owner after all nine profiles load.
#[derive(Debug, Clone)]
pub struct ResolvedListLocale {
    profile: Arc<profiles::ListProfile>,
}
impl ResolvedListLocale {
    pub fn resolved(&self) -> &CanonicalLocaleId {
        self.profile.locale()
    }
}
#[derive(Debug, Clone)]
pub struct CheckedListConfiguration {
    locale: ResolvedListLocale,
    kind: ListType,
    style: ListStyle,
}
impl CheckedListConfiguration {
    pub fn new(locale: ResolvedListLocale, kind: ListType, style: ListStyle) -> Self {
        Self {
            locale,
            kind,
            style,
        }
    }
    pub fn locale(&self) -> &ResolvedListLocale {
        &self.locale
    }
    pub const fn list_type(&self) -> ListType {
        self.kind
    }
    pub const fn style(&self) -> ListStyle {
        self.style
    }
    fn profile(&self) -> &profiles::ListProfile {
        &self.locale.profile
    }
}

#[derive(Debug, Clone)]
pub struct FormatListPartsRequest {
    configuration: CheckedListConfiguration,
    elements: Box<[Box<[u16]>]>,
}
impl FormatListPartsRequest {
    /// Any UTF16 code unit is admitted, including every isolated surrogate.
    pub fn new(
        configuration: CheckedListConfiguration,
        elements: Box<[Box<[u16]>]>,
    ) -> Result<Self, ListFormatOperationError> {
        u32::try_from(elements.len())
            .map_err(|_| ListFormatOperationError::Resource("element count"))?;
        let mut extent = configuration
            .locale()
            .resolved()
            .as_str()
            .len()
            .checked_add(48)
            .ok_or(ListFormatOperationError::Resource("request extent"))?;
        for element in &elements {
            let bytes = element
                .len()
                .checked_mul(2)
                .and_then(|n| n.checked_add(8))
                .ok_or(ListFormatOperationError::Resource("UTF16 element extent"))?;
            extent = extent
                .checked_add(bytes)
                .ok_or(ListFormatOperationError::Resource("request extent"))?;
        }
        u32::try_from(extent)
            .map_err(|_| ListFormatOperationError::Resource("request exceeds Wasm32 span"))?;
        Ok(Self {
            configuration,
            elements,
        })
    }
    pub fn configuration(&self) -> &CheckedListConfiguration {
        &self.configuration
    }
    pub fn elements(&self) -> &[Box<[u16]>] {
        &self.elements
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListPart {
    Literal(Box<[u16]>),
    Element(u32),
}
impl ListPart {
    pub const fn kind(&self) -> ListPartKind {
        match self {
            Self::Literal(_) => ListPartKind::Literal,
            Self::Element(_) => ListPartKind::Element,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListParts {
    parts: Box<[ListPart]>,
    input_count: u32,
}
impl ListParts {
    pub fn parts(&self) -> &[ListPart] {
        &self.parts
    }
    pub const fn input_count(&self) -> u32 {
        self.input_count
    }
    pub(crate) fn checked(
        parts: Vec<ListPart>,
        input_count: u32,
    ) -> Result<Self, ListFormatOperationError> {
        let mut next = 0u32;
        let mut bytes = 24usize;
        for part in &parts {
            let extent = match part {
                ListPart::Element(index) => {
                    if *index != next || next >= input_count {
                        return Err(ListFormatOperationError::InvalidPartition);
                    }
                    next += 1;
                    16
                }
                ListPart::Literal(text) => {
                    if text.is_empty() {
                        return Err(ListFormatOperationError::InvalidPartition);
                    }
                    text.len()
                        .checked_mul(2)
                        .and_then(|n| n.checked_add(16))
                        .ok_or(ListFormatOperationError::Resource("literal extent"))?
                }
            };
            bytes = bytes
                .checked_add(extent)
                .ok_or(ListFormatOperationError::Resource("partition extent"))?;
        }
        if next != input_count || (input_count == 0 && !parts.is_empty()) {
            return Err(ListFormatOperationError::InvalidPartition);
        }
        u32::try_from(bytes)
            .map_err(|_| ListFormatOperationError::Resource("partition exceeds Wasm32 span"))?;
        Ok(Self {
            parts: parts.into_boxed_slice(),
            input_count,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListFormatOperationError {
    UnavailableService(crate::IntlService),
    InvalidResolvedLocale,
    InvalidPartition,
    Data(Box<str>),
    Resource(&'static str),
}
impl fmt::Display for ListFormatOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableService(service) => crate::UnavailableIntlService(*service).fmt(f),
            Self::InvalidResolvedLocale => f.write_str("invalid resolved ListFormat locale"),
            Self::InvalidPartition => f.write_str("invalid indexed ListFormat partition"),
            Self::Data(reason) => write!(f, "invalid pinned ListFormat data: {reason}"),
            Self::Resource(reason) => write!(f, "ListFormat resource limit: {reason}"),
        }
    }
}
impl std::error::Error for ListFormatOperationError {}

pub(crate) fn format_list_parts(
    request: FormatListPartsRequest,
) -> Result<ListParts, ListFormatOperationError> {
    partition::format(&request)
}
