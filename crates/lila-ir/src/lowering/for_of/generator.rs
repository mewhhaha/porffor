use super::*;

/// The prepared head, iterable and body of a generator-owned synchronous walk.
pub(super) struct GeneratorForOfCandidate {
    pub(super) head: ForOfAssignmentIr,
    pub(super) head_environment: Option<ForInOfEnvironmentIr>,
    pub(super) iterable: TypedExpr,
    pub(super) body: StatementIr,
    pub(super) body_kind: ValueKind,
    pub(super) entry_state: u32,
    pub(super) body_exit_state: u32,
}

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_for_of_body_with_generator_region(
        &mut self,
        for_of: &ForOfLoop,
        entry_state: Option<u32>,
    ) -> (StatementIr, ValueKind, Option<u32>) {
        if let Some(entry_state) = entry_state {
            self.current_generator_resume_state = Some(entry_state + 1);
            self.generator_structured_depth += 1;
        }
        let (body, body_kind) = self.lower_loop_body(for_of.body());
        if entry_state.is_some() {
            self.generator_structured_depth -= 1;
        }
        let body_exit_state = entry_state.map(|_| {
            self.current_generator_resume_state
                .expect("generator body retains state")
        });
        (body, body_kind, body_exit_state)
    }

    pub(super) fn generator_for_of_entry_state(
        &mut self,
        for_of: &ForOfLoop,
    ) -> Result<Option<u32>, ForOfLoweringIr> {
        let entry_state = self.current_generator_resume_state.filter(|_| {
            self.current_resumable_plan.is_none()
                && !for_of.r#await()
                && contains(for_of.body(), ContainsSymbol::YieldExpression)
        });
        if entry_state.is_some()
            && !matches!(
                for_of.initializer(),
                IterableLoopInitializer::Let(Binding::Identifier(_))
                    | IterableLoopInitializer::Const(Binding::Identifier(_))
            )
            && !matches!(for_of.initializer(), IterableLoopInitializer::Var(variable)
                if matches!(variable.binding(), Binding::Identifier(_)))
        {
            self.unsupported("resumable generator for-of requires an identifier binding head");
            return Err(ForOfLoweringIr::no_iteration());
        }
        Ok(entry_state)
    }

    pub(super) fn finish_generator_for_of_iterator(
        &mut self,
        candidate: GeneratorForOfCandidate,
    ) -> ForOfLoweringIr {
        let GeneratorForOfCandidate {
            head,
            head_environment,
            iterable,
            body,
            body_kind,
            entry_state,
            body_exit_state,
        } = candidate;
        let record = IteratorRecordIr::new(
            self.alloc_iterator_slot(),
            self.alloc_next_method_slot(),
            self.alloc_done_slot(),
        );
        let statements = match body {
            StatementIr::Block(block) if block.lexical_environment.is_none() => block.statements,
            StatementIr::LexicalBlock(statements) => statements,
            statement => vec![statement],
        };
        let Some(plan) = GeneratorForOfIteratorPlanIr::new(
            head.clone(),
            record,
            head_environment,
            flatten_suspending_lexical_blocks(statements),
            entry_state,
            body_exit_state,
        ) else {
            self.unsupported("generator for-of continuation state overflow");
            return ForOfLoweringIr::no_iteration();
        };
        if matches!(
            plan.iteration_environment(),
            ResumableLoopIterationEnvironmentIr::StorageOnly
        ) {
            self.add_suspension_owned_binding(head.name);
        }
        self.current_generator_resume_state = Some(plan.exit_state());
        ForOfLoweringIr::new(
            StatementIr::GeneratorForOfIterator { iterable, plan },
            body_kind,
            IteratorProtocolWitness::RESUMABLE_SYNC_ITERATOR_PROTOCOL,
        )
    }
}
