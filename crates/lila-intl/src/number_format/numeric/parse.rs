use core::cmp::Ordering;
use core::fmt;
use core::ops::Range;

use super::resource::zeroed_digits;
use super::{
    DecimalCoefficient, ExactDecimal, IntlMathematicalValue, NumberFormatResourceError, NumberSign,
    NumericLimits, ObservedNumericInput,
};

mod thresholds;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericWireError {
    NumberSpelling,
    BigIntSpelling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericNormalizationError {
    InvalidWire(NumericWireError),
    Resource(NumberFormatResourceError),
}

impl From<NumberFormatResourceError> for NumericNormalizationError {
    fn from(error: NumberFormatResourceError) -> Self {
        Self::Resource(error)
    }
}

impl fmt::Display for NumericNormalizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidWire(NumericWireError::NumberSpelling) => {
                formatter.write_str("invalid intrinsic Number decimal spelling")
            }
            Self::InvalidWire(NumericWireError::BigIntSpelling) => {
                formatter.write_str("invalid intrinsic BigInt decimal spelling")
            }
            Self::Resource(error) => fmt::Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for NumericNormalizationError {}

#[derive(Clone, Copy)]
enum NumericSource<'a> {
    Utf16(&'a [u16]),
    Ascii(&'a [u8]),
}

impl NumericSource<'_> {
    fn len(self) -> usize {
        match self {
            Self::Utf16(units) => units.len(),
            Self::Ascii(bytes) => bytes.len(),
        }
    }

    fn at(self, index: usize) -> u16 {
        match self {
            Self::Utf16(units) => units[index],
            Self::Ascii(bytes) => u16::from(bytes[index]),
        }
    }

    fn equals_ascii(self, range: Range<usize>, expected: &[u8]) -> bool {
        range.len() == expected.len()
            && range
                .zip(expected)
                .all(|(index, byte)| self.at(index) == u16::from(*byte))
    }
}

struct ParsedDecimal<'a> {
    source: NumericSource<'a>,
    integer: Range<usize>,
    fraction: Range<usize>,
    exponent: i128,
    sign: NumberSign,
}

impl ParsedDecimal<'_> {
    fn digit_count(&self) -> usize {
        self.integer.len() + self.fraction.len()
    }

    fn digit(&self, index: usize) -> u8 {
        let offset = if index < self.integer.len() {
            self.integer.start + index
        } else {
            self.fraction.start + index - self.integer.len()
        };
        (self.source.at(offset) - u16::from(b'0')) as u8
    }
}

fn is_decimal_digit(unit: u16) -> bool {
    (u16::from(b'0')..=u16::from(b'9')).contains(&unit)
}

fn is_string_whitespace(unit: u16) -> bool {
    matches!(
        unit,
        0x0009..=0x000d | 0x0020 | 0x00a0 | 0x1680 | 0x2000..=0x200a
            | 0x2028..=0x2029 | 0x202f | 0x205f | 0x3000 | 0xfeff
    )
}

fn parse_decimal(
    source: NumericSource<'_>,
    start: usize,
    end: usize,
    sign: NumberSign,
) -> Option<ParsedDecimal<'_>> {
    let mut cursor = start;
    while cursor < end && is_decimal_digit(source.at(cursor)) {
        cursor += 1;
    }
    let integer = start..cursor;
    let mut fraction = cursor..cursor;
    if cursor < end && source.at(cursor) == u16::from(b'.') {
        cursor += 1;
        let first = cursor;
        while cursor < end && is_decimal_digit(source.at(cursor)) {
            cursor += 1;
        }
        fraction = first..cursor;
    }
    if integer.is_empty() && fraction.is_empty() {
        return None;
    }
    let mut exponent = 0_i128;
    if cursor < end && matches!(source.at(cursor), 0x45 | 0x65) {
        cursor += 1;
        let mut negative = false;
        if cursor < end && matches!(source.at(cursor), 0x2b | 0x2d) {
            negative = source.at(cursor) == u16::from(b'-');
            cursor += 1;
        }
        let first = cursor;
        while cursor < end && is_decimal_digit(source.at(cursor)) {
            // The ABI bounds the source at u32 units. Once larger than u64::MAX,
            // an exponent cannot be balanced by the explicit significand.
            exponent = (exponent * 10 + i128::from(source.at(cursor) - u16::from(b'0')))
                .min(i128::from(u64::MAX));
            cursor += 1;
        }
        if cursor == first {
            return None;
        }
        if negative {
            exponent = -exponent;
        }
    }
    if cursor != end {
        return None;
    }
    Some(ParsedDecimal {
        source,
        integer,
        fraction,
        exponent,
        sign,
    })
}

