use core::fmt;
use core::num::NonZeroU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumericLimits {
    maximum_digits: NonZeroU32,
}

impl NumericLimits {
    pub const HOST_ABI: Self = Self {
        maximum_digits: NonZeroU32::MAX,
    };

    pub const fn new(maximum_digits: NonZeroU32) -> Self {
        Self { maximum_digits }
    }

    pub const fn maximum_digits(self) -> u32 {
        self.maximum_digits.get()
    }

    pub(super) fn check(self, count: u128) -> Result<usize, NumberFormatResourceError> {
        if count > u128::from(self.maximum_digits.get()) {
            return Err(NumberFormatResourceError::DigitExtent {
                requested: count,
                maximum: self.maximum_digits.get(),
            });
        }
        usize::try_from(count).map_err(|_| NumberFormatResourceError::DigitExtent {
            requested: count,
            maximum: self.maximum_digits.get(),
        })
    }
}

impl Default for NumericLimits {
    fn default() -> Self {
        Self::HOST_ABI
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberFormatResourceError {
    DigitExtent { requested: u128, maximum: u32 },
    Allocation,
    ExponentOverflow,
}

impl fmt::Display for NumberFormatResourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DigitExtent { requested, maximum } => write!(
                formatter,
                "NumberFormat needs {requested} decimal digits; the resource limit is {maximum}"
            ),
            Self::Allocation => formatter.write_str("NumberFormat digit allocation failed"),
            Self::ExponentOverflow => formatter.write_str("NumberFormat decimal exponent overflow"),
        }
    }
}

impl std::error::Error for NumberFormatResourceError {}

pub(super) fn zeroed_digits(
    count: u128,
    limits: &NumericLimits,
) -> Result<Vec<u8>, NumberFormatResourceError> {
    let count = limits.check(count)?;
    let mut digits = Vec::new();
    digits
        .try_reserve_exact(count)
        .map_err(|_| NumberFormatResourceError::Allocation)?;
    digits.resize(count, 0);
    Ok(digits)
}

pub(super) fn prepend_digit(
    digits: &mut Vec<u8>,
    digit: u8,
    limits: &NumericLimits,
) -> Result<(), NumberFormatResourceError> {
    limits.check(digits.len() as u128 + 1)?;
    digits
        .try_reserve(1)
        .map_err(|_| NumberFormatResourceError::Allocation)?;
    digits.insert(0, digit);
    Ok(())
}
