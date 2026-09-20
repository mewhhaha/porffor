use crate::datetime::*;

use super::super::named_time_zones::NamedTimeZones;
use super::{
    pattern::{Glue, GlueToken, Pattern, Token},
    plan::ValidatedPlan,
    profile::{Interval, Profile},
    render::{self, PreparedInput},
};

mod selection;

pub(super) fn format(
    profile: &Profile,
    plan: &ValidatedPlan<'_>,
    start: DateTimeInput,
    end: DateTimeInput,
    named: &NamedTimeZones,
) -> Result<DateTimeRangeParts, DateTimeFormatError> {
    let selected = plan.format(start.kind())?;
    let start = render::prepare(plan, start, named)?;
    let end = render::prepare(plan, end, named)?;
    let Some(difference) = selection::greatest(plan, &selected.pattern, start.fields, end.fields)?
    else {
        return Ok(with_source(
            render::pattern(profile, plan, &selected.pattern, &start)?,
            DateTimeRangeSource::Shared,
        ));
    };
    if let Some((interval, pattern)) = selection::select(plan, &selected.pattern, difference) {
        return interval_parts(profile, plan, interval, &pattern, &start, &end);
    }
    if let (Some(date), Some(time), Some(glue)) =
        (&selected.date, &selected.time, &selected.range_glue)
    {
        if selection::greatest(plan, date, start.fields, end.fields)?.is_none() {
            let date = with_source(
                render::pattern(profile, plan, date, &start)?,
                DateTimeRangeSource::Shared,
            );
            let time = if let Some((interval, pattern)) = selection::select(plan, time, difference)
            {
                interval_parts(profile, plan, interval, &pattern, &start, &end)?
            } else {
                fallback(profile, plan, time, &start, &end)?
            };
            return Ok(combine(glue, &time, &date));
        }
        // CLDR specifies the standard connector for a range, while a single
        // event may use the grammatical atTime connector.
        return fallback(profile, plan, &glue.combine(time, date)?, &start, &end);
    }
    fallback(profile, plan, &selected.pattern, &start, &end)
}

fn fallback(
    profile: &Profile,
    plan: &ValidatedPlan<'_>,
    pattern: &Pattern,
    start: &PreparedInput,
    end: &PreparedInput,
) -> Result<DateTimeRangeParts, DateTimeFormatError> {
    let start = with_source(
        render::pattern(profile, plan, pattern, start)?,
        DateTimeRangeSource::StartRange,
    );
    let end = with_source(
        render::pattern(profile, plan, pattern, end)?,
        DateTimeRangeSource::EndRange,
    );
    Ok(combine(&plan.calendar.interval_fallback, &start, &end))
}

fn with_source(parts: DateTimeParts, source: DateTimeRangeSource) -> DateTimeRangeParts {
    DateTimeRangeParts {
        parts: parts
            .parts
            .into_iter()
            .map(|part| DateTimeRangePart {
                kind: part.kind,
                value: part.value,
                source,
            })
            .collect(),
    }
}

fn combine(
    glue: &Glue,
    first: &DateTimeRangeParts,
    second: &DateTimeRangeParts,
) -> DateTimeRangeParts {
    let mut parts = Vec::new();
    for token in &glue.tokens {
        match token {
            GlueToken::Literal(value) => append(
                &mut parts,
                DateTimeRangePart {
                    kind: DateTimePartKind::Literal,
                    value: value.clone(),
                    source: DateTimeRangeSource::Shared,
                },
            ),
            GlueToken::First => {
                for part in &first.parts {
                    append(&mut parts, part.clone());
                }
            }
            GlueToken::Second => {
                for part in &second.parts {
                    append(&mut parts, part.clone());
                }
            }
        }
    }
    DateTimeRangeParts { parts }
}

fn interval_parts(
    profile: &Profile,
    plan: &ValidatedPlan<'_>,
    interval: &Interval,
    pattern: &Pattern,
    start: &PreparedInput,
    end: &PreparedInput,
) -> Result<DateTimeRangeParts, DateTimeFormatError> {
    let field_source = |index: usize| {
        let Token::Field(field) = &pattern.tokens[index] else {
            return None;
        };
        let other = if index < interval.second_start {
            &pattern.tokens[interval.second_start..]
        } else {
            &pattern.tokens[..interval.second_start]
        };
        let shared = !other
            .iter()
            .any(|token| matches!(token, Token::Field(other) if field.part() == other.part()));
        Some(if shared {
            DateTimeRangeSource::Shared
        } else if (index < interval.second_start) != interval.latest_first {
            DateTimeRangeSource::StartRange
        } else {
            DateTimeRangeSource::EndRange
        })
    };
    let mut parts = Vec::new();
    for (index, token) in pattern.tokens.iter().enumerate() {
        let source = if let Some(source) = field_source(index) {
            source
        } else {
            let previous = (0..index).rev().find_map(field_source);
            let next = (index + 1..pattern.tokens.len()).find_map(field_source);
            match (previous, next) {
                (Some(left), Some(right)) if left == right => left,
                (Some(source), None) | (None, Some(source)) => source,
                _ => DateTimeRangeSource::Shared,
            }
        };
        let input = match source {
            DateTimeRangeSource::Shared | DateTimeRangeSource::StartRange => start,
            DateTimeRangeSource::EndRange => end,
        };
        let part = render::token_value(profile, plan, pattern, token, input)?;
        append(
            &mut parts,
            DateTimeRangePart {
                kind: part.kind,
                value: part.value,
                source,
            },
        );
    }
    Ok(DateTimeRangeParts { parts })
}

fn append(parts: &mut Vec<DateTimeRangePart>, part: DateTimeRangePart) {
    if part.kind == DateTimePartKind::Literal {
        if let Some(last) = parts
            .last_mut()
            .filter(|last| last.kind == DateTimePartKind::Literal && last.source == part.source)
        {
            last.value.push_str(&part.value);
            return;
        }
    }
    parts.push(part);
}
