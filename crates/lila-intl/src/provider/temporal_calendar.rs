//! Temporal calendar arithmetic and projection over pinned ICU4X algorithms.

use icu_calendar::{
    types::{MonthCode, RataDie, YearInfo},
    AnyCalendar, AnyCalendarKind, Date, DateDuration,
};

use crate::temporal_calendar::{
    TemporalCalendar, TemporalCalendarAnswer, TemporalCalendarDate, TemporalCalendarDateFields,
    TemporalCalendarDuration, TemporalCalendarEra, TemporalCalendarError, TemporalCalendarOverflow,
    TemporalCalendarQuery, TemporalCalendarRangeError, TemporalCalendarRequest, TemporalIsoDate,
    TemporalMonthCode, TEMPORAL_CALENDAR_ISO_DAYS_LIMIT,
};

const RATA_DIE_UNIX_EPOCH: i64 = 719_163;

fn any_calendar(calendar: TemporalCalendar) -> AnyCalendar {
    AnyCalendar::new(match calendar {
        TemporalCalendar::Gregorian => AnyCalendarKind::Gregorian,
        TemporalCalendar::Iso8601 => AnyCalendarKind::Iso,
        TemporalCalendar::Chinese => AnyCalendarKind::Chinese,
        TemporalCalendar::Buddhist => AnyCalendarKind::Buddhist,
        TemporalCalendar::Indian => AnyCalendarKind::Indian,
        TemporalCalendar::Persian => AnyCalendarKind::Persian,
        TemporalCalendar::Roc => AnyCalendarKind::Roc,
        TemporalCalendar::Dangi => AnyCalendarKind::Dangi,
        TemporalCalendar::IslamicCivil => AnyCalendarKind::HijriTabularTypeIIFriday,
        TemporalCalendar::Coptic => AnyCalendarKind::Coptic,
        TemporalCalendar::Ethioaa => AnyCalendarKind::EthiopianAmeteAlem,
        TemporalCalendar::Ethiopic => AnyCalendarKind::Ethiopian,
        TemporalCalendar::Hebrew => AnyCalendarKind::Hebrew,
        TemporalCalendar::IslamicTabular => AnyCalendarKind::HijriTabularTypeIIThursday,
        TemporalCalendar::IslamicUmmAlQura => AnyCalendarKind::HijriUmmAlQura,
        TemporalCalendar::Japanese => AnyCalendarKind::Japanese,
    })
}

pub(super) fn answer(
    request: &TemporalCalendarRequest,
) -> Result<TemporalCalendarAnswer, TemporalCalendarError> {
    let calendar = request.calendar();
    let any = any_calendar(calendar);
    match request.query() {
        TemporalCalendarQuery::Fields { date } => {
            let Some(iso) = iso_date(date) else {
                return Ok(TemporalCalendarAnswer::RangeError(
                    TemporalCalendarRangeError::InvalidDate,
                ));
            };
            project(iso.to_calendar(any), calendar)
        }
        TemporalCalendarQuery::FromFields { fields, overflow } => {
            let date = match from_fields(&any, calendar, fields, overflow) {
                Ok(date) => date,
                Err(error) => return Ok(TemporalCalendarAnswer::RangeError(error)),
            };
            project(date, calendar)
        }
        TemporalCalendarQuery::DateAdd {
            date,
            duration,
            overflow,
        } => {
            let Some(date) = iso_date(date) else {
                return Ok(TemporalCalendarAnswer::RangeError(
                    TemporalCalendarRangeError::InvalidDate,
                ));
            };
            let date = match date_add(date.to_calendar(any), calendar, duration, overflow) {
                Ok(date) => date,
                Err(error) => return Ok(TemporalCalendarAnswer::RangeError(error)),
            };
            project(date, calendar)
        }
    }
}

fn iso_date(date: TemporalIsoDate) -> Option<Date<icu_calendar::cal::Iso>> {
    let date = Date::try_new_iso(date.year(), date.month(), date.day()).ok()?;
    in_temporal_range(date.to_rata_die()).then_some(date)
}

