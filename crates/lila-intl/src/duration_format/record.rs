//! Exact mathematical integers after future AOT ToDurationRecord observations.
use super::{DurationError, DurationUnit};
use crate::number_format::numeric::{
    DecimalCoefficient, ExactDecimal, IntlMathematicalValue, NumberSign,
};
const LIMIT_NS: u128 = (1u128 << 53) * 1_000_000_000;
const NS: [u128; 7] = [
    86_400_000_000_000,
    3_600_000_000_000,
    60_000_000_000,
    1_000_000_000,
    1_000_000,
    1_000,
    1,
];
/// Uniform-sign, finite integral IEEE inputs, with the normative exact duration bounds.
/// Negative-zero input fields contribute zero and do not set DurationSign.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurationRecord {
    magnitude: [u128; 10],
    negative: bool,
}
fn integer_magnitude(value: f64) -> Option<u128> {
    let bits = value.to_bits() & 0x7fff_ffff_ffff_ffff;
    if bits == 0 {
        return Some(0);
    }
    let exponent = ((bits >> 52) & 0x7ff) as i32 - 1023;
    let significand = u128::from((bits & ((1u64 << 52) - 1)) | (1u64 << 52));
    if exponent >= 128 {
        return None;
    }
    if exponent >= 52 {
        significand.checked_shl((exponent - 52) as u32)
    } else {
        Some(significand >> ((52 - exponent) as u32))
    }
}
impl DurationRecord {
    pub fn from_number_fields(fields: [f64; 10]) -> Result<Self, DurationError> {
        // ToIntegerIfIntegral has already observed every input in the future AOT caller.
        // This primitive boundary never converts through decimal-shortest spelling.
        if fields.iter().any(|v| !v.is_finite() || v.fract() != 0.0) {
            return Err(DurationError::InvalidRecord);
        }
        let mut sign = 0i8;
        for value in fields {
            let next = if value < 0.0 {
                -1
            } else if value > 0.0 {
                1
            } else {
                0
            };
            if next != 0 {
                if sign != 0 && sign != next {
                    return Err(DurationError::InvalidRecord);
                }
                sign = next
            }
        }
        let mut magnitude = [0; 10];
        for (i, value) in fields.iter().enumerate() {
            magnitude[i] = integer_magnitude(*value).ok_or(DurationError::InvalidBounds)?;
        }
        if magnitude[..3].iter().any(|v| *v >= (1u128 << 32)) {
            return Err(DurationError::InvalidBounds);
        }
        let total = magnitude[3..]
            .iter()
            .zip(NS)
            .try_fold(0u128, |sum, (value, scale)| {
                value.checked_mul(scale).and_then(|v| sum.checked_add(v))
            })
            .ok_or(DurationError::InvalidBounds)?;
        if total >= LIMIT_NS {
            return Err(DurationError::InvalidBounds);
        }
        Ok(Self {
            magnitude,
            negative: sign < 0,
        })
    }
    pub const fn negative(&self) -> bool {
        self.negative
    }
    pub const fn magnitude(&self, unit: DurationUnit) -> u128 {
        self.magnitude[unit.index()]
    }
    pub(super) fn fraction_value(
        &self,
        unit: DurationUnit,
        fractional: &[bool; 10],
        sign: bool,
    ) -> IntlMathematicalValue {
        // The checked total bound also bounds every subsecond composition by <2^83.
        let mut coefficient = self.magnitude(unit);
        let mut exponent = 0i64;
        for &smaller in &DurationUnit::ALL[unit.index() + 1..] {
            if matches!(
                unit,
                DurationUnit::Second | DurationUnit::Millisecond | DurationUnit::Microsecond
            ) && fractional.get(unit.index() + 1) == Some(&true)
                && fractional[smaller.index()]
            {
                coefficient = coefficient * 1000 + self.magnitude(smaller);
                exponent -= 3;
            }
        }
        exact_decimal(coefficient, exponent, sign && self.negative)
    }
}
pub(super) fn exact_decimal(
    mut coefficient: u128,
    mut exponent: i64,
    negative: bool,
) -> IntlMathematicalValue {
    let sign = if negative {
        NumberSign::Negative
    } else {
        NumberSign::Positive
    };
    if coefficient == 0 {
        return IntlMathematicalValue::Zero(sign);
    }
    while coefficient % 10 == 0 {
        coefficient /= 10;
        exponent += 1;
    }
    let digits = coefficient
        .to_string()
        .bytes()
        .map(|b| b - b'0')
        .collect::<Vec<_>>()
        .into_boxed_slice();
    IntlMathematicalValue::Finite(ExactDecimal::new(
        sign,
        DecimalCoefficient::from_digits(digits).expect("nonzero u128 canonical coefficient"),
        exponent,
    ))
}
