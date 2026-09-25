//! Temporal time-zone queries answered by the pinned time-zone kernel.
//!
//! A Temporal time zone is either a fixed UTC offset or an available named
//! IANA zone ([`TemporalTimeZone`]). Every Temporal operation that needs the
//! zone's rules — `GetOffsetNanosecondsFor`, `GetEpochNanosecondsFor`,
//! `InterpretISODateTimeOffset`'s offset matching, `GetStartOfDay` and the
//! named-zone transition searches — is one closed [`TemporalTimeZoneQuery`]
//! answered by the same kernel that owns `Intl.DateTimeFormat`'s zone data.
//!
//! All transitions in the pinned data happen at whole UTC seconds and every
//! offset is a whole number of seconds, so the queries exchange seconds. A
//! local date-time or an exact time is a whole-second count plus a
//! subsecond remainder in `[0, 10^9)`; only whether that remainder is zero can
//! change an answer (at the upper instant limit and for previous-transition
//! searches), and the queries carry exactly that.

use core::fmt;

use crate::{TimeZoneId, MAX_TIME_ZONE_IDENTIFIER_BYTES};

/// `nsMaxInstant` / 10^9: the last representable exact time in seconds.
pub const TEMPORAL_EPOCH_SECONDS_LIMIT: i64 = 8_640_000_000_000;
/// `CheckISODaysRange`: 10^8 days either side of the epoch.
pub const TEMPORAL_ISO_DAYS_LIMIT: i64 = 100_000_000;

/// The zone an operation runs in, after identifier parsing.
///
/// Offset zones are minute-aligned and strictly inside a day
/// (`FormatOffsetTimeZoneIdentifier`'s domain). Named zones are resolved
/// through the pinned catalogue; the variant only records the normalized
/// spelling the caller supplied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalTimeZone {
    Offset(TemporalOffsetMinutes),
    Named(TimeZoneId),
}

/// Offset time zone identifier minutes, `-1439..=1439`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemporalOffsetMinutes(i16);

impl TemporalOffsetMinutes {
    pub const MAX: i64 = 23 * 60 + 59;

    pub fn new(minutes: i64) -> Result<Self, InvalidTemporalTimeZoneRequest> {
        if minutes.unsigned_abs() > Self::MAX as u64 {
            return Err(InvalidTemporalTimeZoneRequest(
                "offset time zone is outside one day",
            ));
        }
        Ok(Self(minutes as i16))
    }

    pub const fn minutes(self) -> i64 {
        self.0 as i64
    }

    pub const fn seconds(self) -> i64 {
        self.0 as i64 * 60
    }
}

impl TemporalTimeZone {
    /// `ParseTimeZoneIdentifier` over an identifier the compiled program
    /// already produced with `ToTemporalTimeZoneIdentifier`: either the
    /// normalized `±HH:MM` offset spelling or a named identifier. Anything else
    /// is a malformed request, never a user-visible RangeError.
    pub fn parse_stored(identifier: &str) -> Result<Self, InvalidTemporalTimeZoneRequest> {
        let bytes = identifier.as_bytes();
        match bytes.first() {
            Some(b'+' | b'-') => {
                if bytes.len() != 6 || bytes[3] != b':' {
                    return Err(InvalidTemporalTimeZoneRequest(
                        "offset time zone is not in normalized form",
                    ));
                }
                let digit = |index: usize| -> Result<i64, InvalidTemporalTimeZoneRequest> {
                    match bytes[index] {
                        byte @ b'0'..=b'9' => Ok(i64::from(byte - b'0')),
                        _ => Err(InvalidTemporalTimeZoneRequest(
                            "offset time zone has a non-digit",
                        )),
                    }
                };
                let hours = digit(1)? * 10 + digit(2)?;
                let minutes = digit(4)? * 10 + digit(5)?;
                if hours > 23 || minutes > 59 {
                    return Err(InvalidTemporalTimeZoneRequest(
                        "offset time zone field is out of range",
                    ));
                }
                let magnitude = hours * 60 + minutes;
                let signed = if bytes[0] == b'-' {
                    -magnitude
                } else {
                    magnitude
                };
                Ok(Self::Offset(TemporalOffsetMinutes::new(signed)?))
            }
            _ => TimeZoneId::parse(identifier).map(Self::Named).map_err(|_| {
                InvalidTemporalTimeZoneRequest("named time zone identifier is malformed")
            }),
        }
    }
}

