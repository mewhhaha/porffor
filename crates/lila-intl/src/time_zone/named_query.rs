//! Typed data-only named-zone requests and the exact host wire format.

use super::exact::{
    LocalTimeCoordinate, NamedTimeZoneOffsetSeconds, RawTimeZoneEpoch, TimeZoneInstant,
};
use super::{InvalidTimeZoneData, InvalidTimeZoneRequest};
use crate::TimeZoneId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamedTimeZoneDataError {
    UnavailableService(crate::IntlService),
    UnknownIdentifier(TimeZoneId),
    UnavailableIdentifier(TimeZoneId),
    InvalidProviderData(InvalidTimeZoneData),
}
impl core::fmt::Display for NamedTimeZoneDataError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnavailableService(service) => crate::UnavailableIntlService(*service).fmt(f),
            Self::UnknownIdentifier(identifier) => write!(
                f,
                "unknown previously resolved named time zone {:?}",
                identifier.as_str()
            ),
            Self::InvalidProviderData(error) => write!(f, "invalid pinned time-zone data: {error}"),
            Self::UnavailableIdentifier(identifier) => write!(
                f,
                "named time zone {:?} is unavailable in the selected Custom data",
                identifier.as_str()
            ),
        }
    }
}
impl std::error::Error for NamedTimeZoneDataError {}
impl From<InvalidTimeZoneData> for NamedTimeZoneDataError {
    fn from(error: InvalidTimeZoneData) -> Self {
        Self::InvalidProviderData(error)
    }
}

pub const NAMED_TIME_ZONE_REQUEST_HEADER_BYTES: usize = 32;
pub const NAMED_TIME_ZONE_SECONDS_OFFSET: usize = 0;
pub const NAMED_TIME_ZONE_NANOSECOND_OFFSET: usize = 8;
pub const NAMED_TIME_ZONE_QUERY_OFFSET: usize = 16;
pub const NAMED_TIME_ZONE_IDENTIFIER_LENGTH_OFFSET: usize = 24;
pub const POSSIBLE_TIME_ZONE_RESULT_HEADER_BYTES: usize = 16;
pub const POSSIBLE_TIME_ZONE_CANDIDATE_BYTES: usize = 24;
pub const POSSIBLE_TIME_ZONE_GAP_BYTES: usize = 40;
pub const TIME_ZONE_TRANSITION_RESULT_BYTES: usize = 16;
// Every offset is an integer strictly between -86400 and +86400. A distinct
// offset can produce at most one candidate, so even the complete type domain
// fits the existing u32 packed-span result ABI without a two-candidate cap.
const DISTINCT_NAMED_OFFSET_DOMAIN_SIZE: usize = 2 * 86_400 - 1;
const MAX_POSSIBLE_RESULT_BYTES: usize = POSSIBLE_TIME_ZONE_RESULT_HEADER_BYTES
    + DISTINCT_NAMED_OFFSET_DOMAIN_SIZE * POSSIBLE_TIME_ZONE_CANDIDATE_BYTES;
