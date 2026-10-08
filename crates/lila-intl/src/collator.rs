//! Closed Collator domains shared by the native service and emitted Wasm.

macro_rules! closed_collator_domain {
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

closed_collator_domain!(CollatorUsage { Sort = 1 => "sort", Search = 2 => "search" });
closed_collator_domain!(CollatorSensitivity { Base = 1 => "base", Accent = 2 => "accent", Case = 3 => "case", Variant = 4 => "variant" });
closed_collator_domain!(CollatorCaseFirst { False = 0 => "false", Lower = 1 => "lower", Upper = 2 => "upper" });
closed_collator_domain!(CollatorCollationKind { Default = 0 => "default", UnicodeType = 1 => "unicodeType" });
closed_collator_domain!(CollatorOrdering { Equal = 0 => "equal", Less = 1 => "less", Greater = 2 => "greater" });

use crate::number_format::options::LocaleMatcher;
use crate::CanonicalLocaleId;
use core::fmt;
use std::sync::Arc;
mod identity;
mod profiles;
#[cfg(test)]
mod tests;
pub(crate) use identity::COLLATOR_DATA_SHA256;
pub use profiles::{embedded_collator_profiles, CollatorProfiles};

/// Original syntax-valid option spelling. Canonicalization belongs to ResolveLocale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollatorCollationOption(Box<str>);
impl CollatorCollationOption {
    pub fn parse(value: impl Into<Box<str>>) -> Result<Self, CollatorOperationError> {
        let value = value.into();
        if value.is_empty()
            || !value.split('-').all(|part| {
                (3..=8).contains(&part.len()) && part.bytes().all(|b| b.is_ascii_alphanumeric())
            })
        {
            return Err(CollatorOperationError::InvalidConfiguration);
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollatorLocaleRequest {
    pub requested: Box<[CanonicalLocaleId]>,
    pub matcher: LocaleMatcher,
    pub usage: CollatorUsage,
    pub collation: Option<CollatorCollationOption>,
    pub numeric: Option<bool>,
    pub case_first: Option<CollatorCaseFirst>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollatorSupportedLocalesRequest {
    pub requested: Box<[CanonicalLocaleId]>,
    pub matcher: LocaleMatcher,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollatorSupportedLocalesResult {
    pub locales: Box<[CanonicalLocaleId]>,
}

/// The profile owner is constructed only after its real sort/search payloads load.
#[derive(Debug, Clone)]
pub struct ResolvedCollatorLocale {
    resolved: CanonicalLocaleId,
    usage: CollatorUsage,
    profile: Arc<profiles::CollationProfile>,
    numeric: bool,
    case_first: CollatorCaseFirst,
}
impl ResolvedCollatorLocale {
    pub fn resolved(&self) -> &CanonicalLocaleId {
        &self.resolved
    }
    pub const fn usage(&self) -> CollatorUsage {
        self.usage
    }
    pub fn collation(&self) -> Option<&str> {
        self.profile.public_collation()
    }
    pub fn collation_kind(&self) -> CollatorCollationKind {
        if self.collation().is_some() {
            CollatorCollationKind::UnicodeType
        } else {
            CollatorCollationKind::Default
        }
    }
    pub const fn numeric(&self) -> bool {
        self.numeric
    }
    pub const fn case_first(&self) -> CollatorCaseFirst {
        self.case_first
    }
    /// Chosen ILD search policy and prescribed sort policy both use Variant.
    pub const fn default_sensitivity(&self) -> CollatorSensitivity {
        CollatorSensitivity::Variant
    }
    pub fn default_ignore_punctuation(&self) -> bool {
        self.profile.default_ignore_punctuation()
    }
}
#[derive(Debug, Clone)]
pub struct CheckedCollatorConfiguration {
    locale: ResolvedCollatorLocale,
    sensitivity: CollatorSensitivity,
    ignore_punctuation: bool,
}
impl CheckedCollatorConfiguration {
    pub fn new(
        locale: ResolvedCollatorLocale,
        sensitivity: CollatorSensitivity,
        ignore_punctuation: bool,
    ) -> Self {
        Self {
            locale,
            sensitivity,
            ignore_punctuation,
        }
    }
    pub fn locale(&self) -> &ResolvedCollatorLocale {
        &self.locale
    }
    pub const fn sensitivity(&self) -> CollatorSensitivity {
        self.sensitivity
    }
    pub const fn ignore_punctuation(&self) -> bool {
        self.ignore_punctuation
    }
}
#[derive(Debug, Clone)]
pub struct CompareCollatorRequest {
    configuration: CheckedCollatorConfiguration,
    left: Box<[u16]>,
    right: Box<[u16]>,
}
impl CompareCollatorRequest {
    pub fn new(
        configuration: CheckedCollatorConfiguration,
        left: Box<[u16]>,
        right: Box<[u16]>,
    ) -> Result<Self, CollatorOperationError> {
        let collation_extent = configuration.locale().collation().map_or(Ok(0), |co| {
            co.len()
                .checked_add(8)
                .ok_or(CollatorOperationError::Resource("collation extent"))
        })?;
        let extent = configuration
            .locale()
            .resolved()
            .as_str()
            .len()
            .checked_add(88)
            .and_then(|n| n.checked_add(collation_extent))
            .and_then(|n| {
                left.len()
                    .checked_add(right.len())
                    .and_then(|units| units.checked_mul(2))
                    .and_then(|bytes| n.checked_add(bytes))
            })
            .ok_or(CollatorOperationError::Resource("compare request extent"))?;
        u32::try_from(extent)
            .map_err(|_| CollatorOperationError::Resource("compare request exceeds Wasm32"))?;
        Ok(Self {
            configuration,
            left,
            right,
        })
    }
    pub fn configuration(&self) -> &CheckedCollatorConfiguration {
        &self.configuration
    }
    pub fn left(&self) -> &[u16] {
        &self.left
    }
    pub fn right(&self) -> &[u16] {
        &self.right
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollatorOperationError {
    UnavailableService(crate::IntlService),
    InvalidConfiguration,
    Data(Box<str>),
    Resource(&'static str),
}
impl fmt::Display for CollatorOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableService(service) => crate::UnavailableIntlService(*service).fmt(f),
            Self::InvalidConfiguration => f.write_str("invalid admitted Collator configuration"),
            Self::Data(reason) => write!(f, "invalid pinned Collator data: {reason}"),
            Self::Resource(reason) => write!(f, "Collator resource limit: {reason}"),
        }
    }
}
impl std::error::Error for CollatorOperationError {}

pub(crate) fn compare_collator(
    request: CompareCollatorRequest,
) -> Result<CollatorOrdering, CollatorOperationError> {
    use core::cmp::Ordering;
    let config = request.configuration();
    let collator = config.locale.profile.construct(
        config.locale.numeric,
        config.locale.case_first,
        config.sensitivity,
        config.ignore_punctuation,
    )?;
    Ok(
        match collator
            .as_borrowed()
            .compare_utf16(request.left(), request.right())
        {
            Ordering::Equal => CollatorOrdering::Equal,
            Ordering::Less => CollatorOrdering::Less,
            Ordering::Greater => CollatorOrdering::Greater,
        },
    )
}
