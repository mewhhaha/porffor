use crate::datetime::DateTimeFormatError;

use super::{
    names::NameKey,
    pattern::{DayPeriod, Field, NameContext, NameWidth, Pattern, Token},
    profile::{invalid, Calendar, Locale, Profile},
};

impl Profile {
    pub(super) fn validate_names(&self) -> Result<(), DateTimeFormatError> {
        for locale in &self.locales {
            validate_calendar(locale, &locale.gregorian, false)?;
            validate_calendar(locale, &locale.chinese, true)?;
        }
        Ok(())
    }
}

fn validate_calendar(
    locale: &Locale,
    calendar: &Calendar,
    cyclic: bool,
) -> Result<(), DateTimeFormatError> {
    if calendar.append_era.is_some() == cyclic {
        return Err(invalid(
            "calendar era append availability disagrees with its year kind",
        ));
    }
    let names = &calendar.names;
    for width in [NameWidth::Abbreviated, NameWidth::Wide, NameWidth::Narrow] {
        if cyclic {
            for year in 1..=60 {
                names.get(NameKey::CyclicYear(width, year))?;
            }
        } else {
            for era in 0..=1 {
                names.get(NameKey::Era(width, era))?;
            }
        }
        for context in [NameContext::Format, NameContext::Standalone] {
            for month in 1..=12 {
                names.get(NameKey::Month(context, width, month))?;
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
        validate_pattern(calendar, &style.date, cyclic)?;
        validate_pattern(calendar, &style.time, cyclic)?;
    }
    for pattern in &calendar.available {
        validate_pattern(calendar, pattern, cyclic)?;
    }
    for interval in &calendar.intervals {
        validate_pattern(calendar, &interval.pattern, cyclic)?;
    }
    Ok(())
}

fn validate_pattern(
    calendar: &Calendar,
    pattern: &Pattern,
    cyclic: bool,
) -> Result<(), DateTimeFormatError> {
    for token in &pattern.tokens {
        let Token::Field(field) = token else {
            continue;
        };
        match field {
            Field::Era(_) if cyclic => {
                return Err(invalid("cyclic calendar contains an era pattern"));
            }
            Field::RelatedYear(_) | Field::CyclicYear(_) if !cyclic => {
                return Err(invalid("era calendar contains a cyclic-year pattern"));
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
    for (field, _) in &pattern.numbering {
        if let Some(field) = field {
            if !pattern.tokens.iter().any(|token| matches!(token, Token::Field(value) if value.numbering_symbol() == Some(*field))) { return Err(invalid("numbering override does not select a pattern field")); }
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
