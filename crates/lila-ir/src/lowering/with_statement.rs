use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_with_statement(
        &mut self,
        with: &boa_ast::statement::With,
    ) -> (StatementIr, ValueKind) {
        let environment_id = self
            .analysis
            .with_environment_ids
            .get(&(with as *const boa_ast::statement::With as usize))
            .copied()
            .expect("with statement must have an analyzed Object Environment Record");
        let owned_env_slots = self
            .analysis
            .environment_plans
            .get(&environment_id)
            .expect("with environment must be planned")
            .owned_env_slots
            .clone();
        let binding_name = self
            .analysis
            .with_object_environment_plans
            .get(&environment_id)
            .expect("with environment must carry its hidden binding plan")
            .binding_name
            .clone();
        // The source expression belongs to the outer environment. Only after
        // it has been lowered do we enter/materialize the WithObject record.
        let object = self.lower_expression(with.expression());
        let object_info = object.value_info();
        if !object
            .possible_kinds
            .is_subset_of(Self::object_like_kind_set().union(KindSet::all_runtime_tags()))
        {
            self.unsupported("with object expression");
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        let crosses_suspension = (self.current_generator_resume_state.is_some()
            && contains(with.statement(), ContainsSymbol::YieldExpression))
            || (self.current_async_resume_state.is_some()
                && contains(with.statement(), ContainsSymbol::AwaitExpression));
        let resumable_owner = self
            .current_function_id
            .as_ref()
            .and_then(|function_id| self.analysis.function_plans.get(function_id))
            .is_some_and(|function| {
                function.protocol.execution_kind() != FunctionExecutionKind::Ordinary
            });
        if resumable_owner && !owned_env_slots.is_empty() {
            self.unsupported("resumable captured with Object Environment Record");
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        let object_value_name = self.alloc_temp_binding_name("with.object.value.");
        let object_value = TypedExpr::from_info(
            object_info.clone(),
            ExprIr::Identifier(object_value_name.clone()),
        );
        let object_name = binding_name.as_str().to_string();
        if crosses_suspension {
            self.add_suspension_owned_binding(object_name.clone());
        }
        let with_object = ObjectEnvironmentBindingObject::materialized(&binding_name, object_info);
        self.with_environment_chain.enter_current(
            with_object,
            CurrentScopeDepth::at_with_entry(self.scopes.len()),
        );
        let lowered = self.lower_statement(with.statement());
        self.with_environment_chain.leave_current();

        let body = StatementIr::Block(BlockIr {
            statements: vec![
                StatementIr::Lexical {
                    mode: BindingMode::Let,
                    name: object_name,
                    init: object_value,
                },
                lowered.0,
            ],
            result_kind: lowered.1,
            lexical_environment: (!owned_env_slots.is_empty()).then(|| LexicalEnvironmentIr {
                bindings: owned_env_slots
                    .iter()
                    .map(|(name, slot)| OwnedEnvBindingIr {
                        name: name.clone(),
                        slot: *slot,
                    })
                    .collect(),
            }),
        });
        (
            StatementIr::LexicalBlock(vec![
                StatementIr::Lexical {
                    mode: BindingMode::Let,
                    name: object_value_name,
                    init: object,
                },
                body,
            ]),
            lowered.1,
        )
    }
}