/// `disambiguation` option values, as carried on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TemporalDisambiguation {
    Compatible = 0,
    Earlier = 1,
    Later = 2,
    Reject = 3,
}

impl TemporalDisambiguation {
    pub const ALL: [Self; 4] = [Self::Compatible, Self::Earlier, Self::Later, Self::Reject];

    pub const fn wire(self) -> i64 {
        self as i64
    }

    pub const fn from_wire(wire: i64) -> Option<Self> {
        match wire {
            0 => Some(Self::Compatible),
            1 => Some(Self::Earlier),
            2 => Some(Self::Later),
            3 => Some(Self::Reject),
            _ => None,
        }
    }
}

/// The two `offset` option values under which `InterpretISODateTimeOffset`
/// compares a given offset with the zone's candidates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TemporalOffsetMismatch {
    /// `offset: "prefer"`: fall back to disambiguation.
    Prefer = 0,
    /// `offset: "reject"`: a RangeError.
    Reject = 1,
}

impl TemporalOffsetMismatch {
    pub const fn wire(self) -> i64 {
        self as i64
    }

    pub const fn from_wire(wire: i64) -> Option<Self> {
        match wire {
            0 => Some(Self::Prefer),
            1 => Some(Self::Reject),
            _ => None,
        }
    }
}

/// `InterpretISODateTimeOffset`'s matchBehaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TemporalOffsetMatch {
    Exactly = 0,
    Minutes = 1,
}

impl TemporalOffsetMatch {
    pub const fn wire(self) -> i64 {
        self as i64
    }

    pub const fn from_wire(wire: i64) -> Option<Self> {
        match wire {
            0 => Some(Self::Exactly),
            1 => Some(Self::Minutes),
            _ => None,
        }
    }
}

/// Search direction of `GetNamedTimeZoneNextTransition` /
/// `GetNamedTimeZonePreviousTransition`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalTransitionDirection {
    Next,
    Previous,
}

/// A whole-second count plus whether the subsecond remainder is non-zero.
///
/// The remainder itself never changes an answer: offsets are whole seconds,
/// so the kernel returns seconds and the caller re-attaches its remainder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemporalSeconds {
    seconds: i64,
    has_subsecond: bool,
}

impl TemporalSeconds {
    pub const fn new(seconds: i64, has_subsecond: bool) -> Self {
        Self {
            seconds,
            has_subsecond,
        }
    }

    pub const fn seconds(self) -> i64 {
        self.seconds
    }

    pub const fn has_subsecond(self) -> bool {
        self.has_subsecond
    }
}

/// One Temporal time-zone operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalTimeZoneQuery {
    /// `GetOffsetNanosecondsFor(timeZone, epochNs)`; answers
    /// [`TemporalTimeZoneAnswer::Seconds`] with the offset.
    OffsetAt { epoch: TemporalSeconds },
    /// `GetEpochNanosecondsFor(timeZone, isoDateTime, disambiguation)` of the
    /// local date-time `local`; answers the exact time's whole seconds.
    EpochFor {
        local: TemporalSeconds,
        disambiguation: TemporalDisambiguation,
    },
    /// `InterpretISODateTimeOffset` steps 7-14 for `offset: "prefer"` or
    /// `"reject"`, with `offsetNanoseconds` in `offset_nanoseconds`.
    EpochForOffset {
        local: TemporalSeconds,
        offset_nanoseconds: i64,
        mismatch: TemporalOffsetMismatch,
        matching: TemporalOffsetMatch,
        disambiguation: TemporalDisambiguation,
    },
    /// `GetStartOfDay(timeZone, isoDate)` for the local midnight `local`.
    StartOfDay { local_midnight: i64 },
    /// `GetNamedTimeZoneNextTransition` / `PreviousTransition` (always null
    /// for an offset zone), limited to the representable instants.
    Transition {
        epoch: TemporalSeconds,
        direction: TemporalTransitionDirection,
    },
}

