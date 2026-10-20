use super::numeric::{PluralOperand, PluralOperands, RoundedDecimal};
pub(super) use crate::plural_rules::rules::{
    CategoryRule, OperandRelation, PluralCategory as CardinalCategory, PluralRules as CardinalRules,
};
use core::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PluralSelectionPurpose {
    CompactPattern,
    Measurement,
}

impl PluralSelectionPurpose {
    pub fn operands(self, rounded: &RoundedDecimal) -> PluralOperands<'_> {
        match self {
            Self::CompactPattern => rounded.mantissa_plural_operands(),
            // LDML compact source-number operands differ from its pattern
            // selector. Scientific/engineering names use ECMA-402's final
            // scaled RoundedNumber, with no compact c/e exponent.
            Self::Measurement => rounded.plural_operands(),
        }
    }
}

#[derive(Debug)]
pub(super) struct PluralVariants<T> {
    pub categories: [T; 6],
    pub exact_zero: Option<T>,
    pub exact_one: Option<T>,
}

impl<T: Copy> PluralVariants<T> {
    pub fn select(&self, category: CardinalCategory, operands: Option<PluralOperands<'_>>) -> T {
        if let Some(operands) = operands {
            let value = operands.operand(PluralOperand::N);
            if let Some(exact_zero) = self.exact_zero {
                if value.compare_integer(0) == Ordering::Equal {
                    return exact_zero;
                }
            }
            if let Some(exact_one) = self.exact_one {
                if value.compare_integer(1) == Ordering::Equal {
                    return exact_one;
                }
            }
        }
        self.categories[category.index()]
    }

    pub fn all(&self) -> impl Iterator<Item = T> + '_ {
        self.categories
            .iter()
            .copied()
            .chain(self.exact_zero)
            .chain(self.exact_one)
    }
}
