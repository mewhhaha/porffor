use std::{cmp::Ordering, sync::Arc};

/// An exact mathematical natural number retained in canonical decimal form.
/// Construction validates digits once and removes only redundant leading zeros.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegExpNatural {
    digits: Arc<[u8]>,
}

impl RegExpNatural {
    pub fn from_decimal_digits(digits: &[u8]) -> Option<Self> {
        if digits.is_empty() || digits.iter().any(|digit| !digit.is_ascii_digit()) {
            return None;
        }
        let first = digits
            .iter()
            .position(|&digit| digit != b'0')
            .unwrap_or(digits.len() - 1);
        Some(Self {
            digits: Arc::from(&digits[first..]),
        })
    }

    pub fn from_u64(value: u64) -> Self {
        Self {
            digits: Arc::from(value.to_string().into_bytes()),
        }
    }

    pub fn digits(&self) -> &[u8] {
        &self.digits
    }
    pub fn is_zero(&self) -> bool {
        self.digits() == b"0"
    }
    pub fn is_one(&self) -> bool {
        self.digits() == b"1"
    }

    pub fn checked_to_u64(&self) -> Option<u64> {
        self.digits.iter().try_fold(0_u64, |value, digit| {
            value.checked_mul(10)?.checked_add(u64::from(digit - b'0'))
        })
    }

    pub(crate) fn counter_byte_length(&self) -> Option<usize> {
        self.digits
            .len()
            .checked_add(REGEXP_REPEAT_COUNTER_DECIMAL_DIGITS - 1)?
            .checked_div(REGEXP_REPEAT_COUNTER_DECIMAL_DIGITS)?
            .checked_mul(REGEXP_REPEAT_COUNTER_LIMB_WIDTH)
    }
}

impl Ord for RegExpNatural {
    fn cmp(&self, other: &Self) -> Ordering {
        self.digits
            .len()
            .cmp(&other.digits.len())
            .then_with(|| self.digits.cmp(&other.digits))
    }
}
impl PartialOrd for RegExpNatural {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegExpRepeatMaximum {
    Finite(RegExpNatural),
    Unbounded,
}

/// Exact ordered bounds. A finite maximum cannot be below its minimum.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegExpRepeatBounds {
    minimum: RegExpNatural,
    maximum: RegExpRepeatMaximum,
}

impl RegExpRepeatBounds {
    pub fn new(minimum: RegExpNatural, maximum: RegExpRepeatMaximum) -> Option<Self> {
        if matches!(&maximum, RegExpRepeatMaximum::Finite(value) if value < &minimum) {
            return None;
        }
        Some(Self { minimum, maximum })
    }
    pub fn minimum(&self) -> &RegExpNatural {
        &self.minimum
    }
    pub fn maximum(&self) -> &RegExpRepeatMaximum {
        &self.maximum
    }

    pub(crate) fn state_byte_length(&self) -> Option<usize> {
        let maximum = match &self.maximum {
            RegExpRepeatMaximum::Finite(value) => value.counter_byte_length()?,
            RegExpRepeatMaximum::Unbounded => 0,
        };
        REGEXP_REPEAT_STATE_HEADER_SIZE
            .checked_add(self.minimum.counter_byte_length()?)?
            .checked_add(maximum)?
            .checked_add(7)
            .map(|length| length & !7)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum RegExpRepeatMaximumKind {
    Finite = 0,
    Unbounded = 1,
}
impl RegExpRepeatMaximumKind {
    pub const fn from_word(word: u64) -> Option<Self> {
        match word {
            0 => Some(Self::Finite),
            1 => Some(Self::Unbounded),
            _ => None,
        }
    }
    pub const fn word(self) -> u64 {
        self as u64
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum RegExpRepeatBoundWord {
    MinimumDigitsOffset,
    MinimumDigitsLength,
    MaximumKind,
    MaximumDigitsOffset,
    MaximumDigitsLength,
    StateOffset,
}
impl RegExpRepeatBoundWord {
    pub const ALL: [Self; 6] = [
        Self::MinimumDigitsOffset,
        Self::MinimumDigitsLength,
        Self::MaximumKind,
        Self::MaximumDigitsOffset,
        Self::MaximumDigitsLength,
        Self::StateOffset,
    ];
    pub const fn offset(self) -> u64 {
        self as u64 * 8
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum RegExpRepeatStateWord {
    Active,
    MinimumUsed,
    MaximumUsed,
    PreUtf16,
    Stage,
}
impl RegExpRepeatStateWord {
    pub const fn offset(self) -> u64 {
        self as u64 * 8
    }
}

pub const REGEXP_REPEAT_BOUND_RECORD_SIZE: usize = 48;
pub const REGEXP_REPEAT_STATE_HEADER_SIZE: usize = 40;
pub const REGEXP_REPEAT_COUNTER_LIMB_WIDTH: usize = 4;
pub const REGEXP_REPEAT_COUNTER_DECIMAL_DIGITS: usize = 9;
pub const REGEXP_REPEAT_COUNTER_RADIX: u64 = 1_000_000_000;
