use super::super::{
    pattern::{Field, NameWidth, Pattern, PeriodKind, Token},
    profile::Calendar,
};
use crate::datetime::*;
use crate::TimeZoneNameStyle;

pub(in crate::provider::datetime) fn matching_clock(
    pattern: &Pattern,
    desired: DateTimeHourCycle,
) -> bool {
    pattern.tokens.iter().all(|token| match token {
        Token::Field(Field::Hour { cycle, .. }) => {
            cycle.is_twelve_hour() == desired.is_twelve_hour()
        }
        _ => true,
    })
}
pub(in crate::provider::datetime) fn with_cycle(
    mut pattern: Pattern,
    desired: DateTimeHourCycle,
) -> Pattern {
    let symbol = |cycle| match cycle {
        DateTimeHourCycle::H11 => 'K',
        DateTimeHourCycle::H12 => 'h',
        DateTimeHourCycle::H23 => 'H',
        DateTimeHourCycle::H24 => 'k',
    };
    for token in &mut pattern.tokens {
        if let Token::Field(Field::Hour { cycle, .. }) = token {
            let previous = symbol(*cycle);
            for (field, _) in &mut pattern.numbering {
                if *field == Some(previous) {
                    *field = Some(symbol(desired));
                }
            }
            *cycle = desired;
        }
    }
    for field in &mut pattern.skeleton {
        if let Field::Hour { cycle, .. } = field {
            *cycle = desired;
        }
    }
    pattern
}
pub(super) fn has_date(fields: DateTimeComponents) -> bool {
    fields.weekday.is_some()
        || fields.era.is_some()
        || fields.year.is_some()
        || fields.month.is_some()
        || fields.day.is_some()
}
pub(super) fn has_time(fields: DateTimeComponents) -> bool {
    fields.day_period.is_some()
        || fields.hour.is_some()
        || fields.minute.is_some()
        || fields.second.is_some()
        || fields.fractional_second_digits.is_some()
}
pub(super) fn style_index(style: DateTimeStyle) -> usize {
    match style {
        DateTimeStyle::Full => 0,
        DateTimeStyle::Long => 1,
        DateTimeStyle::Medium => 2,
        DateTimeStyle::Short => 3,
    }
}
pub(super) fn combination_style(fields: DateTimeComponents) -> usize {
    match (fields.month, fields.weekday) {
        (Some(DateTimeMonthWidth::Long), Some(_)) => 0,
        (Some(DateTimeMonthWidth::Long), None) => 1,
        (Some(DateTimeMonthWidth::Short), _) => 2,
        _ => 3,
    }
}
pub(in crate::provider::datetime) fn adjust_widths(
    pattern: &mut Pattern,
    wanted: DateTimeComponents,
) {
    for token in &mut pattern.tokens {
        if let Token::Field(field) = token {
            *field = adjust_field(*field, wanted, &pattern.skeleton);
        }
    }
}
pub(super) fn adjust_field(
    original: Field,
    wanted: DateTimeComponents,
    skeleton: &[Field],
) -> Field {
    let mut field = original;
    let number = |width| match width {
        DateTimeNumericWidth::Numeric => 1,
        DateTimeNumericWidth::TwoDigit => 2,
    };
    let text = |width| match width {
        DateTimeTextWidth::Narrow => NameWidth::Narrow,
        DateTimeTextWidth::Short => NameWidth::Abbreviated,
        DateTimeTextWidth::Long => NameWidth::Wide,
    };
    match &mut field {
        Field::Era(width) => {
            if let Some(value) = wanted.era {
                *width = text(value);
            }
        }
        Field::Year(width) | Field::RelatedYear(width) => {
            if let Some(value) = wanted.year {
                *width = number(value);
            }
        }
        Field::CyclicYear(_) => {}
        Field::Month { width, .. } => {
            if let Some(value) = wanted.month {
                let value = match value {
                    DateTimeMonthWidth::Numeric => 1,
                    DateTimeMonthWidth::TwoDigit => 2,
                    DateTimeMonthWidth::Short => 3,
                    DateTimeMonthWidth::Long => 4,
                    DateTimeMonthWidth::Narrow => 5,
                };
                if (*width <= 2) == (value <= 2) {
                    *width = value;
                }
            }
        }
        Field::Day(width) => {
            if let Some(value) = wanted.day {
                *width = number(value);
            }
        }
        Field::Weekday { width, .. } => {
            if let Some(value) = wanted.weekday {
                *width = text(value);
            }
        }
        Field::DayPeriod {
            kind: PeriodKind::Flexible,
            width,
        } => {
            if let Some(value) = wanted.day_period {
                *width = text(value);
            }
        }
        Field::DayPeriod {
            kind: PeriodKind::AmPm | PeriodKind::NoonMidnight,
            ..
        } => {}
        Field::Hour { width, .. } => {
            if wanted.hour == Some(DateTimeNumericWidth::TwoDigit) {
                *width = 2;
            }
        }
        Field::Minute(width) => {
            if wanted.minute == Some(DateTimeNumericWidth::TwoDigit) {
                *width = 2;
            }
        }
        Field::Second(width) => {
            if wanted.second == Some(DateTimeNumericWidth::TwoDigit) {
                *width = 2;
            }
        }
        Field::Fraction(_) | Field::ZoneName(_) => {}
    }
    // LDML Matching Skeletons: an exact requested/skeleton width lets the
    // locale pattern override that width. Numeric/textual families never cross.
    if !matches!(
        field,
        Field::Hour { .. } | Field::Minute(_) | Field::Second(_)
    ) && skeleton.iter().any(|reference| {
        component(*reference) == component(field) && length(*reference) == length(field)
    }) {
        original
    } else {
        field
    }
}
fn component(field: Field) -> DateTimePartKind {
    match field.part() {
        DateTimePartKind::RelatedYear | DateTimePartKind::YearName => DateTimePartKind::Year,
        kind => kind,
    }
}
fn length(field: Field) -> u8 {
    let text = |width| match width {
        NameWidth::Abbreviated => 3,
        NameWidth::Wide => 4,
        NameWidth::Narrow => 5,
        NameWidth::Short => 6,
    };
    match field {
        Field::Era(width)
        | Field::CyclicYear(width)
        | Field::Weekday { width, .. }
        | Field::DayPeriod { width, .. } => text(width),
        Field::Year(width)
        | Field::RelatedYear(width)
        | Field::Month { width, .. }
        | Field::Day(width)
        | Field::Hour { width, .. }
        | Field::Minute(width)
        | Field::Second(width) => width,
        Field::Fraction(width) => width.get(),
        Field::ZoneName(_) => 0,
    }
}

pub(super) fn add_zone(
    calendar: &Calendar,
    pattern: &mut Pattern,
    zone: TimeZoneNameStyle,
) -> Result<(), DateTimeFormatError> {
    let mut found = false;
    for token in &mut pattern.tokens {
        if let Token::Field(Field::ZoneName(style)) = token {
            *style = zone;
            found = true;
        }
    }
    if !found {
        *pattern = calendar
            .append_zone
            .combine(pattern, &Pattern::single(Field::ZoneName(zone)))?;
    }
    Ok(())
}
