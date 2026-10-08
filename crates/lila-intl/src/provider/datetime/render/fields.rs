use crate::datetime::{DateTimeFormatError, DateTimeHourCycle};

use super::super::{
    calendar::{CalendarId, CalendarYear, Fields},
    names::NameKey,
    pattern::{DayPeriod, Field, NameContext, NameWidth, Pattern, PeriodKind},
    plan::ValidatedPlan,
    profile::{invalid, AlgorithmicField, Calendar, Locale, Profile},
};

pub(super) fn format(
    profile: &Profile,
    plan: &ValidatedPlan<'_>,
    pattern: &Pattern,
    field: Field,
    fields: Fields,
) -> Result<String, DateTimeFormatError> {
    format_fields(
        profile,
        plan.locale,
        plan.calendar,
        &plan.numbering,
        pattern,
        field,
        fields,
    )
}

// Both the admitted request path and private expanded-profile acceptance use
// this same renderer; the latter cannot publish a public calendar identifier.
pub(in crate::provider::datetime) fn format_fields(
    profile: &Profile,
    locale: &Locale,
    calendar: &Calendar,
    numbering: &str,
    pattern: &Pattern,
    field: Field,
    fields: Fields,
) -> Result<String, DateTimeFormatError> {
    let names = &calendar.names;
    let name = |key| names.get(key).map(str::to_owned);
    let number = |value: i64, width| {
        let system = pattern
            .numbering
            .iter()
            .find(|(symbol, _)| symbol.is_some() && *symbol == field.numbering_symbol())
            .or_else(|| {
                pattern
                    .numbering
                    .iter()
                    .find(|(symbol, _)| symbol.is_none())
            })
            .map(|(_, system)| system.as_str())
            .unwrap_or(numbering);
        if let Some(algorithm) = profile.algorithmic.get(system) {
            return match algorithm {
                AlgorithmicField::FiniteDay(labels) => {
                    if !matches!(field, Field::Day(_)) {
                        return Err(invalid(
                            "finite algorithmic numbering used for a different field",
                        ));
                    }
                    usize::try_from(value - 1)
                        .ok()
                        .and_then(|index| labels.get(index))
                        .cloned()
                        .ok_or_else(|| invalid("day outside the finite numbering field"))
                }
                AlgorithmicField::JapaneseYear { first_year } => {
                    let CalendarYear::Era { year, .. } = fields.year else {
                        return Err(invalid(
                            "Japanese year numbering used for a cyclic calendar",
                        ));
                    };
                    if fields.calendar != CalendarId::Japanese || !matches!(field, Field::Year(_)) {
                        return Err(invalid(
                            "Japanese year numbering used for a different field",
                        ));
                    }
                    // Inspect the complete year before width2's modulo100.
                    // Year101 must use decimal fallback, never the first-year label.
                    if display_year(i64::from(year)) == 1 {
                        Ok(first_year.clone())
                    } else {
                        super::positional(profile, locale, "latn", value, 1)
                    }
                }
            };
        }
        super::positional(profile, locale, system, value, width)
    };
    match field {
        Field::Era(width) => match fields.year {
            CalendarYear::Era { name: era, .. } => name(era.key(fields.calendar, width)?),
            CalendarYear::Cyclic { .. } => Err(invalid("cyclic calendar pattern contains era")),
        },
        Field::Year(width) => {
            let year = match fields.year {
                CalendarYear::Era { year, .. } => i64::from(year),
                CalendarYear::Cyclic { year, .. } => i64::from(year),
            };
            // FormatDateTimePattern converts nonpositive calendar years before
            // applying numeric or two-digit formatting. Buddhist has one era
            // and can expose such years; Gregorian era years are positive.
            let year = display_year(year);
            number(if width == 2 { year % 100 } else { year }, width)
        }
        Field::RelatedYear(width) => match fields.year {
            CalendarYear::Cyclic { related_iso, .. } => number(i64::from(related_iso), width),
            CalendarYear::Era { .. } => Err(invalid("era calendar pattern contains related year")),
        },
        Field::CyclicYear(width) => match fields.year {
            CalendarYear::Cyclic { year, .. } => name(NameKey::CyclicYear(width, year)),
            CalendarYear::Era { .. } => Err(invalid("era calendar pattern contains cyclic year")),
        },
        Field::Month { context, width } => {
            let month = if fields.calendar == CalendarId::Hebrew {
                // Hebrew numeric requests keep real month names (CLDR-15510).
                // Source name indices remain separate from standard month codes.
                let width = if width <= 2 {
                    NameWidth::Abbreviated
                } else {
                    month_width(width)
                };
                name(fields.month.key(fields.calendar, context, width))?
            } else if width <= 2 {
                number(i64::from(fields.month.name_index(fields.calendar)), width)?
            } else {
                name(
                    fields
                        .month
                        .key(fields.calendar, context, month_width(width)),
                )?
            };
            if !fields.month.uses_leap_placeholder(fields.calendar) {
                return Ok(month);
            }
            let key = if width <= 2 {
                NameKey::NumericLeapMonth
            } else {
                NameKey::LeapMonth(context, month_width(width))
            };
            Ok(names.get(key)?.replace("{0}", &month))
        }
        Field::Day(width) => number(i64::from(fields.day), width),
        Field::Weekday { context, width } => name(NameKey::Weekday(context, width, fields.weekday)),
        Field::NumericWeekday { width, .. } => {
            let first = locale.first_weekday.sunday_index();
            number(i64::from((fields.weekday + 7 - first) % 7 + 1), width)
        }
        Field::DayPeriod { kind, width } => {
            let am_pm = if fields.hour < 12 {
                DayPeriod::Am
            } else {
                DayPeriod::Pm
            };
            let period = match kind {
                PeriodKind::AmPm => am_pm,
                PeriodKind::NoonMidnight => {
                    let on_hour =
                        fields.minute == 0 && fields.second == 0 && fields.nanosecond == 0;
                    let period = match (fields.hour, on_hour) {
                        (12, true) => DayPeriod::Noon,
                        _ => am_pm,
                    };
                    if names
                        .optional(NameKey::Period(NameContext::Format, width, period))
                        .is_some()
                    {
                        period
                    } else {
                        am_pm
                    }
                }
                PeriodKind::Flexible => locale.periods.select(
                    fields.hour,
                    fields.minute,
                    fields.second,
                    fields.nanosecond,
                )?,
            };
            name(NameKey::Period(NameContext::Format, width, period))
        }
        Field::Hour { cycle, width } => {
            let hour = match cycle {
                DateTimeHourCycle::H11 => fields.hour % 12,
                DateTimeHourCycle::H12 => {
                    let value = fields.hour % 12;
                    if value == 0 {
                        12
                    } else {
                        value
                    }
                }
                DateTimeHourCycle::H23 => fields.hour,
                DateTimeHourCycle::H24 => {
                    if fields.hour == 0 {
                        24
                    } else {
                        fields.hour
                    }
                }
            };
            number(i64::from(hour), width)
        }
        Field::Minute(width) => number(i64::from(fields.minute), width),
        Field::Second(width) => number(i64::from(fields.second), width),
        Field::Fraction(width) => number(
            i64::from(fields.nanosecond / 10_u32.pow(9 - u32::from(width.get()))),
            width.get(),
        ),
        Field::ZoneName(_) => Err(invalid("zone names bypassed their transition snapshot")),
    }
}

fn month_width(width: u8) -> NameWidth {
    match width {
        3 => NameWidth::Abbreviated,
        4 => NameWidth::Wide,
        5 => NameWidth::Narrow,
        _ => unreachable!("validated textual month width"),
    }
}

fn display_year(year: i64) -> i64 {
    if year <= 0 {
        1 - year
    } else {
        year
    }
}
