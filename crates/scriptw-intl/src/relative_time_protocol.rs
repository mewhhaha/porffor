//! RelativeTimeFormat primitive frames; the shared host-operation join is explicit.
use crate::number_format::options::LocaleMatcher;
use crate::number_format::{
    NumberLocaleRequest, NumberPart, NumberPartKind, NumberProfiles, NumberingSystemOption,
    PartitionLimits, ResolvedNumberLocale,
};
use crate::relative_time_format::*;
use crate::CanonicalLocaleId;
use core::fmt;
use std::sync::Arc;

mod wire;
use wire::{Reader, Writer};

pub const RELATIVE_WIRE_VERSION: u64 = 1;
pub const RELATIVE_WIRE_HEADER_BYTES: u64 = 16;
pub const RELATIVE_TIME_CONFIGURATION_WORDS: usize = 2;
const _: () = assert!(RELATIVE_WIRE_VERSION == crate::number_protocol::NUMBER_WIRE_VERSION);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeTimeConfigurationWord {
    Style,
    Numeric,
}
impl RelativeTimeConfigurationWord {
    pub const ALL: [Self; RELATIVE_TIME_CONFIGURATION_WORDS] = [Self::Style, Self::Numeric];
    pub const fn index(self) -> usize {
        match self {
            Self::Style => 0,
            Self::Numeric => 1,
        }
    }
    pub const fn offset(self) -> u64 {
        self.index() as u64 * 8
    }
}
impl RelativeTimeConfiguration {
    pub fn wire_words(&self) -> [u64; RELATIVE_TIME_CONFIGURATION_WORDS] {
        let mut words = [0; RELATIVE_TIME_CONFIGURATION_WORDS];
        for word in RelativeTimeConfigurationWord::ALL {
            words[word.index()] = match word {
                RelativeTimeConfigurationWord::Style => self.style().wire_code(),
                RelativeTimeConfigurationWord::Numeric => self.numeric().wire_code(),
            };
        }
        words
    }
}

