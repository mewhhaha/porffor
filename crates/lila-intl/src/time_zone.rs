use core::fmt;

use crate::{CanonicalLocaleId, TimeZoneId};

/// The two identifiers in an AvailableNamedTimeZoneIdentifiers record.
/// A link retains its normalized Identifier even when it has another primary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedTimeZoneIdentity {
    identifier: TimeZoneId,
    primary_identifier: TimeZoneId,
}

impl NamedTimeZoneIdentity {
    pub(crate) fn from_data(
        identifier: &str,
        primary_identifier: &str,
    ) -> Result<Self, InvalidTimeZoneData> {
        let identifier = TimeZoneId::parse(identifier)
            .map_err(|_| InvalidTimeZoneData("invalid normalized named identifier"))?;
        let primary_identifier = TimeZoneId::parse(primary_identifier)
            .map_err(|_| InvalidTimeZoneData("invalid primary named identifier"))?;
        if [identifier.as_str(), primary_identifier.as_str()]
            .iter()
            .any(|name| name.starts_with(['+', '-']))
        {
            return Err(InvalidTimeZoneData(
                "a named identifier cannot be an offset",
            ));
        }
        Ok(Self {
            identifier,
            primary_identifier,
        })
    }

    pub fn identifier(&self) -> &str {
        self.identifier.as_str()
    }
    pub fn primary_identifier(&self) -> &str {
        self.primary_identifier.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTimeZoneData(pub(crate) &'static str);

impl fmt::Display for InvalidTimeZoneData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for InvalidTimeZoneData {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookupNamedTimeZoneRequest {
    identifier: TimeZoneId,
}

impl LookupNamedTimeZoneRequest {
    pub const fn new(identifier: TimeZoneId) -> Self {
        Self { identifier }
    }
    pub const fn identifier(&self) -> &TimeZoneId {
        &self.identifier
    }
    pub fn into_identifier(self) -> TimeZoneId {
        self.identifier
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookupNamedTimeZoneResult {
    identity: NamedTimeZoneIdentity,
}

impl LookupNamedTimeZoneResult {
    pub(crate) const fn new(identity: NamedTimeZoneIdentity) -> Self {
        Self { identity }
    }
    pub const fn identity(&self) -> &NamedTimeZoneIdentity {
        &self.identity
    }

    pub fn encode(&self) -> Vec<u8> {
        let identifier = self.identity.identifier().as_bytes();
        let primary = self.identity.primary_identifier().as_bytes();
        let mut bytes =
            Vec::with_capacity(LOOKUP_TIME_ZONE_HEADER_BYTES + identifier.len() + primary.len());
        bytes.extend_from_slice(&(identifier.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&(primary.len() as u64).to_le_bytes());
        bytes.extend_from_slice(identifier);
        bytes.extend_from_slice(primary);
        bytes
    }
}

pub const LOOKUP_TIME_ZONE_HEADER_BYTES: usize = 16;
pub const LOOKUP_TIME_ZONE_IDENTIFIER_LENGTH_OFFSET: u64 = 0;
pub const LOOKUP_TIME_ZONE_PRIMARY_LENGTH_OFFSET: u64 = 8;

/// The stored DateTimeFormat option codes and provider name styles share one domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TimeZoneNameStyle {
    Short = 1,
    Long = 2,
    ShortOffset = 3,
    LongOffset = 4,
    ShortGeneric = 5,
    LongGeneric = 6,
}

impl TimeZoneNameStyle {
    pub const ALL: [Self; 6] = [
        Self::Short,
        Self::Long,
        Self::ShortOffset,
        Self::LongOffset,
        Self::ShortGeneric,
        Self::LongGeneric,
    ];
    pub const OPTIONS: [(&'static str, i64); Self::ALL.len()] = {
        let mut options = [("", 0); Self::ALL.len()];
        let mut index = 0;
        while index < Self::ALL.len() {
            let style = Self::ALL[index];
            options[index] = (style.spelling(), style.code());
            index += 1;
        }
        options
    };
    pub const fn code(self) -> i64 {
        self as i64
    }
    pub const fn from_code(code: i64) -> Option<Self> {
        match code {
            1 => Some(Self::Short),
            2 => Some(Self::Long),
            3 => Some(Self::ShortOffset),
            4 => Some(Self::LongOffset),
            5 => Some(Self::ShortGeneric),
            6 => Some(Self::LongGeneric),
            _ => None,
        }
    }
    pub const fn spelling(self) -> &'static str {
        match self {
            Self::Short => "short",
            Self::Long => "long",
            Self::ShortOffset => "shortOffset",
            Self::LongOffset => "longOffset",
            Self::ShortGeneric => "shortGeneric",
            Self::LongGeneric => "longGeneric",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TimeZoneKind {
    Named = 1,
    FixedOffset = 2,
}

impl TimeZoneKind {
    pub const fn code(self) -> i64 {
        self as i64
    }
    pub const fn from_code(code: i64) -> Option<Self> {
        match code {
            1 => Some(Self::Named),
            2 => Some(Self::FixedOffset),
            _ => None,
        }
    }
}

/// ECMA-402 accepts minute-aligned fixed offsets strictly below 24 hours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedTimeZoneOffset(i32);

impl FixedTimeZoneOffset {
    pub const MAX_HOUR: i64 = 23;
    pub const MAX_MINUTE: i64 = 59;
    const MAX_SECONDS: u64 = ((Self::MAX_HOUR * 60 + Self::MAX_MINUTE) * 60) as u64;
    pub fn from_seconds(seconds: i64) -> Result<Self, InvalidTimeZoneRequest> {
        if seconds.unsigned_abs() > Self::MAX_SECONDS || seconds % 60 != 0 {
            return Err(InvalidTimeZoneRequest(
                "fixed offset is outside the minute-aligned domain",
            ));
        }
        Ok(Self(seconds as i32))
    }
    pub const fn seconds(self) -> i32 {
        self.0
    }
}

/// Exact input domain of Date and Temporal.Instant, already converted with floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeZoneEpochSeconds(i64);

impl TimeZoneEpochSeconds {
    pub const LIMIT: i64 = 8_640_000_000_000;
    pub fn new(seconds: i64) -> Result<Self, InvalidTimeZoneRequest> {
        if !(-Self::LIMIT..=Self::LIMIT).contains(&seconds) {
            return Err(InvalidTimeZoneRequest(
                "epoch seconds are outside the exact-time domain",
            ));
        }
        Ok(Self(seconds))
    }
    pub fn from_milliseconds(milliseconds: i64) -> Result<Self, InvalidTimeZoneRequest> {
        Self::new(milliseconds.div_euclid(1000))
    }
    pub const fn get(self) -> i64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeZoneSelection {
    Named(TimeZoneId),
    FixedOffset(FixedTimeZoneOffset),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveTimeZoneRequest {
    selection: TimeZoneSelection,
    epoch: TimeZoneEpochSeconds,
    name_style: Option<TimeZoneNameStyle>,
    locale: CanonicalLocaleId,
}

impl ResolveTimeZoneRequest {
    pub const fn new(
        selection: TimeZoneSelection,
        epoch: TimeZoneEpochSeconds,
        name_style: Option<TimeZoneNameStyle>,
        locale: CanonicalLocaleId,
    ) -> Self {
        Self {
            selection,
            epoch,
            name_style,
            locale,
        }
    }
    pub const fn selection(&self) -> &TimeZoneSelection {
        &self.selection
    }
    pub const fn epoch(&self) -> TimeZoneEpochSeconds {
        self.epoch
    }
    pub const fn name_style(&self) -> Option<TimeZoneNameStyle> {
        self.name_style
    }
    pub const fn locale(&self) -> &CanonicalLocaleId {
        &self.locale
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, InvalidTimeZoneRequest> {
        let header = bytes
            .get(..RESOLVE_TIME_ZONE_HEADER_BYTES)
            .ok_or(InvalidTimeZoneRequest("truncated time-zone request header"))?;
        let word = |offset: u64| {
            let offset = offset as usize;
            i64::from_le_bytes(
                header[offset..offset + 8]
                    .try_into()
                    .expect("bounded header word"),
            )
        };
        let kind = TimeZoneKind::from_code(word(RESOLVE_TIME_ZONE_KIND_OFFSET))
            .ok_or(InvalidTimeZoneRequest("unknown time-zone selection kind"))?;
        let fixed_seconds = word(RESOLVE_TIME_ZONE_FIXED_SECONDS_OFFSET);
        let epoch = TimeZoneEpochSeconds::new(word(RESOLVE_TIME_ZONE_EPOCH_SECONDS_OFFSET))?;
        let style = word(RESOLVE_TIME_ZONE_NAME_STYLE_OFFSET);
        let name_style = if style == 0 {
            None
        } else {
            Some(
                TimeZoneNameStyle::from_code(style)
                    .ok_or(InvalidTimeZoneRequest("unknown time-zone name style"))?,
            )
        };
        let identifier_length =
            usize::try_from(word(RESOLVE_TIME_ZONE_IDENTIFIER_LENGTH_OFFSET))
                .map_err(|_| InvalidTimeZoneRequest("invalid named identifier length"))?;
        let locale_length = usize::try_from(word(RESOLVE_TIME_ZONE_LOCALE_LENGTH_OFFSET))
            .map_err(|_| InvalidTimeZoneRequest("invalid name locale length"))?;
        let identifier_end = RESOLVE_TIME_ZONE_HEADER_BYTES
            .checked_add(identifier_length)
            .ok_or(InvalidTimeZoneRequest("named identifier length overflow"))?;
        let end = identifier_end
            .checked_add(locale_length)
            .ok_or(InvalidTimeZoneRequest("name locale length overflow"))?;
        if end != bytes.len() {
            return Err(InvalidTimeZoneRequest("time-zone request extent mismatch"));
        }
        let identifier =
            core::str::from_utf8(&bytes[RESOLVE_TIME_ZONE_HEADER_BYTES..identifier_end])
                .map_err(|_| InvalidTimeZoneRequest("named identifier is not UTF-8"))?;
        let locale = core::str::from_utf8(&bytes[identifier_end..])
            .map_err(|_| InvalidTimeZoneRequest("name locale is not UTF-8"))?;
        let locale = CanonicalLocaleId::from_data(locale)
            .map_err(|_| InvalidTimeZoneRequest("name locale is not canonical"))?;
        let selection = match kind {
            TimeZoneKind::Named => {
                if fixed_seconds != 0 || identifier.starts_with(['+', '-']) {
                    return Err(InvalidTimeZoneRequest(
                        "named selection carries a fixed offset",
                    ));
                }
                TimeZoneSelection::Named(
                    TimeZoneId::parse(identifier).map_err(|_| {
                        InvalidTimeZoneRequest("invalid normalized named identifier")
                    })?,
                )
            }
            TimeZoneKind::FixedOffset => {
                if !identifier.is_empty() {
                    return Err(InvalidTimeZoneRequest(
                        "fixed selection carries a named identifier",
                    ));
                }
                TimeZoneSelection::FixedOffset(FixedTimeZoneOffset::from_seconds(fixed_seconds)?)
            }
        };
        Ok(Self::new(selection, epoch, name_style, locale))
    }

    pub fn encode(&self) -> Vec<u8> {
        let (kind, fixed, identifier) = match &self.selection {
            TimeZoneSelection::Named(identifier) => (TimeZoneKind::Named, 0, identifier.as_str()),
            TimeZoneSelection::FixedOffset(offset) => {
                (TimeZoneKind::FixedOffset, i64::from(offset.seconds()), "")
            }
        };
        let locale = self.locale.as_str();
        let mut bytes =
            Vec::with_capacity(RESOLVE_TIME_ZONE_HEADER_BYTES + identifier.len() + locale.len());
        for word in [
            kind.code(),
            fixed,
            self.epoch.get(),
            self.name_style.map_or(0, TimeZoneNameStyle::code),
            identifier.len() as i64,
            locale.len() as i64,
        ] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes.extend_from_slice(identifier.as_bytes());
        bytes.extend_from_slice(locale.as_bytes());
        bytes
    }
}

pub const RESOLVE_TIME_ZONE_HEADER_BYTES: usize = 48;
pub const RESOLVE_TIME_ZONE_KIND_OFFSET: u64 = 0;
pub const RESOLVE_TIME_ZONE_FIXED_SECONDS_OFFSET: u64 = 8;
pub const RESOLVE_TIME_ZONE_EPOCH_SECONDS_OFFSET: u64 = 16;
pub const RESOLVE_TIME_ZONE_NAME_STYLE_OFFSET: u64 = 24;
pub const RESOLVE_TIME_ZONE_IDENTIFIER_LENGTH_OFFSET: u64 = 32;
pub const RESOLVE_TIME_ZONE_LOCALE_LENGTH_OFFSET: u64 = 40;
pub const RESOLVE_TIME_ZONE_RESULT_HEADER_BYTES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTimeZoneRequest(&'static str);
impl fmt::Display for InvalidTimeZoneRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for InvalidTimeZoneRequest {}

/// A name uses the resolved locale's text and Latin decimal digit skeleton.
/// The AOT shell applies the formatter's already resolved numbering system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTimeZoneSnapshot {
    offset_seconds: i32,
    display_name: Option<String>,
}

impl ResolvedTimeZoneSnapshot {
    pub(crate) fn from_data(
        offset_seconds: i32,
        display_name: Option<String>,
    ) -> Result<Self, InvalidTimeZoneData> {
        if display_name.as_ref().is_some_and(String::is_empty) {
            return Err(InvalidTimeZoneData("requested time-zone name is empty"));
        }
        Ok(Self {
            offset_seconds,
            display_name,
        })
    }
    pub const fn offset_seconds(&self) -> i32 {
        self.offset_seconds
    }
    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }
    pub fn encode(&self) -> Vec<u8> {
        let name = self.display_name.as_deref().unwrap_or("").as_bytes();
        let mut bytes = Vec::with_capacity(RESOLVE_TIME_ZONE_RESULT_HEADER_BYTES + name.len());
        bytes.extend_from_slice(&i64::from(self.offset_seconds).to_le_bytes());
        bytes.extend_from_slice(name);
        bytes
    }
}

#[derive(Debug)]
pub enum TimeZoneResolveError {
    InvalidNamedIdentifier(TimeZoneId),
    UnsupportedNameLocale(CanonicalLocaleId),
    InvalidProviderData(InvalidTimeZoneData),
}

impl fmt::Display for TimeZoneResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNamedIdentifier(identifier) => write!(
                f,
                "unknown previously resolved named time zone {:?}",
                identifier.as_str()
            ),
            Self::UnsupportedNameLocale(locale) => {
                write!(f, "unsupported time-zone name locale {:?}", locale.as_str())
            }
            Self::InvalidProviderData(error) => write!(f, "invalid pinned time-zone data: {error}"),
        }
    }
}
impl std::error::Error for TimeZoneResolveError {}
impl From<InvalidTimeZoneData> for TimeZoneResolveError {
    fn from(error: InvalidTimeZoneData) -> Self {
        Self::InvalidProviderData(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(selection: TimeZoneSelection) -> ResolveTimeZoneRequest {
        ResolveTimeZoneRequest::new(
            selection,
            TimeZoneEpochSeconds::new(-1).unwrap(),
            Some(TimeZoneNameStyle::LongGeneric),
            CanonicalLocaleId::from_data("en-US").unwrap(),
        )
    }

    #[test]
    fn request_codec_roundtrips_named_and_extreme_fixed_offsets() {
        for selection in [
            TimeZoneSelection::Named(TimeZoneId::parse("Pacific/Apia").unwrap()),
            TimeZoneSelection::FixedOffset(FixedTimeZoneOffset::from_seconds(-86_340).unwrap()),
            TimeZoneSelection::FixedOffset(FixedTimeZoneOffset::from_seconds(86_340).unwrap()),
        ] {
            let value = request(selection);
            assert_eq!(
                ResolveTimeZoneRequest::decode(&value.encode()).unwrap(),
                value
            );
        }
    }

    #[test]
    fn request_codec_rejects_cross_kind_fields_and_corrupt_extents() {
        let valid = request(TimeZoneSelection::Named(
            TimeZoneId::parse("Europe/Paris").unwrap(),
        ))
        .encode();
        for (offset, value) in [
            (RESOLVE_TIME_ZONE_KIND_OFFSET, 3),
            (RESOLVE_TIME_ZONE_FIXED_SECONDS_OFFSET, 60),
            (RESOLVE_TIME_ZONE_NAME_STYLE_OFFSET, 7),
            (RESOLVE_TIME_ZONE_IDENTIFIER_LENGTH_OFFSET, -1),
            (RESOLVE_TIME_ZONE_LOCALE_LENGTH_OFFSET, i64::MAX),
            (
                RESOLVE_TIME_ZONE_EPOCH_SECONDS_OFFSET,
                TimeZoneEpochSeconds::LIMIT + 1,
            ),
        ] {
            let mut bytes = valid.clone();
            bytes[offset as usize..offset as usize + 8].copy_from_slice(&value.to_le_bytes());
            assert!(ResolveTimeZoneRequest::decode(&bytes).is_err(), "{offset}");
        }
        for length in 0..valid.len() {
            assert!(ResolveTimeZoneRequest::decode(&valid[..length]).is_err());
        }
        let mut trailing = valid;
        trailing.push(0);
        assert!(ResolveTimeZoneRequest::decode(&trailing).is_err());
    }

    #[test]
    fn exact_time_conversion_floors_negative_subseconds() {
        for (milliseconds, seconds) in [
            (-1001, -2),
            (-1000, -1),
            (-999, -1),
            (-1, -1),
            (0, 0),
            (1, 0),
            (999, 0),
            (1000, 1),
        ] {
            assert_eq!(
                TimeZoneEpochSeconds::from_milliseconds(milliseconds)
                    .unwrap()
                    .get(),
                seconds
            );
        }
        assert!(TimeZoneEpochSeconds::new(-TimeZoneEpochSeconds::LIMIT).is_ok());
        assert!(TimeZoneEpochSeconds::new(TimeZoneEpochSeconds::LIMIT).is_ok());
        assert!(FixedTimeZoneOffset::from_seconds(i64::MIN).is_err());
        assert!(FixedTimeZoneOffset::from_seconds(1).is_err());
    }

    #[test]
    fn lookup_result_keeps_alias_and_primary_identifier_separate() {
        let result = LookupNamedTimeZoneResult::new(
            NamedTimeZoneIdentity::from_data("Etc/UTC", "UTC").unwrap(),
        );
        let bytes = result.encode();
        assert_eq!(u64::from_le_bytes(bytes[..8].try_into().unwrap()), 7);
        assert_eq!(u64::from_le_bytes(bytes[8..16].try_into().unwrap()), 3);
        assert_eq!(&bytes[16..], b"Etc/UTCUTC");
        assert_eq!(result.identity().identifier(), "Etc/UTC");
        assert_eq!(result.identity().primary_identifier(), "UTC");
    }
}
