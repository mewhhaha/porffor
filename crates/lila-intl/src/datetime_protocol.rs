//! Stateless DateTimeFormat messages shared by Wasm emission and the host.

use core::fmt;

use crate::datetime::*;
use crate::{
    CanonicalLocaleId, FixedTimeZoneOffset, TimeZoneId, TimeZoneNameStyle, TimeZoneSelection,
};

pub const DATE_TIME_WIRE_VERSION: u64 = 1;
pub const DATE_TIME_WIRE_HEADER_BYTES: u64 = 16;
pub const DATE_TIME_INPUT_BYTES: u64 = 80;
pub const DATE_TIME_COMPONENT_COUNT: usize = 11;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateTimeWireError {
    Malformed(&'static str),
    Domain(DateTimeFormatError),
    Resource(&'static str),
}

impl fmt::Display for DateTimeWireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(reason) => write!(f, "invalid date/time wire message: {reason}"),
            Self::Domain(error) => error.fmt(f),
            Self::Resource(reason) => write!(f, "date/time wire resource limit: {reason}"),
        }
    }
}
impl std::error::Error for DateTimeWireError {}
impl From<DateTimeFormatError> for DateTimeWireError {
    fn from(error: DateTimeFormatError) -> Self {
        Self::Domain(error)
    }
}

// Exhaustive encoders make adding a domain member require a wire decision.
macro_rules! wire_domain {
    ($domain:ident { $($variant:ident = $code:literal),+ $(,)? }) => {
        impl $domain {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];
            pub const fn wire_code(self) -> u64 {
                match self { $(Self::$variant => $code),+ }
            }
            pub const fn from_wire_code(code: u64) -> Option<Self> {
                match code { $($code => Some(Self::$variant),)+ _ => None }
            }
        }
    };
}
wire_domain!(DateTimeCalendar { Gregorian = 1, Iso8601 = 2, Chinese = 3 });
wire_domain!(DateTimeHourCycle { H11 = 1, H12 = 2, H23 = 3, H24 = 4 });
wire_domain!(DateTimeLocaleMatcher { Lookup = 1, BestFit = 2 });
wire_domain!(DateTimeFormatMatcher { Basic = 1, BestFit = 2 });
wire_domain!(DateTimeNumericWidth { TwoDigit = 1, Numeric = 2 });
wire_domain!(DateTimeTextWidth { Narrow = 1, Short = 2, Long = 3 });
wire_domain!(DateTimeMonthWidth { TwoDigit = 1, Numeric = 2, Narrow = 3, Short = 4, Long = 5 });
wire_domain!(DateTimeStyle { Full = 1, Long = 2, Medium = 3, Short = 4 });
wire_domain!(DateTimeRequired { Any = 1, Date = 2, Time = 3 });
wire_domain!(DateTimeDefaults { All = 1, Date = 2, Time = 3 });
wire_domain!(DateTimeValueKind {
    Legacy = 1, Instant = 2, PlainDate = 3, PlainYearMonth = 4,
    PlainMonthDay = 5, PlainTime = 6, PlainDateTime = 7,
});
wire_domain!(DateTimePartKind {
    Literal = 0, Era = 1, Year = 2, RelatedYear = 3, YearName = 4,
    Month = 5, Day = 6, Weekday = 7, DayPeriod = 8, Hour = 9,
    Minute = 10, Second = 11, FractionalSecond = 12, TimeZoneName = 13,
});
wire_domain!(DateTimeRangeSource { Shared = 0, StartRange = 1, EndRange = 2 });

macro_rules! option_names {
    ($domain:ident { $($variant:ident => $name:literal),+ $(,)? }) => {
        impl $domain {
            pub const OPTIONS: &'static [(&'static str, i64)] = &[
                $((Self::$variant.option_name(), Self::$variant.wire_code() as i64)),+
            ];
            pub const fn option_name(self) -> &'static str {
                match self { $(Self::$variant => $name),+ }
            }
        }
    };
}
option_names!(DateTimeNumericWidth { TwoDigit => "2-digit", Numeric => "numeric" });
option_names!(DateTimeTextWidth { Narrow => "narrow", Short => "short", Long => "long" });
option_names!(DateTimeMonthWidth {
    TwoDigit => "2-digit", Numeric => "numeric", Narrow => "narrow", Short => "short", Long => "long",
});
option_names!(DateTimeStyle { Full => "full", Long => "long", Medium => "medium", Short => "short" });
option_names!(DateTimeLocaleMatcher { Lookup => "lookup", BestFit => "best fit" });
option_names!(DateTimeFormatMatcher { Basic => "basic", BestFit => "best fit" });
impl DateTimeHourCycle {
    pub const OPTIONS: &'static [(&'static str, i64)] = &[
        (Self::H11.as_str(), Self::H11.wire_code() as i64),
        (Self::H12.as_str(), Self::H12.wire_code() as i64),
        (Self::H23.as_str(), Self::H23.wire_code() as i64),
        (Self::H24.as_str(), Self::H24.wire_code() as i64),
    ];
}

