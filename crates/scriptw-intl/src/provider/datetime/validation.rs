use crate::datetime::DateTimeFormatError;

use super::{
    calendar::CalendarId,
    names::{MonthYearType, NameKey},
    pattern::{DayPeriod, Field, NameContext, NameWidth, Pattern, Token},
    profile::{invalid, AlgorithmicField, Calendar, Locale, Profile},
};

impl Profile {
    pub(super) fn validate_names(&self) -> Result<(), DateTimeFormatError> {
        for locale in &self.locales {
            for (kind, calendar) in locale.calendars() {
                validate_calendar(self, locale, calendar, kind)?;
            }
        }
        Ok(())
    }
}

fn validate_calendar(
    profile: &Profile,
    locale: &Locale,
    calendar: &Calendar,
    kind: CalendarId,
) -> Result<(), DateTimeFormatError> {
    let cyclic = kind.is_cyclic();
    if calendar.append_era.is_some() == cyclic {
        return Err(invalid(
            "calendar era append availability disagrees with its year kind",
        ));
    }
    if calendar.kind.month_names() != kind.month_names() {
        return Err(invalid("calculation and physical month domains disagree"));
    }
    let names = &calendar.names;
    for width in [NameWidth::Abbreviated, NameWidth::Wide, NameWidth::Narrow] {
        if cyclic {
            for year in 1..=60 {
                names.get(NameKey::CyclicYear(width, year))?;
            }
        } else {
            for era in kind.eras() {
                names.get(era.key(kind, width)?)?;
            }
        }
        for context in [NameContext::Format, NameContext::Standalone] {
            for month in 1..=kind.month_names() {
                names.get(NameKey::Month(context, width, month, None))?;
            }
            if kind == CalendarId::Hebrew {
                names.get(NameKey::Month(context, width, 7, Some(MonthYearType::Leap)))?;
            }
            for weekday in 0..=6 {
                names.get(NameKey::Weekday(context, width, weekday))?;
            }
            if cyclic {
                placeholder(names.get(NameKey::LeapMonth(context, width))?)?;
            }
        }
        for period in [DayPeriod::Am, DayPeriod::Pm]
            .into_iter()
            .chain(locale.periods.periods())
        {
            names.get(NameKey::Period(NameContext::Format, width, period))?;
        }
    }
    if cyclic {
        placeholder(names.get(NameKey::NumericLeapMonth)?)?;
    }
    for style in &calendar.styles {
        validate_pattern(profile, calendar, kind, &style.date, false)?;
        validate_pattern(profile, calendar, kind, &style.time, false)?;
    }
    for pattern in &calendar.available {
        validate_pattern(profile, calendar, kind, pattern, false)?;
    }
    for interval in &calendar.intervals {
        validate_pattern(profile, calendar, kind, &interval.pattern, true)?;
    }
    Ok(())
}

fn validate_pattern(
    profile: &Profile,
    calendar: &Calendar,
    kind: CalendarId,
    pattern: &Pattern,
    interval: bool,
) -> Result<(), DateTimeFormatError> {
    for token in &pattern.tokens {
        let Token::Field(field) = token else {
            continue;
        };
        match field {
            Field::Era(_) if kind.is_cyclic() => {
                return Err(invalid("cyclic calendar contains an era pattern"));
            }
            Field::RelatedYear(_) | Field::CyclicYear(_) if !kind.is_cyclic() => {
                return Err(invalid("era calendar contains a cyclic-year pattern"));
            }
            Field::NumericWeekday { .. } if !interval => {
                return Err(invalid(
                    "numeric weekday requires its sourced interval skeleton",
                ));
            }
            Field::NumericWeekday { .. } => {
                if !pattern
                    .skeleton
                    .iter()
                    .any(|value| matches!(value, Field::Weekday { .. }))
                {
                    return Err(invalid(
                        "numeric interval weekday has no sourced textual skeleton field",
                    ));
                }
            }
            Field::Weekday {
                context,
                width: NameWidth::Short,
            } => {
                for weekday in 0..=6 {
                    calendar
                        .names
                        .get(NameKey::Weekday(*context, NameWidth::Short, weekday))?;
                }
            }
            _ => {}
        }
    }
    for (field, system) in &pattern.numbering {
        if let Some(field) = field {
            if !pattern.tokens.iter().any(|token| matches!(token, Token::Field(value) if value.numbering_symbol() == Some(*field))) {
                return Err(invalid("numbering override does not select a pattern field"));
            }
        }
        if let Some(algorithm) = profile.algorithmic.get(system) {
            let permitted = match algorithm {
                AlgorithmicField::FiniteDay(_) => *field == Some('d'),
                AlgorithmicField::JapaneseYear { .. } => {
                    kind == CalendarId::Japanese && *field == Some('y')
                }
            };
            if !permitted {
                return Err(invalid(
                    "algorithmic numbering has the wrong calendar or field",
                ));
            }
        }
    }
    Ok(())
}

fn placeholder(pattern: &str) -> Result<(), DateTimeFormatError> {
    if pattern.matches("{0}").count() != 1 || pattern.replace("{0}", "").contains(['{', '}']) {
        return Err(invalid("invalid leap-month placeholder"));
    }
    Ok(())
}