fn in_temporal_range(rata_die: RataDie) -> bool {
    rata_die
        .to_i64_date()
        .checked_sub(RATA_DIE_UNIX_EPOCH)
        .is_some_and(|day| {
            (-TEMPORAL_CALENDAR_ISO_DAYS_LIMIT - 1..=TEMPORAL_CALENDAR_ISO_DAYS_LIMIT)
                .contains(&day)
        })
}

fn project(
    date: Date<AnyCalendar>,
    calendar: TemporalCalendar,
) -> Result<TemporalCalendarAnswer, TemporalCalendarError> {
    if !in_temporal_range(date.to_rata_die()) {
        return Ok(TemporalCalendarAnswer::RangeError(
            TemporalCalendarRangeError::OutOfRange,
        ));
    }
    let iso = date.to_iso();
    let (era, era_year) = if calendar == TemporalCalendar::Iso8601 {
        (None, None)
    } else if calendar == TemporalCalendar::Japanese && iso.extended_year() < 1873 {
        let year = iso.extended_year();
        if year > 0 {
            (Some(TemporalCalendarEra::JapaneseCe), Some(year))
        } else {
            (Some(TemporalCalendarEra::JapaneseBce), Some(1 - year))
        }
    } else {
        match date.year() {
            YearInfo::Era(era_year) => {
                let era = TemporalCalendarEra::from_icu(calendar, era_year.era.as_str()).ok_or(
                    TemporalCalendarError::InvalidCalendarData(
                        "pinned ICU calendar returned an unrecognized era",
                    ),
                )?;
                (Some(era), Some(era_year.year))
            }
            YearInfo::Cyclic(_) => (None, None),
            _ => {
                return Err(TemporalCalendarError::InvalidCalendarData(
                    "unknown ICU year kind",
                ));
            }
        }
    };
    let month_info = date.month();
    // The Temporal era/monthCode proposal uses the canonical ICU standard
    // code. Hebrew formatting codes distinguish CLDR's printed month labels,
    // while the standard code preserves Adar I as M05L.
    let (month_code_number, month_code_leap) =
        month_info
            .standard_code
            .parsed()
            .ok_or(TemporalCalendarError::InvalidCalendarData(
                "pinned ICU calendar returned an invalid standard month code",
            ))?;
    Ok(TemporalCalendarAnswer::Date(
        TemporalCalendarDate::new_projected(
            TemporalIsoDate::new(
                iso.extended_year(),
                iso.month().ordinal,
                iso.day_of_month().0,
            ),
            arithmetic_year(&date)
                .map_err(|_| TemporalCalendarError::InvalidCalendarData("unknown ICU year kind"))?,
            era_year,
            era,
            month_info.ordinal,
            TemporalMonthCode::new(month_code_number, month_code_leap)
                .map_err(TemporalCalendarError::InvalidRequest)?,
            date.day_of_month().0,
            date.months_in_year(),
            date.days_in_month(),
            date.days_in_year(),
            date.day_of_year().0,
            date.is_in_leap_year(),
        ),
    ))
}

fn arithmetic_year(date: &Date<AnyCalendar>) -> Result<i32, TemporalCalendarRangeError> {
    match date.year() {
        YearInfo::Cyclic(year) => Ok(year.related_iso),
        YearInfo::Era(_) => Ok(date.extended_year()),
        _ => Err(TemporalCalendarRangeError::InvalidDate),
    }
}

