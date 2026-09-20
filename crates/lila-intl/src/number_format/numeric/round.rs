use core::cmp::Ordering;

use super::super::options::{
    FractionPrecision, IntegerDigitCount, NumberFormatOptions, Precision, RoundingMode,
    SignificantDigitRange, TrailingZeroDisplay,
};
use super::digits::{add_one, canonical_decimal, divide_small, multiply_small};
use super::resource::zeroed_digits;
use super::{
    ExactDecimal, FiniteValue, NumberFormatResourceError, NumberSign, NumericLimits,
    SelectedNotation,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoundingSettings {
    pub precision: Precision,
    pub mode: RoundingMode,
    pub minimum_integer_digits: IntegerDigitCount,
    pub trailing_zero_display: TrailingZeroDisplay,
}

impl From<&NumberFormatOptions> for RoundingSettings {
    fn from(options: &NumberFormatOptions) -> Self {
        Self {
            precision: options.precision,
            mode: options.rounding_mode,
            minimum_integer_digits: options.minimum_integer_digits,
            trailing_zero_display: options.trailing_zero_display,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoundedDecimal {
    sign: NumberSign,
    rounded_mantissa: Option<ExactDecimal>,
    integer_digits: Box<[u8]>,
    fraction_digits: Box<[u8]>,
    rounding_magnitude: i64,
    notation: SelectedNotation,
}

impl RoundedDecimal {
    pub const fn sign(&self) -> NumberSign {
        self.sign
    }
    pub fn integer_digits(&self) -> &[u8] {
        &self.integer_digits
    }
    pub fn fraction_digits(&self) -> &[u8] {
        &self.fraction_digits
    }
    pub const fn rounding_magnitude(&self) -> i64 {
        self.rounding_magnitude
    }
    pub const fn notation(&self) -> SelectedNotation {
        self.notation
    }

    /// The signed rounded mantissa after percent scaling and division by
    /// 10^notation.exponent(), not the original unscaled input quantity.
    pub fn rounded_value(&self) -> FiniteValue<'_> {
        match &self.rounded_mantissa {
            Some(value) => FiniteValue::Nonzero(value),
            None => FiniteValue::Zero(self.sign),
        }
    }

    pub fn is_zero(&self) -> bool {
        self.rounded_mantissa.is_none()
    }

    pub fn is_integer(&self) -> bool {
        self.rounded_mantissa
            .as_ref()
            .is_none_or(|value| value.exponent() >= 0)
    }

    pub fn magnitude(&self) -> Result<Option<i64>, NumberFormatResourceError> {
        self.rounded_mantissa
            .as_ref()
            .map(|value| checked_magnitude(value.coefficient().digits().len(), value.exponent()))
            .transpose()
    }
}

#[derive(Clone, Copy)]
pub(super) struct ScaledDecimal<'a> {
    pub sign: NumberSign,
    pub digits: &'a [u8],
    pub exponent: i64,
}

impl<'a> From<FiniteValue<'a>> for ScaledDecimal<'a> {
    fn from(value: FiniteValue<'a>) -> Self {
        match value {
            FiniteValue::Nonzero(value) => Self {
                sign: value.sign(),
                digits: value.coefficient().digits(),
                exponent: value.exponent(),
            },
            FiniteValue::Zero(sign) => Self {
                sign,
                digits: &[],
                exponent: 0,
            },
        }
    }
}

impl ScaledDecimal<'_> {
    pub fn magnitude(self) -> Result<Option<i64>, NumberFormatResourceError> {
        if self.digits.is_empty() {
            return Ok(None);
        }
        checked_magnitude(self.digits.len(), self.exponent).map(Some)
    }

    pub fn rescale(self, exponent: i64) -> Result<Self, NumberFormatResourceError> {
        if self.digits.is_empty() {
            return Ok(self);
        }
        let exponent = i64::try_from(i128::from(self.exponent) - i128::from(exponent))
            .map_err(|_| NumberFormatResourceError::ExponentOverflow)?;
        Ok(Self { exponent, ..self })
    }
}

fn checked_magnitude(length: usize, exponent: i64) -> Result<i64, NumberFormatResourceError> {
    i64::try_from(i128::from(exponent) + length as i128 - 1)
        .map_err(|_| NumberFormatResourceError::ExponentOverflow)
}

