//! Checked named records decide generator branch ownership and plan metadata.

use super::*;

impl ScriptLowerer<'_> {
    pub(in crate::lowering) fn generator_if_branch_owns_runtime_environment(
        &self,
        source: &If,
    ) -> bool {
        [Some(source.body()), source.else_node()]
            .into_iter()
            .flatten()
            .any(|branch| {
                let Statement::Block(block) = branch else {
                    return false;
                };
                contains(block, ContainsSymbol::YieldExpression)
                    && self
                        .lower_materialized_lexical_environment(
                            self.analysis
                                .block_environment_ids
                                .get(&(block as *const Block as usize))
                                .copied(),
                        )
                        .is_some()
            })
    }

    pub(in crate::lowering) fn generator_plan_for_lowered_named_environment(
        &self,
        body: &BlockIr,
        source_plan: GeneratorPlanIr,
    ) -> GeneratorPlanIr {
        if !self.uses_runtime_identifier_environment() {
            return source_plan;
        }
        // Source admission still validates the source shape. The finalized
        // checked continuation regions own any additional environment phases.
        let Ok(final_state) = crate::generator_loop_control::sequence_end(&body.statements, 0)
        else {
            return source_plan;
        };
        let mut points = Vec::new();
        crate::generator_loop_control::collect_suspensions(&body.statements, &mut points);
        finish_generator_plan(final_state, points)
            .expect("checked continuation ranges cannot overflow their exit state")
    }
}
