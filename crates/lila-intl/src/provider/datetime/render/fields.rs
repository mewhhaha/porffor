use crate::datetime::{DateTimeFormatError, DateTimeHourCycle};

use super::super::{
    calendar::{Fields, Year},
    names::NameKey,
    pattern::{DayPeriod, Field, NameContext, NameWidth, Pattern, PeriodKind},
    plan::ValidatedPlan,
    profile::{invalid, Profile},
};

pub(super) fn format(
    profile: &Profile,
    plan: &ValidatedPlan<'_>,
    pattern: &Pattern,
    field: Field,
    fields: Fields,
) -> Result<String, DateTimeFormatError> {
    let names = &plan.calendar.names;
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
            .unwrap_or(&plan.numbering);
        if let Some(labels) = profile.algorithmic.get(system) {
            if !matches!(field, Field::Day(_)) {
                return Err(invalid(
                    "finite algorithmic numbering used for a different field",
                ));
            }
            return usize::try_from(value - 1)
                .ok()
                .and_then(|index| labels.get(index))
                .cloned()
                .ok_or_else(|| invalid("day outside the finite numbering field"));
        }
        super::positional(profile, plan.locale, system, value, width)
    };
    match field {
        Field::Era(width) => match fields.year {
            Year::Era { era, .. } => name(NameKey::Era(width, era)),
            Year::Cyclic { .. } => Err(invalid("cyclic calendar pattern contains era")),
        },
        Field::Year(width) => {
            let year = match fields.year {
                Year::Era { year, .. } => i64::from(year),
                Year::Cyclic { year, .. } => i64::from(year),
            };
            number(if width == 2 { year % 100 } else { year }, width)
        }
        Field::RelatedYear(width) => match fields.year {
            Year::Cyclic { related, .. } => number(i64::from(related), width),
            Year::Era { .. } => Err(invalid("era calendar pattern contains related year")),
        },
        Field::CyclicYear(width) => match fields.year {
            Year::Cyclic { year, .. } => name(NameKey::CyclicYear(width, year)),
            Year::Era { .. } => Err(invalid("era calendar pattern contains cyclic year")),
        },
        Field::Month { context, width } => {
            let month = if width <= 2 {
                number(i64::from(fields.month), width)?
            } else {
                name(NameKey::Month(context, month_width(width), fields.month))?
            };
            if !fields.leap_month {
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
                PeriodKind::Flexible => plan.locale.periods.select(
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