/// Why a query is a RangeError for the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TemporalTimeZoneRangeError {
    /// A candidate exact time, or `CheckISODaysRange`, is outside the limits.
    OutOfRange = 1,
    /// `disambiguation: "reject"` met a skipped or repeated local time.
    Ambiguous = 2,
    /// `offset: "reject"` found no candidate with the given offset.
    OffsetMismatch = 3,
}

impl TemporalTimeZoneRangeError {
    pub const ALL: [Self; 3] = [Self::OutOfRange, Self::Ambiguous, Self::OffsetMismatch];

    pub const fn status(self) -> i64 {
        self as i64 + 1
    }

    pub const fn message(self) -> &'static str {
        match self {
            Self::OutOfRange => "Temporal time zone result is outside the supported range",
            Self::Ambiguous => "Temporal local time is ambiguous or skipped in its time zone",
            Self::OffsetMismatch => "Temporal offset does not match the time zone",
        }
    }
}

/// A query's answer, fixed at [`TEMPORAL_TIME_ZONE_RESPONSE_BYTES`] on the
/// wire: a status word and a value word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalTimeZoneAnswer {
    /// Status 0: an offset or an exact time, in seconds.
    Seconds(i64),
    /// Status 1: no transition in the searched direction.
    NoTransition,
    /// Status 2..: a RangeError, whose value word is zero.
    RangeError(TemporalTimeZoneRangeError),
}

pub const TEMPORAL_TIME_ZONE_STATUS_SECONDS: i64 = 0;
pub const TEMPORAL_TIME_ZONE_STATUS_NO_TRANSITION: i64 = 1;

impl TemporalTimeZoneAnswer {
    pub fn encode(self) -> [u8; TEMPORAL_TIME_ZONE_RESPONSE_BYTES] {
        let (status, value) = match self {
            Self::Seconds(seconds) => (TEMPORAL_TIME_ZONE_STATUS_SECONDS, seconds),
            Self::NoTransition => (TEMPORAL_TIME_ZONE_STATUS_NO_TRANSITION, 0),
            Self::RangeError(error) => (error.status(), 0),
        };
        let mut bytes = [0_u8; TEMPORAL_TIME_ZONE_RESPONSE_BYTES];
        bytes[..8].copy_from_slice(&status.to_le_bytes());
        bytes[8..].copy_from_slice(&value.to_le_bytes());
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, InvalidTemporalTimeZoneRequest> {
        if bytes.len() != TEMPORAL_TIME_ZONE_RESPONSE_BYTES {
            return Err(InvalidTemporalTimeZoneRequest(
                "time-zone answer has the wrong extent",
            ));
        }
        let word = |at: usize| i64::from_le_bytes(bytes[at..at + 8].try_into().expect("word"));
        let (status, value) = (word(0), word(8));
        match status {
            TEMPORAL_TIME_ZONE_STATUS_SECONDS => Ok(Self::Seconds(value)),
            TEMPORAL_TIME_ZONE_STATUS_NO_TRANSITION if value == 0 => Ok(Self::NoTransition),
            _ => TemporalTimeZoneRangeError::ALL
                .into_iter()
                .find(|error| error.status() == status && value == 0)
                .map(Self::RangeError)
                .ok_or(InvalidTemporalTimeZoneRequest(
                    "time-zone answer status is unknown",
                )),
        }
    }
}

/// Request wire: six little-endian words, then the identifier bytes.
pub const TEMPORAL_TIME_ZONE_REQUEST_HEADER_BYTES: usize = 48;
pub const TEMPORAL_TIME_ZONE_REQUEST_KIND_OFFSET: u64 = 0;
pub const TEMPORAL_TIME_ZONE_REQUEST_SECONDS_OFFSET: u64 = 8;
pub const TEMPORAL_TIME_ZONE_REQUEST_SUBSECOND_OFFSET: u64 = 16;
pub const TEMPORAL_TIME_ZONE_REQUEST_MODE_OFFSET: u64 = 24;
pub const TEMPORAL_TIME_ZONE_REQUEST_OFFSET_NANOSECONDS_OFFSET: u64 = 32;
pub const TEMPORAL_TIME_ZONE_REQUEST_MATCH_OFFSET: u64 = 40;
pub const TEMPORAL_TIME_ZONE_RESPONSE_BYTES: usize = 16;
pub const TEMPORAL_TIME_ZONE_RESPONSE_STATUS_OFFSET: u64 = 0;
pub const TEMPORAL_TIME_ZONE_RESPONSE_VALUE_OFFSET: u64 = 8;

/// Closed query kinds of the request's first word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TemporalTimeZoneQueryKind {
    OffsetAt = 1,
    EpochFor = 2,
    EpochForOffset = 3,
    StartOfDay = 4,
    NextTransition = 5,
    PreviousTransition = 6,
}

