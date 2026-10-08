use core::{cmp::Ordering, num::NonZeroU64};

use crate::number_format::numeric::{PluralOperand, PluralOperands};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PluralCategory {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

impl PluralCategory {
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

    pub const fn name(self) -> &'static str {
        match self {
            Self::Zero => "zero",
            Self::One => "one",
            Self::Two => "two",
            Self::Few => "few",
            Self::Many => "many",
            Self::Other => "other",
        }
    }
    pub const fn wire_code(self) -> u64 {
        self as u64
    }
    pub fn from_wire_code(code: u64) -> Option<Self> {
        u8::try_from(code).ok().and_then(Self::decode)
    }
    pub fn decode(value: u8) -> Option<Self> {
        Self::ALL.get(usize::from(value)).copied()
    }
}

#[derive(Debug)]
pub(crate) struct OperandRelation {
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
pub(crate) struct CategoryRule {
    pub category: PluralCategory,
    pub alternatives: Box<[Box<[OperandRelation]>]>,
}

#[derive(Debug)]
pub(crate) struct PluralRules(pub Box<[CategoryRule]>);

impl PluralRules {
    pub fn select(&self, operands: PluralOperands<'_>) -> PluralCategory {
        for rule in &self.0 {
            if rule.alternatives.iter().any(|conjunction| {
                conjunction
                    .iter()
                    .all(|relation| relation.matches(operands))
            }) {
                return rule.category;
            }
        }
        PluralCategory::Other
    }
}

/// Checked ordered set of categories; Other is always available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PluralCategorySet(u8);
impl PluralCategorySet {
    pub fn from_wire_mask(mask: u64) -> Option<Self> {
        let mask = u8::try_from(mask).ok()?;
        (mask & !63 == 0 && mask & (1 << PluralCategory::Other.index()) != 0).then_some(Self(mask))
    }
    pub const fn wire_mask(self) -> u64 {
        self.0 as u64
    }
    pub fn contains(self, category: PluralCategory) -> bool {
        self.0 & (1 << category.index()) != 0
    }
}
impl PluralRules {
    pub(crate) fn categories(&self) -> PluralCategorySet {
        let mut mask = 1 << PluralCategory::Other.index();
        for rule in &self.0 {
            mask |= 1 << rule.category.index();
        }
        PluralCategorySet(mask)
    }
}
const _: () = {
    assert!(PluralCategory::ALL.len() < u8::BITS as usize);
};
