//! `Intl.Locale` information queries and `Intl.supportedValuesOf` keys.
//!
//! The Wasm shell performs RequireInternalSlot and ToString; the typed host
//! operations receive only a canonical locale tag and a closed query or key.
//! Responses are primitive identifier lists, a text direction or a week record;
//! object and Array construction stays in the emitted program.

use core::fmt;

use crate::{CanonicalLocaleId, IntlHostOp};

macro_rules! wire_domain {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $($variant:ident => $code:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            #[must_use]
            pub const fn wire_code(self) -> u64 {
                match self {
                    $(Self::$variant => $code),+
                }
            }

            #[must_use]
            pub const fn from_wire_code(code: u64) -> Option<Self> {
                match code {
                    $($code => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

wire_domain! {
    /// One `Intl.Locale.prototype` information method (ECMA-402 15.3.16-15.3.22).
    pub enum LocaleInfoQuery {
        Calendars => 0,
        Collations => 1,
        HourCycles => 2,
        NumberingSystems => 3,
        TimeZones => 4,
        TextDirection => 5,
        WeekInfo => 6,
    }
}

impl LocaleInfoQuery {
    /// The only response shape a well-behaved provider may return.
    #[must_use]
    pub const fn shape(self) -> LocaleInfoShape {
        match self {
            Self::Calendars | Self::Collations | Self::HourCycles | Self::NumberingSystems => {
                LocaleInfoShape::Identifiers
            }
            Self::TimeZones => LocaleInfoShape::OptionalIdentifiers,
            Self::TextDirection => LocaleInfoShape::Direction,
            Self::WeekInfo => LocaleInfoShape::Week,
        }
    }
}

/// Response shape selected statically by each Locale method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocaleInfoShape {
    Identifiers,
    OptionalIdentifiers,
    Direction,
    Week,
}

wire_domain! {
    /// The `key` argument of `Intl.supportedValuesOf` (ECMA-402 8.3.2).
    pub enum SupportedValuesKey {
        Calendar => 0,
        Collation => 1,
        Currency => 2,
        NumberingSystem => 3,
        TimeZone => 4,
        Unit => 5,
    }
}

impl SupportedValuesKey {
    /// The exact JavaScript spelling of the key.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Calendar => "calendar",
            Self::Collation => "collation",
            Self::Currency => "currency",
            Self::NumberingSystem => "numberingSystem",
            Self::TimeZone => "timeZone",
            Self::Unit => "unit",
        }
    }
}

wire_domain! {
    /// TextDirectionOfLocale's two defined results.
    pub enum TextDirection {
        LeftToRight => 1,
        RightToLeft => 2,
    }
}

impl TextDirection {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::LeftToRight => "ltr",
            Self::RightToLeft => "rtl",
        }
    }
}

wire_domain! {
    /// ISO 8601 day of week: Monday is 1 and Sunday is 7 (ECMA-402 Table 27).
    pub enum IsoWeekday {
        Monday => 1,
        Tuesday => 2,
        Wednesday => 3,
        Thursday => 4,
        Friday => 5,
        Saturday => 6,
        Sunday => 7,
    }
}

impl IsoWeekday {
    /// WeekdayUValueToNumber (ECMA-402 15.5.16) for a Unicode `fw` value.
    #[must_use]
    pub fn from_first_day_value(value: &str) -> Option<Self> {
        Some(match value {
            "mon" => Self::Monday,
            "tue" => Self::Tuesday,
            "wed" => Self::Wednesday,
            "thu" => Self::Thursday,
            "fri" => Self::Friday,
            "sat" => Self::Saturday,
            "sun" => Self::Sunday,
            _ => return None,
        })
    }
}

/// ECMA-402 Table 27. `weekend` is nonempty and strictly ascending.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeekInfo {
    first_day: IsoWeekday,
    weekend: Box<[IsoWeekday]>,
}

