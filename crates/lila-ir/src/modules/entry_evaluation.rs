//! The trusted Module entry evaluation and its host completion owner.

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleEntryEvaluationKindIr {
    Promise,
}

/// An entry operation has no JavaScript result. Its canonical evaluation
/// Promise belongs to the host checkpoint rather than rejection tracking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleEntryEvaluationIr {
    kind: ModuleEntryEvaluationKindIr,
    evaluation: Box<TypedExpr>,
}

impl ModuleEntryEvaluationIr {
    pub const fn kind(&self) -> ModuleEntryEvaluationKindIr {
        self.kind
    }

    pub fn evaluation(&self) -> &TypedExpr {
        &self.evaluation
    }

    /// Only linker-owned entry operations occupy this position. A Script with
    /// the same arrow/call source has no private operation and returns None.
    pub fn in_root_block(block: &BlockIr) -> Option<&Self> {
        match block.statements.last()? {
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::ModuleEntryEvaluation(entry),
                ..
            }) => Some(entry),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum LinkedModuleEntry {
    CanonicalGraph(ModuleUnitId),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ModuleEntryEvaluationBoundary<'a> {
    expression: &'a Expression,
    operand: &'a Expression,
}

impl LinkedModuleEntry {
    pub(super) fn apply<'a>(self, script: &'a Script, analysis: &mut Analysis<'a>) {
        let Some(StatementListItem::Statement(statement)) = script.statements().statements().last()
        else {
            panic!("trusted Module entry ends with an evaluation expression");
        };
        let Statement::Expression(expression) = statement.as_ref() else {
            panic!("trusted Module entry ends with an evaluation expression");
        };
        let operand = match self {
            Self::CanonicalGraph(module) => {
                let evaluation = analysis
                    .module_execution
                    .evaluations
                    .get(&(std::ptr::from_ref(expression) as usize))
                    .expect("the final operation evaluates the linked entry");
                assert_eq!(evaluation.module(), module);
                expression
            }
        };
        assert!(analysis.module_entry_evaluation.is_none());
        analysis.module_entry_evaluation = Some(ModuleEntryEvaluationBoundary {
            expression,
            operand,
        });
    }
}

impl<'a> ModuleEntryEvaluationBoundary<'a> {
    pub(crate) fn owns(self, expression: &Expression) -> bool {
        std::ptr::eq(self.expression, expression)
    }

    pub(crate) const fn operand(self) -> &'a Expression {
        self.operand
    }

    pub(crate) fn lower(self, evaluation: TypedExpr) -> TypedExpr {
        TypedExpr::from_info(
            ValueInfo::undefined(),
            ExprIr::ModuleEntryEvaluation(ModuleEntryEvaluationIr {
                kind: ModuleEntryEvaluationKindIr::Promise,
                evaluation: Box::new(evaluation),
            }),
        )
    }
}