impl TemporalTimeZoneQueryKind {
    pub const fn wire(self) -> i64 {
        self as i64
    }

    pub const fn from_wire(wire: i64) -> Option<Self> {
        match wire {
            1 => Some(Self::OffsetAt),
            2 => Some(Self::EpochFor),
            3 => Some(Self::EpochForOffset),
            4 => Some(Self::StartOfDay),
            5 => Some(Self::NextTransition),
            6 => Some(Self::PreviousTransition),
            _ => None,
        }
    }
}

/// A query about one stored Temporal time zone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalTimeZoneRequest {
    zone: TemporalTimeZone,
    query: TemporalTimeZoneQuery,
}

impl TemporalTimeZoneRequest {
    pub const fn new(zone: TemporalTimeZone, query: TemporalTimeZoneQuery) -> Self {
        Self { zone, query }
    }

    pub const fn zone(&self) -> &TemporalTimeZone {
        &self.zone
    }

    pub const fn query(&self) -> TemporalTimeZoneQuery {
        self.query
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, InvalidTemporalTimeZoneRequest> {
        let header = bytes
            .get(..TEMPORAL_TIME_ZONE_REQUEST_HEADER_BYTES)
            .ok_or(InvalidTemporalTimeZoneRequest(
                "truncated time-zone query header",
            ))?;
        let word = |offset: u64| {
            let offset = offset as usize;
            i64::from_le_bytes(header[offset..offset + 8].try_into().expect("header word"))
        };
        let identifier = &bytes[TEMPORAL_TIME_ZONE_REQUEST_HEADER_BYTES..];
        if identifier.is_empty() || identifier.len() > MAX_TIME_ZONE_IDENTIFIER_BYTES {
            return Err(InvalidTemporalTimeZoneRequest(
                "time-zone identifier extent is invalid",
            ));
        }
        let identifier = core::str::from_utf8(identifier).map_err(|_| {
            InvalidTemporalTimeZoneRequest("time-zone identifier is not UTF-8")
        })?;
        let zone = TemporalTimeZone::parse_stored(identifier)?;
        let kind = TemporalTimeZoneQueryKind::from_wire(word(TEMPORAL_TIME_ZONE_REQUEST_KIND_OFFSET))
            .ok_or(InvalidTemporalTimeZoneRequest("unknown time-zone query kind"))?;
        let seconds = word(TEMPORAL_TIME_ZONE_REQUEST_SECONDS_OFFSET);
        let subsecond = word(TEMPORAL_TIME_ZONE_REQUEST_SUBSECOND_OFFSET);
        if !(0..1_000_000_000).contains(&subsecond) {
            return Err(InvalidTemporalTimeZoneRequest(
                "subsecond remainder is outside one second",
            ));
        }
        let at = TemporalSeconds::new(seconds, subsecond != 0);
        let mode = word(TEMPORAL_TIME_ZONE_REQUEST_MODE_OFFSET);
        let offset_nanoseconds = word(TEMPORAL_TIME_ZONE_REQUEST_OFFSET_NANOSECONDS_OFFSET);
        let matching = word(TEMPORAL_TIME_ZONE_REQUEST_MATCH_OFFSET);
        let disambiguation = || {
            TemporalDisambiguation::from_wire(mode)
                .ok_or(InvalidTemporalTimeZoneRequest("unknown disambiguation"))
        };
        let unused = |values: &[i64]| {
            if values.iter().all(|value| *value == 0) {
                Ok(())
            } else {
                Err(InvalidTemporalTimeZoneRequest(
                    "time-zone query carries an unused field",
                ))
            }
        };
        let query = match kind {
            TemporalTimeZoneQueryKind::OffsetAt => {
                unused(&[mode, offset_nanoseconds, matching])?;
                TemporalTimeZoneQuery::OffsetAt { epoch: at }
            }
            TemporalTimeZoneQueryKind::EpochFor => {
                unused(&[offset_nanoseconds, matching])?;
                TemporalTimeZoneQuery::EpochFor {
                    local: at,
                    disambiguation: disambiguation()?,
                }
            }
            TemporalTimeZoneQueryKind::EpochForOffset => {
                if offset_nanoseconds.unsigned_abs() >= 86_400_000_000_000 {
                    return Err(InvalidTemporalTimeZoneRequest(
                        "offset nanoseconds exceed one day",
                    ));
                }
                let mismatch = TemporalOffsetMismatch::from_wire(matching & 0xff)
                    .ok_or(InvalidTemporalTimeZoneRequest("unknown offset option"))?;
                let matching = TemporalOffsetMatch::from_wire(matching >> 8)
                    .ok_or(InvalidTemporalTimeZoneRequest("unknown offset matching"))?;
                TemporalTimeZoneQuery::EpochForOffset {
                    local: at,
                    offset_nanoseconds,
                    mismatch,
                    matching,
                    disambiguation: disambiguation()?,
                }
            }
            TemporalTimeZoneQueryKind::StartOfDay => {
                unused(&[subsecond, mode, offset_nanoseconds, matching])?;
                TemporalTimeZoneQuery::StartOfDay {
                    local_midnight: seconds,
                }
            }
            TemporalTimeZoneQueryKind::NextTransition
            | TemporalTimeZoneQueryKind::PreviousTransition => {
                unused(&[mode, offset_nanoseconds, matching])?;
                TemporalTimeZoneQuery::Transition {
                    epoch: at,
                    direction: if kind == TemporalTimeZoneQueryKind::NextTransition {
                        TemporalTransitionDirection::Next
                    } else {
                        TemporalTransitionDirection::Previous
                    },
                }
            }
        };
        Ok(Self::new(zone, query))
    }

    /// The request as the compiled program lays it out. The subsecond word
    /// only records whether a remainder exists.
    pub fn encode(&self) -> Vec<u8> {
        let identifier = match &self.zone {
            TemporalTimeZone::Named(identifier) => identifier.as_str().to_owned(),
            TemporalTimeZone::Offset(offset) => {
                let minutes = offset.minutes();
                let sign = if minutes < 0 { '-' } else { '+' };
                let magnitude = minutes.unsigned_abs();
                format!("{sign}{:02}:{:02}", magnitude / 60, magnitude % 60)
            }
        };
        let (kind, at, mode, offset_nanoseconds, matching) = match self.query {
            TemporalTimeZoneQuery::OffsetAt { epoch } => {
                (TemporalTimeZoneQueryKind::OffsetAt, epoch, 0, 0, 0)
            }
            TemporalTimeZoneQuery::EpochFor {
                local,
                disambiguation,
            } => (
                TemporalTimeZoneQueryKind::EpochFor,
                local,
                disambiguation.wire(),
                0,
                0,
            ),
            TemporalTimeZoneQuery::EpochForOffset {
                local,
                offset_nanoseconds,
                mismatch,
                matching,
                disambiguation,
            } => (
                TemporalTimeZoneQueryKind::EpochForOffset,
                local,
                disambiguation.wire(),
                offset_nanoseconds,
                mismatch.wire() | (matching.wire() << 8),
            ),
            TemporalTimeZoneQuery::StartOfDay { local_midnight } => (
                TemporalTimeZoneQueryKind::StartOfDay,
                TemporalSeconds::new(local_midnight, false),
                0,
                0,
                0,
            ),
            TemporalTimeZoneQuery::Transition { epoch, direction } => (
                match direction {
                    TemporalTransitionDirection::Next => TemporalTimeZoneQueryKind::NextTransition,
                    TemporalTransitionDirection::Previous => {
                        TemporalTimeZoneQueryKind::PreviousTransition
                    }
                },
                epoch,
                0,
                0,
                0,
            ),
        };
        let mut bytes = Vec::with_capacity(TEMPORAL_TIME_ZONE_REQUEST_HEADER_BYTES + identifier.len());
        for word in [
            kind.wire(),
            at.seconds(),
            i64::from(at.has_subsecond()),
            mode,
            offset_nanoseconds,
            matching,
        ] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes.extend_from_slice(identifier.as_bytes());
        bytes
    }
}

/// `offset` option and matchBehaviour packed into the request's match word:
/// the low byte is the [`TemporalOffsetMismatch`], the next the
/// [`TemporalOffsetMatch`].
pub const fn temporal_offset_match_word(
    mismatch: TemporalOffsetMismatch,
    matching: TemporalOffsetMatch,
) -> i64 {
    mismatch.wire() | (matching.wire() << 8)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTemporalTimeZoneRequest(pub(crate) &'static str);

impl fmt::Display for InvalidTemporalTimeZoneRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for InvalidTemporalTimeZoneRequest {}

/// Kernel failure for a Temporal time-zone query. Both variants are faults of
/// the compiled program or the pinned data, never JavaScript exceptions: a
/// named identifier reaches the kernel only after `ToTemporalTimeZoneIdentifier`
/// accepted it.
#[derive(Debug)]
pub enum TemporalTimeZoneError {
    UnknownNamedZone(TimeZoneId),
    InvalidRequest(InvalidTemporalTimeZoneRequest),
    InvalidProviderData(crate::InvalidTimeZoneData),
}

impl fmt::Display for TemporalTimeZoneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownNamedZone(identifier) => write!(
                f,
                "unknown previously resolved named time zone {:?}",
                identifier.as_str()
            ),
            Self::InvalidRequest(error) => write!(f, "invalid Temporal time-zone query: {error}"),
            Self::InvalidProviderData(error) => write!(f, "invalid pinned time-zone data: {error}"),
        }
    }
}
impl std::error::Error for TemporalTimeZoneError {}

