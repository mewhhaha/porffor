use crate::datetime::*;
use crate::{TimeZoneEpochSeconds, TimeZoneSelection};

use super::super::named_time_zones::NamedTimeZones;
use super::{
    calendar,
    pattern::{Field, Pattern, Token},
    plan::ValidatedPlan,
    profile::{invalid, Locale, Profile},
    zones::{self, Snapshot},
};

mod fields;

pub(super) struct PreparedInput {
    pub(super) fields: calendar::Fields,
    pub(super) zone: Snapshot,
}

pub(super) fn prepare(
    plan: &ValidatedPlan<'_>,
    input: DateTimeInput,
    named: &NamedTimeZones,
) -> Result<PreparedInput, DateTimeFormatError> {
    let (iso, zone) = match input {
        DateTimeInput::Plain(value) => (value.iso(), Snapshot::Plain),
        DateTimeInput::Exact(value) => {
            let (offset, snapshot) = match &plan.time_zone {
                TimeZoneSelection::FixedOffset(offset) => {
                    (offset.seconds(), Snapshot::Fixed(offset.seconds()))
                }
                TimeZoneSelection::Named(identifier) => {
                    let identity = named
                        .lookup(identifier)
                        .map_err(|_| DateTimeFormatError::InvalidPlan("unknown named time zone"))?;
                    let epoch = TimeZoneEpochSeconds::new(value.epoch_seconds()).map_err(|_| {
                        DateTimeFormatError::InvalidRequest("invalid exact epoch seconds")
                    })?;
                    let transition = named
                        .transition(&identity, epoch)
                        .map_err(|error| invalid(error.to_string()))?;
                    (
                        transition.offset_seconds(),
                        Snapshot::Named {
                            identity,
                            epoch,
                            transition,
                        },
                    )
                }
            };
            (
                calendar::local_iso(value.epoch_seconds(), value.nanosecond(), offset)?,
                snapshot,
            )
        }
    };
    Ok(PreparedInput {
        fields: calendar::convert(plan.calendar_kind, iso)?,
        zone,
    })
}

pub(super) fn format(
    profile: &Profile,
    plan: &ValidatedPlan<'_>,
    input: DateTimeInput,
    named: &NamedTimeZones,
) -> Result<DateTimeParts, DateTimeFormatError> {
    let selected = plan.format(input.kind())?;
    let prepared = prepare(plan, input, named)?;
    pattern(profile, plan, &selected.pattern, &prepared)
}

pub(super) fn pattern(
    profile: &Profile,
    plan: &ValidatedPlan<'_>,
    pattern: &Pattern,
    input: &PreparedInput,
) -> Result<DateTimeParts, DateTimeFormatError> {
    let mut parts = Vec::new();
    for token in &pattern.tokens {
        let part = token_value(profile, plan, pattern, token, input)?;
        append(&mut parts, part);
    }
    Ok(DateTimeParts { parts })
}

pub(super) fn token_value(
    profile: &Profile,
    plan: &ValidatedPlan<'_>,
    pattern: &Pattern,
    token: &Token,
    input: &PreparedInput,
) -> Result<DateTimePart, DateTimeFormatError> {
    match token {
        Token::Literal(value) => Ok(DateTimePart {
            kind: DateTimePartKind::Literal,
            value: value.clone(),
        }),
        Token::Field(field) => Ok(DateTimePart {
            kind: field.part(),
            value: match field {
                Field::ZoneName(style) => {
                    zones::format(profile, plan.locale, &plan.numbering, &input.zone, *style)?
                }
                _ => fields::format(profile, plan, pattern, *field, input.fields)?,
            },
        }),
    }
}

pub(super) fn append(parts: &mut Vec<DateTimePart>, part: DateTimePart) {
    if part.kind == DateTimePartKind::Literal {
        if let Some(last) = parts
            .last_mut()
            .filter(|last| last.kind == DateTimePartKind::Literal)
        {
            last.value.push_str(&part.value);
            return;
        }
    }
    parts.push(part);
}

pub(super) fn positional(
    profile: &Profile,
    locale: &Locale,
    numbering: &str,
    value: i64,
    minimum_digits: u8,
) -> Result<String, DateTimeFormatError> {
    let digits = profile
        .digits
        .get(numbering)
        .ok_or_else(|| invalid("numeric field requires a positional numbering system"))?;
    let mut result = String::new();
    if value < 0 {
        result.push_str(
            locale
                .minus
                .get(numbering)
                .ok_or_else(|| invalid("missing numeric minus sign"))?,
        );
    }
    let decimal = value.unsigned_abs().to_string();
    for _ in decimal.len()..usize::from(minimum_digits) {
        result.push(digits[0]);
    }
    for digit in decimal.bytes() {
        result.push(digits[usize::from(digit - b'0')]);
    }
    Ok(result)
}
