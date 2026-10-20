//! Binding initialization consumes the RHS and then its selected pattern plan.
use super::*;

pub(crate) struct GeneratorPatternInitializerSource<'ast> {
    pattern: &'ast Pattern,
    initializer: &'ast Expression,
}

impl<'ast> GeneratorPatternInitializerSource<'ast> {
    pub(crate) fn new(
        variable: &'ast Variable,
        admission: GeneratorValueBranchAdmission,
    ) -> Option<Self> {
        if !matches!(
            admission,
            GeneratorValueBranchAdmission::OrdinaryOutsideLoops
        ) {
            return None;
        }
        let Binding::Pattern(pattern) = variable.binding() else {
            return None;
        };
        if contains(variable.binding(), ContainsSymbol::AwaitExpression) {
            return None;
        }
        let initializer = variable.init()?;
        if !contains(initializer, ContainsSymbol::YieldExpression)
            && !contains(pattern, ContainsSymbol::YieldExpression)
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
        GeneratorExpressionSourcePlan::new(initializer, admission)?;
        Some(Self {
            pattern,
            initializer,
        })
    }

    pub(crate) fn append(
        &self,
        cursor: &mut u32,
        points: &mut Vec<GeneratorSuspensionPointIr>,
    ) -> Option<()> {
        GeneratorExpressionSourcePlan::new(
            self.initializer,
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
        (self.pattern, self.initializer)
    }
}
