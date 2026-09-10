//! One runtime ResolveBinding lifecycle; no Reference escapes as a JS value.
use crate::{
    ArithmeticBinaryOp, BitwiseBinaryOp, LogicalBinaryOp, NumericUpdateOp, Strictness, TypedExpr,
    UpdateReturnMode,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentIdentifierIr {
    pub name: String,
    pub strictness: Strictness,
    pub operation: EnvironmentIdentifierOperationIr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentIdentifierOperationIr {
    Read,
    Typeof,
    Assign {
        value: Box<TypedExpr>,
    },
    Delete,
    Update {
        operation: NumericUpdateOp,
        return_mode: UpdateReturnMode,
    },
    EagerCompound {
        operation: EnvironmentCompoundOperationIr,
        rhs: Box<TypedExpr>,
    },
    LogicalCompound {
        operation: LogicalBinaryOp,
        rhs: Box<TypedExpr>,
    },
    Call {
        args: Vec<TypedExpr>,
        direct_eval: Option<crate::DirectEvalContextIr>,
    },
}

impl EnvironmentIdentifierOperationIr {
    pub fn operands(&self) -> impl Iterator<Item = &TypedExpr> {
        let operands: &[TypedExpr] = match self {
            Self::Read | Self::Typeof | Self::Delete | Self::Update { .. } => &[],
            Self::Assign { value } => std::slice::from_ref(value),
            Self::EagerCompound { rhs, .. } | Self::LogicalCompound { rhs, .. } => {
                std::slice::from_ref(rhs)
            }
            Self::Call { args, .. } => args,
        };
        operands.iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentCompoundOperationIr {
    Add,
    Arithmetic(ArithmeticBinaryOp),
    Bitwise(BitwiseBinaryOp),
}
