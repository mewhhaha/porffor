use core::fmt;

use super::super::options::{Notation, NumberFormatOptions, NumberStyle};
use super::round::{round_scaled, ScaledDecimal};
use super::{
    FiniteValue, NumberFormatResourceError, NumberSign, NumericLimits, RoundedDecimal,
    RoundingSettings,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidCompactExponentTable {
    ExponentExceedsMagnitude,
    NonIncreasingMagnitude,
}

impl fmt::Display for InvalidCompactExponentTable {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExponentExceedsMagnitude => {
                formatter.write_str("compact exponent exceeds its pattern magnitude")
            }
            Self::NonIncreasingMagnitude => {
                formatter.write_str("compact magnitudes must be sorted and unique")
            }
        }
    }
}
impl std::error::Error for InvalidCompactExponentTable {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompactExponentRow {
    magnitude: u32,
    exponent: u32,
}

impl CompactExponentRow {
    pub fn new(magnitude: u32, exponent: u32) -> Result<Self, InvalidCompactExponentTable> {
        if exponent > magnitude {
            return Err(InvalidCompactExponentTable::ExponentExceedsMagnitude);
        }
        Ok(Self {
            magnitude,
            exponent,
        })
    }
    pub const fn magnitude(self) -> u32 {
        self.magnitude
    }
    pub const fn exponent(self) -> u32 {
        self.exponent
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactExponentTable(Box<[CompactExponentRow]>);

impl CompactExponentTable {
    pub fn new(rows: Box<[CompactExponentRow]>) -> Result<Self, InvalidCompactExponentTable> {
        if rows
            .windows(2)
            .any(|pair| pair[0].magnitude >= pair[1].magnitude)
        {
            return Err(InvalidCompactExponentTable::NonIncreasingMagnitude);
        }
        Ok(Self(rows))
    }
    pub fn empty() -> Self {
        Self(Box::default())
    }
    pub fn rows(&self) -> &[CompactExponentRow] {
        &self.0
    }
    fn select(&self, magnitude: i64) -> Option<CompactExponentRow> {
        let end = self
            .0
            .partition_point(|row| i64::from(row.magnitude) <= magnitude);
        end.checked_sub(1).map(|index| self.0[index])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecimalScale {
    Unit,
    Percent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotationScaling<'a> {
    Standard,
    Scientific,
    Engineering,
    Compact(&'a CompactExponentTable),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedNotation {
    Standard,
    Scientific { exponent: i64 },
    Engineering { exponent: i64 },
    Compact { row: Option<CompactExponentRow> },
}

impl SelectedNotation {
    pub const fn exponent(self) -> i64 {
        match self {
            Self::Standard => 0,
            Self::Scientific { exponent } | Self::Engineering { exponent } => exponent,
            Self::Compact { row: Some(row) } => row.exponent as i64,
            Self::Compact { row: None } => 0,
        }
    }
    pub const fn compact_pattern_magnitude(self) -> Option<u32> {
        match self {
            Self::Compact { row: Some(row) } => Some(row.magnitude),
            Self::Standard
            | Self::Scientific { .. }
            | Self::Engineering { .. }
            | Self::Compact { row: None } => None,
        }
    }
    pub const fn compact_exponent(self) -> u32 {
        match self {
            Self::Compact { row: Some(row) } => row.exponent,
            Self::Standard
            | Self::Scientific { .. }
            | Self::Engineering { .. }
            | Self::Compact { row: None } => 0,
        }
    }
}

impl NotationScaling<'_> {
    fn select(self, magnitude: i64) -> Result<SelectedNotation, NumberFormatResourceError> {
        Ok(match self {
            Self::Standard => SelectedNotation::Standard,
            Self::Scientific => SelectedNotation::Scientific {
                exponent: magnitude,
            },
            Self::Engineering => SelectedNotation::Engineering {
                exponent: magnitude
                    .div_euclid(3)
                    .checked_mul(3)
                    .ok_or(NumberFormatResourceError::ExponentOverflow)?,
            },
            Self::Compact(table) => SelectedNotation::Compact {
                row: table.select(magnitude),
            },
        })
    }
    fn zero(self) -> SelectedNotation {
        match self {
            Self::Standard => SelectedNotation::Standard,
            Self::Scientific => SelectedNotation::Scientific { exponent: 0 },
            Self::Engineering => SelectedNotation::Engineering { exponent: 0 },
            Self::Compact(_) => SelectedNotation::Compact { row: None },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecimalFormatSettings<'a> {
    pub rounding: RoundingSettings,
    pub scale: DecimalScale,
    pub notation: NotationScaling<'a>,
}

impl<'a> DecimalFormatSettings<'a> {
    pub fn from_options(options: &NumberFormatOptions, compact: &'a CompactExponentTable) -> Self {
        Self {
            rounding: options.into(),
            scale: match &options.style {
                NumberStyle::Percent => DecimalScale::Percent,
                NumberStyle::Decimal | NumberStyle::Currency { .. } | NumberStyle::Unit { .. } => {
                    DecimalScale::Unit
                }
            },
            notation: match options.notation {
                Notation::Standard => NotationScaling::Standard,
                Notation::Scientific => NotationScaling::Scientific,
                Notation::Engineering => NotationScaling::Engineering,
                Notation::Compact(_) => NotationScaling::Compact(compact),
            },
        }
    }
}

pub fn format_decimal(
    value: FiniteValue<'_>,
    settings: &DecimalFormatSettings<'_>,
    limits: &NumericLimits,
) -> Result<RoundedDecimal, NumberFormatResourceError> {
    let mut value = ScaledDecimal::from(value);
    if settings.scale == DecimalScale::Percent {
        value = value.rescale(-2)?;
    }
    let selected = match value.magnitude()? {
        None => settings.notation.zero(),
        Some(magnitude) => {
            let initial = settings.notation.select(magnitude)?;
            // ComputeExponent probes the absolute value with positive-sign
            // rounding, then final formatting rounds the signed input afresh.
            let positive = ScaledDecimal {
                sign: NumberSign::Positive,
                ..value
            };
            let probe = round_scaled(
                positive.rescale(initial.exponent())?,
                &settings.rounding,
                initial,
                limits,
            )?;
            match probe.magnitude()? {
                Some(rounded_magnitude)
                    if i128::from(rounded_magnitude)
                        != i128::from(magnitude) - i128::from(initial.exponent()) =>
                {
                    let next_magnitude = magnitude
                        .checked_add(1)
                        .ok_or(NumberFormatResourceError::ExponentOverflow)?;
                    settings.notation.select(next_magnitude)?
                }
                Some(_) | None => initial,
            }
        }
    };
    round_scaled(
        value.rescale(selected.exponent())?,
        &settings.rounding,
        selected,
        limits,
    )
}
