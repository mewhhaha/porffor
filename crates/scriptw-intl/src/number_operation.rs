//! Typed NumberFormat operation inputs after JavaScript observations.
use std::sync::Arc;

use crate::number_format::numeric::{
    normalize_numeric_input, NumberRange, NumericNormalizationError, ObservedNumericInput,
};
use crate::number_format::{
    partition_number, partition_number_range, NumberFormatConfiguration, NumberFormatKernelError,
    NumberProfiles, PartitionLimits, RangeNumberPartition, ScalarNumberPartition,
};
use crate::CanonicalLocaleId;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberFormatRequest {
    pub configuration: NumberFormatConfiguration,
    pub input: ObservedNumericInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberRangeFormatRequest {
    pub configuration: NumberFormatConfiguration,
    pub start: ObservedNumericInput,
    pub end: ObservedNumericInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberSupportedLocalesResult {
    pub locales: Box<[CanonicalLocaleId]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberFormatOperationError {
    UnavailableService(crate::IntlService),
    Kernel(NumberFormatKernelError),
    Numeric(NumericNormalizationError),
    NaNRangeEndpoint,
}
impl From<NumberFormatKernelError> for NumberFormatOperationError {
    fn from(error: NumberFormatKernelError) -> Self {
        Self::Kernel(error)
    }
}
impl From<NumericNormalizationError> for NumberFormatOperationError {
    fn from(error: NumericNormalizationError) -> Self {
        Self::Numeric(error)
    }
}
impl fmt::Display for NumberFormatOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableService(service) => crate::UnavailableIntlService(*service).fmt(f),
            Self::Kernel(error) => error.fmt(f),
            Self::Numeric(error) => error.fmt(f),
            Self::NaNRangeEndpoint => f.write_str("NumberFormat range endpoint is NaN"),
        }
    }
}
impl std::error::Error for NumberFormatOperationError {}

pub fn format_number_parts_operation(
    request: NumberFormatRequest,
    profiles: &Arc<NumberProfiles>,
    limits: &PartitionLimits,
) -> Result<ScalarNumberPartition, NumberFormatOperationError> {
    request.configuration.locale.ensure_profiles(profiles)?;
    let value = normalize_numeric_input(request.input, &limits.numeric())?;
    Ok(partition_number(
        &request.configuration,
        &value,
        profiles,
        limits,
    )?)
}
pub fn format_number_range_parts_operation(
    request: NumberRangeFormatRequest,
    profiles: &Arc<NumberProfiles>,
    limits: &PartitionLimits,
) -> Result<RangeNumberPartition, NumberFormatOperationError> {
    request.configuration.locale.ensure_profiles(profiles)?;
    // Both source-level conversions have already completed. Normalization is
    // pure and retains string/BigInt precision before excluding NaN endpoints.
    let start = normalize_numeric_input(request.start, &limits.numeric())?;
    let end = normalize_numeric_input(request.end, &limits.numeric())?;
    let range =
        NumberRange::new(start, end).map_err(|_| NumberFormatOperationError::NaNRangeEndpoint)?;
    Ok(partition_number_range(
        &request.configuration,
        &range,
        profiles,
        limits,
    )?)
}