fn compare_to_threshold(
    decimal: &ParsedDecimal<'_>,
    first: usize,
    length: usize,
    threshold: &[u8],
) -> Ordering {
    for index in 0..length.max(threshold.len()) {
        let left = if index < length {
            decimal.digit(first + index)
        } else {
            0
        };
        let right = threshold.get(index).map_or(0, |digit| *digit - b'0');
        match left.cmp(&right) {
            Ordering::Equal => {}
            ordering => return ordering,
        }
    }
    Ordering::Equal
}

fn normalize_decimal(
    decimal: ParsedDecimal<'_>,
    limits: &NumericLimits,
) -> Result<IntlMathematicalValue, NumberFormatResourceError> {
    let length = decimal.digit_count();
    let Some(first) = (0..length).find(|index| decimal.digit(*index) != 0) else {
        return Ok(IntlMathematicalValue::Zero(decimal.sign));
    };
    let last = (first..length)
        .rfind(|index| decimal.digit(*index) != 0)
        .unwrap();
    let retained = last - first + 1;
    let exponent = decimal.exponent - decimal.fraction.len() as i128 + (length - last - 1) as i128;
    let magnitude = exponent + retained as i128 - 1;
    if magnitude < -324
        || (magnitude == -324
            && compare_to_threshold(&decimal, first, retained, thresholds::UNDERFLOW)
                != Ordering::Greater)
    {
        return Ok(IntlMathematicalValue::Zero(decimal.sign));
    }
    if magnitude > 308
        || (magnitude == 308
            && compare_to_threshold(&decimal, first, retained, thresholds::OVERFLOW)
                != Ordering::Less)
    {
        return Ok(IntlMathematicalValue::Infinity(decimal.sign));
    }
    let mut digits = zeroed_digits(retained as u128, limits)?;
    for (index, digit) in digits.iter_mut().enumerate() {
        *digit = decimal.digit(first + index);
    }
    let exponent =
        i64::try_from(exponent).map_err(|_| NumberFormatResourceError::ExponentOverflow)?;
    Ok(IntlMathematicalValue::Finite(ExactDecimal::new(
        decimal.sign,
        DecimalCoefficient::from_digits(digits.into_boxed_slice())
            .expect("the parsed coefficient has nonzero endpoints"),
        exponent,
    )))
}

fn radix_digit(unit: u16, radix: u16) -> Option<u16> {
    let digit = match unit {
        0x30..=0x39 => unit - 0x30,
        0x41..=0x46 => unit - 0x41 + 10,
        0x61..=0x66 => unit - 0x61 + 10,
        _ => return None,
    };
    (digit < radix).then_some(digit)
}

