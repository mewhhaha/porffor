//! Plain Async With retains its original analyzed Object Environment Record.

use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn lower_plain_async_with(
        &mut self,
        with: &boa_ast::statement::With,
    ) -> (StatementIr, ValueKind) {
        match self.lower_checked_plain_async_with(with) {
            Some((plan, kind)) => (StatementIr::AsyncFunctionWith(Box::new(plan)), kind),
            None => {
                self.unsupported("plain async With differs from its complete source and original environment plan");
                (StatementIr::Empty, ValueKind::Undefined)
            }
        }
    }

    fn lower_checked_plain_async_with(
        &mut self,
        with: &boa_ast::statement::With,
    ) -> Option<(AsyncFunctionWithIr, ValueKind)> {
        let source = AsyncWithSource::new(with)?;
        let states = source.states(self.plain_async_entry_state()?)?;
        let previous = std::mem::replace(
            &mut self.async_value_branch_context,
            AsyncValueBranchContext::WithHead {
                loop_depth: self.loop_depth,
            },
        );
        let expression = source.source().expression();
        let staged = self.lower_async_prefixed_expression(expression);
        let (mut head, raw) = match staged {
            Some(staged) => staged,
            None if !contains(expression, ContainsSymbol::AwaitExpression)
                && !contains(expression, ContainsSymbol::YieldExpression) =>
            {
                (Vec::new(), self.lower_expression(expression))
            }
            None => {
                self.async_value_branch_context = previous;
                return None;
            }
        };
        self.async_value_branch_context = previous;
        if self.plain_async_entry_state()? != states.head_ready() {
            return None;
        }
        // One completed ToObject publication precedes creation of this record.
        let object = TypedExpr::spec_to_object(raw);
        let object_info = object.value_info();
        let head_name =
            self.alloc_suspension_owned_binding("async.with.head.", object_info.clone());
        let head_binding = self
            .generated_owned_env_bindings
            .iter()
            .find(|binding| binding.name == head_name)?
            .clone();
        head.push(StatementIr::Lexical {
            mode: BindingMode::Let,
            name: head_name.clone(),
            init: object,
        });
        let head_value = TypedExpr::from_info(object_info.clone(), ExprIr::Identifier(head_name));
        let head = BlockIr {
            statements: head,
            result_kind: ValueKind::Undefined,
            lexical_environment: None,
        };
        let environment_id = *self
            .analysis
            .with_environment_ids
            .get(&(with as *const boa_ast::statement::With as usize))?;
        let lexical_environment = self.lower_runtime_lexical_environment(Some(environment_id))?;
        let binding_name = self
            .analysis
            .with_object_environment_plans
            .get(&environment_id)?
            .binding_name
            .clone();
        let object_binding = lexical_environment
            .bindings
            .iter()
            .find(|binding| binding.name == binding_name.as_str())?
            .clone();
        let with_object = ObjectEnvironmentBindingObject::materialized(&binding_name, object_info);
        self.current_async_resume_state = Some(states.body_entry());
        self.with_environment_chain.enter_current(
            with_object,
            CurrentScopeDepth::at_with_entry(self.scopes.len()),
        );
        let previous = std::mem::replace(
            &mut self.async_value_branch_context,
            AsyncValueBranchContext::WithBody {
                loop_depth: self.loop_depth,
            },
        );
        self.plain_async_with_depth += 1;
        let (statement, kind) = self.lower_statement(source.source().statement());
        self.plain_async_with_depth -= 1;
        self.async_value_branch_context = previous;
        self.with_environment_chain.leave_current();
        if self.plain_async_entry_state()? != states.body_end() {
            return None;
        }
        let body = BlockIr {
            statements: vec![statement],
            result_kind: kind,
            lexical_environment: None,
        };
        let plan = AsyncFunctionWithIr::new(
            states,
            head,
            head_value,
            head_binding,
            object_binding,
            lexical_environment,
            body,
            &self.generated_owned_env_bindings,
        )
        .ok()?;
        self.current_async_resume_state = Some(plan.exit_state());
        Some((plan, kind))
    }
}