struct RawRounded {
    value: Option<ExactDecimal>,
    magnitude: i64,
    fraction_width: u128,
    trimmable_zeroes: u8,
}

fn choose_upper(mode: RoundingMode, sign: NumberSign, half: Ordering, lower_is_even: bool) -> bool {
    let negative = sign == NumberSign::Negative;
    match mode {
        RoundingMode::Ceil => !negative,
        RoundingMode::Floor => negative,
        RoundingMode::Expand => true,
        RoundingMode::Trunc => false,
        RoundingMode::HalfCeil
        | RoundingMode::HalfFloor
        | RoundingMode::HalfExpand
        | RoundingMode::HalfTrunc
        | RoundingMode::HalfEven => match half {
            Ordering::Less => false,
            Ordering::Greater => true,
            Ordering::Equal => match mode {
                RoundingMode::HalfCeil => !negative,
                RoundingMode::HalfFloor => negative,
                RoundingMode::HalfExpand => true,
                RoundingMode::HalfTrunc => false,
                RoundingMode::HalfEven => !lower_is_even,
                RoundingMode::Ceil
                | RoundingMode::Floor
                | RoundingMode::Expand
                | RoundingMode::Trunc => unreachable!(),
            },
        },
    }
}

fn compare_fraction_to_half(digits: &[u8], removed: i128) -> Ordering {
    if removed > digits.len() as i128 {
        return Ordering::Less;
    }
    let start = digits.len() - removed as usize;
    match digits[start].cmp(&5) {
        Ordering::Equal if digits[start + 1..].iter().any(|digit| *digit != 0) => Ordering::Greater,
        ordering => ordering,
    }
}

fn round_grid(
    value: ScaledDecimal<'_>,
    magnitude: i64,
    increment: u16,
    mode: RoundingMode,
    limits: &NumericLimits,
) -> Result<Option<ExactDecimal>, NumberFormatResourceError> {
    if value.digits.is_empty() {
        return Ok(None);
    }
    let shift = i128::from(value.exponent) - i128::from(magnitude);
    let integer_length = (value.digits.len() as i128 + shift).max(0) as u128;
    let mut quotient = zeroed_digits(integer_length, limits)?;
    let retained = quotient.len().min(value.digits.len());
    quotient[..retained].copy_from_slice(&value.digits[..retained]);
    let remainder = divide_small(&mut quotient, increment);
    // Canonical coefficients end in a nonzero digit; every nonempty discarded
    // tail therefore makes the quotient inexact.
    let fractional = shift < 0;
    if remainder != 0 || fractional {
        let twice = u32::from(remainder) * 2;
        let half = match twice.cmp(&u32::from(increment)) {
            Ordering::Equal if fractional => Ordering::Greater,
            Ordering::Less if fractional && u32::from(increment) - twice == 1 => {
                compare_fraction_to_half(value.digits, -shift)
            }
            ordering => ordering,
        };
        let even = quotient.last().is_none_or(|digit| digit % 2 == 0);
        if choose_upper(mode, value.sign, half, even) {
            add_one(&mut quotient, limits)?;
        }
    }
    multiply_small(&mut quotient, increment, limits)?;
    canonical_decimal(quotient, magnitude, value.sign)
}

fn raw_significant(
    value: ScaledDecimal<'_>,
    range: SignificantDigitRange,
    mode: RoundingMode,
    limits: &NumericLimits,
) -> Result<RawRounded, NumberFormatResourceError> {
    let precision = i64::from(range.maximum().get());
    let initial_magnitude = value.magnitude()?.unwrap_or(0);
    let grid = initial_magnitude
        .checked_sub(precision - 1)
        .ok_or(NumberFormatResourceError::ExponentOverflow)?;
    let rounded = round_grid(value, grid, 1, mode, limits)?;
    let rounded_magnitude = match &rounded {
        Some(value) => checked_magnitude(value.coefficient().digits().len(), value.exponent())?,
        None => 0,
    };
    // Significant carry changes e and therefore the reported rounding
    // magnitude. Combined precision compares this final magnitude.
    let magnitude = rounded_magnitude
        .checked_sub(precision - 1)
        .ok_or(NumberFormatResourceError::ExponentOverflow)?;
    Ok(RawRounded {
        value: rounded,
        magnitude,
        fraction_width: (-i128::from(magnitude)).max(0) as u128,
        trimmable_zeroes: range.maximum().get() - range.minimum().get(),
    })
}