impl WeekInfo {
    pub fn new(
        first_day: IsoWeekday,
        weekend: Box<[IsoWeekday]>,
    ) -> Result<Self, LocaleInfoWireError> {
        let ascending = weekend
            .windows(2)
            .all(|pair| pair[0].wire_code() < pair[1].wire_code());
        if weekend.is_empty() || !ascending {
            return Err(LocaleInfoWireError::Malformed(
                "weekend days are empty or unordered",
            ));
        }
        Ok(Self { first_day, weekend })
    }

    #[must_use]
    pub const fn first_day(&self) -> IsoWeekday {
        self.first_day
    }

    #[must_use]
    pub fn weekend(&self) -> &[IsoWeekday] {
        &self.weekend
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleInfoRequest {
    query: LocaleInfoQuery,
    locale: CanonicalLocaleId,
}

impl LocaleInfoRequest {
    #[must_use]
    pub const fn new(query: LocaleInfoQuery, locale: CanonicalLocaleId) -> Self {
        Self { query, locale }
    }

    #[must_use]
    pub const fn query(&self) -> LocaleInfoQuery {
        self.query
    }

    #[must_use]
    pub const fn locale(&self) -> &CanonicalLocaleId {
        &self.locale
    }
}

/// Result of one Locale information query. The variant is fixed by
/// [`LocaleInfoQuery::shape`]; encoding rejects any mismatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocaleInfoResult {
    /// getTimeZones on a locale without a region subtag, or a direction that
    /// cannot be determined: the JavaScript value `undefined`.
    Undefined,
    Identifiers(Box<[Box<str>]>),
    Direction(TextDirection),
    Week(WeekInfo),
}

impl LocaleInfoResult {
    const fn matches(&self, shape: LocaleInfoShape) -> bool {
        matches!(
            (self, shape),
            (Self::Identifiers(_), LocaleInfoShape::Identifiers)
                | (
                    Self::Undefined | Self::Identifiers(_),
                    LocaleInfoShape::OptionalIdentifiers
                )
                | (
                    Self::Undefined | Self::Direction(_),
                    LocaleInfoShape::Direction
                )
                | (Self::Week(_), LocaleInfoShape::Week)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupportedValuesRequest {
    key: SupportedValuesKey,
}

impl SupportedValuesRequest {
    #[must_use]
    pub const fn new(key: SupportedValuesKey) -> Self {
        Self { key }
    }

    #[must_use]
    pub const fn key(&self) -> SupportedValuesKey {
        self.key
    }
}

/// A sorted, duplicate-free identifier list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportedValuesResult {
    values: Box<[Box<str>]>,
}

impl SupportedValuesResult {
    pub fn new(values: Box<[Box<str>]>) -> Result<Self, LocaleInfoWireError> {
        if !values.windows(2).all(|pair| pair[0] < pair[1]) {
            return Err(LocaleInfoWireError::Malformed(
                "supported values are not strictly sorted",
            ));
        }
        Ok(Self { values })
    }

    #[must_use]
    pub fn values(&self) -> &[Box<str>] {
        &self.values
    }
}

/// Pinned-data failure while answering a query. These are provider defects,
/// never JavaScript-observable errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocaleInfoError {
    InvalidLocale(&'static str),
    InvalidData(&'static str),
}

impl fmt::Display for LocaleInfoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLocale(reason) => write!(f, "invalid Locale information input: {reason}"),
            Self::InvalidData(reason) => {
                write!(f, "invalid pinned Locale information data: {reason}")
            }
        }
    }
}

impl std::error::Error for LocaleInfoError {}

pub const LOCALE_INFO_WIRE_VERSION: u64 = 1;
pub const LOCALE_INFO_WIRE_HEADER_BYTES: u64 = 16;

wire_domain! {
    /// Leading word of a Locale information response.
    pub enum LocaleInfoResponseKind {
        Undefined => 0,
        Identifiers => 1,
        Direction => 2,
        Week => 3,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocaleInfoWireError {
    Malformed(&'static str),
    Resource(&'static str),
}

impl fmt::Display for LocaleInfoWireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(reason) => write!(f, "invalid Locale information message: {reason}"),
            Self::Resource(reason) => write!(f, "Locale information resource limit: {reason}"),
        }
    }
}

impl std::error::Error for LocaleInfoWireError {}

#[derive(Clone, Copy)]
enum Direction {
    Request,
    Response,
}

fn message_code(operation: IntlHostOp, direction: Direction) -> u64 {
    u64::from(operation.code()) * 2
        + match direction {
            Direction::Request => 0,
            Direction::Response => 1,
        }
}

struct Writer(Vec<u8>);

impl Writer {
    fn new(operation: IntlHostOp, direction: Direction) -> Result<Self, LocaleInfoWireError> {
        let mut writer = Self(Vec::new());
        writer.word(LOCALE_INFO_WIRE_VERSION)?;
        writer.word(message_code(operation, direction))?;
        Ok(writer)
    }

    fn append(&mut self, bytes: &[u8]) -> Result<(), LocaleInfoWireError> {
        let length = self
            .0
            .len()
            .checked_add(bytes.len())
            .ok_or(LocaleInfoWireError::Resource("message length overflow"))?;
        u32::try_from(length)
            .map_err(|_| LocaleInfoWireError::Resource("message exceeds Wasm32 span"))?;
        self.0
            .try_reserve(bytes.len())
            .map_err(|_| LocaleInfoWireError::Resource("message allocation"))?;
        self.0.extend_from_slice(bytes);
        Ok(())
    }

    fn word(&mut self, value: u64) -> Result<(), LocaleInfoWireError> {
        self.append(&value.to_le_bytes())
    }

    fn text(&mut self, text: &str) -> Result<(), LocaleInfoWireError> {
        self.word(text.len() as u64)?;
        self.append(text.as_bytes())
    }

    fn identifiers(&mut self, values: &[Box<str>]) -> Result<(), LocaleInfoWireError> {
        self.word(values.len() as u64)?;
        for value in values {
            self.text(value)?;
        }
        Ok(())
    }
}

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn new(
        bytes: &'a [u8],
        operation: IntlHostOp,
        direction: Direction,
    ) -> Result<Self, LocaleInfoWireError> {
        u32::try_from(bytes.len())
            .map_err(|_| LocaleInfoWireError::Resource("message exceeds Wasm32 span"))?;
        let mut reader = Self(bytes);
        if reader.word()? != LOCALE_INFO_WIRE_VERSION
            || reader.word()? != message_code(operation, direction)
        {
            return Err(LocaleInfoWireError::Malformed("version or operation"));
        }
        Ok(reader)
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], LocaleInfoWireError> {
        let (prefix, remaining) = self
            .0
            .split_at_checked(count)
            .ok_or(LocaleInfoWireError::Malformed("truncated field"))?;
        self.0 = remaining;
        Ok(prefix)
    }

    fn word(&mut self) -> Result<u64, LocaleInfoWireError> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("eight-byte word"),
        ))
    }

    fn domain<T>(&mut self, decode: fn(u64) -> Option<T>) -> Result<T, LocaleInfoWireError> {
        decode(self.word()?).ok_or(LocaleInfoWireError::Malformed("unknown closed-domain code"))
    }

    fn count(&mut self, minimum_record_bytes: usize) -> Result<usize, LocaleInfoWireError> {
        let count = u32::try_from(self.word()?)
            .map_err(|_| LocaleInfoWireError::Malformed("count exceeds Wasm32"))?
            as usize;
        if count > self.0.len() / minimum_record_bytes {
            return Err(LocaleInfoWireError::Malformed(
                "count exceeds owned message",
            ));
        }
        Ok(count)
    }

    fn text(&mut self) -> Result<&'a str, LocaleInfoWireError> {
        let length = u32::try_from(self.word()?)
            .map_err(|_| LocaleInfoWireError::Malformed("field exceeds Wasm32"))?
            as usize;
        core::str::from_utf8(self.take(length)?)
            .map_err(|_| LocaleInfoWireError::Malformed("invalid UTF8 text"))
    }

    fn identifiers(&mut self) -> Result<Box<[Box<str>]>, LocaleInfoWireError> {
        let count = self.count(8)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| LocaleInfoWireError::Resource("identifier list allocation"))?;
        for _ in 0..count {
            values.push(Box::from(self.text()?));
        }
        Ok(values.into_boxed_slice())
    }

    fn finish(self) -> Result<(), LocaleInfoWireError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(LocaleInfoWireError::Malformed("trailing bytes"))
        }
    }
}

