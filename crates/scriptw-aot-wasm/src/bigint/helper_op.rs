use super::*;

// One row creates the enum, its complete dispatcher census and its i32 wire
// projection. A new operation cannot be omitted from the dispatcher census.
macro_rules! bigint_helper_operations {
    ($( $(#[$doc:meta])* $operation:ident => $code:literal, )+) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) enum BigIntHelperOp { $( $(#[$doc])* $operation, )+ }
        impl BigIntHelperOp {
            pub(crate) const ALL: &'static [Self] = &[$(Self::$operation),+];
            pub(crate) const fn runtime_code(&self) -> i32 {
                match self { $( Self::$operation => $code, )+ }
            }
        }
    };
}

bigint_helper_operations! {
    Add => 0,
    Sub => 1,
    Mul => 2,
    Div => 3,
    Rem => 4,
    Exp => 5,
    /// Exact three-way BigInt comparison, returned as a Number.
    Compare => 6,
    /// Unary negation; the right magnitude is not used by the operation.
    Negate => 7,
    /// Exact BigInt/Number comparison; NaN remains unordered.
    CompareWithNumber => 8,
    BitAnd => 9,
    BitOr => 10,
    BitXor => 11,
    Shl => 12,
    Shr => 13,
}

impl BigIntHelperOp {
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