fn raw_fraction(
    value: ScaledDecimal<'_>,
    precision: FractionPrecision,
    mode: RoundingMode,
    limits: &NumericLimits,
) -> Result<RawRounded, NumberFormatResourceError> {
    let (minimum, maximum, increment) = match precision {
        FractionPrecision::Range(range) => (range.minimum().get(), range.maximum().get(), 1),
        FractionPrecision::Increment { digits, increment } => {
            (digits.get(), digits.get(), increment.value())
        }
    };
    let magnitude = -i64::from(maximum);
    Ok(RawRounded {
        value: round_grid(value, magnitude, increment, mode, limits)?,
        magnitude,
        fraction_width: u128::from(maximum),
        trimmable_zeroes: maximum - minimum,
    })
}

fn render_digits(
    value: Option<&ExactDecimal>,
    fraction_width: u128,
    minimum_integer_digits: IntegerDigitCount,
    limits: &NumericLimits,
) -> Result<(Box<[u8]>, Box<[u8]>), NumberFormatResourceError> {
    let point = value.map_or(1, |value| {
        value.coefficient().digits().len() as i128 + i128::from(value.exponent())
    });
    let unpadded_integer = point.max(1) as u128;
    let integer_width = unpadded_integer.max(u128::from(minimum_integer_digits.get()));
    limits.check(integer_width + fraction_width)?;
    let mut integer = zeroed_digits(integer_width, limits)?;
    let mut fraction = zeroed_digits(fraction_width, limits)?;
    if let Some(value) = value {
        let padding = (integer_width - unpadded_integer) as usize;
        for (index, digit) in value.coefficient().digits().iter().copied().enumerate() {
            if (index as i128) < point {
                integer[padding + index] = digit;
            } else {
                let offset = (index as i128 - point) as usize;
                fraction[offset] = digit;
            }
        }
    }
    Ok((integer.into_boxed_slice(), fraction.into_boxed_slice()))
}

pub(super) fn round_scaled(
    value: ScaledDecimal<'_>,
    settings: &RoundingSettings,
    notation: SelectedNotation,
    limits: &NumericLimits,
) -> Result<RoundedDecimal, NumberFormatResourceError> {
    let raw = match settings.precision {
        Precision::Fraction(precision) => raw_fraction(value, precision, settings.mode, limits)?,
        Precision::Significant(range) => raw_significant(value, range, settings.mode, limits)?,
        Precision::More {
            fraction,
            significant,
        }
        | Precision::Less {
            fraction,
            significant,
        } => {
            let significant = raw_significant(value, significant, settings.mode, limits)?;
            let fraction = raw_fraction(
                value,
                FractionPrecision::Range(fraction),
                settings.mode,
                limits,
            )?;
            let fixed_is_more_precise = fraction.magnitude < significant.magnitude;
            let choose_fraction = match settings.precision {
                Precision::More { .. } => fixed_is_more_precise,
                Precision::Less { .. } => !fixed_is_more_precise,
                Precision::Fraction(_) | Precision::Significant(_) => unreachable!(),
            };
            if choose_fraction {
                fraction
            } else {
                significant
            }
        }
    };
    let required_fraction = raw
        .value
        .as_ref()
        .map_or(0, |value| (-i128::from(value.exponent())).max(0) as u128);
    let removable = raw.fraction_width - required_fraction;
    let mut fraction_width = raw.fraction_width - removable.min(u128::from(raw.trimmable_zeroes));
    if settings.trailing_zero_display == TrailingZeroDisplay::StripIfInteger
        && required_fraction == 0
    {
        fraction_width = 0;
    }
    let (integer_digits, fraction_digits) = render_digits(
        raw.value.as_ref(),
        fraction_width,
        settings.minimum_integer_digits,
        limits,
    )?;
    Ok(RoundedDecimal {
        sign: value.sign,
        rounded_mantissa: raw.value,
        integer_digits,
        fraction_digits,
        rounding_magnitude: raw.magnitude,
        notation,
    })
}

pub fn round_decimal(
    value: FiniteValue<'_>,
    settings: &RoundingSettings,
    limits: &NumericLimits,
) -> Result<RoundedDecimal, NumberFormatResourceError> {
    round_scaled(value.into(), settings, SelectedNotation::Standard, limits)
}