impl LocaleInfoRequest {
    pub fn encode(&self) -> Result<Vec<u8>, LocaleInfoWireError> {
        let mut writer = Writer::new(IntlHostOp::LocaleInfo, Direction::Request)?;
        writer.word(self.query.wire_code())?;
        writer.text(self.locale.as_str())?;
        Ok(writer.0)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, LocaleInfoWireError> {
        let mut reader = Reader::new(bytes, IntlHostOp::LocaleInfo, Direction::Request)?;
        let query = reader.domain(LocaleInfoQuery::from_wire_code)?;
        let locale = CanonicalLocaleId::from_data(reader.text()?)
            .map_err(|_| LocaleInfoWireError::Malformed("non-canonical locale"))?;
        reader.finish()?;
        Ok(Self { query, locale })
    }
}

impl LocaleInfoResult {
    /// Encodes the result of `query`, rejecting a shape the query cannot have.
    pub fn encode(&self, query: LocaleInfoQuery) -> Result<Vec<u8>, LocaleInfoWireError> {
        if !self.matches(query.shape()) {
            return Err(LocaleInfoWireError::Malformed(
                "result shape does not match the query",
            ));
        }
        let mut writer = Writer::new(IntlHostOp::LocaleInfo, Direction::Response)?;
        match self {
            Self::Undefined => writer.word(LocaleInfoResponseKind::Undefined.wire_code())?,
            Self::Identifiers(values) => {
                writer.word(LocaleInfoResponseKind::Identifiers.wire_code())?;
                writer.identifiers(values)?;
            }
            Self::Direction(direction) => {
                writer.word(LocaleInfoResponseKind::Direction.wire_code())?;
                writer.word(direction.wire_code())?;
            }
            Self::Week(week) => {
                writer.word(LocaleInfoResponseKind::Week.wire_code())?;
                writer.word(week.first_day.wire_code())?;
                writer.word(week.weekend.len() as u64)?;
                for day in week.weekend.iter() {
                    writer.word(day.wire_code())?;
                }
            }
        }
        Ok(writer.0)
    }

