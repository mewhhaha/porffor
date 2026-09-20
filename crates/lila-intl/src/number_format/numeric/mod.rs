//! Exact NumberFormat configuration and mathematical value domains.

mod digits;
mod notation;
mod parse;
mod plural;
mod resource;
mod round;

pub use notation::{
    format_decimal, CompactExponentRow, CompactExponentTable, DecimalFormatSettings, DecimalScale,
    InvalidCompactExponentTable, NotationScaling, SelectedNotation,
};
pub use parse::{normalize_numeric_input, NumericNormalizationError, NumericWireError};
pub use plural::{ExactPluralOperand, PluralOperand, PluralOperands};
pub use resource::{NumberFormatResourceError, NumericLimits};
pub use round::{round_decimal, RoundedDecimal, RoundingSettings};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumberSign {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidExactDecimal {
    EmptyCoefficient,
    LeadingZero,
    TrailingZero,
    NonDecimalDigit,
}

/// Nonzero canonical coefficient, stored as numeric digits 0..=9. Zero has its
/// own signed variant and never crosses this constructor.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DecimalCoefficient(Box<[u8]>);
impl DecimalCoefficient {
    pub fn from_digits(digits: Box<[u8]>) -> Result<Self, InvalidExactDecimal> {
        if digits.is_empty() {
            return Err(InvalidExactDecimal::EmptyCoefficient);
        }
        if digits.first() == Some(&0) {
            return Err(InvalidExactDecimal::LeadingZero);
        }
        if digits.last() == Some(&0) {
            return Err(InvalidExactDecimal::TrailingZero);
        }
        if digits.iter().any(|digit| *digit > 9) {
            return Err(InvalidExactDecimal::NonDecimalDigit);
        }
        Ok(Self(digits))
    }
    pub fn digits(&self) -> &[u8] {
        &self.0
    }
}

/// Exact sign * coefficient * 10^exponent. This is a mathematical decimal,
/// not a binary64. BigInts can exceed the binary64 magnitude limits.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExactDecimal {
    sign: NumberSign,
    coefficient: DecimalCoefficient,
    exponent: i64,
}
impl ExactDecimal {
    pub const fn new(sign: NumberSign, coefficient: DecimalCoefficient, exponent: i64) -> Self {
        Self {
            sign,
            coefficient,
            exponent,
        }
    }
    pub const fn sign(&self) -> NumberSign {
        self.sign
    }
    pub fn coefficient(&self) -> &DecimalCoefficient {
        &self.coefficient
    }
    pub const fn exponent(&self) -> i64 {
        self.exponent
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IntlMathematicalValue {
    Finite(ExactDecimal),
    Zero(NumberSign),
    Infinity(NumberSign),
    NaN,
}

/// A finite signed input. NaN and infinity are partition-layer cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiniteValue<'a> {
    Nonzero(&'a ExactDecimal),
    Zero(NumberSign),
}
impl IntlMathematicalValue {
    pub fn finite_value(&self) -> Option<FiniteValue<'_>> {
        match self {
            Self::Finite(value) => Some(FiniteValue::Nonzero(value)),
            Self::Zero(sign) => Some(FiniteValue::Zero(*sign)),
            Self::Infinity(_) | Self::NaN => None,
        }
    }
}
impl FiniteValue<'_> {
    pub const fn sign(self) -> NumberSign {
        match self {
            Self::Nonzero(value) => value.sign(),
            Self::Zero(sign) => sign,
        }
    }
}

/// Boundary material after AOT ToPrimitive/ToNumber. Strings retain exact
/// UTF-16, including lone surrogates which the numeric parser must reject as
/// NaN. BigInt decimal grammar is a trusted compiler/protocol invariant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservedNumericInput {
    StringNumericLiteral(Box<[u16]>),
    NumberShortestDecimal(Box<str>),
    NegativeZero,
    BigIntDecimal(Box<str>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NumberRangeEndpoint {
    Finite(ExactDecimal),
    Zero(NumberSign),
    Infinity(NumberSign),
}
impl TryFrom<IntlMathematicalValue> for NumberRangeEndpoint {
    type Error = NaNRangeEndpoint;
    fn try_from(value: IntlMathematicalValue) -> Result<Self, Self::Error> {
        match value {
            IntlMathematicalValue::Finite(value) => Ok(Self::Finite(value)),
            IntlMathematicalValue::Zero(sign) => Ok(Self::Zero(sign)),
            IntlMathematicalValue::Infinity(sign) => Ok(Self::Infinity(sign)),
            IntlMathematicalValue::NaN => Err(NaNRangeEndpoint),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NaNRangeEndpoint;

/// Both AOT conversions are complete before construction. Descending and
/// differently signed-zero ranges are valid; only NaN is excluded here.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NumberRange {
    start: NumberRangeEndpoint,
    end: NumberRangeEndpoint,
}
impl NumberRange {
    pub fn new(
        start: IntlMathematicalValue,
        end: IntlMathematicalValue,
    ) -> Result<Self, NaNRangeEndpoint> {
        Ok(Self {
            start: start.try_into()?,
            end: end.try_into()?,
        })
    }
    pub fn start(&self) -> &NumberRangeEndpoint {
        &self.start
    }
    pub fn end(&self) -> &NumberRangeEndpoint {
        &self.end
    }
}