/// Era fields select an arithmetic year; they do not constrain the resulting
/// date to the historical era interval. NonISOResolveFields intentionally
/// erases them after this conversion (including Japanese midyear eras).
fn resolve_year(
    calendar: TemporalCalendar,
    fields: TemporalCalendarDateFields,
) -> Result<i32, TemporalCalendarRangeError> {
    use TemporalCalendarEra as Era;
    use TemporalCalendarRangeError as Error;
    let derived = match (fields.era(), fields.era_year()) {
        (None, None) => return fields.year().ok_or(Error::InvalidDate),
        (Some(era), Some(year)) if era.allowed_in(calendar) => match era {
            Era::Bce | Era::JapaneseBce | Era::Broc | Era::Bh => 1_i32.checked_sub(year),
            Era::Meiji => year.checked_add(1867),
            Era::Taisho => year.checked_add(1911),
            Era::Showa => year.checked_add(1925),
            Era::Heisei => year.checked_add(1988),
            Era::Reiwa => year.checked_add(2018),
            Era::Ethioaa if calendar == TemporalCalendar::Ethiopic => year.checked_sub(5500),
            Era::Ce
            | Era::JapaneseCe
            | Era::Buddhist
            | Era::Shaka
            | Era::Persian
            | Era::Roc
            | Era::Ah
            | Era::Coptic
            | Era::Ethioaa
            | Era::EthiopicAm
            | Era::Hebrew => Some(year),
        }
        .ok_or(Error::OutOfRange)?,
        _ => return Err(Error::InvalidDate),
    };
    if fields.year().is_some_and(|year| year != derived) {
        return Err(Error::InvalidDate);
    }
    Ok(derived)
}

fn year_start(
    calendar: &AnyCalendar,
    id: TemporalCalendar,
    year: i32,
) -> Result<Date<AnyCalendar>, TemporalCalendarRangeError> {
    use TemporalCalendar as Cal;
    use TemporalCalendarRangeError as Error;
    // All selected calendars have at least 353 days per year and epochs
    // within 6000 ISO years. Beyond this conservative bound no part of the
    // requested year can intersect Temporal's +/-100,000,000-day interval.
    // Reject before entering calendar algorithms with bounded i32 arithmetic.
    if year.unsigned_abs() > 300_000 {
        return Err(Error::OutOfRange);
    }
    let result = match id {
        Cal::Chinese => {
            Date::try_new_chinese_with_calendar(year, 1, 1, icu_calendar::cal::Chinese::new())
                .map(Date::to_any)
                .map_err(|_| Error::InvalidDate)
        }
        Cal::Dangi => {
            Date::try_new_dangi_with_calendar(year, 1, 1, icu_calendar::cal::Dangi::new())
                .map(Date::to_any)
                .map_err(|_| Error::InvalidDate)
        }
        Cal::Gregorian | Cal::Iso8601 | Cal::Japanese => Date::try_new_iso(year, 1, 1)
            .map(|date| date.to_calendar(calendar.clone()))
            .map_err(|_| Error::InvalidDate),
        Cal::Buddhist
        | Cal::Indian
        | Cal::Persian
        | Cal::Roc
        | Cal::IslamicCivil
        | Cal::Coptic
        | Cal::Ethioaa
        | Cal::Ethiopic
        | Cal::Hebrew
        | Cal::IslamicTabular
        | Cal::IslamicUmmAlQura => {
            let (era, era_year) = match id {
                Cal::Roc if year <= 0 => (Some("broc"), 1 - year),
                Cal::IslamicCivil | Cal::IslamicTabular | Cal::IslamicUmmAlQura if year <= 0 => {
                    (Some("bh"), 1 - year)
                }
                Cal::Ethiopic if year <= 0 => (Some("aa"), year + 5500),
                Cal::Gregorian
                | Cal::Iso8601
                | Cal::Chinese
                | Cal::Buddhist
                | Cal::Indian
                | Cal::Persian
                | Cal::Roc
                | Cal::Dangi
                | Cal::IslamicCivil
                | Cal::Coptic
                | Cal::Ethioaa
                | Cal::Ethiopic
                | Cal::Hebrew
                | Cal::IslamicTabular
                | Cal::IslamicUmmAlQura
                | Cal::Japanese => (None, year),
            };
            Date::try_new_from_codes(
                era,
                era_year,
                MonthCode::new_normal(1).expect("M01"),
                1,
                calendar.clone(),
            )
            .map_err(|_| Error::InvalidDate)
        }
    };
    result
}