impl DateTimeHourCyclePreference {
    pub const fn wire_code(self) -> u64 {
        match self {
            Self::Default => 0,
            Self::Cycle(cycle) => cycle.wire_code(),
            Self::TwelveHour => 5,
            Self::TwentyFourHour => 6,
        }
    }
    pub const fn from_wire_code(code: u64) -> Option<Self> {
        match code {
            0 => Some(Self::Default),
            5 => Some(Self::TwelveHour),
            6 => Some(Self::TwentyFourHour),
            code => match DateTimeHourCycle::from_wire_code(code) {
                Some(cycle) => Some(Self::Cycle(cycle)),
                None => None,
            },
        }
    }
}

struct DateTimeWireWriter(Vec<u8>);
impl DateTimeWireWriter {
    fn append(&mut self, bytes: &[u8]) -> Result<(), DateTimeWireError> {
        let length = self
            .0
            .len()
            .checked_add(bytes.len())
            .ok_or(DateTimeWireError::Resource("message length overflow"))?;
        u32::try_from(length)
            .map_err(|_| DateTimeWireError::Resource("message exceeds the Wasm32 host span"))?;
        self.0
            .try_reserve(bytes.len())
            .map_err(|_| DateTimeWireError::Resource("message allocation failed"))?;
        self.0.extend_from_slice(bytes);
        Ok(())
    }
    fn word(&mut self, word: u64) -> Result<(), DateTimeWireError> {
        self.append(&word.to_le_bytes())
    }
    fn blob(&mut self, bytes: &[u8]) -> Result<(), DateTimeWireError> {
        self.word(bytes.len() as u64)?;
        self.append(bytes)
    }
    fn text(&mut self, value: &str) -> Result<(), DateTimeWireError> {
        self.blob(value.as_bytes())
    }
    fn locales(&mut self, locales: &[CanonicalLocaleId]) -> Result<(), DateTimeWireError> {
        self.word(locales.len() as u64)?;
        for locale in locales {
            self.text(locale.as_str())?;
        }
        Ok(())
    }
    fn keyword(&mut self, keyword: &Option<DateTimeKeyword>) -> Result<(), DateTimeWireError> {
        self.text(keyword.as_ref().map_or("", DateTimeKeyword::as_str))
    }
    fn locale_result(&mut self, locale: &DateTimeLocaleResult) -> Result<(), DateTimeWireError> {
        self.text(locale.locale.as_str())?;
        self.text(locale.data_locale.as_str())?;
        self.word(locale.calendar.wire_code())?;
        self.text(locale.numbering_system.as_str())?;
        self.word(locale.hour_cycle.wire_code())
    }
    fn zone(&mut self, zone: &TimeZoneSelection) -> Result<(), DateTimeWireError> {
        match zone {
            TimeZoneSelection::Named(identifier) => {
                self.word(1)?;
                self.text(identifier.as_str())
            }
            TimeZoneSelection::FixedOffset(offset) => {
                self.word(2)?;
                self.word(i64::from(offset.seconds()) as u64)
            }
        }
    }
    fn components(&mut self, components: DateTimeComponents) -> Result<(), DateTimeWireError> {
        for code in [
            components.weekday.map_or(0, DateTimeTextWidth::wire_code),
            components.era.map_or(0, DateTimeTextWidth::wire_code),
            components.year.map_or(0, DateTimeNumericWidth::wire_code),
            components.month.map_or(0, DateTimeMonthWidth::wire_code),
            components.day.map_or(0, DateTimeNumericWidth::wire_code),
            components
                .day_period
                .map_or(0, DateTimeTextWidth::wire_code),
            components.hour.map_or(0, DateTimeNumericWidth::wire_code),
            components.minute.map_or(0, DateTimeNumericWidth::wire_code),
            components.second.map_or(0, DateTimeNumericWidth::wire_code),
            components
                .fractional_second_digits
                .map_or(0, |digits| u64::from(digits.get())),
            components
                .time_zone_name
                .map_or(0, |style| style.code() as u64),
        ] {
            self.word(code)?;
        }
        Ok(())
    }
    fn styles(&mut self, styles: Option<DateTimeStyles>) -> Result<(), DateTimeWireError> {
        self.word(
            styles
                .and_then(DateTimeStyles::date)
                .map_or(0, DateTimeStyle::wire_code),
        )?;
        self.word(
            styles
                .and_then(DateTimeStyles::time)
                .map_or(0, DateTimeStyle::wire_code),
        )
    }
    fn input(&mut self, input: DateTimeInput) -> Result<(), DateTimeWireError> {
        // Every input occupies ten words. Unused fields must be zero, so a kind
        // change cannot reinterpret leftover fields as another valid record.
        self.word(input.kind().wire_code())?;
        let fields = match input {
            DateTimeInput::Exact(input) => [
                input.epoch_seconds() as u64,
                u64::from(input.nanosecond()),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
            DateTimeInput::Plain(input) => {
                let iso = input.iso();
                [
                    0,
                    0,
                    i64::from(iso.year) as u64,
                    u64::from(iso.month),
                    u64::from(iso.day),
                    u64::from(iso.hour),
                    u64::from(iso.minute),
                    u64::from(iso.second),
                    u64::from(iso.nanosecond),
                ]
            }
        };
        for field in fields {
            self.word(field)?;
        }
        Ok(())
    }
}

struct DateTimeWireReader<'a>(&'a [u8]);
impl<'a> DateTimeWireReader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], DateTimeWireError> {
        let Some((prefix, remaining)) = self.0.split_at_checked(count) else {
            return Err(DateTimeWireError::Malformed("truncated field"));
        };
        self.0 = remaining;
        Ok(prefix)
    }
    fn word(&mut self) -> Result<u64, DateTimeWireError> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("eight-byte field"),
        ))
    }
    fn domain<T>(&mut self, decode: fn(u64) -> Option<T>) -> Result<T, DateTimeWireError> {
        decode(self.word()?).ok_or(DateTimeWireError::Malformed("unknown closed-domain code"))
    }
    fn optional<T>(
        &mut self,
        decode: fn(u64) -> Option<T>,
    ) -> Result<Option<T>, DateTimeWireError> {
        let code = self.word()?;
        if code == 0 {
            return Ok(None);
        }
        decode(code)
            .map(Some)
            .ok_or(DateTimeWireError::Malformed("unknown optional-domain code"))
    }
    fn blob(&mut self) -> Result<&'a [u8], DateTimeWireError> {
        let length = u32::try_from(self.word()?)
            .map_err(|_| DateTimeWireError::Malformed("blob exceeds the Wasm32 host span"))?;
        self.take(length as usize)
    }
    fn text(&mut self) -> Result<&'a str, DateTimeWireError> {
        core::str::from_utf8(self.blob()?)
            .map_err(|_| DateTimeWireError::Malformed("ill-formed UTF-8"))
    }
    fn owned_text(&mut self) -> Result<String, DateTimeWireError> {
        let text = self.text()?;
        let mut owned = String::new();
        owned
            .try_reserve_exact(text.len())
            .map_err(|_| DateTimeWireError::Resource("text allocation failed"))?;
        owned.push_str(text);
        Ok(owned)
    }
    fn locale(&mut self) -> Result<CanonicalLocaleId, DateTimeWireError> {
        CanonicalLocaleId::from_data(self.text()?)
            .map_err(|_| DateTimeWireError::Malformed("non-canonical locale"))
    }
    fn list<T>(
        &mut self,
        minimum_bytes: usize,
        mut read: impl FnMut(&mut Self) -> Result<T, DateTimeWireError>,
    ) -> Result<Vec<T>, DateTimeWireError> {
        let count = usize::try_from(self.word()?)
            .map_err(|_| DateTimeWireError::Malformed("list count overflow"))?;
        if count > self.0.len() / minimum_bytes {
            return Err(DateTimeWireError::Malformed(
                "list count exceeds remaining fields",
            ));
        }
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(count)
            .map_err(|_| DateTimeWireError::Resource("list allocation failed"))?;
        for _ in 0..count {
            entries.push(read(self)?);
        }
        Ok(entries)
    }
    fn locales(&mut self) -> Result<Vec<CanonicalLocaleId>, DateTimeWireError> {
        self.list(8, Self::locale)
    }
    fn keyword(&mut self) -> Result<Option<DateTimeKeyword>, DateTimeWireError> {
        let text = self.text()?;
        if text.is_empty() {
            return Ok(None);
        }
        Ok(Some(DateTimeKeyword::parse(text)?))
    }
    fn locale_result(&mut self) -> Result<DateTimeLocaleResult, DateTimeWireError> {
        Ok(DateTimeLocaleResult {
            locale: self.locale()?,
            data_locale: self.locale()?,
            calendar: self.domain(DateTimeCalendar::from_wire_code)?,
            numbering_system: DateTimeKeyword::parse(self.text()?)?,
            hour_cycle: self.domain(DateTimeHourCycle::from_wire_code)?,
        })
    }
    fn zone(&mut self) -> Result<TimeZoneSelection, DateTimeWireError> {
        match self.word()? {
            1 => Ok(TimeZoneSelection::Named(
                TimeZoneId::parse(self.text()?)
                    .map_err(|_| DateTimeWireError::Malformed("invalid named time zone"))?,
            )),
            2 => Ok(TimeZoneSelection::FixedOffset(
                FixedTimeZoneOffset::from_seconds(self.word()? as i64)
                    .map_err(|_| DateTimeWireError::Malformed("invalid fixed time-zone offset"))?,
            )),
            _ => Err(DateTimeWireError::Malformed("unknown time-zone kind")),
        }
    }
    fn components(&mut self) -> Result<DateTimeComponents, DateTimeWireError> {
        Ok(DateTimeComponents {
            weekday: self.optional(DateTimeTextWidth::from_wire_code)?,
            era: self.optional(DateTimeTextWidth::from_wire_code)?,
            year: self.optional(DateTimeNumericWidth::from_wire_code)?,
            month: self.optional(DateTimeMonthWidth::from_wire_code)?,
            day: self.optional(DateTimeNumericWidth::from_wire_code)?,
            day_period: self.optional(DateTimeTextWidth::from_wire_code)?,
            hour: self.optional(DateTimeNumericWidth::from_wire_code)?,
            minute: self.optional(DateTimeNumericWidth::from_wire_code)?,
            second: self.optional(DateTimeNumericWidth::from_wire_code)?,
            fractional_second_digits: self.optional(|code| {
                u8::try_from(code)
                    .ok()
                    .and_then(|value| DateTimeFractionalDigits::new(value).ok())
            })?,
            time_zone_name: self.optional(|code| {
                i64::try_from(code)
                    .ok()
                    .and_then(TimeZoneNameStyle::from_code)
            })?,
        })
    }
    fn styles(&mut self) -> Result<Option<DateTimeStyles>, DateTimeWireError> {
        let date = self.optional(DateTimeStyle::from_wire_code)?;
        let time = self.optional(DateTimeStyle::from_wire_code)?;
        if date.is_none() && time.is_none() {
            return Ok(None);
        }
        Ok(Some(DateTimeStyles::new(date, time)?))
    }
    fn input(&mut self) -> Result<DateTimeInput, DateTimeWireError> {
        let kind = self.domain(DateTimeValueKind::from_wire_code)?;
        let fields = [
            self.word()?,
            self.word()?,
            self.word()?,
            self.word()?,
            self.word()?,
            self.word()?,
            self.word()?,
            self.word()?,
            self.word()?,
        ];
        let invalid_field = || DateTimeWireError::Malformed("input integer outside its domain");
        match kind {
            DateTimeValueKind::Legacy | DateTimeValueKind::Instant => {
                if fields[2..].iter().any(|field| *field != 0) {
                    return Err(DateTimeWireError::Malformed("exact input has Plain fields"));
                }
                Ok(DateTimeInput::Exact(DateTimeExactInput::new(
                    kind,
                    fields[0] as i64,
                    u32::try_from(fields[1]).map_err(|_| invalid_field())?,
                )?))
            }
            DateTimeValueKind::PlainDate
            | DateTimeValueKind::PlainYearMonth
            | DateTimeValueKind::PlainMonthDay
            | DateTimeValueKind::PlainTime
            | DateTimeValueKind::PlainDateTime => {
                if fields[..2].iter().any(|field| *field != 0) {
                    return Err(DateTimeWireError::Malformed("Plain input has epoch fields"));
                }
                let iso = DateTimeIsoFields {
                    year: i32::try_from(fields[2] as i64).map_err(|_| invalid_field())?,
                    month: u8::try_from(fields[3]).map_err(|_| invalid_field())?,
                    day: u8::try_from(fields[4]).map_err(|_| invalid_field())?,
                    hour: u8::try_from(fields[5]).map_err(|_| invalid_field())?,
                    minute: u8::try_from(fields[6]).map_err(|_| invalid_field())?,
                    second: u8::try_from(fields[7]).map_err(|_| invalid_field())?,
                    nanosecond: u32::try_from(fields[8]).map_err(|_| invalid_field())?,
                };
                Ok(DateTimeInput::Plain(DateTimePlainInput::new(kind, iso)?))
            }
        }
    }
    fn plan(&mut self) -> Result<EncodedDateTimePlan, DateTimeWireError> {
        let bytes = self.blob()?;
        let mut owned = Vec::new();
        owned
            .try_reserve_exact(bytes.len())
            .map_err(|_| DateTimeWireError::Resource("plan allocation failed"))?;
        owned.extend_from_slice(bytes);
        Ok(EncodedDateTimePlan::from_bytes(owned))
    }
}

