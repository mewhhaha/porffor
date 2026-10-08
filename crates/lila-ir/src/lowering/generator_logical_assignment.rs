use super::*;
use crate::generator_loop_control::{GeneratorLoopRegionIr, GeneratorLoopSourceRange};

fn completed_region(
    statements: Vec<StatementIr>,
    range: GeneratorLoopSourceRange,
) -> Option<GeneratorLoopRegionIr> {
    GeneratorLoopRegionIr::new(
        BlockIr {
            statements,
            result_kind: ValueKind::Undefined,
            lexical_environment: None,
        },
        range,
    )
    .ok()
}

impl ScriptLowerer<'_> {
    pub(super) fn lower_generator_logical_assignment(
        &mut self,
        source: CheckedGeneratorCompoundAssignmentSource<'_>,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let operation = source.logical_operation()?;
        let mut prefix = Vec::new();
        let (reference, old) =
            self.capture_resumable_assignment_reference(source.lhs(), &mut prefix)?;
        let states = source.logical_states(self.plain_generator_entry_state()?)?;
        let (_, rhs, _) = source.into_parts();
        let condition = match operation {
            LogicalBinaryOp::And => old.clone(),
            LogicalBinaryOp::Or => TypedExpr::from_info(
                ValueInfo::new(ValueKind::Boolean),
                ExprIr::LogicalNot {
                    expr: Box::new(old.clone()),
                },
            ),
            LogicalBinaryOp::Coalesce => generator_value_branch::nullish_condition(old.clone()),
        };
        let result_name = self.alloc_suspension_owned_binding(
            "generator.logical.assignment.result.",
            unknown_runtime_value_info(),
        );
        prefix.push(StatementIr::Lexical {
            mode: BindingMode::Let,
            name: result_name.clone(),
            init: old,
        });
        let before = self.capture_conditional_flow_facts();
        self.current_generator_resume_state = Some(states.selected.entry);
        // The selected arm owns its generated operands. Keep the Reference's
        // PutValue and result publication inside that scope, then join only the
        // original surrounding bindings with the skipped arm.
        self.push_scope();
        let selected = (|| {
            let (mut selected, rhs) = self.lower_staged_generator_expression(rhs)?;
            if self.plain_generator_entry_state()? != states.selected.end {
                return None;
            }
            let skipped_value = reference.release_operation();
            let value = reference.put_value(self, rhs);
            selected.push(StatementIr::Expression(
                self.lower_identifier_assign_value(result_name.clone(), value),
            ));
            Some((selected, skipped_value))
        })();
        self.pop_scope();
        let (selected, skipped_value) = selected?;
        let selected_facts = self.capture_conditional_flow_facts();
        self.install_conditional_flow_facts(before);
        self.current_generator_resume_state = Some(states.skipped.entry);
        // A skipped logical assignment does not call SetMutableBinding. Retire
        // the private Reference and keep the original whole GetValue result.
        let skipped = skipped_value
            .into_iter()
            .map(|(name, operation)| {
                StatementIr::Expression(self.environment_identifier(name, operation))
            })
            .collect();
        let skipped_facts = self.capture_conditional_flow_facts();
        self.merge_conditional_flow_facts(selected_facts, skipped_facts);
        let branch = OrdinaryGeneratorIfIr::new(
            condition,
            states.entry,
            completed_region(selected, states.selected)?,
            completed_region(skipped, states.skipped)?,
            states.exit,
        )
        .ok()?;
        prefix.push(StatementIr::OrdinaryGeneratorIf(Box::new(branch)));
        self.current_generator_resume_state = Some(states.exit);
        self.set_binding_value_info(&result_name, unknown_runtime_value_info());
        Some((
            prefix,
            TypedExpr::from_info(
                unknown_runtime_value_info(),
                ExprIr::Identifier(result_name),
            ),
        ))
    }
}
