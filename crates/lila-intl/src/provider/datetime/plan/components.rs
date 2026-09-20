use crate::datetime::*;
use crate::TimeZoneNameStyle;

use super::super::pattern::{Field, NameWidth, Pattern, PeriodKind, Token};

pub(in crate::provider::datetime) fn components(pattern: &Pattern) -> DateTimeComponents {
    from_fields(pattern.tokens.iter().filter_map(|token| match token {
        Token::Field(field) => Some(*field),
        Token::Literal(_) => None,
    }))
}
pub(super) fn from_fields(fields: impl Iterator<Item = Field>) -> DateTimeComponents {
    let mut result = DateTimeComponents::default();
    for field in fields {
        match field {
            Field::Era(width) => result.era = Some(text(width)),
            Field::Year(width) | Field::RelatedYear(width) => result.year = Some(numeric(width)),
            Field::CyclicYear(_) => {
                result.year.get_or_insert(DateTimeNumericWidth::Numeric);
            }
            Field::Month { width, .. } => {
                result.month = Some(match width {
                    1 => DateTimeMonthWidth::Numeric,
                    2 => DateTimeMonthWidth::TwoDigit,
                    3 => DateTimeMonthWidth::Short,
                    4 => DateTimeMonthWidth::Long,
                    _ => DateTimeMonthWidth::Narrow,
                })
            }
            Field::Day(width) => result.day = Some(numeric(width)),
            Field::Weekday { width, .. } => result.weekday = Some(text(width)),
            Field::DayPeriod {
                kind: PeriodKind::Flexible,
                width,
            } => result.day_period = Some(text(width)),
            Field::DayPeriod {
                kind: PeriodKind::AmPm | PeriodKind::NoonMidnight,
                ..
            } => {}
            Field::Hour { width, .. } => result.hour = Some(numeric(width)),
            Field::Minute(width) => result.minute = Some(numeric(width)),
            Field::Second(width) => result.second = Some(numeric(width)),
            Field::Fraction(width) => result.fractional_second_digits = Some(width),
            Field::ZoneName(style) => result.time_zone_name = Some(style),
        }
    }
    result
}
fn numeric(width: u8) -> DateTimeNumericWidth {
    if width == 2 {
        DateTimeNumericWidth::TwoDigit
    } else {
        DateTimeNumericWidth::Numeric
    }
}
fn text(width: NameWidth) -> DateTimeTextWidth {
    match width {
        NameWidth::Abbreviated | NameWidth::Short => DateTimeTextWidth::Short,
        NameWidth::Wide => DateTimeTextWidth::Long,
        NameWidth::Narrow => DateTimeTextWidth::Narrow,
    }
}

pub(in crate::provider::datetime) fn score(
    requested: DateTimeComponents,
    candidate: DateTimeComponents,
) -> i32 {
    let number = |value: Option<DateTimeNumericWidth>| {
        value.map(|width| match width {
            DateTimeNumericWidth::TwoDigit => 0,
            DateTimeNumericWidth::Numeric => 1,
        })
    };
    let text = |value: Option<DateTimeTextWidth>| {
        value.map(|width| match width {
            DateTimeTextWidth::Narrow => 2,
            DateTimeTextWidth::Short => 3,
            DateTimeTextWidth::Long => 4,
        })
    };
    let month = |value: Option<DateTimeMonthWidth>| {
        value.map(|width| match width {
            DateTimeMonthWidth::TwoDigit => 0,
            DateTimeMonthWidth::Numeric => 1,
            DateTimeMonthWidth::Narrow => 2,
            DateTimeMonthWidth::Short => 3,
            DateTimeMonthWidth::Long => 4,
        })
    };
    let mut score = 0;
    for (wanted, found) in [
        (text(requested.weekday), text(candidate.weekday)),
        (text(requested.era), text(candidate.era)),
        (number(requested.year), number(candidate.year)),
        (month(requested.month), month(candidate.month)),
        (number(requested.day), number(candidate.day)),
        (text(requested.day_period), text(candidate.day_period)),
        (number(requested.hour), number(candidate.hour)),
        (number(requested.minute), number(candidate.minute)),
        (number(requested.second), number(candidate.second)),
        (
            requested
                .fractional_second_digits
                .map(|value| i32::from(value.get())),
            candidate
                .fractional_second_digits
                .map(|value| i32::from(value.get())),
        ),
    ] {
        score -= match (wanted, found) {
            (None, Some(_)) => 20,
            (Some(_), None) => 120,
            (None, None) => 0,
            (Some(wanted), Some(found)) => match (found - wanted).clamp(-2, 2) {
                -2 => 8,
                -1 => 6,
                0 => 0,
                1 => 3,
                _ => 6,
            },
        };
    }
    score - zone_penalty(requested.time_zone_name, candidate.time_zone_name)
}
fn zone_penalty(wanted: Option<TimeZoneNameStyle>, found: Option<TimeZoneNameStyle>) -> i32 {
    use TimeZoneNameStyle::*;
    match (wanted, found) {
        (None, None) => 0,
        (None, Some(_)) => 20,
        (Some(_), None) => 120,
        (Some(a), Some(b)) if a == b => 0,
        (Some(Short | ShortGeneric), Some(ShortOffset))
        | (Some(Long | LongGeneric), Some(LongOffset)) => 1,
        (Some(Short | ShortGeneric), Some(LongOffset)) => 4,
        (Some(Long | LongGeneric), Some(ShortOffset)) => 9,
        (Some(Short), Some(Long))
        | (Some(ShortGeneric), Some(LongGeneric))
        | (Some(ShortOffset), Some(LongOffset)) => 3,
        (Some(Long), Some(Short))
        | (Some(LongGeneric), Some(ShortGeneric))
        | (Some(LongOffset), Some(ShortOffset)) => 8,
        _ => 120,
    }
}