macro_rules! wire_message {
    ($record:ident, $tag:expr, |$value:ident, $writer:ident| $write:block, |$reader:ident| $read:block) => {
        impl $record {
            pub const WIRE_TAG: u64 = $tag;
            pub fn encode(&self) -> Result<Vec<u8>, DateTimeWireError> {
                let mut $writer = DateTimeWireWriter(Vec::new());
                $writer.word(DATE_TIME_WIRE_VERSION)?;
                $writer.word(Self::WIRE_TAG)?;
                let $value = self;
                $write
                Ok($writer.0)
            }
            pub fn decode(bytes: &[u8]) -> Result<Self, DateTimeWireError> {
                if bytes.len() > u32::MAX as usize {
                    return Err(DateTimeWireError::Malformed("message exceeds the Wasm32 host span"));
                }
                let mut $reader = DateTimeWireReader(bytes);
                if $reader.word()? != DATE_TIME_WIRE_VERSION || $reader.word()? != Self::WIRE_TAG {
                    return Err(DateTimeWireError::Malformed("wrong message version or kind"));
                }
                let decoded: Self = $read;
                if !$reader.0.is_empty() {
                    return Err(DateTimeWireError::Malformed("trailing message bytes"));
                }
                Ok(decoded)
            }
        }
    };
}