fn from_fields(
    calendar: &AnyCalendar,
    calendar_id: TemporalCalendar,
    fields: TemporalCalendarDateFields,
    overflow: TemporalCalendarOverflow,
) -> Result<Date<AnyCalendar>, TemporalCalendarRangeError> {
    use TemporalCalendarRangeError as Error;
    let year = resolve_year(calendar_id, fields)?;
    let first = year_start(calendar, calendar_id, year)?;
    let month_start = if let Some(code) = fields.month_code() {
        let month = select_month_code(first, calendar_id, code, overflow)?;
        if fields
            .month()
            .is_some_and(|ordinal| ordinal != month.month().ordinal)
        {
            return Err(Error::InvalidDate);
        }
        month
    } else {
        let month = fields.month().ok_or(Error::InvalidDate)?;
        if month == 0 {
            return Err(Error::InvalidDate);
        }
        let maximum = first.months_in_year();
        if month > maximum && overflow == TemporalCalendarOverflow::Reject {
            return Err(Error::Overflow);
        }
        first.added(DateDuration::new(
            0,
            i32::from(month.min(maximum)) - 1,
            0,
            0,
        ))
    };
    if fields.day() == 0 {
        return Err(Error::InvalidDate);
    }
    let maximum = month_start.days_in_month();
    if fields.day() > maximum && overflow == TemporalCalendarOverflow::Reject {
        return Err(Error::Overflow);
    }
    let date = month_start.added(DateDuration::new(
        0,
        0,
        0,
        i32::from(fields.day().min(maximum)) - 1,
    ));
    if in_temporal_range(date.to_rata_die()) {
        Ok(date)
    } else {
        Err(Error::OutOfRange)
    }
}

fn valid_month_code(calendar: TemporalCalendar, code: TemporalMonthCode) -> bool {
    use TemporalCalendar as Cal;
    if !code.is_leap() && code.month() <= 12 {
        return true;
    }
    match calendar {
        Cal::Chinese | Cal::Dangi => code.is_leap() && code.month() <= 12,
        Cal::Hebrew => code.is_leap() && code.month() == 5,
        Cal::Coptic | Cal::Ethioaa | Cal::Ethiopic => !code.is_leap() && code.month() == 13,
        Cal::Gregorian
        | Cal::Iso8601
        | Cal::Buddhist
        | Cal::Indian
        | Cal::Persian
        | Cal::Roc
        | Cal::IslamicCivil
        | Cal::IslamicTabular
        | Cal::IslamicUmmAlQura
        | Cal::Japanese => false,
    }
}

fn select_month_code(
    first: Date<AnyCalendar>,
    calendar: TemporalCalendar,
    wanted: TemporalMonthCode,
    overflow: TemporalCalendarOverflow,
) -> Result<Date<AnyCalendar>, TemporalCalendarRangeError> {
    use TemporalCalendarRangeError as Error;
    if !valid_month_code(calendar, wanted) {
        return Err(Error::InvalidDate);
    }
    let mut fallback = None;
    let fallback_code = if calendar == TemporalCalendar::Hebrew && wanted.is_leap() {
        (6, false) // missing Adar I constrains forward to common Adar
    } else {
        (wanted.month(), false) // Chinese/Dangi constrain backward
    };
    for ordinal in 0..first.months_in_year() {
        let candidate = first
            .clone()
            .added(DateDuration::new(0, i32::from(ordinal), 0, 0));
        let code = candidate.month().standard_code.parsed();
        if code == Some((wanted.month(), wanted.is_leap())) {
            return Ok(candidate);
        }
        if code == Some(fallback_code) {
            fallback = Some(candidate);
        }
    }
    if overflow == TemporalCalendarOverflow::Reject {
        return Err(Error::Overflow);
    }
    fallback.ok_or(Error::InvalidDate)
}

