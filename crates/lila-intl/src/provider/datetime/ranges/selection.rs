use crate::datetime::*;

use super::super::{
    calendar::{Fields, Year},
    pattern::{Field, Pattern, PeriodKind, Token},
    plan::{
        adjustment::{adjust_widths, matching_clock, with_cycle},
        components::{components, score},
        ValidatedPlan,
    },
    profile::Interval,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Difference {
    Era,
    Year,
    Month,
    Day,
    AmPm,
    DayPeriod,
    Hour,
    Minute,
    Second,
    Fraction,
}

pub(super) fn greatest(
    plan: &ValidatedPlan<'_>,
    pattern: &Pattern,
    start: Fields,
    end: Fields,
) -> Result<Option<Difference>, DateTimeFormatError> {
    let fields: Vec<_> = pattern
        .tokens
        .iter()
        .filter_map(|token| match token {
            Token::Field(field) => Some(*field),
            Token::Literal(_) => None,
        })
        .collect();
    let finest = fields.iter().filter_map(|field| rank(*field)).max();
    let Some(finest) = finest else {
        return Ok(None);
    };
    let am_pm = fields.iter().any(|field| {
        matches!(
            field,
            Field::DayPeriod {
                kind: PeriodKind::AmPm | PeriodKind::NoonMidnight,
                ..
            }
        )
    });
    let flexible = fields.iter().any(|field| {
        matches!(
            field,
            Field::DayPeriod {
                kind: PeriodKind::Flexible,
                ..
            }
        )
    });
    let fraction = fields
        .iter()
        .find_map(|field| match field {
            Field::Fraction(digits) => Some(digits.get()),
            _ => None,
        })
        .unwrap_or(3);
    for difference in [
        Difference::Era,
        Difference::Year,
        Difference::Month,
        Difference::Day,
        Difference::AmPm,
        Difference::DayPeriod,
        Difference::Hour,
        Difference::Minute,
        Difference::Second,
        Difference::Fraction,
    ] {
        if difference > finest {
            break;
        }
        let different = match difference {
            Difference::Era => era(start.year) != era(end.year),
            Difference::Year => year(start.year) != year(end.year),
            Difference::Month => (start.month, start.leap_month) != (end.month, end.leap_month),
            Difference::Day => start.day != end.day,
            Difference::AmPm => am_pm && (start.hour < 12) != (end.hour < 12),
            Difference::DayPeriod => {
                flexible
                    && plan.locale.periods.select(
                        start.hour,
                        start.minute,
                        start.second,
                        start.nanosecond,
                    )? != plan.locale.periods.select(
                        end.hour,
                        end.minute,
                        end.second,
                        end.nanosecond,
                    )?
            }
            Difference::Hour => start.hour != end.hour,
            Difference::Minute => start.minute != end.minute,
            Difference::Second => start.second != end.second,
            Difference::Fraction => {
                start.nanosecond / 10_u32.pow(9 - u32::from(fraction))
                    != end.nanosecond / 10_u32.pow(9 - u32::from(fraction))
            }
        };
        if different {
            return Ok(Some(difference));
        }
    }
    Ok(None)
}

fn era(year: Year) -> u8 {
    match year {
        Year::Era { era, .. } => era,
        Year::Cyclic { .. } => 0,
    }
}
fn year(year: Year) -> i32 {
    match year {
        Year::Era { year, .. } => year,
        Year::Cyclic { related, .. } => related,
    }
}
fn rank(field: Field) -> Option<Difference> {
    Some(match field {
        Field::Era(_) => Difference::Era,
        Field::Year(_) | Field::RelatedYear(_) | Field::CyclicYear(_) => Difference::Year,
        Field::Month { .. } => Difference::Month,
        Field::Day(_) | Field::Weekday { .. } => Difference::Day,
        Field::DayPeriod {
            kind: PeriodKind::AmPm | PeriodKind::NoonMidnight,
            ..
        } => Difference::AmPm,
        Field::DayPeriod {
            kind: PeriodKind::Flexible,
            ..
        } => Difference::DayPeriod,
        Field::Hour { .. } => Difference::Hour,
        Field::Minute(_) => Difference::Minute,
        Field::Second(_) => Difference::Second,
        Field::Fraction(_) => Difference::Fraction,
        Field::ZoneName(_) => return None,
    })
}

pub(super) fn select<'p>(
    plan: &'p ValidatedPlan<'_>,
    base: &Pattern,
    difference: Difference,
) -> Option<(&'p Interval, Pattern)> {
    let wanted = components(base);
    let wanted_years = year_fields(base);
    let mut selected = None;
    let mut best_score = i32::MIN;
    for interval in &plan.calendar.intervals {
        if !matches_difference(interval.difference, difference)
            || !matching_clock(&interval.pattern, plan.hour_cycle)
        {
            continue;
        }
        let mut pattern = with_cycle(interval.pattern.clone(), plan.hour_cycle);
        adjust_widths(&mut pattern, wanted);
        let found = components(&pattern);
        if !same_fields(wanted, found) || year_fields(&pattern) != wanted_years {
            continue;
        }
        let candidate_score = score(wanted, found);
        if candidate_score > best_score {
            best_score = candidate_score;
            selected = Some((interval, pattern));
        }
    }
    selected
}

fn matches_difference(symbol: char, difference: Difference) -> bool {
    match difference {
        Difference::Era => symbol == 'G',
        Difference::Year => symbol == 'y',
        Difference::Month => symbol == 'M',
        Difference::Day => symbol == 'd',
        Difference::AmPm => symbol == 'a',
        Difference::DayPeriod => symbol == 'B',
        Difference::Hour => matches!(symbol, 'h' | 'H'),
        Difference::Minute => symbol == 'm',
        Difference::Second => symbol == 's',
        Difference::Fraction => false,
    }
}

fn same_fields(left: DateTimeComponents, right: DateTimeComponents) -> bool {
    [
        left.weekday.is_some() == right.weekday.is_some(),
        left.era.is_some() == right.era.is_some(),
        left.year.is_some() == right.year.is_some(),
        left.month.is_some() == right.month.is_some(),
        left.day.is_some() == right.day.is_some(),
        left.day_period.is_some() == right.day_period.is_some(),
        left.hour.is_some() == right.hour.is_some(),
        left.minute.is_some() == right.minute.is_some(),
        left.second.is_some() == right.second.is_some(),
        left.fractional_second_digits.is_some() == right.fractional_second_digits.is_some(),
        left.time_zone_name.is_some() == right.time_zone_name.is_some(),
    ]
    .into_iter()
    .all(|same| same)
}

// Numeric calendar years, related Gregorian years and cyclic names share a
// DateTimeFormat option but are distinct output fields. An incompatible
// interval must use the locale's fallback with the selected endpoint pattern.
fn year_fields(pattern: &Pattern) -> [bool; 3] {
    let mut fields = [false; 3];
    for token in &pattern.tokens {
        match token {
            Token::Field(Field::Year(_)) => fields[0] = true,
            Token::Field(Field::RelatedYear(_)) => fields[1] = true,
            Token::Field(Field::CyclicYear(_)) => fields[2] = true,
            Token::Literal(_)
            | Token::Field(
                Field::Era(_)
                | Field::Month { .. }
                | Field::Day(_)
                | Field::Weekday { .. }
                | Field::DayPeriod { .. }
                | Field::Hour { .. }
                | Field::Minute(_)
                | Field::Second(_)
                | Field::Fraction(_)
                | Field::ZoneName(_),
            ) => {}
        }
    }
    fields
}