wire_message!(
    DateTimeLocaleRequest,
    2 * crate::IntlHostOp::ResolveDateTimeLocale.code() as u64,
    |value, writer| {
        writer.word(value.matcher.wire_code())?;
        writer.word(value.hour_cycle.wire_code())?;
        writer.keyword(&value.calendar)?;
        writer.keyword(&value.numbering_system)?;
        writer.locales(&value.requested)?;
    },
    |reader| {
        let matcher = reader.domain(DateTimeLocaleMatcher::from_wire_code)?;
        let hour_cycle = reader.domain(DateTimeHourCyclePreference::from_wire_code)?;
        let calendar = reader.keyword()?;
        let numbering_system = reader.keyword()?;
        DateTimeLocaleRequest {
            matcher,
            hour_cycle,
            calendar,
            numbering_system,
            requested: reader.locales()?,
        }
    }
);
wire_message!(
    DateTimeLocaleResult,
    2 * crate::IntlHostOp::ResolveDateTimeLocale.code() as u64 + 1,
    |value, writer| {
        writer.locale_result(value)?;
    },
    |reader| { reader.locale_result()? }
);
wire_message!(
    DateTimeSupportedLocalesRequest,
    2 * crate::IntlHostOp::SupportedDateTimeLocales.code() as u64,
    |value, writer| {
        writer.word(value.matcher.wire_code())?;
        writer.locales(&value.requested)?;
    },
    |reader| {
        DateTimeSupportedLocalesRequest {
            matcher: reader.domain(DateTimeLocaleMatcher::from_wire_code)?,
            requested: reader.locales()?,
        }
    }
);
wire_message!(
    DateTimeSupportedLocalesResult,
    2 * crate::IntlHostOp::SupportedDateTimeLocales.code() as u64 + 1,
    |value, writer| {
        writer.locales(&value.locales)?;
    },
    |reader| {
        DateTimeSupportedLocalesResult {
            locales: reader.locales()?,
        }
    }
);
wire_message!(
    DateTimePlanRequest,
    2 * crate::IntlHostOp::SelectDateTimeFormat.code() as u64,
    |value, writer| {
        writer.locale_result(&value.locale)?;
        writer.zone(&value.time_zone)?;
        writer.word(value.matcher.wire_code())?;
        writer.word(value.required.wire_code())?;
        writer.word(value.defaults.wire_code())?;
        match value.selection {
            DateTimeStyleSelection::Components(components) => {
                writer.word(1)?;
                writer.components(components)?;
            }
            DateTimeStyleSelection::Styles(styles) => {
                writer.word(2)?;
                writer.styles(Some(styles))?;
            }
        }
    },
    |reader| {
        let locale = reader.locale_result()?;
        let time_zone = reader.zone()?;
        let matcher = reader.domain(DateTimeFormatMatcher::from_wire_code)?;
        let required = reader.domain(DateTimeRequired::from_wire_code)?;
        let defaults = reader.domain(DateTimeDefaults::from_wire_code)?;
        let selection = match reader.word()? {
            1 => DateTimeStyleSelection::Components(reader.components()?),
            2 => DateTimeStyleSelection::Styles(
                reader
                    .styles()?
                    .ok_or(DateTimeWireError::Malformed("empty styles selection"))?,
            ),
            _ => return Err(DateTimeWireError::Malformed("unknown selection kind")),
        };
        DateTimePlanRequest {
            locale,
            time_zone,
            matcher,
            required,
            defaults,
            selection,
        }
    }
);
wire_message!(
    DateTimePlanResult,
    2 * crate::IntlHostOp::SelectDateTimeFormat.code() as u64 + 1,
    |value, writer| {
        writer.locale_result(&value.locale)?;
        writer.zone(&value.time_zone)?;
        writer.components(value.components)?;
        writer.styles(value.styles)?;
        writer.word(value.available_formats.wire_code())?;
        writer.blob(value.plan.as_bytes())?;
    },
    |reader| {
        DateTimePlanResult {
            locale: reader.locale_result()?,
            time_zone: reader.zone()?,
            components: reader.components()?,
            styles: reader.styles()?,
            available_formats: reader.domain(DateTimeFormatAvailability::from_wire_code)?,
            plan: reader.plan()?,
        }
    }
);
wire_message!(
    DateTimeFormatRequest,
    2 * crate::IntlHostOp::FormatDateTimeParts.code() as u64,
    |value, writer| {
        writer.input(value.input)?;
        writer.blob(value.plan.as_bytes())?;
    },
    |reader| {
        DateTimeFormatRequest {
            input: reader.input()?,
            plan: reader.plan()?,
        }
    }
);
wire_message!(
    DateTimeParts,
    2 * crate::IntlHostOp::FormatDateTimeParts.code() as u64 + 1,
    |value, writer| {
        writer.word(value.parts.len() as u64)?;
        for part in &value.parts {
            writer.word(part.kind.wire_code())?;
            writer.text(&part.value)?;
        }
    },
    |reader| {
        DateTimeParts {
            parts: reader.list(16, |reader| {
                Ok(DateTimePart {
                    kind: reader.domain(DateTimePartKind::from_wire_code)?,
                    value: reader.owned_text()?,
                })
            })?,
        }
    }
);
wire_message!(
    DateTimeRangeRequest,
    2 * crate::IntlHostOp::FormatDateTimeRangeParts.code() as u64,
    |value, writer| {
        writer.input(value.start)?;
        writer.input(value.end)?;
        writer.blob(value.plan.as_bytes())?;
    },
    |reader| {
        DateTimeRangeRequest {
            start: reader.input()?,
            end: reader.input()?,
            plan: reader.plan()?,
        }
    }
);
wire_message!(
    DateTimeRangeParts,
    2 * crate::IntlHostOp::FormatDateTimeRangeParts.code() as u64 + 1,
    |value, writer| {
        writer.word(value.parts.len() as u64)?;
        for part in &value.parts {
            writer.word(part.kind.wire_code())?;
            writer.word(part.source.wire_code())?;
            writer.text(&part.value)?;
        }
    },
    |reader| {
        DateTimeRangeParts {
            parts: reader.list(24, |reader| {
                Ok(DateTimeRangePart {
                    kind: reader.domain(DateTimePartKind::from_wire_code)?,
                    source: reader.domain(DateTimeRangeSource::from_wire_code)?,
                    value: reader.owned_text()?,
                })
            })?,
        }
    }
);

#[cfg(test)]
mod tests;