impl From<crate::InvalidTimeZoneData> for TemporalTimeZoneError {
    fn from(error: crate::InvalidTimeZoneData) -> Self {
        Self::InvalidProviderData(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(identifier: &str, query: TemporalTimeZoneQuery) -> Vec<u8> {
        TemporalTimeZoneRequest::new(
            TemporalTimeZone::parse_stored(identifier).unwrap(),
            query,
        )
        .encode()
    }

    #[test]
    fn every_query_kind_roundtrips() {
        let at = TemporalSeconds::new(-5, true);
        for query in [
            TemporalTimeZoneQuery::OffsetAt { epoch: at },
            TemporalTimeZoneQuery::EpochFor {
                local: at,
                disambiguation: TemporalDisambiguation::Later,
            },
            TemporalTimeZoneQuery::EpochForOffset {
                local: at,
                offset_nanoseconds: -3_600_000_000_001,
                mismatch: TemporalOffsetMismatch::Reject,
                matching: TemporalOffsetMatch::Minutes,
                disambiguation: TemporalDisambiguation::Reject,
            },
            TemporalTimeZoneQuery::StartOfDay {
                local_midnight: 86_400,
            },
            TemporalTimeZoneQuery::Transition {
                epoch: at,
                direction: TemporalTransitionDirection::Previous,
            },
        ] {
            for identifier in ["America/New_York", "-05:30", "UTC"] {
                let bytes = request(identifier, query);
                let decoded = TemporalTimeZoneRequest::decode(&bytes).unwrap();
                assert_eq!(decoded.query(), query);
                assert_eq!(
                    decoded.zone(),
                    &TemporalTimeZone::parse_stored(identifier).unwrap()
                );
            }
        }
    }

    #[test]
    fn stored_identifiers_are_normalized_or_rejected() {
        assert_eq!(
            TemporalTimeZone::parse_stored("-05:30").unwrap(),
            TemporalTimeZone::Offset(TemporalOffsetMinutes::new(-330).unwrap())
        );
        assert_eq!(
            TemporalTimeZone::parse_stored("+23:59").unwrap(),
            TemporalTimeZone::Offset(TemporalOffsetMinutes::new(1439).unwrap())
        );
        for invalid in ["+24:00", "+0530", "-05:60", "+05:3x", "", "+05:30:00"] {
            assert!(TemporalTimeZone::parse_stored(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn malformed_requests_never_decode() {
        let valid = request(
            "Europe/Paris",
            TemporalTimeZoneQuery::OffsetAt {
                epoch: TemporalSeconds::new(0, false),
            },
        );
        for length in 0..TEMPORAL_TIME_ZONE_REQUEST_HEADER_BYTES + 1 {
            assert!(TemporalTimeZoneRequest::decode(&valid[..length]).is_err());
        }
        for (offset, value) in [
            (TEMPORAL_TIME_ZONE_REQUEST_KIND_OFFSET, 9_i64),
            (TEMPORAL_TIME_ZONE_REQUEST_SUBSECOND_OFFSET, 1_000_000_000),
            (TEMPORAL_TIME_ZONE_REQUEST_SUBSECOND_OFFSET, -1),
            (TEMPORAL_TIME_ZONE_REQUEST_MODE_OFFSET, 1),
            (TEMPORAL_TIME_ZONE_REQUEST_OFFSET_NANOSECONDS_OFFSET, 1),
        ] {
            let mut bytes = valid.clone();
            bytes[offset as usize..offset as usize + 8].copy_from_slice(&value.to_le_bytes());
            assert!(TemporalTimeZoneRequest::decode(&bytes).is_err(), "{offset}");
        }
    }

    #[test]
    fn answers_roundtrip_and_reject_unknown_status() {
        for answer in [
            TemporalTimeZoneAnswer::Seconds(-18_000),
            TemporalTimeZoneAnswer::NoTransition,
            TemporalTimeZoneAnswer::RangeError(TemporalTimeZoneRangeError::OutOfRange),
            TemporalTimeZoneAnswer::RangeError(TemporalTimeZoneRangeError::Ambiguous),
            TemporalTimeZoneAnswer::RangeError(TemporalTimeZoneRangeError::OffsetMismatch),
        ] {
            assert_eq!(
                TemporalTimeZoneAnswer::decode(&answer.encode()).unwrap(),
                answer
            );
        }
        let mut bytes = TemporalTimeZoneAnswer::NoTransition.encode();
        bytes[0] = 9;
        assert!(TemporalTimeZoneAnswer::decode(&bytes).is_err());
    }
}
