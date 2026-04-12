//! Implementation of `ResolvedCalendarFields`

use alloc::{format, vec::Vec};
use tinystr::tinystr;
use tinystr::TinyAsciiStr;

use crate::fields::CalendarFields;
use crate::iso::{constrain_iso_day, is_valid_iso_day, IsoDate};
use crate::options::Overflow;
use crate::Calendar;
use crate::{TemporalError, TemporalResult};
use icu_calendar::{AnyCalendarKind, Calendar as IcuCalendar, Iso, types::MonthCode as IcuMonthCode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResolutionType {
    Date,
    YearMonth,
    MonthDay,
}

/// `ResolvedCalendarFields` represents the resolved field values necessary for
/// creating a Date from potentially partial values.
#[derive(Debug)]
pub struct ResolvedCalendarFields {
    pub(crate) era_year: EraYear,
    pub(crate) month_code: MonthCode,
    pub(crate) day: u8,
}

impl ResolvedCalendarFields {
    // TODO: Potentially make a method on `Calendar`.
    #[inline]
    pub fn try_from_fields(
        calendar: &Calendar,
        fields: &CalendarFields,
        overflow: Overflow,
        resolve_type: ResolutionType,
    ) -> TemporalResult<Self> {
        fields.check_year_in_safe_arithmetical_range()?;
        let era_year = EraYear::try_from_fields(calendar, fields, resolve_type)?;
        if calendar.is_iso() {
            let month_code = resolve_iso_month(calendar, fields, overflow)?;
            let day = resolve_day(
                fields.day,
                resolve_type == ResolutionType::YearMonth,
                &era_year,
                month_code,
                calendar,
            )?;
            let day = if overflow == Overflow::Constrain {
                constrain_iso_day(era_year.year, month_code.to_month_integer(), day)
            } else {
                if !is_valid_iso_day(era_year.year, month_code.to_month_integer(), day) {
                    return Err(
                        TemporalError::range().with_message("day value is not in a valid range.")
                    );
                }
                day
            };
            return Ok(Self {
                era_year,
                month_code,
                day,
            });
        }

        let month_code = resolve_non_iso_month(calendar, fields, &era_year, overflow)?;
        let day = resolve_day(
            fields.day,
            resolve_type == ResolutionType::YearMonth,
            &era_year,
            month_code,
            calendar,
        )?;

        Ok(Self {
            era_year,
            month_code,
            day,
        })
    }
}

fn resolve_day(
    day: Option<u8>,
    is_year_month: bool,
    _year: &EraYear,
    _month_code: MonthCode,
    _calendar: &Calendar,
) -> TemporalResult<u8> {
    if is_year_month {
        Ok(1)
    } else {
        day.ok_or(TemporalError::r#type().with_message("Required day field is empty."))
    }
}

pub(super) fn probe_day_for_year_month(
    calendar: &Calendar,
    year: &EraYear,
    month_code: MonthCode,
) -> u8 {
    if calendar.kind() == AnyCalendarKind::Japanese {
        match (year.arithmetic_year, month_code.to_month_integer()) {
            // Meiji begins Oct 23, 1868
            (1868, 10) => 23,
            // Taisho begins Jul 30, 1912
            (1912, 7) => 30,
            // Showa begins Dec 25, 1926
            (1926, 12) => 25,
            // Heisei begins Jan 8, 1989
            (1989, 1) => 8,
            // Reiwa begins May 1, 2019
            (2019, 5) => 1,
            _ => 1,
        }
    } else {
        // PlainYearMonth construction paths all use the first day of the calendar month.
        1
    }
}

#[derive(Debug)]
pub struct Era(pub(crate) TinyAsciiStr<16>);

// TODO(Manishearth) We should just be using arithmetic_year unconditionally.
// so that https://github.com/boa-dev/temporal/issues/448 is handled.
//
// However, ICU4X has some bugs
// (https://github.com/unicode-org/icu4x/pull/6762/, https://github.com/unicode-org/icu4x/pull/6800)
// so for now we store both.
#[derive(Debug)]
pub struct EraYear {
    pub(crate) era: Option<Era>,
    pub(crate) year: i32,
    pub(crate) arithmetic_year: i32,
}

impl EraYear {
    pub(crate) fn try_from_fields(
        calendar: &Calendar,
        partial: &CalendarFields,
        resolution_type: ResolutionType,
    ) -> TemporalResult<Self> {
        if !Calendar::calendar_has_eras(calendar.kind()) && resolution_type != ResolutionType::MonthDay {
            if let Some(year) = partial.year {
                return Ok(Self {
                    era: None,
                    year,
                    arithmetic_year: year,
                });
            }

            return Err(TemporalError::r#type()
                .with_message("Required fields missing to determine an era and year."));
        }

        match (partial.year, partial.era, partial.era_year) {
            _ if resolution_type == ResolutionType::MonthDay => {
                let day = partial
                    .day
                    .ok_or(TemporalError::r#type().with_message("MonthDay must specify day"))?;

                let arithmetic_year = Self::reference_arithmetic_year_for_month_day(
                    calendar,
                    partial.month_code,
                    day,
                )?;
                Ok(Self {
                    // We should just specify these as arithmetic years, no need
                    // to muck with eras
                    era: None,
                    arithmetic_year,
                    year: arithmetic_year,
                })
            }
            (maybe_year, Some(era), Some(era_year)) => {
                let Some(era_info) = calendar.get_era_info(&era) else {
                    return Err(TemporalError::range().with_message("Invalid era provided."));
                };
                let calculated_arith = era_info.arithmetic_year_for(era_year);
                // or a RangeError exception if the fields are sufficient but their values are internally inconsistent
                // within the calendar (e.g., when fields such as [[Month]] and [[MonthCode]] have conflicting non-unset values). For example:
                if let Some(arith) = maybe_year {
                    if calculated_arith != arith {
                        return Err(
                            TemporalError::range().with_message("Conflicting year/eraYear info")
                        );
                    }
                }
                Ok(Self {
                    year: era_year,
                    era: Some(Era(era_info.name)),
                    arithmetic_year: calculated_arith,
                })
            }
            (Some(year), None, None) => Ok(Self {
                era: None,
                year,
                arithmetic_year: year,
            }),
            _ => Err(TemporalError::r#type()
                .with_message("Required fields missing to determine an era and year.")),
        }
    }

    pub(crate) fn reference_arithmetic_year_for_month_day(
        calendar: &Calendar,
        month_code: Option<MonthCode>,
        day: u8,
    ) -> TemporalResult<i32> {
        fn find_reference_year_for_lunisolar_month_day(
            calendar: &Calendar,
            month_code: MonthCode,
            day: u8,
            start_year: i32,
            end_year: i32,
        ) -> TemporalResult<i32> {
            let reference_limit = IsoDate::new_unchecked(1972, 12, 31);
            let mut latest_on_or_before = None;
            let mut earliest_after = None;

            for arithmetic_year in start_year..=end_year {
                let Ok(date) = calendar.0.from_codes(
                    None,
                    arithmetic_year,
                    IcuMonthCode(month_code.0),
                    day,
                ) else {
                    continue;
                };
                let iso = calendar.0.to_iso(&date);
                let iso_date = IsoDate::new_unchecked(
                    Iso.extended_year(&iso),
                    Iso.month(&iso).ordinal,
                    Iso.day_of_month(&iso).0,
                );
                if calendar.month_code(&iso_date) != month_code || calendar.day(&iso_date) != day {
                    continue;
                }

                if iso_date <= reference_limit {
                    let replace = latest_on_or_before
                        .map(|(candidate, _)| candidate < iso_date)
                        .unwrap_or(true);
                    if replace {
                        latest_on_or_before = Some((iso_date, arithmetic_year));
                    }
                } else {
                    let replace = earliest_after
                        .map(|(candidate, _)| iso_date < candidate)
                        .unwrap_or(true);
                    if replace {
                        earliest_after = Some((iso_date, arithmetic_year));
                    }
                }
            }

            latest_on_or_before
                .or(earliest_after)
                .map(|(_, arithmetic_year)| arithmetic_year)
                .ok_or_else(|| {
                    TemporalError::range()
                        .with_message("Do not currently support MonthDay with this calendar")
                })
        }

        let kind = calendar.kind();

        // This behavior is required by tests, but is not yet specced.
        // https://github.com/tc39/proposal-intl-era-monthcode/issues/60
        let Some(month_code) = month_code else {
            if kind == AnyCalendarKind::Iso {
                return Ok(1972);
            } else {
                return Err(TemporalError::r#type()
                    .with_message("MonthDay must be created with a monthCode for non-ISO"));
            }
        };

        let (start_year, end_year) = match kind {
            AnyCalendarKind::Chinese => (1899, 2099),
            AnyCalendarKind::Dangi => (1899, 2049),
            _ => {
                let reference_limit = IsoDate::new_unchecked(1972, 12, 31);
                let calendar_date = calendar.0.from_iso(*reference_limit.to_icu4x().inner());
                let center_year = calendar.0.extended_year(&calendar_date);
                (center_year - 8, center_year + 8)
            }
        };

        find_reference_year_for_lunisolar_month_day(
            calendar,
            month_code,
            day,
            start_year,
            end_year,
        )
    }
}

// MonthCode constants.
const MONTH_ONE: TinyAsciiStr<4> = tinystr!(4, "M01");
const MONTH_ONE_LEAP: TinyAsciiStr<4> = tinystr!(4, "M01L");
const MONTH_TWO: TinyAsciiStr<4> = tinystr!(4, "M02");
const MONTH_TWO_LEAP: TinyAsciiStr<4> = tinystr!(4, "M02L");
const MONTH_THREE: TinyAsciiStr<4> = tinystr!(4, "M03");
const MONTH_THREE_LEAP: TinyAsciiStr<4> = tinystr!(4, "M03L");
const MONTH_FOUR: TinyAsciiStr<4> = tinystr!(4, "M04");
const MONTH_FOUR_LEAP: TinyAsciiStr<4> = tinystr!(4, "M04L");
const MONTH_FIVE: TinyAsciiStr<4> = tinystr!(4, "M05");
const MONTH_FIVE_LEAP: TinyAsciiStr<4> = tinystr!(4, "M05L");
const MONTH_SIX: TinyAsciiStr<4> = tinystr!(4, "M06");
const MONTH_SIX_LEAP: TinyAsciiStr<4> = tinystr!(4, "M06L");
const MONTH_SEVEN: TinyAsciiStr<4> = tinystr!(4, "M07");
const MONTH_SEVEN_LEAP: TinyAsciiStr<4> = tinystr!(4, "M07L");
const MONTH_EIGHT: TinyAsciiStr<4> = tinystr!(4, "M08");
const MONTH_EIGHT_LEAP: TinyAsciiStr<4> = tinystr!(4, "M08L");
const MONTH_NINE: TinyAsciiStr<4> = tinystr!(4, "M09");
const MONTH_NINE_LEAP: TinyAsciiStr<4> = tinystr!(4, "M09L");
const MONTH_TEN: TinyAsciiStr<4> = tinystr!(4, "M10");
const MONTH_TEN_LEAP: TinyAsciiStr<4> = tinystr!(4, "M10L");
const MONTH_ELEVEN: TinyAsciiStr<4> = tinystr!(4, "M11");
const MONTH_ELEVEN_LEAP: TinyAsciiStr<4> = tinystr!(4, "M11L");
const MONTH_TWELVE: TinyAsciiStr<4> = tinystr!(4, "M12");
const MONTH_TWELVE_LEAP: TinyAsciiStr<4> = tinystr!(4, "M12L");
const MONTH_THIRTEEN: TinyAsciiStr<4> = tinystr!(4, "M13");

// TODO: Handle instances where month values may be outside of valid
// bounds. In other words, it is totally possible for a value to be
// passed in that is { month: 300 } with overflow::constrain.
/// A MonthCode identifier
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonthCode(pub(crate) TinyAsciiStr<4>);

impl MonthCode {
    pub(crate) fn validate(&self, calendar: &Calendar) -> TemporalResult<()> {
        const COMMON_MONTH_CODES: [TinyAsciiStr<4>; 12] = [
            MONTH_ONE,
            MONTH_TWO,
            MONTH_THREE,
            MONTH_FOUR,
            MONTH_FIVE,
            MONTH_SIX,
            MONTH_SEVEN,
            MONTH_EIGHT,
            MONTH_NINE,
            MONTH_TEN,
            MONTH_ELEVEN,
            MONTH_TWELVE,
        ];

        const LUNAR_LEAP_MONTHS: [TinyAsciiStr<4>; 12] = [
            MONTH_ONE_LEAP,
            MONTH_TWO_LEAP,
            MONTH_THREE_LEAP,
            MONTH_FOUR_LEAP,
            MONTH_FIVE_LEAP,
            MONTH_SIX_LEAP,
            MONTH_SEVEN_LEAP,
            MONTH_EIGHT_LEAP,
            MONTH_NINE_LEAP,
            MONTH_TEN_LEAP,
            MONTH_ELEVEN_LEAP,
            MONTH_TWELVE_LEAP,
        ];

        if COMMON_MONTH_CODES.contains(&self.0) {
            return Ok(());
        }

        match calendar.identifier() {
            "chinese" | "dangi"
                if LUNAR_LEAP_MONTHS.contains(&self.0) || MONTH_THIRTEEN == self.0 =>
            {
                Ok(())
            }
            "coptic" | "ethiopic" | "ethioaa" if MONTH_THIRTEEN == self.0 => Ok(()),
            "hebrew" if MONTH_FIVE_LEAP == self.0 => Ok(()),
            _ => Err(TemporalError::range()
                .with_message("MonthCode was not valid for the current calendar.")),
        }
    }

    pub(crate) fn try_from_fields(
        calendar: &Calendar,
        fields: &CalendarFields,
    ) -> TemporalResult<Self> {
        match fields {
            CalendarFields {
                month: Some(month),
                month_code: None,
                ..
            } => {
                // TODO(manishearth) this is incorrect,
                // see https://github.com/unicode-org/icu4x/issues/6790
                let month_code = month_to_month_code(*month)?;
                month_code.validate(calendar)?;
                Ok(month_code)
            }
            CalendarFields {
                month_code: Some(month_code),
                month: None,
                ..
            } => {
                month_code.validate(calendar)?;
                Ok(*month_code)
            }
            CalendarFields {
                month: Some(month),
                month_code: Some(month_code),
                ..
            } => {
                are_month_and_month_code_resolvable(*month, month_code)?;
                month_code.validate(calendar)?;
                Ok(*month_code)
            }
            _ => Err(TemporalError::r#type()
                .with_message("Month or monthCode is required to determine date.")),
        }
    }

    /// Returns the `MonthCode` as an integer
    pub fn to_month_integer(&self) -> u8 {
        // Sometimes icu_calendar returns "und"
        // when the month is calculated to be out of range (usually for
        // out-of-astronomic range Islamic and Chinese calendars)
        //
        // Normalize to something sensible, since ascii_four_to_integer
        // will assert for non-digits.
        if self.0 == tinystr!(4, "und") {
            return 13;
        }
        ascii_four_to_integer(self.0)
    }

    /// Returns whether the `MonthCode` is a leap month.
    pub fn is_leap_month(&self) -> bool {
        let bytes = self.0.all_bytes();
        bytes[3] == b'L'
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn as_tinystr(&self) -> TinyAsciiStr<4> {
        self.0
    }

    pub fn try_from_utf8(src: &[u8]) -> TemporalResult<Self> {
        if !(3..=4).contains(&src.len()) {
            return Err(
                TemporalError::range().with_message("Month codes must have 3 or 4 characters.")
            );
        }

        let inner = TinyAsciiStr::<4>::try_from_utf8(src).map_err(|_e| TemporalError::range())?;

        let bytes = inner.all_bytes();
        if bytes[0] != b'M' {
            return Err(
                TemporalError::range().with_message("First month code character must be 'M'.")
            );
        }
        if !bytes[1].is_ascii_digit() || !bytes[2].is_ascii_digit() {
            return Err(TemporalError::range().with_message("Invalid month code digit."));
        }
        if src.len() == 4 && bytes[3] != b'L' {
            return Err(TemporalError::range().with_message("Leap month code must end with 'L'."));
        }

        Ok(Self(inner))
    }
}

impl core::str::FromStr for MonthCode {
    type Err = TemporalError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from_utf8(s.as_bytes())
    }
}

// NOTE: This is a greedy function, should handle differently for all calendars.
#[inline]
pub(crate) fn month_to_month_code(month: u8) -> TemporalResult<MonthCode> {
    if !(1..=13).contains(&month) {
        return Err(TemporalError::range().with_message("Month not in a valid range."));
    }
    let first = month / 10;
    let second = month % 10;
    let tinystr = TinyAsciiStr::<4>::try_from_raw([b'M', first + 48, second + 48, b'\0'])
        .map_err(|_| TemporalError::range().with_message("Invalid month code"))?;
    Ok(MonthCode(tinystr))
}

#[inline]
fn are_month_and_month_code_resolvable(_month: u8, _mc: &MonthCode) -> TemporalResult<()> {
    // TODO(Manishearth) month is an ordinal month, this check needs year/calendar info
    // https://github.com/unicode-org/icu4x/issues/6790
    Ok(())
}

// Potentially greedy. Need to verify for all calendars that
// the month code integer aligns with the month integer, which
// may require calendar info
#[inline]
fn ascii_four_to_integer(mc: TinyAsciiStr<4>) -> u8 {
    let bytes = mc.all_bytes();
    // Invariant: second and third character (index 1 and 2) are ascii digits.
    debug_assert!(bytes[1].is_ascii_digit());
    debug_assert!(bytes[2].is_ascii_digit());
    let first = ascii_digit_to_int(bytes[1]) * 10;
    first + ascii_digit_to_int(bytes[2])
}

#[inline]
const fn ascii_digit_to_int(ascii_digit: u8) -> u8 {
    ascii_digit - 48
}

fn resolve_iso_month(
    calendar: &Calendar,
    fields: &CalendarFields,
    overflow: Overflow,
) -> TemporalResult<MonthCode> {
    let month_code = match (fields.month_code, fields.month) {
        (None, None) => {
            return Err(TemporalError::r#type().with_message("Month or monthCode must be provided."))
        }
        (None, Some(month)) => {
            if overflow == Overflow::Constrain {
                return month_to_month_code(month.clamp(1, 12));
            }
            if !(1..=12).contains(&month) {
                return Err(
                    TemporalError::range().with_message("month value is not in a valid range.")
                );
            }
            month_to_month_code(month)?
        }
        (Some(month_code), None) => month_code,
        (Some(month_code), Some(month)) => {
            if month != month_code.to_month_integer() {
                return Err(TemporalError::range()
                    .with_message("month and monthCode could not be resolved."));
            }
            month_code
        }
    };
    month_code.validate(calendar)?;
    Ok(month_code)
}

pub(crate) fn resolve_non_iso_month(
    calendar: &Calendar,
    fields: &CalendarFields,
    era_year: &EraYear,
    overflow: Overflow,
) -> TemporalResult<MonthCode> {
    if matches!(calendar.identifier(), "chinese" | "dangi")
        && era_year.arithmetic_year.unsigned_abs() > 10_000
    {
        return match (fields.month_code, fields.month) {
            (None, None) => Err(TemporalError::r#type()
                .with_message("Month or monthCode is required to determine date.")),
            (None, Some(month)) => resolve_ordinal_month_code(calendar, era_year, month, overflow),
            (Some(month_code), None) | (Some(month_code), Some(_)) => {
                month_code.validate(calendar)?;
                if month_code.is_leap_month() {
                    if overflow == Overflow::Reject {
                        Err(TemporalError::range()
                            .with_message("month and monthCode could not be resolved."))
                    } else {
                        month_to_month_code(month_code.to_month_integer())
                    }
                } else if month_code.to_month_integer() == 13 {
                    month_to_month_code(12)
                } else {
                    Ok(month_code)
                }
            }
        };
    }

    let year_month_codes = calendar.month_codes_in_year(era_year.arithmetic_year)?;
    match (fields.month_code, fields.month) {
        (None, None) => Err(
            TemporalError::r#type().with_message("Month or monthCode is required to determine date.")
        ),
        (Some(month_code), None) => {
            month_code.validate(calendar)?;
            if year_month_codes.contains(&month_code) {
                Ok(month_code)
            } else if overflow == Overflow::Constrain {
                calendar.constrain_month_code_for_year(era_year.arithmetic_year, month_code, 1)
            } else {
                Err(TemporalError::range()
                    .with_message("month and monthCode could not be resolved."))
            }
        }
        (None, Some(month)) => resolve_ordinal_month_code(calendar, era_year, month, overflow),
        (Some(month_code), Some(month)) => {
            month_code.validate(calendar)?;
            if year_month_codes.contains(&month_code) {
                let actual_month = month_number_for_code(calendar, era_year, month_code)?;
                if actual_month != month {
                    return Err(TemporalError::range()
                        .with_message("month and monthCode could not be resolved."));
                }
                return Ok(month_code);
            }
            if overflow == Overflow::Constrain {
                return calendar
                    .constrain_month_code_for_year(era_year.arithmetic_year, month_code, 1);
            }
            Err(TemporalError::range()
                .with_message("month and monthCode could not be resolved."))
        }
    }
}

fn month_number_for_code(
    calendar: &Calendar,
    era_year: &EraYear,
    month_code: MonthCode,
) -> TemporalResult<u8> {
    let day = probe_day_for_year_month(calendar, era_year, month_code);
    let date = calendar.icu_date_from_codes(era_year, month_code, day)?;
    Ok(date.month().ordinal)
}

fn resolve_ordinal_month_code(
    calendar: &Calendar,
    era_year: &EraYear,
    month: u8,
    overflow: Overflow,
) -> TemporalResult<MonthCode> {
    if matches!(calendar.identifier(), "chinese" | "dangi")
        && era_year.arithmetic_year.unsigned_abs() > 10_000
    {
        let month = if overflow == Overflow::Constrain {
            month.clamp(1, 13)
        } else {
            month
        };
        let month_code = if month == 13 {
            month_to_month_code(12)?
        } else {
            month_to_month_code(month)?
        };
        month_code.validate(calendar)?;
        return Ok(month_code);
    }

    let month = if overflow == Overflow::Constrain {
        month.clamp(1, calendar.ordinal_months_in_year(era_year.arithmetic_year)?)
    } else {
        month
    };

    for candidate in calendar_month_code_candidates(calendar.identifier()) {
        let Ok(date) = calendar.icu_date_from_codes(era_year, candidate, 1) else {
            continue;
        };
        if date.month().ordinal == month {
            return Ok(candidate);
        }
    }

    let month_code = if matches!(calendar.identifier(), "chinese" | "dangi") && month == 13 {
        month_to_month_code(12)?
    } else {
        month_to_month_code(month)?
    };
    month_code.validate(calendar)?;
    Ok(month_code)
}

pub(super) fn calendar_month_code_candidates(identifier: &str) -> Vec<MonthCode> {
    let mut candidates = (1..=13)
        .filter_map(|month| month_to_month_code(month).ok())
        .collect::<Vec<_>>();

    if matches!(identifier, "chinese" | "dangi") {
        for month in 1..=12 {
            if let Ok(code) = MonthCode::try_from_utf8(format!("M{month:02}L").as_bytes()) {
                candidates.push(code);
            }
        }
    } else if identifier == "hebrew" {
        if let Ok(code) = MonthCode::try_from_utf8(b"M05L") {
            candidates.push(code);
        }
    }

    candidates
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use tinystr::{tinystr, TinyAsciiStr};

    use crate::{
        builtins::{
            calendar::{types::ResolutionType, CalendarFields},
            core::{calendar::Calendar, PartialDate, PlainDate},
        },
        options::Overflow,
    };

    use super::{month_to_month_code, MonthCode, ResolvedCalendarFields};

    #[test]
    fn valid_month_code() {
        let month_code = MonthCode::from_str("M01").unwrap();
        assert!(!month_code.is_leap_month());
        assert_eq!(month_code.to_month_integer(), 1);

        let month_code = MonthCode::from_str("M12").unwrap();
        assert!(!month_code.is_leap_month());
        assert_eq!(month_code.to_month_integer(), 12);

        let month_code = MonthCode::from_str("M13L").unwrap();
        assert!(month_code.is_leap_month());
        assert_eq!(month_code.to_month_integer(), 13);
    }

    #[test]
    fn invalid_month_code() {
        let _ = MonthCode::from_str("01").unwrap_err();
        let _ = MonthCode::from_str("N01").unwrap_err();
        let _ = MonthCode::from_str("M01R").unwrap_err();
        let _ = MonthCode::from_str("M1").unwrap_err();
        let _ = MonthCode::from_str("M1L").unwrap_err();
    }

    #[test]
    fn month_to_mc() {
        let mc = month_to_month_code(1).unwrap();
        assert_eq!(mc.as_str(), "M01");

        let mc = month_to_month_code(13).unwrap();
        assert_eq!(mc.as_str(), "M13");

        let _ = month_to_month_code(0).unwrap_err();
        let _ = month_to_month_code(14).unwrap_err();
    }

    #[test]
    fn day_overflow_test() {
        let bad_fields = CalendarFields {
            year: Some(2019),
            month: Some(1),
            day: Some(32),
            ..Default::default()
        };

        let cal = Calendar::default();

        let err = cal.date_from_fields(bad_fields.clone(), Overflow::Reject);
        assert!(err.is_err());
        let result = cal.date_from_fields(bad_fields, Overflow::Constrain);
        assert!(result.is_ok());
    }

    #[test]
    fn self_consistent_era_year() {
        use crate::builtins::core::calendar::era::ALL_ALLOWED_ERAS;
        use icu_calendar::AnyCalendarKind;

        for (cal, eras) in ALL_ALLOWED_ERAS {
            for era in *eras {
                let expect_str = alloc::format!("Trying {cal:?} with era {}", era.name);
                let mut calendar_fields = CalendarFields::new();

                // We want to pick some valid date. year=1 month=1, day=1 is valid for basically
                // all calendars except for Japanese, which has mid-year eras. For Japanese we pick December 31 instead
                if *cal == AnyCalendarKind::Japanese {
                    calendar_fields.month = Some(12);
                    calendar_fields.day = Some(31);
                } else {
                    calendar_fields.month = Some(1);
                    calendar_fields.day = Some(1);
                }
                calendar_fields.era = Some(TinyAsciiStr::from_str(&era.name).unwrap());
                calendar_fields.era_year = Some(1);
                let partial = PartialDate {
                    calendar_fields,
                    calendar: Calendar::new(*cal),
                };

                let plain_date =
                    PlainDate::from_partial(partial, Some(Overflow::Constrain)).expect(&expect_str);

                assert_eq!(
                    plain_date.year(),
                    era.arithmetic_year_for(1),
                    "Mismatched year/eraYear for {cal:?} and {}",
                    era.name
                );

                // Get the full partial date.
                let full_partial = CalendarFields::default()
                    .with_fallback_date(&plain_date, *cal, Overflow::Constrain)
                    .expect(&expect_str);

                let era_year = super::EraYear::try_from_fields(
                    &Calendar::new(*cal),
                    &full_partial,
                    ResolutionType::Date,
                )
                .expect(&expect_str);

                assert_eq!(
                    &*era_year.era.expect("only testing calendars with era").0,
                    &*era.name,
                    "Backcalculated era must match"
                );
                assert_eq!(era_year.year, 1, "Backcalculated era must match");
                assert_eq!(
                    era_year.arithmetic_year,
                    plain_date.year(),
                    "Backcalculated arithmetic_year must match"
                );
            }
        }
    }

    #[test]
    fn unresolved_month_and_month_code() {
        let bad_fields = CalendarFields {
            year: Some(1976),
            month: Some(11),
            month_code: Some(MonthCode(tinystr!(4, "M12"))),
            day: Some(18),
            ..Default::default()
        };

        let err = ResolvedCalendarFields::try_from_fields(
            &Calendar::ISO,
            &bad_fields,
            Overflow::Reject,
            ResolutionType::Date,
        );
        assert!(err.is_err());
    }

    #[test]
    fn missing_partial_fields() {
        let bad_fields = CalendarFields {
            year: Some(2019),
            day: Some(19),
            ..Default::default()
        };

        let err = ResolvedCalendarFields::try_from_fields(
            &Calendar::ISO,
            &bad_fields,
            Overflow::Reject,
            ResolutionType::Date,
        );
        assert!(err.is_err());

        let bad_fields = CalendarFields::default();
        let err = ResolvedCalendarFields::try_from_fields(
            &Calendar::ISO,
            &bad_fields,
            Overflow::Reject,
            ResolutionType::Date,
        );
        assert!(err.is_err());
    }

    #[test]
    fn constrain_missing_chinese_leap_month_code_to_common_month() {
        let fields = CalendarFields::new()
            .with_year(2022)
            .with_month_code(MonthCode::from_str("M02L").unwrap());
        let resolved = super::ResolvedCalendarFields::try_from_fields(
            &Calendar::CHINESE,
            &fields,
            Overflow::Constrain,
            ResolutionType::YearMonth,
        )
        .unwrap();
        assert_eq!(resolved.month_code.as_str(), "M02");
    }

    #[test]
    fn keep_month_code_authoritative_when_constraining_year_change() {
        let fields = CalendarFields::new()
            .with_year(2024)
            .with_month(7)
            .with_month_code(MonthCode::from_str("M06L").unwrap())
            .with_day(1);
        let resolved = super::ResolvedCalendarFields::try_from_fields(
            &Calendar::CHINESE,
            &fields,
            Overflow::Constrain,
            ResolutionType::Date,
        )
        .unwrap();
        assert_eq!(resolved.month_code.as_str(), "M06");
    }
}
