//! Primitive DateTimeFormat options and results. JavaScript observation stays in Wasm.

use core::fmt;

use crate::{CanonicalLocaleId, TimeZoneNameStyle, TimeZoneSelection};

mod input;
pub use input::{
    DateTimeExactInput, DateTimeInput, DateTimeIsoFields, DateTimePlainInput, DateTimeValueKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeCalendar {
    Gregorian,
    Iso8601,
    Chinese,
}

impl DateTimeCalendar {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Gregorian => "gregory",
            Self::Iso8601 => "iso8601",
            Self::Chinese => "chinese",
        }
    }
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "gregory" => Some(Self::Gregorian),
            "iso8601" => Some(Self::Iso8601),
            "chinese" => Some(Self::Chinese),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeHourCycle {
    H11,
    H12,
    H23,
    H24,
}

impl DateTimeHourCycle {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::H11 => "h11",
            Self::H12 => "h12",
            Self::H23 => "h23",
            Self::H24 => "h24",
        }
    }
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "h11" => Some(Self::H11),
            "h12" => Some(Self::H12),
            "h23" => Some(Self::H23),
            "h24" => Some(Self::H24),
            _ => None,
        }
    }
    pub const fn is_twelve_hour(self) -> bool {
        matches!(self, Self::H11 | Self::H12)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeHourCyclePreference {
    Default,
    Cycle(DateTimeHourCycle),
    TwelveHour,
    TwentyFourHour,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeLocaleMatcher {
    Lookup,
    BestFit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeFormatMatcher {
    Basic,
    BestFit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeNumericWidth {
    Numeric,
    TwoDigit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeTextWidth {
    Narrow,
    Short,
    Long,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeMonthWidth {
    Numeric,
    TwoDigit,
    Narrow,
    Short,
    Long,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeStyle {
    Full,
    Long,
    Medium,
    Short,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeRequired {
    Any,
    Date,
    Time,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeDefaults {
    All,
    Date,
    Time,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTimeFractionalDigits(u8);
impl DateTimeFractionalDigits {
    pub fn new(value: u8) -> Result<Self, DateTimeFormatError> {
        (1..=3)
            .contains(&value)
            .then_some(Self(value))
            .ok_or(DateTimeFormatError::InvalidRequest(
                "fractional-second digits must be 1..=3",
            ))
    }
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// Valid options may name an unsupported Unicode keyword. Resolution owns support.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeKeyword(Box<str>);
impl DateTimeKeyword {
    pub fn parse(value: impl Into<Box<str>>) -> Result<Self, DateTimeFormatError> {
        let value = value.into();
        if !value.split('-').all(|part| {
            (3..=8).contains(&part.len()) && part.bytes().all(|byte| byte.is_ascii_alphanumeric())
        }) {
            return Err(DateTimeFormatError::InvalidRequest(
                "malformed Unicode keyword type",
            ));
        }
        Ok(Self(value.to_ascii_lowercase().into_boxed_str()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeLocaleRequest {
    pub requested: Vec<CanonicalLocaleId>,
    pub matcher: DateTimeLocaleMatcher,
    pub calendar: Option<DateTimeKeyword>,
    pub numbering_system: Option<DateTimeKeyword>,
    pub hour_cycle: DateTimeHourCyclePreference,
}

/// Primitive selection; the provider revalidates it against its immutable profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeLocaleResult {
    pub locale: CanonicalLocaleId,
    pub data_locale: CanonicalLocaleId,
    pub calendar: DateTimeCalendar,
    pub numbering_system: DateTimeKeyword,
    pub hour_cycle: DateTimeHourCycle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeSupportedLocalesRequest {
    pub requested: Vec<CanonicalLocaleId>,
    pub matcher: DateTimeLocaleMatcher,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeSupportedLocalesResult {
    pub locales: Vec<CanonicalLocaleId>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DateTimeComponents {
    pub weekday: Option<DateTimeTextWidth>,
    pub era: Option<DateTimeTextWidth>,
    pub year: Option<DateTimeNumericWidth>,
    pub month: Option<DateTimeMonthWidth>,
    pub day: Option<DateTimeNumericWidth>,
    pub day_period: Option<DateTimeTextWidth>,
    pub hour: Option<DateTimeNumericWidth>,
    pub minute: Option<DateTimeNumericWidth>,
    pub second: Option<DateTimeNumericWidth>,
    pub fractional_second_digits: Option<DateTimeFractionalDigits>,
    pub time_zone_name: Option<TimeZoneNameStyle>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTimeStyles {
    date: Option<DateTimeStyle>,
    time: Option<DateTimeStyle>,
}
impl DateTimeStyles {
    pub fn new(
        date: Option<DateTimeStyle>,
        time: Option<DateTimeStyle>,
    ) -> Result<Self, DateTimeFormatError> {
        if date.is_none() && time.is_none() {
            return Err(DateTimeFormatError::InvalidRequest(
                "a style selection needs a date or time style",
            ));
        }
        Ok(Self { date, time })
    }
    pub const fn date(self) -> Option<DateTimeStyle> {
        self.date
    }
    pub const fn time(self) -> Option<DateTimeStyle> {
        self.time
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateTimeStyleSelection {
    Components(DateTimeComponents),
    Styles(DateTimeStyles),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimePlanRequest {
    pub locale: DateTimeLocaleResult,
    pub time_zone: TimeZoneSelection,
    pub selection: DateTimeStyleSelection,
    pub matcher: DateTimeFormatMatcher,
    pub required: DateTimeRequired,
    pub defaults: DateTimeDefaults,
}

/// Untrusted plan bytes are never confused with the provider's validated plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedDateTimePlan(Vec<u8>);
impl EncodedDateTimePlan {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimePlanResult {
    pub plan: EncodedDateTimePlan,
    pub locale: DateTimeLocaleResult,
    pub time_zone: TimeZoneSelection,
    pub components: DateTimeComponents,
    pub styles: Option<DateTimeStyles>,
    pub available_formats: DateTimeFormatAvailability,
}

/// Legacy and Instant always have formats; only Plain formats may be absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTimeFormatAvailability(u64);
impl DateTimeFormatAvailability {
    const KNOWN_MASK: u64 = Self::mask_for(DateTimeValueKind::PlainDate)
        | Self::mask_for(DateTimeValueKind::PlainYearMonth)
        | Self::mask_for(DateTimeValueKind::PlainMonthDay)
        | Self::mask_for(DateTimeValueKind::PlainTime)
        | Self::mask_for(DateTimeValueKind::PlainDateTime);

    pub const fn mask_for(kind: DateTimeValueKind) -> u64 {
        match kind {
            DateTimeValueKind::Legacy | DateTimeValueKind::Instant => 0,
            DateTimeValueKind::PlainDate => 1 << 0,
            DateTimeValueKind::PlainYearMonth => 1 << 1,
            DateTimeValueKind::PlainMonthDay => 1 << 2,
            DateTimeValueKind::PlainTime => 1 << 3,
            DateTimeValueKind::PlainDateTime => 1 << 4,
        }
    }
    pub fn from_available_kinds(kinds: impl IntoIterator<Item = DateTimeValueKind>) -> Self {
        Self(
            kinds
                .into_iter()
                .fold(0, |mask, kind| mask | Self::mask_for(kind)),
        )
    }
    pub const fn contains(self, kind: DateTimeValueKind) -> bool {
        let mask = Self::mask_for(kind);
        self.0 & mask == mask
    }
    pub const fn wire_code(self) -> u64 {
        self.0
    }
    pub const fn from_wire_code(value: u64) -> Option<Self> {
        if value & !Self::KNOWN_MASK == 0 {
            Some(Self(value))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeFormatRequest {
    pub plan: EncodedDateTimePlan,
    pub input: DateTimeInput,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeRangeRequest {
    pub plan: EncodedDateTimePlan,
    pub start: DateTimeInput,
    pub end: DateTimeInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimePartKind {
    Literal,
    Era,
    Year,
    RelatedYear,
    YearName,
    Month,
    Day,
    Weekday,
    DayPeriod,
    Hour,
    Minute,
    Second,
    FractionalSecond,
    TimeZoneName,
}
impl DateTimePartKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Literal => "literal",
            Self::Era => "era",
            Self::Year => "year",
            Self::RelatedYear => "relatedYear",
            Self::YearName => "yearName",
            Self::Month => "month",
            Self::Day => "day",
            Self::Weekday => "weekday",
            Self::DayPeriod => "dayPeriod",
            Self::Hour => "hour",
            Self::Minute => "minute",
            Self::Second => "second",
            Self::FractionalSecond => "fractionalSecond",
            Self::TimeZoneName => "timeZoneName",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimePart {
    pub kind: DateTimePartKind,
    pub value: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeParts {
    pub parts: Vec<DateTimePart>,
}
impl DateTimeParts {
    pub fn to_formatted_string(&self) -> String {
        self.parts.iter().map(|part| part.value.as_str()).collect()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeRangeSource {
    Shared,
    StartRange,
    EndRange,
}
impl DateTimeRangeSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Shared => "shared",
            Self::StartRange => "startRange",
            Self::EndRange => "endRange",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeRangePart {
    pub kind: DateTimePartKind,
    pub value: String,
    pub source: DateTimeRangeSource,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeRangeParts {
    pub parts: Vec<DateTimeRangePart>,
}
impl DateTimeRangeParts {
    pub fn to_formatted_string(&self) -> String {
        self.parts.iter().map(|part| part.value.as_str()).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateTimeFormatError {
    InvalidRequest(&'static str),
    InvalidPlan(&'static str),
    InvalidProfile(String),
    UnavailableFormat,
    InputKindMismatch,
}
impl fmt::Display for DateTimeFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(reason) => write!(f, "invalid date/time request: {reason}"),
            Self::InvalidPlan(reason) => write!(f, "invalid date/time plan: {reason}"),
            Self::InvalidProfile(reason) => write!(f, "invalid pinned date/time profile: {reason}"),
            Self::UnavailableFormat => {
                f.write_str("the formatter has no format for this Temporal kind")
            }
            Self::InputKindMismatch => f.write_str("date/time range inputs have different kinds"),
        }
    }
}
impl std::error::Error for DateTimeFormatError {}