fn normalize_radix(
    source: NumericSource<'_>,
    start: usize,
    end: usize,
    radix: u16,
    limits: &NumericLimits,
) -> Result<IntlMathematicalValue, NumberFormatResourceError> {
    if start == end || (start..end).any(|index| radix_digit(source.at(index), radix).is_none()) {
        return Ok(IntlMathematicalValue::NaN);
    }
    // Any integer with 310 decimal digits overflows binary64. This fixed
    // scratch bound also avoids input-sized allocation for a huge radix string.
    let mut digits = [0_u8; 310];
    let mut used = 1;
    for index in start..end {
        let mut carry = radix_digit(source.at(index), radix).unwrap();
        for digit in &mut digits[..used] {
            let product = u16::from(*digit) * radix + carry;
            *digit = (product % 10) as u8;
            carry = product / 10;
        }
        while carry != 0 {
            if used == digits.len() {
                return Ok(IntlMathematicalValue::Infinity(NumberSign::Positive));
            }
            digits[used] = (carry % 10) as u8;
            used += 1;
            carry /= 10;
        }
        if used == 310 {
            return Ok(IntlMathematicalValue::Infinity(NumberSign::Positive));
        }
    }
    if used == 1 && digits[0] == 0 {
        return Ok(IntlMathematicalValue::Zero(NumberSign::Positive));
    }
    if used == 309 {
        let ordering = digits[..used]
            .iter()
            .rev()
            .copied()
            .cmp(thresholds::OVERFLOW.iter().map(|byte| byte - b'0'));
        if ordering != Ordering::Less {
            return Ok(IntlMathematicalValue::Infinity(NumberSign::Positive));
        }
    }
    let zeroes = digits[..used]
        .iter()
        .take_while(|digit| **digit == 0)
        .count();
    let mut coefficient = zeroed_digits((used - zeroes) as u128, limits)?;
    for (target, source) in coefficient
        .iter_mut()
        .zip(digits[zeroes..used].iter().rev())
    {
        *target = *source;
    }
    Ok(IntlMathematicalValue::Finite(ExactDecimal::new(
        NumberSign::Positive,
        DecimalCoefficient::from_digits(coefficient.into_boxed_slice()).unwrap(),
        zeroes as i64,
    )))
}

fn normalize_string(
    source: NumericSource<'_>,
    limits: &NumericLimits,
) -> Result<IntlMathematicalValue, NumberFormatResourceError> {
    let mut start = 0;
    let mut end = source.len();
    while start < end && is_string_whitespace(source.at(start)) {
        start += 1;
    }
    while end > start && is_string_whitespace(source.at(end - 1)) {
        end -= 1;
    }
    if start == end {
        return Ok(IntlMathematicalValue::Zero(NumberSign::Positive));
    }
    if end - start >= 2 && source.at(start) == u16::from(b'0') {
        let radix = match source.at(start + 1) {
            0x42 | 0x62 => Some(2),
            0x4f | 0x6f => Some(8),
            0x58 | 0x78 => Some(16),
            _ => None,
        };
        if let Some(radix) = radix {
            return normalize_radix(source, start + 2, end, radix, limits);
        }
    }
    let mut sign = NumberSign::Positive;
    if matches!(source.at(start), 0x2b | 0x2d) {
        if source.at(start) == u16::from(b'-') {
            sign = NumberSign::Negative;
        }
        start += 1;
    }
    if source.equals_ascii(start..end, b"Infinity") {
        return Ok(IntlMathematicalValue::Infinity(sign));
    }
    match parse_decimal(source, start, end, sign) {
        Some(decimal) => normalize_decimal(decimal, limits),
        None => Ok(IntlMathematicalValue::NaN),
    }
}

fn normalize_bigint(
    spelling: &str,
    limits: &NumericLimits,
) -> Result<IntlMathematicalValue, NumericNormalizationError> {
    let (sign, digits) = match spelling.strip_prefix('-') {
        Some(digits) => (NumberSign::Negative, digits.as_bytes()),
        None => (NumberSign::Positive, spelling.as_bytes()),
    };
    if digits.is_empty()
        || !digits.iter().all(u8::is_ascii_digit)
        || (digits[0] == b'0' && (digits.len() != 1 || sign == NumberSign::Negative))
    {
        return Err(NumericNormalizationError::InvalidWire(
            NumericWireError::BigIntSpelling,
        ));
    }
    if digits == b"0" {
        return Ok(IntlMathematicalValue::Zero(NumberSign::Positive));
    }
    let last = digits.iter().rposition(|byte| *byte != b'0').unwrap() + 1;
    let mut coefficient = zeroed_digits(last as u128, limits)?;
    for (target, source) in coefficient.iter_mut().zip(&digits[..last]) {
        *target = *source - b'0';
    }
    let exponent = i64::try_from(digits.len() - last)
        .map_err(|_| NumberFormatResourceError::ExponentOverflow)?;
    Ok(IntlMathematicalValue::Finite(ExactDecimal::new(
        sign,
        DecimalCoefficient::from_digits(coefficient.into_boxed_slice()).unwrap(),
        exponent,
    )))
}

