use super::generator_identifier_reference::RetainedGeneratorIdentifierTarget;
use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn lower_staged_generator_identifier_assignment(
        &mut self,
        name: String,
        rhs: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        if !matches!(
            self.generator_value_branch_admission(),
            GeneratorValueBranchAdmission::OrdinaryOutsideLoops
        ) {
            return None;
        }
        let mut prefix = Vec::new();
        // Plain assignment evaluates the Reference before the complete RHS,
        // without GetValue or an early TDZ/unresolvable/immutable write check.
        let target = RetainedGeneratorIdentifierTarget::capture_write_only(self, &mut prefix, name);
        let (rhs_prefix, value) = self.lower_staged_generator_expression(rhs)?;
        prefix.extend(rhs_prefix);
        Some((prefix, target.put_value(self, value)))
    }
}
