use super::resource::prepend_digit;
use super::{
    DecimalCoefficient, ExactDecimal, NumberFormatResourceError, NumberSign, NumericLimits,
};

pub(super) fn trim_leading_zeroes(digits: &mut Vec<u8>) {
    let first = digits
        .iter()
        .position(|digit| *digit != 0)
        .unwrap_or(digits.len());
    digits.drain(..first);
}

pub(super) fn add_one(
    digits: &mut Vec<u8>,
    limits: &NumericLimits,
) -> Result<(), NumberFormatResourceError> {
    for digit in digits.iter_mut().rev() {
        if *digit != 9 {
            *digit += 1;
            return Ok(());
        }
        *digit = 0;
    }
    prepend_digit(digits, 1, limits)
}

pub(super) fn multiply_small(
    digits: &mut Vec<u8>,
    multiplier: u16,
    limits: &NumericLimits,
) -> Result<(), NumberFormatResourceError> {
    let mut carry = 0_u32;
    for digit in digits.iter_mut().rev() {
        let product = u32::from(*digit) * u32::from(multiplier) + carry;
        *digit = (product % 10) as u8;
        carry = product / 10;
    }
    while carry != 0 {
        prepend_digit(digits, (carry % 10) as u8, limits)?;
        carry /= 10;
    }
    Ok(())
}

pub(super) fn divide_small(digits: &mut Vec<u8>, divisor: u16) -> u16 {
    let mut remainder = 0_u32;
    for digit in digits.iter_mut() {
        let dividend = remainder * 10 + u32::from(*digit);
        *digit = (dividend / u32::from(divisor)) as u8;
        remainder = dividend % u32::from(divisor);
    }
    trim_leading_zeroes(digits);
    remainder as u16
}

pub(super) fn canonical_decimal(
    mut digits: Vec<u8>,
    exponent: i64,
    sign: NumberSign,
) -> Result<Option<ExactDecimal>, NumberFormatResourceError> {
    trim_leading_zeroes(&mut digits);
    if digits.is_empty() {
        return Ok(None);
    }
    let retained = digits.iter().rposition(|digit| *digit != 0).unwrap() + 1;
    let removed = i64::try_from(digits.len() - retained)
        .map_err(|_| NumberFormatResourceError::ExponentOverflow)?;
    let exponent = exponent
        .checked_add(removed)
        .ok_or(NumberFormatResourceError::ExponentOverflow)?;
    digits.truncate(retained);
    let coefficient = DecimalCoefficient::from_digits(digits.into_boxed_slice())
        .expect("decimal arithmetic produces digits with nonzero endpoints");
    Ok(Some(ExactDecimal::new(sign, coefficient, exponent)))
}
