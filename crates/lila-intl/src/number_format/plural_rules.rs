use core::{cmp::Ordering, num::NonZeroU64};

use super::numeric::{PluralOperand, PluralOperands, RoundedDecimal};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum CardinalCategory {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

impl CardinalCategory {
    pub const ALL: [Self; 6] = [
        Self::Zero,
        Self::One,
        Self::Two,
        Self::Few,
        Self::Many,
        Self::Other,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    pub fn decode(value: u8) -> Option<Self> {
        Self::ALL.get(usize::from(value)).copied()
    }
}

#[derive(Debug)]
pub(super) struct OperandRelation {
    pub operand: PluralOperand,
    pub modulus: Option<NonZeroU64>,
    pub integer_only: bool,
    pub negate: bool,
    pub ranges: Box<[(u64, u64)]>,
}

impl OperandRelation {
    fn matches(&self, operands: PluralOperands<'_>) -> bool {
        let mut operand = operands.operand(self.operand);
        if let Some(modulus) = self.modulus {
            operand = operand.modulo(modulus);
        }
        let included = (!self.integer_only || operand.is_integer())
            && self.ranges.iter().any(|&(lower, upper)| {
                operand.compare_integer(lower) != Ordering::Less
                    && operand.compare_integer(upper) != Ordering::Greater
            });
        included != self.negate
    }
}

#[derive(Debug)]
pub(super) struct CategoryRule {
    pub category: CardinalCategory,
    pub alternatives: Box<[Box<[OperandRelation]>]>,
}

#[derive(Debug)]
pub(super) struct CardinalRules(pub Box<[CategoryRule]>);

impl CardinalRules {
    pub fn select(&self, operands: PluralOperands<'_>) -> CardinalCategory {
        for rule in &self.0 {
            if rule.alternatives.iter().any(|conjunction| {
                conjunction
                    .iter()
                    .all(|relation| relation.matches(operands))
            }) {
                return rule.category;
            }
        }
        CardinalCategory::Other
    }
}

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