const _: () = assert!(MAX_POSSIBLE_RESULT_BYTES <= u32::MAX as usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedTimeZoneOffsetRequest {
    identifier: TimeZoneId,
    instant: TimeZoneInstant,
}
impl NamedTimeZoneOffsetRequest {
    pub fn new(
        identifier: TimeZoneId,
        instant: TimeZoneInstant,
    ) -> Result<Self, InvalidTimeZoneRequest> {
        named(&identifier)?;
        Ok(Self {
            identifier,
            instant,
        })
    }
    pub fn identifier(&self) -> &TimeZoneId {
        &self.identifier
    }
    pub const fn instant(&self) -> TimeZoneInstant {
        self.instant
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, InvalidTimeZoneRequest> {
        let (identifier, seconds, nanos, query) = decode_request(bytes)?;
        if query != 0 {
            return Err(InvalidTimeZoneRequest(
                "offset request carries a query kind",
            ));
        }
        Self::new(identifier, TimeZoneInstant::new(seconds, nanos)?)
    }
    pub fn encode(&self) -> Vec<u8> {
        encode_request(
            &self.identifier,
            self.instant.seconds(),
            self.instant.nanosecond(),
            0,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PossibleNamedTimeZoneEpochsRequest {
    identifier: TimeZoneId,
    local: LocalTimeCoordinate,
}
impl PossibleNamedTimeZoneEpochsRequest {
    pub fn new(
        identifier: TimeZoneId,
        local: LocalTimeCoordinate,
    ) -> Result<Self, InvalidTimeZoneRequest> {
        named(&identifier)?;
        Ok(Self { identifier, local })
    }
    pub fn identifier(&self) -> &TimeZoneId {
        &self.identifier
    }
    pub const fn local(&self) -> LocalTimeCoordinate {
        self.local
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, InvalidTimeZoneRequest> {
        let (identifier, seconds, nanos, query) = decode_request(bytes)?;
        if query != 0 {
            return Err(InvalidTimeZoneRequest(
                "inverse request carries a query kind",
            ));
        }
        Self::new(identifier, LocalTimeCoordinate::new(seconds, nanos)?)
    }
    pub fn encode(&self) -> Vec<u8> {
        encode_request(
            &self.identifier,
            self.local.seconds(),
            self.local.nanosecond(),
            0,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NamedTimeZoneTransitionDirection {
    Next = 1,
    Previous = 2,
}
impl NamedTimeZoneTransitionDirection {
    fn decode(word: i64) -> Result<Self, InvalidTimeZoneRequest> {
        match word {
            1 => Ok(Self::Next),
            2 => Ok(Self::Previous),
            _ => Err(InvalidTimeZoneRequest("invalid named transition direction")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindNamedTimeZoneTransitionRequest {
    identifier: TimeZoneId,
    instant: TimeZoneInstant,
    direction: NamedTimeZoneTransitionDirection,
}
impl FindNamedTimeZoneTransitionRequest {
    pub fn new(
        identifier: TimeZoneId,
        instant: TimeZoneInstant,
        direction: NamedTimeZoneTransitionDirection,
    ) -> Result<Self, InvalidTimeZoneRequest> {
        named(&identifier)?;
        Ok(Self {
            identifier,
            instant,
            direction,
        })
    }
    pub fn identifier(&self) -> &TimeZoneId {
        &self.identifier
    }
    pub const fn instant(&self) -> TimeZoneInstant {
        self.instant
    }
    pub const fn direction(&self) -> NamedTimeZoneTransitionDirection {
        self.direction
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, InvalidTimeZoneRequest> {
        let (identifier, seconds, nanos, query) = decode_request(bytes)?;
        Self::new(
            identifier,
            TimeZoneInstant::new(seconds, nanos)?,
            NamedTimeZoneTransitionDirection::decode(query)?,
        )
    }
    pub fn encode(&self) -> Vec<u8> {
        encode_request(
            &self.identifier,
            self.instant.seconds(),
            self.instant.nanosecond(),
            self.direction as i64,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NamedTimeZoneCandidate {
    epoch: RawTimeZoneEpoch,
    offset: NamedTimeZoneOffsetSeconds,
}
impl NamedTimeZoneCandidate {
    pub(crate) fn from_forward_match(
        local: LocalTimeCoordinate,
        epoch: RawTimeZoneEpoch,
        tested: NamedTimeZoneOffsetSeconds,
        actual: NamedTimeZoneOffsetSeconds,
    ) -> Result<Self, InvalidTimeZoneData> {
        if tested != actual
            || epoch.nanoseconds() + i128::from(actual.seconds()) * 1_000_000_000
                != local.nanoseconds()
        {
            return Err(InvalidTimeZoneData(
                "inverse candidate failed exact forward round trip",
            ));
        }
        Ok(Self {
            epoch,
            offset: actual,
        })
    }
    pub const fn epoch(self) -> RawTimeZoneEpoch {
        self.epoch
    }
    pub const fn offset(self) -> NamedTimeZoneOffsetSeconds {
        self.offset
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortedNamedTimeZoneCandidates(Vec<NamedTimeZoneCandidate>);
impl SortedNamedTimeZoneCandidates {
    /// The constructor and emitted response decoder share this exact bound.
    pub const fn maximum_count() -> usize {
        DISTINCT_NAMED_OFFSET_DOMAIN_SIZE
    }
    pub(crate) fn from_sorted(
        values: Vec<NamedTimeZoneCandidate>,
        distinct_offset_count: usize,
    ) -> Result<Self, InvalidTimeZoneData> {
        if values.is_empty()
            || distinct_offset_count > DISTINCT_NAMED_OFFSET_DOMAIN_SIZE
            || values.len() > distinct_offset_count
            || values.windows(2).any(|pair| pair[0].epoch >= pair[1].epoch)
        {
            return Err(InvalidTimeZoneData(
                "invalid sorted named-zone candidate catalogue",
            ));
        }
        Ok(Self(values))
    }
    pub fn as_slice(&self) -> &[NamedTimeZoneCandidate] {
        &self.0
    }
}

/// A containing forward increase whose immutable catalogue has certified
/// global gap emptiness and sole nearest local endpoints. The provider carries
/// data proof only; compiled Wasm still owns disambiguation and range policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NamedTimeZoneGap {
    transition_seconds: i64,
    before: NamedTimeZoneOffsetSeconds,
    after: NamedTimeZoneOffsetSeconds,
}
impl NamedTimeZoneGap {
    pub(crate) fn from_certified_boundary(
        boundary: crate::provider::CertifiedNamedGapBoundary,
    ) -> Self {
        Self {
            transition_seconds: boundary.transition_seconds(),
            before: boundary.before(),
            after: boundary.after(),
        }
    }
    pub const fn transition_seconds(self) -> i64 {
        self.transition_seconds
    }
    pub const fn before(self) -> NamedTimeZoneOffsetSeconds {
        self.before
    }
    pub const fn after(self) -> NamedTimeZoneOffsetSeconds {
        self.after
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PossibleNamedTimeZoneEpochsResult {
    Candidates(SortedNamedTimeZoneCandidates),
    Gap(NamedTimeZoneGap),
}
impl PossibleNamedTimeZoneEpochsResult {
    pub fn encode(&self) -> Vec<u8> {
        let capacity = match self {
            Self::Candidates(candidates) => {
                POSSIBLE_TIME_ZONE_RESULT_HEADER_BYTES
                    + candidates.0.len() * POSSIBLE_TIME_ZONE_CANDIDATE_BYTES
            }
            Self::Gap(_) => POSSIBLE_TIME_ZONE_GAP_BYTES,
        };
        let mut bytes = Vec::with_capacity(capacity);
        match self {
            Self::Candidates(candidates) => {
                words(&mut bytes, &[1, candidates.0.len() as i64]);
                for candidate in &candidates.0 {
                    words(
                        &mut bytes,
                        &[
                            candidate.epoch.seconds(),
                            i64::from(candidate.epoch.nanosecond()),
                            i64::from(candidate.offset.seconds()),
                        ],
                    );
                }
            }
            Self::Gap(gap) => words(
                &mut bytes,
                &[
                    0,
                    0,
                    gap.transition_seconds,
                    i64::from(gap.before.seconds()),
                    i64::from(gap.after.seconds()),
                ],
            ),
        }
        debug_assert_eq!(bytes.len(), capacity);
        bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindNamedTimeZoneTransitionResult(Option<TimeZoneInstant>);
impl FindNamedTimeZoneTransitionResult {
    pub(crate) fn from_seconds(seconds: Option<i64>) -> Result<Self, InvalidTimeZoneData> {
        let instant = seconds
            .map(|seconds| {
                TimeZoneInstant::new(seconds, 0)
                    .map_err(|_| InvalidTimeZoneData("transition exceeds Instant domain"))
            })
            .transpose()?;
        Ok(Self(instant))
    }
    pub const fn instant(self) -> Option<TimeZoneInstant> {
        self.0
    }
    pub fn encode(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(TIME_ZONE_TRANSITION_RESULT_BYTES);
        match self.0 {
            Some(instant) => words(&mut bytes, &[1, instant.seconds()]),
            None => words(&mut bytes, &[0, 0]),
        }
        bytes
    }
}

fn named(identifier: &TimeZoneId) -> Result<(), InvalidTimeZoneRequest> {
    if identifier.as_str().starts_with(['+', '-']) {
        return Err(InvalidTimeZoneRequest("named query carries a fixed offset"));
    }
    Ok(())
}
fn decode_request(bytes: &[u8]) -> Result<(TimeZoneId, i64, u32, i64), InvalidTimeZoneRequest> {
    if bytes.len() < NAMED_TIME_ZONE_REQUEST_HEADER_BYTES {
        return Err(InvalidTimeZoneRequest("truncated named query header"));
    }
    let word = |offset: usize| {
        i64::from_le_bytes(
            bytes[offset..offset + 8]
                .try_into()
                .expect("validated header"),
        )
    };
    let nanos = u32::try_from(word(NAMED_TIME_ZONE_NANOSECOND_OFFSET))
        .map_err(|_| InvalidTimeZoneRequest("invalid nanosecond word"))?;
    let length = usize::try_from(word(NAMED_TIME_ZONE_IDENTIFIER_LENGTH_OFFSET))
        .map_err(|_| InvalidTimeZoneRequest("invalid identifier extent"))?;
    if NAMED_TIME_ZONE_REQUEST_HEADER_BYTES.checked_add(length) != Some(bytes.len()) {
        return Err(InvalidTimeZoneRequest("named query extent mismatch"));
    }
    let text = core::str::from_utf8(&bytes[NAMED_TIME_ZONE_REQUEST_HEADER_BYTES..])
        .map_err(|_| InvalidTimeZoneRequest("named query identifier is not UTF-8"))?;
    let identifier = TimeZoneId::parse(text)
        .map_err(|_| InvalidTimeZoneRequest("invalid named query identifier"))?;
    named(&identifier)?;
    Ok((
        identifier,
        word(NAMED_TIME_ZONE_SECONDS_OFFSET),
        nanos,
        word(NAMED_TIME_ZONE_QUERY_OFFSET),
    ))
}
fn encode_request(identifier: &TimeZoneId, seconds: i64, nanos: u32, query: i64) -> Vec<u8> {
    let mut bytes =
        Vec::with_capacity(NAMED_TIME_ZONE_REQUEST_HEADER_BYTES + identifier.as_str().len());
    words(
        &mut bytes,
        &[
            seconds,
            i64::from(nanos),
            query,
            identifier.as_str().len() as i64,
        ],
    );
    bytes.extend_from_slice(identifier.as_str().as_bytes());
    bytes
}
fn words(bytes: &mut Vec<u8>, values: &[i64]) {
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn request_codecs_distinguish_local_context_instant_and_direction() {
        let id = TimeZoneId::parse("US/Eastern").unwrap();
        let offset = NamedTimeZoneOffsetRequest::new(
            id.clone(),
            TimeZoneInstant::from_nanoseconds(-1).unwrap(),
        )
        .unwrap();
        assert_eq!(
            NamedTimeZoneOffsetRequest::decode(&offset.encode()).unwrap(),
            offset
        );
        let inverse = PossibleNamedTimeZoneEpochsRequest::new(
            id.clone(),
            LocalTimeCoordinate::new(TimeZoneInstant::limit_seconds() + 1, 0).unwrap(),
        )
        .unwrap();
        assert_eq!(
            PossibleNamedTimeZoneEpochsRequest::decode(&inverse.encode()).unwrap(),
            inverse
        );
        assert!(NamedTimeZoneOffsetRequest::decode(&inverse.encode()).is_err());
        let transition = FindNamedTimeZoneTransitionRequest::new(
            id,
            offset.instant(),
            NamedTimeZoneTransitionDirection::Previous,
        )
        .unwrap();
        assert_eq!(
            FindNamedTimeZoneTransitionRequest::decode(&transition.encode()).unwrap(),
            transition
        );
        assert!(PossibleNamedTimeZoneEpochsRequest::decode(&transition.encode()).is_err());
        for (offset, invalid) in [(8, 1_000_000_000_i64), (16, 3), (24, -1), (24, i64::MAX)] {
            let mut bytes = transition.encode();
            bytes[offset..offset + 8].copy_from_slice(&invalid.to_le_bytes());
            assert!(FindNamedTimeZoneTransitionRequest::decode(&bytes).is_err());
        }
        for len in 0..transition.encode().len() {
            assert!(
                FindNamedTimeZoneTransitionRequest::decode(&transition.encode()[..len]).is_err()
            );
        }
        assert!(NamedTimeZoneOffsetRequest::new(
            TimeZoneId::parse("+01:00").unwrap(),
            offset.instant()
        )
        .is_err());
    }
}