/// These reserved codes map exhaustively to IntlHostOp during the future join.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeHostOp {
    ResolveRelativeTimeLocale,
    SupportedRelativeTimeLocales,
    FormatRelativeTimeParts,
}
impl RelativeHostOp {
    pub const ALL: [Self; 3] = [
        Self::ResolveRelativeTimeLocale,
        Self::SupportedRelativeTimeLocales,
        Self::FormatRelativeTimeParts,
    ];
    pub const fn code(self) -> u64 {
        match self {
            Self::ResolveRelativeTimeLocale => 30,
            Self::SupportedRelativeTimeLocales => 31,
            Self::FormatRelativeTimeParts => 32,
        }
    }
    pub const fn from_code(code: u64) -> Option<Self> {
        match code {
            30 => Some(Self::ResolveRelativeTimeLocale),
            31 => Some(Self::SupportedRelativeTimeLocales),
            32 => Some(Self::FormatRelativeTimeParts),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelativeRequest {
    Resolve(NumberLocaleRequest),
    Supported {
        requested: Box<[CanonicalLocaleId]>,
        matcher: LocaleMatcher,
    },
    Parts {
        configuration: RelativeTimeConfiguration,
        value: FiniteRelativeNumber,
        unit: RelativeUnit,
    },
}
impl RelativeRequest {
    pub const fn operation(&self) -> RelativeHostOp {
        match self {
            Self::Resolve(_) => RelativeHostOp::ResolveRelativeTimeLocale,
            Self::Supported { .. } => RelativeHostOp::SupportedRelativeTimeLocales,
            Self::Parts { .. } => RelativeHostOp::FormatRelativeTimeParts,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelativeResponse {
    Resolved(ResolvedRelativeTimeLocale),
    Supported(Box<[CanonicalLocaleId]>),
    Parts(RelativePartition),
}
impl RelativeResponse {
    pub const fn operation(&self) -> RelativeHostOp {
        match self {
            Self::Resolved(_) => RelativeHostOp::ResolveRelativeTimeLocale,
            Self::Supported(_) => RelativeHostOp::SupportedRelativeTimeLocales,
            Self::Parts(_) => RelativeHostOp::FormatRelativeTimeParts,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelativeWireError {
    Malformed(&'static str),
    Operation(RelativeTimeError),
    Resource(&'static str),
}
impl fmt::Display for RelativeWireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(reason) => write!(f, "invalid relative-time frame: {reason}"),
            Self::Operation(error) => error.fmt(f),
            Self::Resource(reason) => write!(f, "relative-time frame resource limit: {reason}"),
        }
    }
}
impl std::error::Error for RelativeWireError {}
impl From<RelativeTimeError> for RelativeWireError {
    fn from(error: RelativeTimeError) -> Self {
        Self::Operation(error)
    }
}

pub fn execute_relative_request(
    request: RelativeRequest,
    profiles: &RelativeProfiles,
    number_profiles: &Arc<NumberProfiles>,
    limits: &PartitionLimits,
) -> Result<RelativeResponse, RelativeTimeError> {
    profiles.ensure_number_profiles(number_profiles)?;
    match request {
        RelativeRequest::Resolve(request) => Ok(RelativeResponse::Resolved(
            profiles.resolve_locale(&request, number_profiles, limits)?,
        )),
        RelativeRequest::Supported { requested, matcher } => Ok(RelativeResponse::Supported(
            profiles.supported_locales(&requested, matcher, limits)?,
        )),
        RelativeRequest::Parts {
            configuration,
            value,
            unit,
        } => Ok(RelativeResponse::Parts(format_relative_time_parts(
            &configuration,
            value,
            unit,
            profiles,
            limits,
        )?)),
    }
}

fn write_locales(
    writer: &mut Writer,
    locales: &[CanonicalLocaleId],
) -> Result<(), RelativeWireError> {
    writer.word(locales.len() as u64)?;
    for locale in locales {
        writer.text(locale.as_str())?;
    }
    Ok(())
}
fn read_locales(
    reader: &mut Reader<'_>,
    limits: &PartitionLimits,
) -> Result<Box<[CanonicalLocaleId]>, RelativeWireError> {
    let count = reader.count(8)?;
    if count as u128 > u128::from(limits.part_count()) {
        return Err(RelativeWireError::Resource("locale count"));
    }
    let mut result = Vec::new();
    result
        .try_reserve_exact(count)
        .map_err(|_| RelativeWireError::Resource("locale allocation"))?;
    for _ in 0..count {
        result.push(reader.locale(limits)?);
    }
    Ok(result.into_boxed_slice())
}
fn write_resolved(
    writer: &mut Writer,
    locale: &ResolvedRelativeTimeLocale,
) -> Result<(), RelativeWireError> {
    writer.text(locale.resolved().as_str())?;
    writer.text(locale.formatting().as_str())?;
    writer.text(locale.numbering_system())
}
fn read_resolved(
    reader: &mut Reader<'_>,
    profiles: &RelativeProfiles,
    number_profiles: &Arc<NumberProfiles>,
    limits: &PartitionLimits,
) -> Result<ResolvedRelativeTimeLocale, RelativeWireError> {
    let resolved = reader.locale(limits)?;
    let formatting = reader.locale(limits)?;
    let numbering = reader.text()?;
    let number = ResolvedNumberLocale::from_resolved_in(
        resolved,
        formatting,
        numbering,
        crate::number_format::NumberLocaleView::relative(number_profiles),
    )
    .map_err(RelativeTimeError::from)?;
    Ok(ResolvedRelativeTimeLocale::from_resolved(number, profiles)?)
}

pub fn encode_relative_request(request: &RelativeRequest) -> Result<Vec<u8>, RelativeWireError> {
    let mut writer = Writer::new(request.operation(), false)?;
    match request {
        RelativeRequest::Resolve(request) => {
            write_locales(&mut writer, &request.requested)?;
            writer.word(request.matcher.wire_code())?;
            writer.word(u64::from(request.numbering_system.is_some()))?;
            if let Some(numbering) = &request.numbering_system {
                writer.text(numbering.name())?;
            }
        }
        RelativeRequest::Supported { requested, matcher } => {
            write_locales(&mut writer, requested)?;
            writer.word(matcher.wire_code())?;
        }
        RelativeRequest::Parts {
            configuration,
            value,
            unit,
        } => {
            write_resolved(&mut writer, configuration.locale())?;
            for word in configuration.wire_words() {
                writer.word(word)?;
            }
            writer.word(value.bits())?;
            writer.word(unit.wire_code())?;
        }
    }
    Ok(writer.finish())
}
pub fn decode_relative_request(
    operation: RelativeHostOp,
    bytes: &[u8],
    profiles: &RelativeProfiles,
    number_profiles: &Arc<NumberProfiles>,
    limits: &PartitionLimits,
) -> Result<RelativeRequest, RelativeWireError> {
    profiles.ensure_number_profiles(number_profiles)?;
    let mut reader = Reader::new(bytes, operation, false)?;
    let request = match operation {
        RelativeHostOp::ResolveRelativeTimeLocale => {
            let requested = read_locales(&mut reader, limits)?;
            let matcher = reader.domain(LocaleMatcher::from_wire_code)?;
            let numbering_system = match reader.word()? {
                0 => None,
                1 => Some(
                    NumberingSystemOption::parse(reader.text()?)
                        .map_err(|_| RelativeWireError::Malformed("numbering system syntax"))?,
                ),
                _ => return Err(RelativeWireError::Malformed("numbering system presence")),
            };
            RelativeRequest::Resolve(NumberLocaleRequest {
                requested,
                matcher,
                numbering_system,
            })
        }
        RelativeHostOp::SupportedRelativeTimeLocales => RelativeRequest::Supported {
            requested: read_locales(&mut reader, limits)?,
            matcher: reader.domain(LocaleMatcher::from_wire_code)?,
        },
        RelativeHostOp::FormatRelativeTimeParts => {
            let locale = read_resolved(&mut reader, profiles, number_profiles, limits)?;
            let style = reader.domain(RelativeStyle::from_wire_code)?;
            let numeric = reader.domain(RelativeNumeric::from_wire_code)?;
            let value = FiniteRelativeNumber::from_bits(reader.word()?)?;
            let unit = reader.domain(RelativeUnit::from_wire_code)?;
            let configuration =
                RelativeTimeConfiguration::new(locale, style, numeric, number_profiles)?;
            RelativeRequest::Parts {
                configuration,
                value,
                unit,
            }
        }
    };
    reader.finish()?;
    Ok(request)
}

pub fn encode_relative_response(response: &RelativeResponse) -> Result<Vec<u8>, RelativeWireError> {
    let mut writer = Writer::new(response.operation(), true)?;
    match response {
        RelativeResponse::Resolved(locale) => write_resolved(&mut writer, locale)?,
        RelativeResponse::Supported(locales) => write_locales(&mut writer, locales)?,
        RelativeResponse::Parts(partition) => {
            writer.word(partition.parts().len() as u64)?;
            for part in partition.parts() {
                writer.word(part.kind().wire_code())?;
                writer.text(part.text())?;
                writer.word(u64::from(part.unit().is_some()))?;
                if let Some(unit) = part.unit() {
                    writer.word(unit.wire_code())?;
                }
            }
        }
    }
    Ok(writer.finish())
}
pub fn decode_relative_response(
    operation: RelativeHostOp,
    bytes: &[u8],
    profiles: &RelativeProfiles,
    number_profiles: &Arc<NumberProfiles>,
    limits: &PartitionLimits,
) -> Result<RelativeResponse, RelativeWireError> {
    profiles.ensure_number_profiles(number_profiles)?;
    let mut reader = Reader::new(bytes, operation, true)?;
    let response = match operation {
        RelativeHostOp::ResolveRelativeTimeLocale => RelativeResponse::Resolved(read_resolved(
            &mut reader,
            profiles,
            number_profiles,
            limits,
        )?),
        RelativeHostOp::SupportedRelativeTimeLocales => {
            let locales = read_locales(&mut reader, limits)?;
            if profiles
                .supported_locales(&locales, LocaleMatcher::Lookup, limits)?
                .len()
                != locales.len()
            {
                return Err(RelativeWireError::Malformed("unsupported locale response"));
            }
            RelativeResponse::Supported(locales)
        }
        RelativeHostOp::FormatRelativeTimeParts => {
            let count = reader.count(24)?;
            if count as u128 > u128::from(limits.part_count()) {
                return Err(RelativeWireError::Resource("part count"));
            }
            let mut parts = Vec::new();
            parts
                .try_reserve_exact(count)
                .map_err(|_| RelativeWireError::Resource("part allocation"))?;
            let mut numeric_unit = None;
            for _ in 0..count {
                let kind = reader.domain(NumberPartKind::from_wire_code)?;
                let text = reader.text()?;
                let unit = match reader.word()? {
                    0 => None,
                    1 => Some(reader.domain(RelativeUnit::from_wire_code)?),
                    _ => return Err(RelativeWireError::Malformed("unit presence")),
                };
                if text.is_empty() || text.contains('\0') {
                    return Err(RelativeWireError::Malformed("empty/NUL part"));
                }
                let text = crate::number_format::owned_text(text, limits)
                    .map_err(RelativeTimeError::from)?;
                if let Some(unit) = unit {
                    if !matches!(
                        kind,
                        NumberPartKind::Literal
                            | NumberPartKind::Integer
                            | NumberPartKind::Group
                            | NumberPartKind::Decimal
                            | NumberPartKind::Fraction
                    ) {
                        return Err(RelativeWireError::Malformed(
                            "non-decimal relative number part",
                        ));
                    }
                    if numeric_unit.is_some_and(|previous| previous != unit) {
                        return Err(RelativeWireError::Malformed("mixed numeric units"));
                    }
                    numeric_unit = Some(unit);
                    parts.push(RelativePart::Number {
                        part: NumberPart::new(kind, text),
                        unit,
                    });
                } else {
                    if kind != NumberPartKind::Literal {
                        return Err(RelativeWireError::Malformed("unitless numeric part"));
                    }
                    parts.push(RelativePart::Literal(text));
                }
            }
            RelativeResponse::Parts(RelativePartition::from_parts(parts, limits)?)
        }
    };
    reader.finish()?;
    Ok(response)
}

#[cfg(test)]
mod tests;

impl RelativeHostOp {
    pub const fn global_operation(self) -> crate::IntlHostOp {
        match self {
            Self::ResolveRelativeTimeLocale => crate::IntlHostOp::ResolveRelativeTimeLocale,
            Self::SupportedRelativeTimeLocales => crate::IntlHostOp::SupportedRelativeTimeLocales,
            Self::FormatRelativeTimeParts => crate::IntlHostOp::FormatRelativeTimeParts,
        }
    }
}
const _: () = {
    let mut index = 0;
    while index < RelativeHostOp::ALL.len() {
        let operation = RelativeHostOp::ALL[index];
        assert!(operation.code() == operation.global_operation().code() as u64);
        index += 1;
    }
};
