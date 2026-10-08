//! Canonical, bounded decimal conversion of the real native BigInt limbs.
use super::*;

pub(super) fn decimal(
    negative: bool,
    mut limbs: Vec<u64>,
    budget: &mut SnapshotBudget,
) -> Result<SnapshotBigInt, SnapshotRejection> {
    if limbs.last() == Some(&0) || (limbs.is_empty() && negative) {
        return Err(invariant("noncanonical BigInt"));
    }
    let max_digits = budget.remaining(SnapshotBudgetDimension::BigIntDigits) as usize;
    let mut chunks = Vec::new();
    while !limbs.is_empty() {
        budget.work(limbs.len())?;
        if chunks.len() >= max_digits.div_ceil(9) {
            return Err(cap(SnapshotBudgetDimension::BigIntDigits));
        }
        let mut carry = 0u128;
        for limb in limbs.iter_mut().rev() {
            let value = (carry << 64) | u128::from(*limb);
            *limb = (value / 1_000_000_000) as u64;
            carry = value % 1_000_000_000;
        }
        chunks.push(carry as u32);
        while limbs.last() == Some(&0) {
            limbs.pop();
        }
    }
    let first = chunks.last().copied().unwrap_or(0);
    let digits =
        first.to_string().len() + chunks.len().saturating_sub(1) * 9 + usize::from(negative);
    if digits > max_digits {
        return Err(cap(SnapshotBudgetDimension::BigIntDigits));
    }
    let mut decimal = String::with_capacity(digits);
    if negative {
        decimal.push('-');
    }
    use std::fmt::Write;
    write!(&mut decimal, "{first}").map_err(invariant)?;
    for chunk in chunks.iter().rev().skip(1) {
        write!(&mut decimal, "{chunk:09}").map_err(invariant)?;
    }
    budget.take_bigint(decimal)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn budget(digits: u32) -> SnapshotBudget {
        SnapshotBudget::new(SnapshotLimits::new(1, 1, 1, 1, 1, digits, 4096, 1).unwrap())
    }
    #[test]
    fn sign_and_cross_limb_magnitude_fit_the_exact_decimal_budget() {
        assert_eq!(
            decimal(false, vec![], &mut budget(1)).unwrap().decimal(),
            "0"
        );
        assert_eq!(
            decimal(false, vec![0, 1], &mut budget(20))
                .unwrap()
                .decimal(),
            "18446744073709551616"
        );
        assert_eq!(
            decimal(true, vec![0, 1], &mut budget(21))
                .unwrap()
                .decimal(),
            "-18446744073709551616"
        );
        assert_eq!(
            decimal(true, vec![0, 1], &mut budget(20)),
            Err(cap(SnapshotBudgetDimension::BigIntDigits))
        );
        assert_eq!(
            decimal(false, vec![u64::MAX], &mut budget(20))
                .unwrap()
                .decimal(),
            "18446744073709551615"
        );
    }
    #[test]
    fn invalid_magnitude_and_budget_failure_cannot_publish_partial_decimal() {
        assert!(matches!(
            decimal(true, vec![], &mut budget(4)),
            Err(SnapshotRejection::BackendInvariant { .. })
        ));
        assert!(matches!(
            decimal(false, vec![1, 0], &mut budget(4)),
            Err(SnapshotRejection::BackendInvariant { .. })
        ));
        assert_eq!(
            decimal(false, vec![10_000_000_000], &mut budget(10)),
            Err(cap(SnapshotBudgetDimension::BigIntDigits))
        );
    }
}
