use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn lower_generator_compound_assignment(
        &mut self,
        source: CheckedGeneratorCompoundAssignmentSource<'_>,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        if !matches!(
            self.generator_value_branch_admission(),
            GeneratorValueBranchAdmission::OrdinaryOutsideLoops
        ) {
            return None;
        }
        if source.logical_operation().is_some() {
            return self.lower_generator_logical_assignment(source);
        }
        let (lhs, rhs, operation) = source.into_parts();
        let mut prefix = Vec::new();
        let (reference, old) = self.capture_resumable_assignment_reference(lhs, &mut prefix)?;
        let (rhs_prefix, rhs) = self.lower_staged_generator_expression(rhs)?;
        prefix.extend(rhs_prefix);
        let value = match operation {
            GeneratorCompoundAssignmentOperation::Arithmetic(op) => {
                self.combine_arithmetic(op, old, rhs)
            }
            GeneratorCompoundAssignmentOperation::Bitwise(op) => self.combine_bitwise(op, old, rhs),
            GeneratorCompoundAssignmentOperation::Logical(_) => return None,
        };
        Some((prefix, reference.put_value(self, value)))
    }
}
