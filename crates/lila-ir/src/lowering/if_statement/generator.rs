use super::*;

impl<'a> ScriptLowerer<'a> {
    /// Complete the legacy one-yield conditional plan. The structured path
    /// already owns its validated branches and returns before this fallback.
    pub(super) fn finish_simple_generator_if(
        &mut self,
        condition: &TypedExpr,
        then_branch: &StatementIr,
        else_branch: Option<&StatementIr>,
        result_kind: ValueKind,
        entry_state: u32,
    ) -> Option<(StatementIr, ValueKind)> {
        let (then_before_yield, then_yield_statement, then_after_yield) =
            Self::split_generator_if_branch(then_branch.clone());
        let (else_before_yield, else_yield_statement, else_after_yield) = else_branch
            .cloned()
            .map(Self::split_generator_if_branch)
            .unwrap_or_default();
        if then_yield_statement.is_none() && else_yield_statement.is_none() {
            return None;
        }
        let then_resume_state = then_yield_statement.as_ref().and_then(|statement| {
            let StatementIr::GeneratorYield { resume_state, .. } = statement else {
                return None;
            };
            Some(*resume_state)
        });
        let else_resume_state = else_yield_statement.as_ref().and_then(|statement| {
            let StatementIr::GeneratorYield { resume_state, .. } = statement else {
                return None;
            };
            Some(*resume_state)
        });
        let exit_state = self.current_generator_resume_state.unwrap_or(entry_state) + 1;
        if let Some(plan) = self.current_resumable_plan.as_mut() {
            for suspension in plan
                .suspension_points
                .iter_mut()
                .skip(self.next_resumable_suspension_index)
            {
                suspension.suspend_state += 1;
                suspension.resume_state += 1;
            }
            plan.state_count += 1;
            self.current_async_resume_state = Some(exit_state);
        }
        self.current_generator_resume_state = Some(exit_state);
        Some((
            StatementIr::GeneratorIf {
                condition: condition.clone(),
                then_before_yield,
                then_yield_statement: then_yield_statement.map(Box::new),
                then_after_yield,
                else_before_yield,
                else_yield_statement: else_yield_statement.map(Box::new),
                else_after_yield,
                entry_state,
                then_resume_state,
                else_resume_state,
                exit_state,
            },
            result_kind,
        ))
    }
}
