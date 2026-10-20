//! Assignment acquires its RHS before the complete selected pattern plan.
use super::*;

pub(crate) struct GeneratorPatternAssignmentSource<'ast> {
    pattern: &'ast Pattern,
    rhs: &'ast Expression,
}

impl<'ast> GeneratorPatternAssignmentSource<'ast> {
    pub(crate) fn new(
        source: &'ast boa_ast::expression::operator::assign::Assign,
        admission: GeneratorValueBranchAdmission,
    ) -> Option<Self> {
        if source.op() != AssignOp::Assign
            || !matches!(
                admission,
                GeneratorValueBranchAdmission::OrdinaryOutsideLoops
            )
        {
            return None;
        }
        let AssignTarget::Pattern(pattern) = source.lhs() else {
            return None;
        };
        if contains(pattern, ContainsSymbol::AwaitExpression)
            || (!contains(pattern, ContainsSymbol::YieldExpression)
                && !contains(source.rhs(), ContainsSymbol::YieldExpression))
        {
            return None;
        }
        if contains(pattern, ContainsSymbol::YieldExpression) {
            match pattern {
                Pattern::Object(_) => {
                    GeneratorObjectPatternSource::new(pattern)?;
                }
                Pattern::Array(_) => {
                    GeneratorArrayPatternSource::new(pattern)?;
                }
            }
        }
        GeneratorExpressionSourcePlan::new(source.rhs(), admission)?;
        Some(Self {
            pattern,
            rhs: source.rhs(),
        })
    }

    pub(crate) fn rhs(&self) -> &'ast Expression {
        self.rhs
    }

    pub(crate) fn append(
        &self,
        cursor: &mut u32,
        points: &mut Vec<GeneratorSuspensionPointIr>,
    ) -> Option<()> {
        GeneratorExpressionSourcePlan::new(
            self.rhs,
            GeneratorValueBranchAdmission::OrdinaryOutsideLoops,
        )?
        .append(cursor, points)?;
        if contains(self.pattern, ContainsSymbol::YieldExpression) {
            match self.pattern {
                Pattern::Object(_) => {
                    GeneratorObjectPatternSource::append_from_pattern(self.pattern, cursor, points)?
                }
                Pattern::Array(_) => {
                    GeneratorArrayPatternSource::append_from_pattern(self.pattern, cursor, points)?
                }
            }
        }
        Some(())
    }

    pub(crate) fn into_parts(self) -> (&'ast Pattern, &'ast Expression) {
        (self.pattern, self.rhs)
    }
}