fn date_add(
    date: Date<AnyCalendar>,
    calendar: TemporalCalendar,
    duration: TemporalCalendarDuration,
    overflow: TemporalCalendarOverflow,
) -> Result<Date<AnyCalendar>, TemporalCalendarRangeError> {
    use TemporalCalendarRangeError as Error;
    let year = i64::from(arithmetic_year(&date)?)
        .checked_add(duration.years())
        .and_then(|year| i32::try_from(year).ok())
        .ok_or(Error::OutOfRange)?;
    let first = year_start(date.calendar(), calendar, year)?;
    let (number, leap) = date
        .month()
        .standard_code
        .parsed()
        .ok_or(Error::InvalidDate)?;
    let code = TemporalMonthCode::new(number, leap).map_err(|_| Error::InvalidDate)?;
    let first = select_month_code(first, calendar, code, overflow)?;
    let months = i32::try_from(duration.months()).map_err(|_| Error::OutOfRange)?;
    // Even the shortest calendar month has five days; a displacement larger
    // than this cannot stay in the 200,000,000-day supported interval.
    if months.unsigned_abs() > 40_000_002 {
        return Err(Error::OutOfRange);
    }
    let month_start = first.added(DateDuration::new(0, months, 0, 0));
    let maximum = month_start.days_in_month();
    let day = date.day_of_month().0;
    if day > maximum && overflow == TemporalCalendarOverflow::Reject {
        return Err(Error::Overflow);
    }
    let offset = duration
        .weeks()
        .checked_mul(7)
        .and_then(|days| days.checked_add(duration.days()))
        .and_then(|days| days.checked_add(i64::from(day.min(maximum)) - 1))
        .ok_or(Error::OutOfRange)?;
    let rata_die = month_start
        .to_rata_die()
        .to_i64_date()
        .checked_add(offset)
        .ok_or(Error::OutOfRange)?;
    let rata_die = RataDie::new(rata_die);
    if !in_temporal_range(rata_die) {
        return Err(Error::OutOfRange);
    }
    Ok(Date::from_rata_die(rata_die, date.calendar().clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TemporalCalendarQuery, TemporalCalendarRequest};

    fn date(calendar: TemporalCalendar, year: i32, month: u8, day: u8) -> TemporalCalendarDate {
        let request = TemporalCalendarRequest::new(
            calendar,
            TemporalCalendarQuery::Fields {
                date: TemporalIsoDate::new(year, month, day),
            },
        );
        match answer(&request).unwrap() {
            TemporalCalendarAnswer::Date(date) => date,
            TemporalCalendarAnswer::RangeError(error) => panic!("{error:?}"),
        }
    }

    #[test]
    fn all_calendar_kinds_project_real_dates_and_roundtrip_the_wire() {
        for calendar in TemporalCalendar::ALL {
            let fields = date(calendar, 2024, 3, 15);
            assert_eq!(
                TemporalCalendarAnswer::decode(&fields.encode()).unwrap(),
                TemporalCalendarAnswer::Date(fields),
                "{}",
                calendar.identifier()
            );
            assert_eq!(fields.iso(), TemporalIsoDate::new(2024, 3, 15));
            assert!(fields.months_in_year() >= 12);
            assert!((1..=31).contains(&fields.days_in_month()));
            assert!((1..=fields.days_in_year()).contains(&fields.day_of_year()));
        }
    }

    #[test]
    fn dangi_projection_and_construction_share_pinned_calendar_arithmetic() {
        let expected = date(TemporalCalendar::Dangi, 2000, 1, 1);
        assert_eq!(expected.year(), 1999);
        assert_eq!(expected.month(), 11);
        assert_eq!(expected.day(), 25);
        let answer = answer(&TemporalCalendarRequest::new(
            TemporalCalendar::Dangi,
            TemporalCalendarQuery::FromFields {
                fields: crate::TemporalCalendarDateFields::new(
                    Some(expected.year()),
                    None,
                    None,
                    Some(expected.month()),
                    Some(expected.month_code()),
                    31,
                ),
                overflow: TemporalCalendarOverflow::Constrain,
            },
        ))
        .unwrap();
        let TemporalCalendarAnswer::Date(reconstructed) = answer else {
            panic!("expected constrained Dangi date, got {answer:?}");
        };
        assert_eq!(reconstructed.month_code(), expected.month_code());
        assert_eq!(reconstructed.day(), 30);
    }

    #[test]
    fn calendar_date_add_preserves_the_calendar_month_and_constrains_day() {
        let answer = answer(&TemporalCalendarRequest::new(
            TemporalCalendar::Hebrew,
            TemporalCalendarQuery::DateAdd {
                date: TemporalIsoDate::new(2023, 9, 16),
                duration: TemporalCalendarDuration::new(0, 1, 0, 0),
                overflow: TemporalCalendarOverflow::Constrain,
            },
        ))
        .unwrap();
        let TemporalCalendarAnswer::Date(date) = answer else {
            panic!("expected Hebrew date, got {answer:?}");
        };
        assert_eq!(date.year(), 5784);
        assert_eq!(date.month_code().month(), 2);
    }
}

#[cfg(test)]
mod edge_tests {
    use super::*;
    use TemporalCalendar as Cal;
    use TemporalCalendarOverflow::{Constrain, Reject};

    fn projected(calendar: Cal, year: i32, month: u8, day: u8) -> TemporalCalendarDate {
        let iso = Date::try_new_iso(year, month, day).unwrap();
        let TemporalCalendarAnswer::Date(fields) =
            project(iso.to_calendar(any_calendar(calendar)), calendar).unwrap()
        else {
            panic!("date should be in range");
        };
        fields
    }

    fn fields(
        year: i32,
        month: Option<u8>,
        code: Option<(u8, bool)>,
        day: u8,
    ) -> TemporalCalendarDateFields {
        TemporalCalendarDateFields::new(
            Some(year),
            None,
            None,
            month,
            code.map(|(month, leap)| TemporalMonthCode::new(month, leap).unwrap()),
            day,
        )
    }

    #[test]
    fn lunisolar_projection_distinguishes_month_ordinal_and_code() {
        for (calendar, iso, year, ordinal, code) in [
            (Cal::Chinese, (2023, 3, 22), 2023, 3, (2, true)),
            (Cal::Chinese, (2023, 4, 20), 2023, 4, (3, false)),
            (Cal::Hebrew, (2023, 3, 1), 5783, 6, (6, false)),
            (Cal::Hebrew, (2024, 3, 1), 5784, 6, (5, true)),
            (Cal::Hebrew, (2024, 3, 15), 5784, 7, (6, false)),
        ] {
            let result = projected(calendar, iso.0, iso.1, iso.2);
            assert_eq!((result.year(), result.month()), (year, ordinal));
            assert_eq!(
                (result.month_code().month(), result.month_code().is_leap()),
                code
            );
            let rebuilt = from_fields(
                &any_calendar(calendar),
                calendar,
                fields(year, Some(ordinal), Some(code), result.day()),
                Reject,
            )
            .unwrap();
            assert_eq!(
                rebuilt.to_iso(),
                Date::try_new_iso(iso.0, iso.1, iso.2).unwrap()
            );
        }
    }

    #[test]
    fn month_code_validation_and_missing_leap_month_rules_are_calendar_specific() {
        for overflow in [Constrain, Reject] {
            for (calendar, year, code) in [
                (Cal::Gregorian, 2024, (13, false)),
                (Cal::Gregorian, 2024, (2, true)),
                (Cal::Hebrew, 5784, (6, true)),
                (Cal::Chinese, 2024, (13, true)),
            ] {
                assert!(from_fields(
                    &any_calendar(calendar),
                    calendar,
                    fields(year, None, Some(code), 1),
                    overflow
                )
                .is_err());
            }
            assert!(from_fields(
                &any_calendar(Cal::Hebrew),
                Cal::Hebrew,
                fields(5784, Some(6), Some((6, false)), 1),
                overflow
            )
            .is_err());
        }
        for (calendar, year, absent, expected) in [
            (Cal::Hebrew, 5783, (5, true), (6, false)),
            (Cal::Chinese, 2024, (2, true), (2, false)),
            (Cal::Dangi, 2024, (2, true), (2, false)),
        ] {
            let input = fields(year, None, Some(absent), 1);
            let result = from_fields(&any_calendar(calendar), calendar, input, Constrain).unwrap();
            assert_eq!(result.month().standard_code.parsed(), Some(expected));
            assert!(from_fields(&any_calendar(calendar), calendar, input, Reject).is_err());
        }
    }

    #[test]
    fn era_fields_select_year_leniently_but_must_agree_with_supplied_year() {
        let calendar = any_calendar(Cal::Japanese);
        let input = TemporalCalendarDateFields::new(
            None,
            Some(TemporalCalendarEra::Reiwa),
            Some(1),
            Some(1),
            None,
            1,
        );
        let result = from_fields(&calendar, Cal::Japanese, input, Reject).unwrap();
        assert_eq!(result.to_iso(), Date::try_new_iso(2019, 1, 1).unwrap());
        assert_eq!(
            projected(Cal::Japanese, 2019, 1, 1).era(),
            Some(TemporalCalendarEra::Heisei)
        );
        let mismatch = TemporalCalendarDateFields::new(
            Some(2020),
            Some(TemporalCalendarEra::Reiwa),
            Some(1),
            Some(1),
            None,
            1,
        );
        assert!(from_fields(&calendar, Cal::Japanese, mismatch, Constrain).is_err());
        for (era, era_year) in [(Some(TemporalCalendarEra::Reiwa), None), (None, Some(1))] {
            let partial =
                TemporalCalendarDateFields::new(Some(2020), era, era_year, Some(1), None, 1);
            assert!(from_fields(&calendar, Cal::Japanese, partial, Reject).is_err());
            let missing = TemporalCalendarDateFields::new(None, era, era_year, Some(1), None, 1);
            assert!(from_fields(&calendar, Cal::Japanese, missing, Reject).is_err());
        }
        assert_eq!(
            projected(Cal::Japanese, 1872, 12, 31).era(),
            Some(TemporalCalendarEra::JapaneseCe)
        );
        assert_eq!(projected(Cal::Japanese, 1873, 1, 1).era_year(), Some(6));
    }

    #[test]
    fn date_add_clamps_after_year_and_month_steps_before_adding_days() {
        let date = Date::try_new_iso(2024, 1, 31)
            .unwrap()
            .to_calendar(any_calendar(Cal::Gregorian));
        let duration = TemporalCalendarDuration::new(0, 1, 0, 1);
        let result = date_add(date.clone(), Cal::Gregorian, duration, Constrain).unwrap();
        assert_eq!(result.to_iso(), Date::try_new_iso(2024, 3, 1).unwrap());
        assert!(date_add(date, Cal::Gregorian, duration, Reject).is_err());
        let leap = from_fields(
            &any_calendar(Cal::Hebrew),
            Cal::Hebrew,
            fields(5784, None, Some((5, true)), 30),
            Reject,
        )
        .unwrap();
        let result = date_add(
            leap.clone(),
            Cal::Hebrew,
            TemporalCalendarDuration::new(1, 0, 0, 0),
            Constrain,
        )
        .unwrap();
        assert_eq!(arithmetic_year(&result).unwrap(), 5785);
        assert_eq!(result.month().standard_code.parsed(), Some((6, false)));
        assert_eq!(result.day_of_month().0, 29);
        assert!(date_add(
            leap,
            Cal::Hebrew,
            TemporalCalendarDuration::new(1, 0, 0, 0),
            Reject
        )
        .is_err());
    }

    #[test]
    fn every_calendar_roundtrips_fields_across_epochs_and_temporal_limits() {
        for calendar in Cal::ALL {
            for (year, month, day) in [
                (-100, 4, 20),
                (1, 1, 1),
                (1872, 12, 31),
                (2024, 3, 22),
                (275760, 9, 13),
                (-271821, 4, 19),
            ] {
                let projected = projected(calendar, year, month, day);
                let input = TemporalCalendarDateFields::new(
                    Some(projected.year()),
                    None,
                    None,
                    Some(projected.month()),
                    Some(projected.month_code()),
                    projected.day(),
                );
                let rebuilt = from_fields(&any_calendar(calendar), calendar, input, Reject)
                    .unwrap_or_else(|error| panic!("{calendar:?} {year}-{month}-{day}: {error:?}"));
                assert_eq!(
                    rebuilt.to_iso(),
                    Date::try_new_iso(year, month, day).unwrap()
                );
            }
        }
        assert!(!in_temporal_range(RataDie::new(i64::MIN)));
        assert!(!in_temporal_range(RataDie::new(i64::MAX)));
    }
}
