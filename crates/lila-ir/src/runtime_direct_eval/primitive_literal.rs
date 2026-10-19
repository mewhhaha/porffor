//! An inert literal is admitted from actual lowered IR, never source matching.
use super::RuntimeDirectEvalProgram;
use crate::{ExprIr, StatementIr, ValueInfo, ValueKind};

/// Closed scalar operands; strings, symbols, BigInts and objects need ownership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDirectEvalPrimitive {
    Undefined,
    Null,
    Boolean(bool),
    Number(u64),
}

/// Only successful parsing/lowering and inert preparation can mint this proof.
#[derive(Debug)]
pub struct PrimitiveRuntimeDirectEval<'a> {
    _program: &'a RuntimeDirectEvalProgram,
    value: RuntimeDirectEvalPrimitive,
}

impl PrimitiveRuntimeDirectEval<'_> {
    pub fn value(&self) -> RuntimeDirectEvalPrimitive {
        self.value
    }
}

impl RuntimeDirectEvalProgram {
    /// The sole executed expression is a scalar literal. No declaration,
    /// environment, source value, function or allocation can escape this P.
    pub fn primitive_literal(&self) -> Option<PrimitiveRuntimeDirectEval<'_>> {
        let unit = self.inert_preparation()?;
        let [StatementIr::Expression(expression)] = unit.body.statements.as_slice() else {
            return None;
        };
        let (value, kind) = match expression.expr {
            ExprIr::Undefined => (RuntimeDirectEvalPrimitive::Undefined, ValueKind::Undefined),
            ExprIr::Null => (RuntimeDirectEvalPrimitive::Null, ValueKind::Null),
            ExprIr::Boolean(value) => (
                RuntimeDirectEvalPrimitive::Boolean(value),
                ValueKind::Boolean,
            ),
            ExprIr::Number(bits) => (RuntimeDirectEvalPrimitive::Number(bits), ValueKind::Number),
            _ => return None,
        };
        if expression.value_info() != ValueInfo::new(kind) {
            return None;
        }
        Some(PrimitiveRuntimeDirectEval {
            _program: self,
            value,
        })
    }
}
