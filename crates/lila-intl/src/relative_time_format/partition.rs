use super::{
    profiles::Pattern, FiniteRelativeNumber, RelativeNumeric, RelativeTimeConfiguration,
    RelativeTimeError, RelativeUnit,
};
use crate::number_format::numeric::{normalize_numeric_input, ObservedNumericInput};
use crate::number_format::{
    owned_text, partition_number, NumberPart, NumberPartKind, NumberProfiles, PartitionLimits,
};
use crate::plural_rules::{select_plural_operation, SelectPluralRequest};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelativePart {
    Literal(Box<str>),
    Number {
        part: NumberPart,
        unit: RelativeUnit,
    },
}
impl RelativePart {
    pub fn text(&self) -> &str {
        match self {
            Self::Literal(text) => text,
            Self::Number { part, .. } => part.text(),
        }
    }
    pub const fn unit(&self) -> Option<RelativeUnit> {
        match self {
            Self::Literal(_) => None,
            Self::Number { unit, .. } => Some(*unit),
        }
    }
    pub const fn kind(&self) -> NumberPartKind {
        match self {
            Self::Literal(_) => NumberPartKind::Literal,
            Self::Number { part, .. } => part.kind(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativePartition {
    parts: Box<[RelativePart]>,
    bytes: usize,
}
impl RelativePartition {
    pub fn parts(&self) -> &[RelativePart] {
        &self.parts
    }
    pub fn to_text(&self) -> Result<String, RelativeTimeError> {
        let mut result = String::new();
        result
            .try_reserve_exact(self.bytes)
            .map_err(|_| RelativeTimeError::Resource("text allocation"))?;
        for part in &self.parts {
            result.push_str(part.text());
        }
        Ok(result)
    }
    pub(crate) fn from_parts(
        parts: Vec<RelativePart>,
        limits: &PartitionLimits,
    ) -> Result<Self, RelativeTimeError> {
        if parts.is_empty() || parts.len() as u128 > u128::from(limits.part_count()) {
            return Err(RelativeTimeError::Resource("part extent"));
        }
        let bytes = parts
            .iter()
            .try_fold(0usize, |sum, part| sum.checked_add(part.text().len()))
            .ok_or(RelativeTimeError::Resource("text extent"))?;
        if bytes as u128 > u128::from(limits.output_bytes()) {
            return Err(RelativeTimeError::Resource("text extent"));
        }
        Ok(Self {
            parts: parts.into_boxed_slice(),
            bytes,
        })
    }
}

pub(super) fn partition_relative_time(
    configuration: &RelativeTimeConfiguration,
    value: FiniteRelativeNumber,
    unit: RelativeUnit,
    number_profiles: &Arc<NumberProfiles>,
    limits: &PartitionLimits,
) -> Result<RelativePartition, RelativeTimeError> {
    configuration
        .plural()
        .locale()
        .ensure_profiles(number_profiles)?;
    let field = configuration.field(unit);
    if configuration.numeric() == RelativeNumeric::Auto {
        if let Some(text) = value
            .auto_offset()
            .and_then(|offset| field.relative[(offset + 2) as usize].as_deref())
        {
            return RelativePartition::from_parts(
                vec![RelativePart::Literal(owned_text(text, limits)?)],
                limits,
            );
        }
    }
    // The checked finite owner makes format_finite's precondition permanent.
    // The NumberShortestDecimal frame requires ECMAScript Number::toString
    // spelling, including its exponent thresholds and explicit exponent sign.
    let mut spelling = ryu_js::Buffer::new();
    let magnitude = ObservedNumericInput::NumberShortestDecimal(owned_text(
        spelling.format_finite(value.value().abs()),
        limits,
    )?);
    let category = select_plural_operation(
        SelectPluralRequest::new(configuration.plural().clone(), magnitude.clone()),
        number_profiles,
        &limits.numeric(),
    )?;
    let numeric = normalize_numeric_input(magnitude, &limits.numeric())?;
    let number = partition_number(configuration.number(), &numeric, number_profiles, limits)?;
    let pattern = if value.past() {
        &field.past
    } else {
        &field.future
    };
    let mut result = Vec::new();
    match &pattern[category.index()] {
        Pattern::Literal(text) => {
            result
                .try_reserve_exact(1)
                .map_err(|_| RelativeTimeError::Resource("part allocation"))?;
            result.push(RelativePart::Literal(owned_text(text, limits)?));
        }
        Pattern::Number { before, after } => {
            let count = number
                .parts()
                .len()
                .checked_add(usize::from(!before.is_empty()) + usize::from(!after.is_empty()))
                .ok_or(RelativeTimeError::Resource("part extent"))?;
            if count as u128 > u128::from(limits.part_count()) {
                return Err(RelativeTimeError::Resource("part extent"));
            }
            result
                .try_reserve_exact(count)
                .map_err(|_| RelativeTimeError::Resource("part allocation"))?;
            if !before.is_empty() {
                result.push(RelativePart::Literal(owned_text(before, limits)?));
            }
            for part in number.parts() {
                result.push(RelativePart::Number {
                    part: NumberPart::new(part.kind(), owned_text(part.text(), limits)?),
                    unit,
                });
            }
            if !after.is_empty() {
                result.push(RelativePart::Literal(owned_text(after, limits)?));
            }
        }
    }
    RelativePartition::from_parts(result, limits)
}
