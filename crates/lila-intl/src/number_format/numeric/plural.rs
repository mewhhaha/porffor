use core::cmp::Ordering;
use core::num::NonZeroU64;

use super::RoundedDecimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluralOperand {
    N,
    I,
    V,
    W,
    F,
    T,
    C,
    E,
}

impl PluralOperand {
    pub const fn name(self) -> &'static str {
        match self {
            Self::N => "n",
            Self::I => "i",
            Self::V => "v",
            Self::W => "w",
            Self::F => "f",
            Self::T => "t",
            Self::C => "c",
            Self::E => "e",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct DigitSpan<'a> {
    integer: &'a [u8],
    fraction: &'a [u8],
    start: u64,
    length: u64,
}

impl<'a> DigitSpan<'a> {
    const EMPTY: Self = Self {
        integer: &[],
        fraction: &[],
        start: 0,
        length: 0,
    };

    fn stored_length(self) -> u64 {
        self.integer.len() as u64 + self.fraction.len() as u64
    }

    fn stored_end(self) -> u64 {
        (self.start + self.length).min(self.stored_length())
    }

    fn stored_digit(self, absolute: u64) -> u8 {
        if absolute < self.integer.len() as u64 {
            self.integer[absolute as usize]
        } else {
            self.fraction[(absolute - self.integer.len() as u64) as usize]
        }
    }

    fn without_leading_zeroes(self) -> Self {
        match (self.start..self.stored_end()).find(|index| self.stored_digit(*index) != 0) {
            Some(start) => Self {
                start,
                length: self.length - (start - self.start),
                ..self
            },
            None => Self::EMPTY,
        }
    }

    fn without_trailing_zeroes(self) -> Self {
        match (self.start..self.stored_end()).rfind(|index| self.stored_digit(*index) != 0) {
            Some(last) => Self {
                length: last - self.start + 1,
                ..self
            },
            None => Self::EMPTY,
        }
    }

    fn has_nonzero(self) -> bool {
        (self.start..self.stored_end()).any(|index| self.stored_digit(index) != 0)
    }

    fn compare_integer(self, value: u64) -> Ordering {
        let digits = self.without_leading_zeroes();
        if digits.length == 0 {
            return 0_u64.cmp(&value);
        }
        let mut buffer = [0_u8; 20];
        let mut start = buffer.len();
        let mut remaining = value;
        loop {
            start -= 1;
            buffer[start] = (remaining % 10) as u8;
            remaining /= 10;
            if remaining == 0 {
                break;
            }
        }
        match digits.length.cmp(&((buffer.len() - start) as u64)) {
            Ordering::Equal => {}
            ordering => return ordering,
        }
        for (index, expected) in buffer[start..].iter().enumerate() {
            let absolute = digits.start + index as u64;
            let digit = if absolute < digits.stored_length() {
                digits.stored_digit(absolute)
            } else {
                0
            };
            match digit.cmp(expected) {
                Ordering::Equal => {}
                ordering => return ordering,
            }
        }
        Ordering::Equal
    }

    fn modulo(self, divisor: NonZeroU64) -> u64 {
        let divisor = u128::from(divisor.get());
        let mut remainder = 0_u128;
        let end = self.stored_end().max(self.start);
        for index in self.start..end {
            remainder = (remainder * 10 + u128::from(self.stored_digit(index))) % divisor;
        }
        let mut zeroes = self.length - (end - self.start);
        let mut power = 10_u128 % divisor;
        while zeroes != 0 {
            if zeroes & 1 != 0 {
                remainder = remainder * power % divisor;
            }
            power = power * power % divisor;
            zeroes >>= 1;
        }
        remainder as u64
    }
}

#[derive(Debug, Clone, Copy)]
enum PluralInteger<'a> {
    Small(u64),
    Digits(DigitSpan<'a>),
}

#[derive(Debug, Clone, Copy)]
pub struct ExactPluralOperand<'a> {
    integer: PluralInteger<'a>,
    fraction: DigitSpan<'a>,
}

