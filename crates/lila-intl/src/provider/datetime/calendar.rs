use icu_calendar::{cal::Iso, types::RataDie, Date};

use crate::datetime::{DateTimeCalendar, DateTimeFormatError, DateTimeIsoFields};

mod kernels;
mod kind;
mod month;
mod names;
mod projection;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(super) use kernels::pinned as pinned_kernels;
pub(super) use kernels::CalendarKernels;
pub(super) use kind::{CalendarId, EraName};
pub(super) use month::Month;
use projection::CalendarProjection;
pub(super) use projection::CalendarYear;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Fields {
    pub(super) calendar: CalendarId,
    pub(super) year: CalendarYear,
    pub(super) month: Month,
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
    kernels: &CalendarKernels,
) -> Result<Fields, DateTimeFormatError> {
    convert_with_kernels(CalendarId::from_admitted(calendar), fields, kernels)
}

/// Native calculation and formatting identity remain closed independently of
/// the public request domain, which is admitted separately with genuine data.
fn convert_with_kernels(
    calendar: CalendarId,
    fields: DateTimeIsoFields,
    kernels: &CalendarKernels,
) -> Result<Fields, DateTimeFormatError> {
    if fields.hour > 23
        || fields.minute > 59
        || fields.second > 59
        || fields.nanosecond >= 1_000_000_000
    {
        return Err(DateTimeFormatError::InvalidRequest(
            "invalid local ISO clock",
        ));
    }
    let converted = CalendarProjection::from_iso(calendar, fields, kernels)?;
    Ok(Fields {
        calendar,
        year: converted.year(),
        month: converted.month(),
        day: converted.day(),
        weekday: converted.weekday(),
        hour: fields.hour,
        minute: fields.minute,
        second: fields.second,
        nanosecond: fields.nanosecond,
    })
}

#[cfg(test)]
pub(super) fn convert_kind(
    calendar: CalendarId,
    fields: DateTimeIsoFields,
) -> Result<Fields, DateTimeFormatError> {
    convert_with_kernels(calendar, fields, pinned_kernels())
}
