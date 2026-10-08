//! Checked Duration operations, exhaustively joined to native/global/host dispatch.
use crate::duration_format::{
    format_duration_parts, CheckedDurationConfiguration, DurationError, DurationPartition,
    DurationProfiles, DurationRecord,
};
use crate::number_format::{NumberLocaleRequest, NumberProfiles, PartitionLimits};
use crate::CanonicalLocaleId;
use std::sync::Arc;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurationHostOp {
    Resolve = 36,
    SupportedLocales = 37,
    Parts = 38,
}
impl DurationHostOp {
    pub const ALL: &'static [Self] = &[Self::Resolve, Self::SupportedLocales, Self::Parts];
    pub const fn proposed_global_tag(self) -> u32 {
        self as u32
    }
}
#[derive(Debug, Clone)]
pub enum DurationRequest {
    Resolve(NumberLocaleRequest),
    SupportedLocales(crate::duration_format::DurationSupportedLocalesRequest),
    Parts {
        configuration: CheckedDurationConfiguration,
        record: DurationRecord,
    },
}
impl DurationRequest {
    pub const fn operation(&self) -> DurationHostOp {
        match self {
            Self::Resolve(_) => DurationHostOp::Resolve,
            Self::SupportedLocales(_) => DurationHostOp::SupportedLocales,
            Self::Parts { .. } => DurationHostOp::Parts,
        }
    }
}
#[derive(Debug, Clone)]
pub enum DurationResponse {
    Resolved(crate::duration_format::ResolvedDurationLocale),
    SupportedLocales(Box<[CanonicalLocaleId]>),
    Parts(DurationPartition),
}
pub fn execute_duration_request(
    request: DurationRequest,
    profiles: &DurationProfiles,
    numbers: &Arc<NumberProfiles>,
    limits: &PartitionLimits,
) -> Result<DurationResponse, DurationError> {
    profiles.ensure_numbers(numbers)?;
    match request {
        DurationRequest::Resolve(request) => Ok(DurationResponse::Resolved(
            profiles.resolve_locale(&request, numbers)?,
        )),
        DurationRequest::SupportedLocales(requested) => Ok(DurationResponse::SupportedLocales(
            profiles.supported_locales(requested),
        )),
        DurationRequest::Parts {
            configuration,
            record,
        } => Ok(DurationResponse::Parts(format_duration_parts(
            &configuration,
            &record,
            profiles,
            limits,
        )?)),
    }
}

impl DurationHostOp {
    pub const fn global_operation(self) -> crate::IntlHostOp {
        match self {
            Self::Resolve => crate::IntlHostOp::ResolveDurationFormatLocale,
            Self::SupportedLocales => crate::IntlHostOp::SupportedDurationFormatLocales,
            Self::Parts => crate::IntlHostOp::PartitionDurationFormat,
        }
    }
}
const _: () = {
    let mut index = 0;
    while index < DurationHostOp::ALL.len() {
        let operation = DurationHostOp::ALL[index];
        assert!(operation.proposed_global_tag() == operation.global_operation().code() as u32);
        index += 1;
    }
};
