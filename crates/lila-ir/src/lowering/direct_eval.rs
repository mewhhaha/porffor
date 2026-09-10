use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn borrows_direct_eval_variable_environment(&self) -> bool {
        self.current_owner_id == SCRIPT_OWNER_ID
            && !self.analysis.owner_plans[SCRIPT_OWNER_ID].strict
            && matches!(
                &self.analysis.script_instantiation,
                ScriptInstantiation::Prepared(PreparedScriptKind::DirectEval(_))
            )
    }

    pub(super) fn direct_eval_invocation(&self) -> Option<lila_front::EvalInvocationContext> {
        if self.current_owner_id != SCRIPT_OWNER_ID
            && !self
                .analysis
                .function_plans
                .get(&self.current_owner_id)
                .is_some_and(|function| {
                    function
                        .captures
                        .contains_key(DIRECT_EVAL_EXECUTION_CONTEXT_NAME)
                })
        {
            return None;
        }
        match &self.analysis.script_instantiation {
            ScriptInstantiation::Prepared(PreparedScriptKind::DirectEval(context)) => {
                Some(context.invocation())
            }
            _ => None,
        }
    }

    pub(super) fn direct_eval_context(&self) -> DirectEvalContextIr {
        let mut private_names = BTreeMap::new();
        let mut environment_id = self.private_environment_id;
        while let Some(id) = environment_id {
            let environment = &self.analysis.private_environment_plans[&id];
            for (name, private_name) in &environment.bindings {
                private_names.entry(name.clone()).or_insert(*private_name);
            }
            environment_id = environment.parent;
        }
        let mut owner_id = Some(self.current_owner_id.as_str());
        let mut derived_constructor_owner = None;
        while let Some(id) = owner_id {
            let owner = &self.analysis.owner_plans[id];
            if owner.lexical_super_owner_role == LexicalSuperOwnerRole::DerivedConstructorActivation
            {
                derived_constructor_owner = Some(id.to_string());
                break;
            }
            if id == SCRIPT_OWNER_ID {
                if let ScriptInstantiation::Prepared(PreparedScriptKind::DirectEval(context)) =
                    &self.analysis.script_instantiation
                {
                    derived_constructor_owner = context.derived_constructor_owner().cloned();
                }
                break;
            }
            if owner.flavor != FunctionFlavor::Arrow {
                break;
            }
            owner_id = owner.parent_owner_id.as_deref();
        }
        DirectEvalContextIr::new(
            self.reference_strictness(),
            self.analysis.owner_plans[&self.current_owner_id].eval_invocation,
            private_names,
            derived_constructor_owner,
        )
    }

    pub(super) fn register_direct_eval_source(
        &mut self,
        context: &DirectEvalContextIr,
        arguments: &[Expression],
    ) {
        let Some(argument) = arguments.first() else {
            return;
        };
        let mut pending = vec![argument];
        let mut sources = BTreeSet::new();
        while let Some(argument) = pending.pop() {
            match Self::unwrap_parenthesized_expr(argument) {
                Expression::Binary(binary) if binary.op() == BinaryOp::Comma => {
                    pending.push(binary.rhs())
                }
                Expression::Conditional(conditional) => {
                    pending.push(conditional.if_true());
                    pending.push(conditional.if_false());
                }
                Expression::Spread(spread) => {
                    if let Expression::ArrayLiteral(array) =
                        Self::unwrap_parenthesized_expr(spread.target())
                    {
                        if let Some(Some(first)) = array.as_ref().first() {
                            pending.push(first);
                        }
                    }
                }
                argument => {
                    if let Some(source) = self
                        .aot_source_text(argument)
                        .or_else(|| self.static_string_receiver_value(argument))
                    {
                        sources.insert(source);
                    }
                }
            }
        }
        for source in sources {
            self.register_dynamic_script_source(DynamicScriptSource {
                admission: PreparedScriptAdmission::RuntimeCandidate,
                kind: PreparedScriptKind::DirectEval(context.clone()),
                source: source.clone(),
            });
            self.register_dynamic_script_source(DynamicScriptSource {
                admission: PreparedScriptAdmission::RuntimeCandidate,
                kind: PreparedScriptKind::IndirectEval,
                source,
            });
        }
    }
}
