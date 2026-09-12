use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn class_heritage_prototype_get_may_call_user_code(
        &self,
        heritage: &TypedExpr,
    ) -> bool {
        if heritage.possible_kinds != KindSet::from_kind(ValueKind::Function) {
            return true;
        }
        let Some(function_id) = self.resolve_single_function_target(heritage) else {
            return true;
        };
        let function_id = self.original_exact_function_id(&function_id);
        if !self
            .function_signatures
            .get(&function_id)
            .is_some_and(|signature| signature.protocol.is_constructable())
        {
            return true;
        }
        // MakeConstructor gives source/class constructors an own,
        // nonconfigurable data prototype; Array has the same guarantee.
        // Its value can require runtime validation without its Get invoking
        // user code. Generic constructable shapes do not prove this: bound
        // functions and Proxy lack that own-property guarantee.
        !self
            .analysis
            .planned_source_function_ids
            .contains(&function_id)
            && function_id != StandardBuiltinId::ArrayConstructor.function_id()
    }

    pub(super) fn class_evaluation_state(&self) -> Option<u32> {
        self.current_generator_resume_state
            .or(self.current_async_resume_state)
    }

    pub(super) fn empty_class_evaluation_prefix(&self) -> ClassEvaluationPrefixIr {
        let state = self.class_evaluation_state().unwrap_or(0);
        ClassEvaluationPrefixIr::new(Vec::new(), state, state)
    }

    pub(super) fn lower_class_evaluation_operand(
        &mut self,
        expression: &Expression,
    ) -> (ClassEvaluationPrefixIr, TypedExpr) {
        let entry_state = self.class_evaluation_state().unwrap_or(0);
        let staged = if self.current_generator_resume_state.is_some()
            && contains(expression, ContainsSymbol::YieldExpression)
        {
            self.lower_staged_generator_expression(expression)
        } else if self.current_async_resume_state.is_some()
            && contains(expression, ContainsSymbol::AwaitExpression)
        {
            self.lower_async_prefixed_expression(expression)
        } else {
            Some((Vec::new(), self.lower_expression(expression)))
        };
        let (statements, value) = staged.unwrap_or_else(|| {
            (
                Vec::new(),
                self.unsupported_expr("class operand suspension expression"),
            )
        });
        let exit_state = self.class_evaluation_state().unwrap_or(entry_state);
        (
            ClassEvaluationPrefixIr::new(statements, entry_state, exit_state),
            value,
        )
    }

    pub(super) fn lower_class_property_key(
        &mut self,
        expression: &Expression,
        private_environment_id: Option<PrivateEnvironmentId>,
    ) -> Option<(PropertyKeyIr, Option<ClassEvaluationPrefixIr>)> {
        let enclosing_private_environment_id = self.private_environment_id;
        self.private_environment_id = private_environment_id;
        let result = if self.class_evaluation_state().is_some()
            && (contains(expression, ContainsSymbol::YieldExpression)
                || contains(expression, ContainsSymbol::AwaitExpression))
        {
            let (prefix, value) = self.lower_class_evaluation_operand(expression);
            self.record_possible_to_primitive_effects(&value.value_info());
            Some((PropertyKeyIr::StringExpr(Box::new(value)), Some(prefix)))
        } else {
            self.lower_dynamic_object_property_key(expression)
                .map(|key| (key, None))
        };
        self.private_environment_id = enclosing_private_environment_id;
        result
    }

    pub(super) fn finish_class_evaluation(
        &mut self,
        expression: TypedExpr,
        entry_state: Option<u32>,
        heritage_prefix: ClassEvaluationPrefixIr,
        element_prefixes: BTreeMap<usize, ClassEvaluationPrefixIr>,
    ) -> TypedExpr {
        let Some(entry_state) = entry_state else {
            return expression;
        };
        let exit_state = self
            .class_evaluation_state()
            .expect("class evaluation keeps its owner");
        if entry_state == exit_state {
            return expression;
        }
        let class_info = expression.value_info();
        let ExprIr::ClassDefinition(class) = &expression.expr else {
            unreachable!()
        };
        let needs_name_environment = class.name_binding.is_some();
        let constructor_binding =
            self.alloc_suspension_owned_binding("class.constructor.", class_info);
        let name_environment_binding = needs_name_environment.then(|| {
            self.alloc_suspension_owned_binding(
                "class.name.environment.",
                ValueInfo::new(ValueKind::Number),
            )
        });
        let completion_binding =
            self.alloc_suspension_owned_binding("class.completion.", unknown_runtime_value_info());
        let plan = ResumableClassDefinitionIr::new(
            expression,
            constructor_binding.clone(),
            name_environment_binding,
            completion_binding,
            heritage_prefix,
            element_prefixes,
            exit_state,
        );
        self.async_expression_prefix
            .as_mut()
            .expect("resumable class evaluation must be staged by its expression owner")
            .push(StatementIr::ResumableClassDefinition(Box::new(plan)));
        self.lower_identifier_name(constructor_binding, false)
    }
}