fn normalize_number(
    spelling: &str,
    limits: &NumericLimits,
) -> Result<IntlMathematicalValue, NumericNormalizationError> {
    let invalid = || NumericNormalizationError::InvalidWire(NumericWireError::NumberSpelling);
    match spelling {
        "NaN" => return Ok(IntlMathematicalValue::NaN),
        "Infinity" => return Ok(IntlMathematicalValue::Infinity(NumberSign::Positive)),
        "-Infinity" => return Ok(IntlMathematicalValue::Infinity(NumberSign::Negative)),
        "0" => return Ok(IntlMathematicalValue::Zero(NumberSign::Positive)),
        _ => {}
    }
    // The origin is compiler-proven. Validate the Number::toString spelling
    // shape here; do not reconstruct its value through the host's f64 parser.
    let unsigned = spelling.strip_prefix('-').unwrap_or(spelling);
    if spelling.len() > 25 || !spelling.is_ascii() || unsigned.is_empty() {
        return Err(invalid());
    }
    let sign = if spelling.starts_with('-') {
        NumberSign::Negative
    } else {
        NumberSign::Positive
    };
    let source = NumericSource::Ascii(unsigned.as_bytes());
    let Some(decimal) = parse_decimal(source, 0, source.len(), sign) else {
        return Err(invalid());
    };
    if decimal.integer.is_empty()
        || (decimal.integer.len() > 1 && source.at(0) == u16::from(b'0'))
        || (!decimal.fraction.is_empty() && source.at(decimal.fraction.end - 1) == u16::from(b'0'))
        || (decimal.integer.end < source.len()
            && source.at(decimal.integer.end) == u16::from(b'.')
            && decimal.fraction.is_empty())
        || unsigned.ends_with('.')
        || unsigned.contains('E')
    {
        return Err(invalid());
    }
    let scientific = unsigned.find('e');
    if let Some(index) = scientific {
        let exponent = &unsigned[index + 1..];
        if decimal.integer.len() != 1
            || source.at(0) == u16::from(b'0')
            || !exponent.starts_with(['+', '-'])
            || exponent.len() < 2
            || exponent.as_bytes()[1] == b'0'
        {
            return Err(invalid());
        }
    }
    let value = normalize_decimal(decimal, limits)?;
    let IntlMathematicalValue::Finite(ref finite) = value else {
        return Err(invalid());
    };
    let magnitude = i128::from(finite.exponent()) + finite.coefficient().digits().len() as i128 - 1;
    if finite.coefficient().digits().len() > 17
        || scientific.is_some() != !(-6..21).contains(&magnitude)
    {
        return Err(invalid());
    }
    Ok(value)
}

pub fn normalize_numeric_input(
    input: ObservedNumericInput,
    limits: &NumericLimits,
) -> Result<IntlMathematicalValue, NumericNormalizationError> {
    match input {
        ObservedNumericInput::StringNumericLiteral(units) => {
            normalize_string(NumericSource::Utf16(&units), limits).map_err(Into::into)
        }
        ObservedNumericInput::NumberShortestDecimal(spelling) => {
            normalize_number(&spelling, limits)
        }
        ObservedNumericInput::NegativeZero => Ok(IntlMathematicalValue::Zero(NumberSign::Negative)),
        ObservedNumericInput::BigIntDecimal(spelling) => normalize_bigint(&spelling, limits),
    }
}