    pub fn decode(bytes: &[u8], query: LocaleInfoQuery) -> Result<Self, LocaleInfoWireError> {
        let mut reader = Reader::new(bytes, IntlHostOp::LocaleInfo, Direction::Response)?;
        let result = match reader.domain(LocaleInfoResponseKind::from_wire_code)? {
            LocaleInfoResponseKind::Undefined => Self::Undefined,
            LocaleInfoResponseKind::Identifiers => Self::Identifiers(reader.identifiers()?),
            LocaleInfoResponseKind::Direction => {
                Self::Direction(reader.domain(TextDirection::from_wire_code)?)
            }
            LocaleInfoResponseKind::Week => {
                let first_day = reader.domain(IsoWeekday::from_wire_code)?;
                let count = reader.count(8)?;
                let mut weekend = Vec::with_capacity(count);
                for _ in 0..count {
                    weekend.push(reader.domain(IsoWeekday::from_wire_code)?);
                }
                Self::Week(WeekInfo::new(first_day, weekend.into_boxed_slice())?)
            }
        };
        reader.finish()?;
        if !result.matches(query.shape()) {
            return Err(LocaleInfoWireError::Malformed(
                "result shape does not match the query",
            ));
        }
        Ok(result)
    }
}

impl SupportedValuesRequest {
    pub fn encode(&self) -> Result<Vec<u8>, LocaleInfoWireError> {
        let mut writer = Writer::new(IntlHostOp::SupportedValues, Direction::Request)?;
        writer.word(self.key.wire_code())?;
        Ok(writer.0)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, LocaleInfoWireError> {
        let mut reader = Reader::new(bytes, IntlHostOp::SupportedValues, Direction::Request)?;
        let key = reader.domain(SupportedValuesKey::from_wire_code)?;
        reader.finish()?;
        Ok(Self { key })
    }
}

impl SupportedValuesResult {
    pub fn encode(&self) -> Result<Vec<u8>, LocaleInfoWireError> {
        let mut writer = Writer::new(IntlHostOp::SupportedValues, Direction::Response)?;
        writer.identifiers(&self.values)?;
        Ok(writer.0)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, LocaleInfoWireError> {
        let mut reader = Reader::new(bytes, IntlHostOp::SupportedValues, Direction::Response)?;
        let values = reader.identifiers()?;
        reader.finish()?;
        Self::new(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locale(tag: &str) -> CanonicalLocaleId {
        CanonicalLocaleId::from_data(tag).unwrap()
    }

    #[test]
    fn requests_and_results_round_trip_through_the_wire() {
        for query in LocaleInfoQuery::ALL {
            let request = LocaleInfoRequest::new(*query, locale("en-US-u-ca-gregory"));
            assert_eq!(
                LocaleInfoRequest::decode(&request.encode().unwrap()).unwrap(),
                request
            );
        }
        let week = LocaleInfoResult::Week(
            WeekInfo::new(
                IsoWeekday::Sunday,
                Box::new([IsoWeekday::Friday, IsoWeekday::Saturday]),
            )
            .unwrap(),
        );
        let bytes = week.encode(LocaleInfoQuery::WeekInfo).unwrap();
        assert_eq!(
            LocaleInfoResult::decode(&bytes, LocaleInfoQuery::WeekInfo).unwrap(),
            week
        );
        let zones = LocaleInfoResult::Identifiers(Box::new(["Europe/Paris".into()]));
        let bytes = zones.encode(LocaleInfoQuery::TimeZones).unwrap();
        assert_eq!(
            LocaleInfoResult::decode(&bytes, LocaleInfoQuery::TimeZones).unwrap(),
            zones
        );
        let values = SupportedValuesResult::new(Box::new(["a".into(), "b".into()])).unwrap();
        assert_eq!(
            SupportedValuesResult::decode(&values.encode().unwrap()).unwrap(),
            values
        );
        for key in SupportedValuesKey::ALL {
            let request = SupportedValuesRequest::new(*key);
            assert_eq!(
                SupportedValuesRequest::decode(&request.encode().unwrap()).unwrap(),
                request
            );
        }
    }

    #[test]
    fn a_result_cannot_cross_the_wire_with_another_query_shape() {
        assert!(LocaleInfoResult::Undefined
            .encode(LocaleInfoQuery::Calendars)
            .is_err());
        assert!(LocaleInfoResult::Direction(TextDirection::RightToLeft)
            .encode(LocaleInfoQuery::WeekInfo)
            .is_err());
        let bytes = LocaleInfoResult::Undefined
            .encode(LocaleInfoQuery::TimeZones)
            .unwrap();
        assert!(LocaleInfoResult::decode(&bytes, LocaleInfoQuery::Collations).is_err());
        assert!(WeekInfo::new(IsoWeekday::Monday, Box::new([])).is_err());
        assert!(WeekInfo::new(
            IsoWeekday::Monday,
            Box::new([IsoWeekday::Sunday, IsoWeekday::Saturday])
        )
        .is_err());
        assert!(SupportedValuesResult::new(Box::new(["b".into(), "a".into()])).is_err());
    }
}