impl<'a> ExactPluralOperand<'a> {
    fn small(value: u64) -> Self {
        Self {
            integer: PluralInteger::Small(value),
            fraction: DigitSpan::EMPTY,
        }
    }

    fn integer_digits(digits: DigitSpan<'a>) -> Self {
        Self {
            integer: PluralInteger::Digits(digits),
            fraction: DigitSpan::EMPTY,
        }
    }

    pub fn is_integer(self) -> bool {
        !self.fraction.has_nonzero()
    }

    pub fn compare_integer(self, value: u64) -> Ordering {
        let integer = match self.integer {
            PluralInteger::Small(integer) => integer.cmp(&value),
            PluralInteger::Digits(digits) => digits.compare_integer(value),
        };
        if integer == Ordering::Equal && self.fraction.has_nonzero() {
            Ordering::Greater
        } else {
            integer
        }
    }

    /// The fractional component is unchanged by integer-modulus remainder.
    pub fn modulo(self, divisor: NonZeroU64) -> Self {
        let remainder = match self.integer {
            PluralInteger::Small(integer) => integer % divisor.get(),
            PluralInteger::Digits(digits) => digits.modulo(divisor),
        };
        Self {
            integer: PluralInteger::Small(remainder),
            fraction: self.fraction,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PluralOperands<'a> {
    integer: DigitSpan<'a>,
    fraction: DigitSpan<'a>,
    trimmed_fraction: DigitSpan<'a>,
    compact_exponent: u32,
}

impl<'a> PluralOperands<'a> {
    pub fn new(rounded: &'a RoundedDecimal) -> Self {
        Self::with_compact_exponent(rounded, rounded.notation().compact_exponent())
    }

    fn with_compact_exponent(rounded: &'a RoundedDecimal, compact_exponent: u32) -> Self {
        let integer = rounded.integer_digits();
        let fraction = rounded.fraction_digits();
        // RoundedDecimal's total stored extent is <= u32::MAX. Adding a u32
        // compact exponent fits u64 and leaves large zero tails virtual.
        let point = integer.len() as u64 + u64::from(compact_exponent);
        let fraction_length = (fraction.len() as u64).saturating_sub(u64::from(compact_exponent));
        let integer_span = DigitSpan {
            integer,
            fraction,
            start: 0,
            length: point,
        };
        let fraction_span = DigitSpan {
            integer,
            fraction,
            start: point,
            length: fraction_length,
        };
        Self {
            integer: integer_span,
            fraction: fraction_span,
            trimmed_fraction: fraction_span.without_trailing_zeroes(),
            compact_exponent,
        }
    }

    pub fn operand(self, operand: PluralOperand) -> ExactPluralOperand<'a> {
        match operand {
            PluralOperand::N => ExactPluralOperand {
                integer: PluralInteger::Digits(self.integer),
                fraction: self.fraction,
            },
            PluralOperand::I => ExactPluralOperand::integer_digits(self.integer),
            PluralOperand::V => ExactPluralOperand::small(self.fraction.length),
            PluralOperand::W => ExactPluralOperand::small(self.trimmed_fraction.length),
            PluralOperand::F => ExactPluralOperand::integer_digits(self.fraction),
            PluralOperand::T => ExactPluralOperand::integer_digits(self.trimmed_fraction),
            PluralOperand::C | PluralOperand::E => {
                ExactPluralOperand::small(u64::from(self.compact_exponent))
            }
        }
    }
}

impl RoundedDecimal {
    /// Compact pattern selection uses the displayed mantissa before the compact
    /// exponent is attached; final source-number plural operands retain it.
    pub fn mantissa_plural_operands(&self) -> PluralOperands<'_> {
        PluralOperands::with_compact_exponent(self, 0)
    }

    pub fn plural_operands(&self) -> PluralOperands<'_> {
        PluralOperands::new(self)
    }
}
