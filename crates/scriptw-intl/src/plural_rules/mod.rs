//! Exact primitive PluralRules service. JavaScript observations stay in Wasm.
use core::fmt;
use std::sync::Arc;

use crate::number_format::numeric::{
    normalize_numeric_input, round_decimal, IntlMathematicalValue, NumberFormatResourceError,
    NumberSign, NumericLimits, NumericNormalizationError, ObservedNumericInput, RoundedDecimal,
    RoundingSettings,
};
use crate::number_format::options::{LocaleMatcher, Notation};
use crate::number_format::{
    filter_number_locales, matching_locale, owned_text, NumberFormatKernelError, NumberProfiles,
    NumberSupportedLocalesRequest, PartitionLimits,
};
use crate::CanonicalLocaleId;

pub(crate) mod rules;
pub use rules::{PluralCategory, PluralCategorySet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluralType {
    Cardinal,
    Ordinal,
}
impl PluralType {
    pub const ALL: [Self; 2] = [Self::Cardinal, Self::Ordinal];
    pub const OPTIONS: &'static [(&'static str, i64)] = &[
        (Self::Cardinal.name(), Self::Cardinal.wire_code() as i64),
        (Self::Ordinal.name(), Self::Ordinal.wire_code() as i64),
    ];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Cardinal => "cardinal",
            Self::Ordinal => "ordinal",
        }
    }
    pub const fn wire_code(self) -> u64 {
        match self {
            Self::Cardinal => 1,
            Self::Ordinal => 2,
        }
    }
    pub const fn from_wire_code(code: u64) -> Option<Self> {
        match code {
            1 => Some(Self::Cardinal),
            2 => Some(Self::Ordinal),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluralLocaleRequest {
    pub requested: Box<[CanonicalLocaleId]>,
    pub matcher: LocaleMatcher,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluralSupportedLocalesRequest {
    pub requested: Box<[CanonicalLocaleId]>,
    pub matcher: LocaleMatcher,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluralSupportedLocalesResult {
    pub locales: Box<[CanonicalLocaleId]>,
}

/// The data-backed resolver/decoder retains the admitted owner with its masks.
#[derive(Clone)]
pub struct ResolvedPluralLocale {
    resolved: CanonicalLocaleId,
    data: CanonicalLocaleId,
    cardinal: PluralCategorySet,
    ordinal: PluralCategorySet,
    profiles: Arc<NumberProfiles>,
}
impl fmt::Debug for ResolvedPluralLocale {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResolvedPluralLocale")
            .field("resolved", &self.resolved)
            .field("data", &self.data)
            .field("cardinal", &self.cardinal)
            .field("ordinal", &self.ordinal)
            .finish_non_exhaustive()
    }
}
impl PartialEq for ResolvedPluralLocale {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.profiles, &other.profiles)
            && self.resolved == other.resolved
            && self.data == other.data
            && self.cardinal == other.cardinal
            && self.ordinal == other.ordinal
    }
}
impl Eq for ResolvedPluralLocale {}
impl ResolvedPluralLocale {
    pub fn from_resolved(
        resolved: CanonicalLocaleId,
        data: CanonicalLocaleId,
        profiles: &Arc<NumberProfiles>,
    ) -> Result<Self, PluralRulesOperationError> {
        Self::from_resolved_in(
            resolved,
            data,
            crate::number_format::NumberLocaleView::public(profiles),
        )
    }

    pub(crate) fn from_resolved_in(
        resolved: CanonicalLocaleId,
        data: CanonicalLocaleId,
        view: crate::number_format::NumberLocaleView<'_>,
    ) -> Result<Self, PluralRulesOperationError> {
        let profiles = view.owner();
        if resolved != data || !view.contains(data.as_str()) {
            return Err(PluralRulesOperationError::InvalidResolvedLocale);
        }
        let cardinal = profiles
            .plural_categories(data.as_str(), PluralType::Cardinal)
            .ok_or(PluralRulesOperationError::InvalidResolvedLocale)?;
        let ordinal = profiles
            .plural_categories(data.as_str(), PluralType::Ordinal)
            .ok_or(PluralRulesOperationError::InvalidResolvedLocale)?;
        Ok(Self {
            resolved,
            data,
            cardinal,
            ordinal,
            profiles: Arc::clone(profiles),
        })
    }
    pub(crate) fn ensure_profiles(
        &self,
        profiles: &Arc<NumberProfiles>,
    ) -> Result<(), PluralRulesOperationError> {
        if Arc::ptr_eq(&self.profiles, profiles) {
            Ok(())
        } else {
            Err(PluralRulesOperationError::InvalidResolvedLocale)
        }
    }
    pub fn resolved(&self) -> &CanonicalLocaleId {
        &self.resolved
    }
    pub fn data(&self) -> &CanonicalLocaleId {
        &self.data
    }
    pub const fn categories(&self, kind: PluralType) -> PluralCategorySet {
        match kind {
            PluralType::Cardinal => self.cardinal,
            PluralType::Ordinal => self.ordinal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedPluralConfiguration {
    locale: ResolvedPluralLocale,
    kind: PluralType,
    notation: Notation,
    rounding: RoundingSettings,
}
impl CheckedPluralConfiguration {
    pub fn new(
        locale: ResolvedPluralLocale,
        kind: PluralType,
        notation: Notation,
        rounding: RoundingSettings,
    ) -> Self {
        Self {
            locale,
            kind,
            notation,
            rounding,
        }
    }
    pub fn locale(&self) -> &ResolvedPluralLocale {
        &self.locale
    }
    pub const fn plural_type(&self) -> PluralType {
        self.kind
    }
    pub const fn notation(&self) -> Notation {
        self.notation
    }
    pub const fn rounding(&self) -> &RoundingSettings {
        &self.rounding
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectPluralRequest {
    configuration: CheckedPluralConfiguration,
    input: ObservedNumericInput,
}
impl SelectPluralRequest {
    pub const fn new(
        configuration: CheckedPluralConfiguration,
        input: ObservedNumericInput,
    ) -> Self {
        Self {
            configuration,
            input,
        }
    }
    pub fn configuration(&self) -> &CheckedPluralConfiguration {
        &self.configuration
    }
    pub fn input(&self) -> &ObservedNumericInput {
        &self.input
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectPluralRangeRequest {
    configuration: CheckedPluralConfiguration,
    start: ObservedNumericInput,
    end: ObservedNumericInput,
}
impl SelectPluralRangeRequest {
    pub const fn new(
        configuration: CheckedPluralConfiguration,
        start: ObservedNumericInput,
        end: ObservedNumericInput,
    ) -> Self {
        Self {
            configuration,
            start,
            end,
        }
    }
    pub fn configuration(&self) -> &CheckedPluralConfiguration {
        &self.configuration
    }
    pub fn start(&self) -> &ObservedNumericInput {
        &self.start
    }
    pub fn end(&self) -> &ObservedNumericInput {
        &self.end
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluralRulesOperationError {
    UnavailableService(crate::IntlService),
    InvalidResolvedLocale,
    InvalidCategory,
    Data(NumberFormatKernelError),
    Numeric(NumericNormalizationError),
    Rounding(NumberFormatResourceError),
    NaNRangeEndpoint,
}
impl fmt::Display for PluralRulesOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableService(service) => crate::UnavailableIntlService(*service).fmt(f),
            Self::InvalidResolvedLocale => f.write_str("invalid resolved PluralRules locale"),
            Self::InvalidCategory => {
                f.write_str("PluralRules result is outside its data category set")
            }
            Self::Data(e) => e.fmt(f),
            Self::Numeric(e) => e.fmt(f),
            Self::Rounding(e) => e.fmt(f),
            Self::NaNRangeEndpoint => f.write_str("PluralRules range endpoint is NaN"),
        }
    }
}
impl std::error::Error for PluralRulesOperationError {}
impl From<NumberFormatKernelError> for PluralRulesOperationError {
    fn from(e: NumberFormatKernelError) -> Self {
        Self::Data(e)
    }
}
impl From<NumericNormalizationError> for PluralRulesOperationError {
    fn from(e: NumericNormalizationError) -> Self {
        Self::Numeric(e)
    }
}
impl From<NumberFormatResourceError> for PluralRulesOperationError {
    fn from(e: NumberFormatResourceError) -> Self {
        Self::Rounding(e)
    }
}

pub fn resolve_plural_locale(
    request: &PluralLocaleRequest,
    profiles: &Arc<NumberProfiles>,
    limits: &PartitionLimits,
) -> Result<ResolvedPluralLocale, PluralRulesOperationError> {
    let data = request
        .requested
        .iter()
        .find_map(|locale| matching_locale(locale.as_str(), request.matcher, profiles))
        .unwrap_or("en-US");
    let copy = || {
        CanonicalLocaleId::from_data(owned_text(data, limits)?)
            .map_err(|_| NumberFormatKernelError::InvalidResolvedLocale)
    };
    ResolvedPluralLocale::from_resolved(copy()?, copy()?, profiles)
}
pub fn supported_plural_locales(
    request: PluralSupportedLocalesRequest,
    profiles: &NumberProfiles,
) -> Result<PluralSupportedLocalesResult, PluralRulesOperationError> {
    Ok(PluralSupportedLocalesResult {
        locales: filter_number_locales(
            &NumberSupportedLocalesRequest {
                requested: request.requested,
                matcher: request.matcher,
            },
            profiles,
        )?,
    })
}

// Finite identity is FormatNumericToString's bare, unsigned visible digits.
// Nonfinite strings are stable ILD tokens; +/- infinity remain distinct.
enum FormattedKey {
    Finite(RoundedDecimal),
    Infinity(NumberSign),
    NaN,
}
impl FormattedKey {
    fn same_string(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Finite(a), Self::Finite(b)) => {
                a.integer_digits() == b.integer_digits()
                    && a.fraction_digits() == b.fraction_digits()
            }
            (Self::Infinity(a), Self::Infinity(b)) => a == b,
            (Self::NaN, Self::NaN) => true,
            (Self::Finite(_), Self::Infinity(_) | Self::NaN)
            | (Self::Infinity(_), Self::Finite(_) | Self::NaN)
            | (Self::NaN, Self::Finite(_) | Self::Infinity(_)) => false,
        }
    }
}
struct ResolvedPlural {
    category: PluralCategory,
    formatted: FormattedKey,
}
fn resolve_plural(
    configuration: &CheckedPluralConfiguration,
    value: &IntlMathematicalValue,
    profiles: &NumberProfiles,
    limits: &NumericLimits,
) -> Result<ResolvedPlural, PluralRulesOperationError> {
    let formatted = match value {
        IntlMathematicalValue::NaN => FormattedKey::NaN,
        IntlMathematicalValue::Infinity(sign) => FormattedKey::Infinity(*sign),
        IntlMathematicalValue::Finite(_) | IntlMathematicalValue::Zero(_) => {
            FormattedKey::Finite(round_decimal(
                value.finite_value().expect("matched finite domain"),
                configuration.rounding(),
                limits,
            )?)
        }
    };
    let category = match &formatted {
        FormattedKey::NaN | FormattedKey::Infinity(_) => PluralCategory::Other,
        FormattedKey::Finite(rounded) => profiles
            .select_plural_category(
                configuration.locale.data.as_str(),
                configuration.kind,
                configuration.notation,
                rounded,
            )?
            .ok_or(PluralRulesOperationError::InvalidResolvedLocale)?,
    };
    if !configuration
        .locale
        .categories(configuration.kind)
        .contains(category)
    {
        return Err(PluralRulesOperationError::InvalidCategory);
    }
    Ok(ResolvedPlural {
        category,
        formatted,
    })
}
pub fn select_plural_operation(
    request: SelectPluralRequest,
    profiles: &Arc<NumberProfiles>,
    limits: &NumericLimits,
) -> Result<PluralCategory, PluralRulesOperationError> {
    request.configuration.locale.ensure_profiles(profiles)?;
    let value = normalize_numeric_input(request.input, limits)?;
    Ok(resolve_plural(&request.configuration, &value, profiles, limits)?.category)
}
pub fn select_plural_range_operation(
    request: SelectPluralRangeRequest,
    profiles: &Arc<NumberProfiles>,
    limits: &NumericLimits,
) -> Result<PluralCategory, PluralRulesOperationError> {
    request.configuration.locale.ensure_profiles(profiles)?;
    let start = normalize_numeric_input(request.start, limits)?;
    let end = normalize_numeric_input(request.end, limits)?;
    if matches!(&start, IntlMathematicalValue::NaN) || matches!(&end, IntlMathematicalValue::NaN) {
        return Err(PluralRulesOperationError::NaNRangeEndpoint);
    }
    let x = resolve_plural(&request.configuration, &start, profiles, limits)?;
    let y = resolve_plural(&request.configuration, &end, profiles, limits)?;
    if x.formatted.same_string(&y.formatted) {
        return Ok(x.category);
    }
    let category = profiles
        .plural_range_category(
            request.configuration.locale.data.as_str(),
            request.configuration.kind,
            x.category,
            y.category,
        )
        .ok_or(PluralRulesOperationError::InvalidResolvedLocale)?;
    if !request
        .configuration
        .locale
        .categories(request.configuration.kind)
        .contains(category)
    {
        return Err(PluralRulesOperationError::InvalidCategory);
    }
    Ok(category)
}

#[cfg(test)]
mod tests;
