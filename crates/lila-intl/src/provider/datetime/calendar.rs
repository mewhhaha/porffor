use icu_calendar::{
    cal::{Chinese, Gregorian, Iso},
    types::RataDie,
    Date,
};

use crate::datetime::{DateTimeCalendar, DateTimeFormatError, DateTimeIsoFields};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Year {
    Era { era: u8, year: i32 },
    Cyclic { year: u8, related: i32 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Fields {
    pub(super) year: Year,
    pub(super) month: u8,
    pub(super) leap_month: bool,
    pub(super) day: u8,
    pub(super) weekday: u8,
    pub(super) hour: u8,
    pub(super) minute: u8,
    pub(super) second: u8,
    pub(super) nanosecond: u32,
}

pub(super) fn local_iso(
    epoch_seconds: i64,
    nanosecond: u32,
    offset_seconds: i32,
) -> Result<DateTimeIsoFields, DateTimeFormatError> {
    let local = epoch_seconds
        .checked_add(i64::from(offset_seconds))
        .ok_or(DateTimeFormatError::InvalidRequest("local epoch overflow"))?;
    let day = local.div_euclid(86_400);
    let seconds = local.rem_euclid(86_400);
    let iso = Date::from_rata_die(RataDie::new(day + 719_163), Iso);
    Ok(DateTimeIsoFields {
        year: iso.extended_year(),
        month: iso.month().ordinal,
        day: iso.day_of_month().0,
        hour: (seconds / 3600) as u8,
        minute: ((seconds % 3600) / 60) as u8,
        second: (seconds % 60) as u8,
        nanosecond,
    })
}

pub(super) fn convert(
    calendar: DateTimeCalendar,
    fields: DateTimeIsoFields,
) -> Result<Fields, DateTimeFormatError> {
    let iso = Date::try_new_iso(fields.year, fields.month, fields.day)
        .map_err(|_| DateTimeFormatError::InvalidRequest("invalid local ISO date"))?;
    let weekday = iso.day_of_week() as u8 % 7;
    let (year, month, leap_month, day) = match calendar {
        DateTimeCalendar::Gregorian | DateTimeCalendar::Iso8601 => {
            let date = iso.to_calendar(Gregorian);
            let year = date.era_year();
            let era = match year.era.as_str() {
                "bce" => 0,
                "ce" => 1,
                _ => return Err(super::profile::invalid("unexpected Gregorian era")),
            };
            (
                Year::Era {
                    era,
                    year: year.year,
                },
                date.month().month_number(),
                false,
                date.day_of_month().0,
            )
        }
        DateTimeCalendar::Chinese => {
            let date = iso.to_calendar(Chinese::new());
            let year = date.cyclic_year();
            let (month, leap) =
                date.month().formatting_code.parsed().ok_or_else(|| {
                    super::profile::invalid("invalid converted Chinese month code")
                })?;
            (
                Year::Cyclic {
                    year: year.year,
                    related: year.related_iso,
                },
                month,
                leap,
                date.day_of_month().0,
            )
        }
    };
    Ok(Fields {
        year,
        month,
        leap_month,
        day,
        weekday,
        hour: fields.hour,
        minute: fields.minute,
        second: fields.second,
        nanosecond: fields.nanosecond,
    })
}
