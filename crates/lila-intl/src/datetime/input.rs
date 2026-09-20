use icu_calendar::Date;

use super::DateTimeFormatError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeValueKind {
    Legacy,
    Instant,
    PlainDate,
    PlainYearMonth,
    PlainMonthDay,
    PlainTime,
    PlainDateTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTimeExactInput {
    kind: DateTimeValueKind,
    epoch_seconds: i64,
    nanosecond: u32,
}
impl DateTimeExactInput {
    pub fn new(
        kind: DateTimeValueKind,
        epoch_seconds: i64,
        nanosecond: u32,
    ) -> Result<Self, DateTimeFormatError> {
        let kind_valid = match kind {
            DateTimeValueKind::Legacy => nanosecond.is_multiple_of(1_000_000),
            DateTimeValueKind::Instant => true,
            DateTimeValueKind::PlainDate
            | DateTimeValueKind::PlainYearMonth
            | DateTimeValueKind::PlainMonthDay
            | DateTimeValueKind::PlainTime
            | DateTimeValueKind::PlainDateTime => false,
        };
        if !kind_valid
            || nanosecond >= 1_000_000_000
            || !(-8_640_000_000_000..=8_640_000_000_000).contains(&epoch_seconds)
            || (epoch_seconds == 8_640_000_000_000 && nanosecond != 0)
        {
            return Err(DateTimeFormatError::InvalidRequest(
                "invalid normalized exact input",
            ));
        }
        Ok(Self {
            kind,
            epoch_seconds,
            nanosecond,
        })
    }
    pub const fn kind(self) -> DateTimeValueKind {
        self.kind
    }
    pub const fn epoch_seconds(self) -> i64 {
        self.epoch_seconds
    }
    pub const fn nanosecond(self) -> u32 {
        self.nanosecond
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTimeIsoFields {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub nanosecond: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTimePlainInput {
    kind: DateTimeValueKind,
    iso: DateTimeIsoFields,
}
impl DateTimePlainInput {
    pub fn new(
        kind: DateTimeValueKind,
        iso: DateTimeIsoFields,
    ) -> Result<Self, DateTimeFormatError> {
        if matches!(kind, DateTimeValueKind::Legacy | DateTimeValueKind::Instant)
            || !(-271_821..=275_760).contains(&iso.year)
            || Date::try_new_iso(iso.year, iso.month, iso.day).is_err()
            || iso.hour >= 24
            || iso.minute >= 60
            || iso.second >= 60
            || iso.nanosecond >= 1_000_000_000
        {
            return Err(DateTimeFormatError::InvalidRequest(
                "invalid exact Plain ISO fields",
            ));
        }
        let date = (iso.year, iso.month, iso.day);
        let time = (iso.hour, iso.minute, iso.second, iso.nanosecond);
        let date_in_bounds = ((-271_821, 4, 19)..=(275_760, 9, 13)).contains(&date);
        let admitted = match kind {
            DateTimeValueKind::PlainDate | DateTimeValueKind::PlainMonthDay => {
                date_in_bounds && time == (12, 0, 0, 0)
            }
            DateTimeValueKind::PlainYearMonth => {
                ((-271_821, 4)..=(275_760, 9)).contains(&(iso.year, iso.month))
                    && time == (12, 0, 0, 0)
            }
            DateTimeValueKind::PlainTime => date == (1970, 1, 1),
            DateTimeValueKind::PlainDateTime => {
                date_in_bounds && (date != (-271_821, 4, 19) || time != (0, 0, 0, 0))
            }
            DateTimeValueKind::Legacy | DateTimeValueKind::Instant => false,
        };
        if !admitted {
            return Err(DateTimeFormatError::InvalidRequest(
                "Plain input violates its Temporal limits or reference time",
            ));
        }
        Ok(Self { kind, iso })
    }
    pub const fn kind(self) -> DateTimeValueKind {
        self.kind
    }
    pub const fn iso(self) -> DateTimeIsoFields {
        self.iso
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeInput {
    Exact(DateTimeExactInput),
    Plain(DateTimePlainInput),
}
impl DateTimeInput {
    pub const fn kind(self) -> DateTimeValueKind {
        match self {
            Self::Exact(input) => input.kind(),
            Self::Plain(input) => input.kind(),
        }
    }
}
