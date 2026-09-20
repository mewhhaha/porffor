use crate::datetime::*;

use super::super::{
    pattern::{Field, NameWidth, Pattern, PeriodKind, Token},
    profile::{invalid, Calendar, Locale},
};
use super::{
    adjustment::{
        add_zone, adjust_field, adjust_widths, combination_style, has_date, has_time,
        matching_clock, style_index, with_cycle,
    },
    components::{components, from_fields, score},
    plain, Format,
};

pub(super) struct Candidates<'p> {
    calendar: &'p Calendar,
    cycle: DateTimeHourCycle,
    values: Vec<Format>,
}

impl<'p> Candidates<'p> {
    pub(super) fn new(
        locale: &Locale,
        calendar: &'p Calendar,
        cycle: DateTimeHourCycle,
        numbering: &str,
    ) -> Result<Self, DateTimeFormatError> {
        let decimal = locale
            .decimal
            .get(numbering)
            .ok_or_else(|| invalid("missing decimal separator"))?;
        let mut patterns = Vec::new();
        for available in &calendar.available {
            if matching_clock(available, cycle) {
                patterns.push(with_cycle(available.clone(), cycle));
            }
        }
        for style in &calendar.styles {
            patterns.push(style.date.clone());
            if matching_clock(&style.time, cycle) {
                patterns.push(with_cycle(style.time.clone(), cycle));
            }
        }
        // A standalone scalar field has no locale-dependent ordering or
        // punctuation. Its names and digits still come from this profile.
        for field in [
            Field::Minute(1),
            Field::Second(1),
            Field::DayPeriod {
                kind: PeriodKind::Flexible,
                width: NameWidth::Abbreviated,
            },
        ] {
            patterns.push(Pattern::single(field));
        }
        for count in 1..=3 {
            patterns.push(Pattern::single(Field::Fraction(
                DateTimeFractionalDigits::new(count)?,
            )));
        }
        let original = patterns.len();
        for index in 0..original {
            if patterns[index]
                .tokens
                .iter()
                .any(|token| matches!(token, Token::Field(Field::Second(_))))
            {
                for count in 1..=3 {
                    let mut pattern = patterns[index].clone();
                    let mut tokens = Vec::new();
                    for token in pattern.tokens {
                        let second = matches!(token, Token::Field(Field::Second(_)));
                        tokens.push(token);
                        if second {
                            tokens.push(Token::Literal(decimal.clone()));
                            tokens.push(Token::Field(Field::Fraction(
                                DateTimeFractionalDigits::new(count)?,
                            )));
                        }
                    }
                    pattern.tokens = tokens;
                    pattern
                        .skeleton
                        .push(Field::Fraction(DateTimeFractionalDigits::new(count)?));
                    patterns.push(pattern);
                }
            }
        }
        let dates: Vec<_> = patterns
            .iter()
            .filter(|pattern| {
                let fields = components(pattern);
                has_date(fields) && !has_time(fields) && fields.time_zone_name.is_none()
            })
            .cloned()
            .collect();
        let times: Vec<_> = patterns
            .iter()
            .filter(|pattern| {
                let fields = components(pattern);
                !has_date(fields) && has_time(fields)
            })
            .cloned()
            .collect();
        let mut values: Vec<_> = patterns.into_iter().map(plain).collect();
        for date in &dates {
            let style = &calendar.styles[combination_style(components(date))];
            for time in &times {
                let pattern = style.at_time.combine(time, date)?;
                values.push(Format {
                    components: components(&pattern),
                    pattern,
                    date: Some(date.clone()),
                    time: Some(time.clone()),
                    range_glue: Some(style.standard.clone()),
                });
            }
        }
        Ok(Self {
            calendar,
            cycle,
            values,
        })
    }

    pub(super) fn best(
        &self,
        requested: DateTimeComponents,
    ) -> Result<Format, DateTimeFormatError> {
        let mut best = None;
        let mut best_score = i32::MIN;
        for candidate in &self.values {
            // Width and zone families are scored without materializing their
            // Cartesian product. The exact requested zone style dominates
            // every other style in the same candidate family.
            let mut actual =
                from_fields(
                    candidate
                        .pattern
                        .tokens
                        .iter()
                        .filter_map(|token| match token {
                            Token::Field(field) => {
                                Some(adjust_field(*field, requested, &candidate.pattern.skeleton))
                            }
                            Token::Literal(_) => None,
                        }),
                );
            if requested.era.is_some() && actual.era.is_none() && self.calendar.append_era.is_some()
            {
                actual.era = requested.era;
            }
            if requested.time_zone_name.is_some() {
                actual.time_zone_name = requested.time_zone_name;
            }
            let candidate_score = score(requested, actual);
            if candidate_score > best_score {
                best = Some(candidate);
                best_score = candidate_score;
            }
        }
        let mut selected = best
            .ok_or_else(|| invalid("empty format candidate closure"))?
            .clone();
        adjust_widths(&mut selected.pattern, requested);
        if let Some(date) = &mut selected.date {
            adjust_widths(date, requested);
        }
        if let Some(time) = &mut selected.time {
            adjust_widths(time, requested);
        }
        if components(&selected.pattern).era.is_none() {
            if let (Some(width), Some(append)) = (requested.era, &self.calendar.append_era) {
                let width = match width {
                    DateTimeTextWidth::Short => NameWidth::Abbreviated,
                    DateTimeTextWidth::Long => NameWidth::Wide,
                    DateTimeTextWidth::Narrow => NameWidth::Narrow,
                };
                let era = Pattern::single(Field::Era(width));
                if let Some(date) = &mut selected.date {
                    *date = append.combine(date, &era)?;
                } else {
                    selected.pattern = append.combine(&selected.pattern, &era)?;
                }
            }
        }
        if let (Some(date), Some(time)) = (&selected.date, &selected.time) {
            let style = &self.calendar.styles[combination_style(requested)];
            selected.pattern = style.at_time.combine(time, date)?;
            selected.range_glue = Some(style.standard.clone());
        }
        if let Some(zone) = requested.time_zone_name {
            add_zone(self.calendar, &mut selected.pattern, zone)?;
            if let Some(time) = &mut selected.time {
                add_zone(self.calendar, time, zone)?;
            }
        }
        selected.components = components(&selected.pattern);
        Ok(selected)
    }

    pub(super) fn style(&self, styles: DateTimeStyles) -> Result<Format, DateTimeFormatError> {
        let date = styles
            .date()
            .map(|style| self.calendar.styles[style_index(style)].date.clone());
        let time = styles
            .time()
            .map(|style| {
                let pattern = &self.calendar.styles[style_index(style)].time;
                if matching_clock(pattern, self.cycle) {
                    Ok(with_cycle(pattern.clone(), self.cycle))
                } else {
                    self.best(components(pattern)).map(|format| format.pattern)
                }
            })
            .transpose()?;
        match (date, time) {
            (Some(date), Some(time)) => {
                let style = &self.calendar.styles
                    [style_index(styles.date().ok_or_else(|| invalid("date style missing"))?)];
                let pattern = style.at_time.combine(&time, &date)?;
                Ok(Format {
                    components: components(&pattern),
                    pattern,
                    date: Some(date),
                    time: Some(time),
                    range_glue: Some(style.standard.clone()),
                })
            }
            (Some(pattern), None) | (None, Some(pattern)) => Ok(plain(pattern)),
            (None, None) => Err(invalid("empty style selection")),
        }
    }
}
