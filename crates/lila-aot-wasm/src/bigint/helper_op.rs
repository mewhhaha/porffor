use super::*;

/// Operation selector passed to the shared BigInt helper in parameter 4.
pub(crate) enum BigIntHelperOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Exp,
    /// Numeric three-way comparison; the result is a Number (-1, 0 or 1).
    Compare,
    /// Unary negation of the left operand; the right operand is ignored.
    Negate,
    /// Exact three-way comparison of a BigInt against a Number (right operand
    /// payload is f64 bits). Yields NaN when the Number is NaN, so the caller's
    /// float comparison against zero is false for every relational operator —
    /// which is what ECMA-262 7.2.13 requires.
    CompareWithNumber,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

impl BigIntHelperOp {
    pub(crate) const fn runtime_code(&self) -> i64 {
        match self {
            Self::Add => 0,
            Self::Sub => 1,
            Self::Mul => 2,
            Self::Div => 3,
            Self::Rem => 4,
            Self::Exp => 5,
            Self::Compare => 6,
            Self::Negate => 7,
            Self::CompareWithNumber => 8,
            Self::BitAnd => 9,
            Self::BitOr => 10,
            Self::BitXor => 11,
            Self::Shl => 12,
            Self::Shr => 13,
        }
    }

    pub(crate) const fn from_arithmetic(op: ArithmeticBinaryOp) -> Self {
        match op {
            ArithmeticBinaryOp::Add => Self::Add,
            ArithmeticBinaryOp::Sub => Self::Sub,
            ArithmeticBinaryOp::Mul => Self::Mul,
            ArithmeticBinaryOp::Div => Self::Div,
            ArithmeticBinaryOp::Mod => Self::Rem,
            ArithmeticBinaryOp::Exp => Self::Exp,
        }
    }

    pub(crate) const fn from_bitwise(op: BigIntBitwiseOp) -> Self {
        match op {
            BigIntBitwiseOp::And => Self::BitAnd,
            BigIntBitwiseOp::Or => Self::BitOr,
            BigIntBitwiseOp::Xor => Self::BitXor,
            BigIntBitwiseOp::Shl => Self::Shl,
            BigIntBitwiseOp::Shr => Self::Shr,
        }
    }
}