#[derive(Clone, Copy)]
pub(super) enum Required {
    Any,
    Date,
    YearMonth,
    MonthDay,
    Time,
}
pub(super) fn filter(mut fields: DateTimeComponents, required: Required) -> DateTimeComponents {
    let date = matches!(required, Required::Any | Required::Date);
    let year = date || matches!(required, Required::YearMonth);
    let month = year || matches!(required, Required::MonthDay);
    let time = matches!(required, Required::Any | Required::Time);
    if !date {
        fields.weekday = None;
    }
    if !year {
        fields.era = None;
        fields.year = None;
    }
    if !month {
        fields.month = None;
    }
    if !date && !matches!(required, Required::MonthDay) {
        fields.day = None;
    }
    if !time {
        fields.day_period = None;
        fields.hour = None;
        fields.minute = None;
        fields.second = None;
        fields.fractional_second_digits = None;
    }
    fields.time_zone_name = None;
    fields
}
pub(super) fn any(fields: DateTimeComponents) -> bool {
    fields.weekday.is_some()
        || fields.year.is_some()
        || fields.month.is_some()
        || fields.day.is_some()
        || fields.day_period.is_some()
        || fields.hour.is_some()
        || fields.minute.is_some()
        || fields.second.is_some()
        || fields.fractional_second_digits.is_some()
}
pub(super) fn defaults(
    mut fields: DateTimeComponents,
    required: Required,
    inherit_all: bool,
    defaults: DateTimeDefaults,
) -> Option<DateTimeComponents> {
    let any_present = any(fields);
    let relevant = filter(fields, required);
    if !inherit_all {
        fields = relevant;
    }
    if !any(relevant) {
        if !inherit_all && any_present {
            return None;
        }
        let date = matches!(
            required,
            Required::Date | Required::YearMonth | Required::MonthDay
        ) || matches!(required, Required::Any)
            && matches!(defaults, DateTimeDefaults::All | DateTimeDefaults::Date);
        let time = matches!(required, Required::Time)
            || matches!(required, Required::Any)
                && matches!(defaults, DateTimeDefaults::All | DateTimeDefaults::Time);
        if date {
            if !matches!(required, Required::MonthDay) {
                fields.year = Some(DateTimeNumericWidth::Numeric);
            }
            fields.month = Some(DateTimeMonthWidth::Numeric);
            if !matches!(required, Required::YearMonth) {
                fields.day = Some(DateTimeNumericWidth::Numeric);
            }
        }
        if time {
            fields.hour = Some(DateTimeNumericWidth::Numeric);
            fields.minute = Some(DateTimeNumericWidth::Numeric);
            fields.second = Some(DateTimeNumericWidth::Numeric);
        }
    }
    Some(fields)
}
