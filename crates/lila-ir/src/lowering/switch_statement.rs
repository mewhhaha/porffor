use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_switch(&mut self, switch: &AstSwitch) -> (StatementIr, ValueKind) {
        if self.async_generator_entry_state().is_some() {
            return self.lower_async_generator_switch(switch);
        }
        if crate::async_generator_source::switch_has_direct_resources(switch) {
            if self.plain_async_entry_state().is_some() {
                return self
                    .lower_complete_resource_switch(switch, ResumableRegionProtocolIr::Async);
            }
            if self.plain_generator_entry_state().is_some() {
                return self
                    .lower_complete_resource_switch(switch, ResumableRegionProtocolIr::Generator);
            }
        }
        self.lower_switch_legacy(switch)
    }

    fn lower_switch_legacy(&mut self, switch: &AstSwitch) -> (StatementIr, ValueKind) {
        if self.plain_async_entry_state().is_some() {
            return self.lower_async_function_switch(switch);
        }
        if self.plain_generator_entry_state().is_some()
            && matches!(
                self.generator_value_branch_admission(),
                GeneratorValueBranchAdmission::OrdinaryOutsideLoops
            )
            && (contains(switch, ContainsSymbol::YieldExpression)
                || contains_ordinary_generator_phase_owner(switch))
        {
            return self.lower_ordinary_generator_switch(switch);
        }
        let discriminant = self.lower_expression(switch.val());
        let before_vars = self.var_bindings.clone();
        let before_globals = self.global_properties.clone();
        self.breakable_depth += 1;
        // 14.12.4 CaseBlockEvaluation pushes the CaseBlock's Environment Record
        // and instantiates its LexicallyScopedDeclarations once for the whole
        // block, not per case, so one token map is shared by every case body
        // below. The push is the constructor's; the pop is `scope.finish`.
        let mut scope = LexicalScopeInstantiation::instantiate_switch(self, switch);

        let mut last_function_by_name = BTreeMap::new();
        for case in switch.cases() {
            for item in case.body().statements() {
                let Some(function) = statement_list_item_function_declaration(item) else {
                    continue;
                };
                last_function_by_name
                    .insert(function_name(self.interner, function, None), function);
            }
        }
        let lexical_declarations = last_function_by_name
            .into_values()
            .map(|function| self.lower_function_declaration(function))
            .collect::<Vec<_>>();

        let case_conditions = switch
            .cases()
            .iter()
            .map(|case| {
                self.var_bindings = before_vars.clone();
                self.global_properties = before_globals.clone();
                (
                    case.condition().map(|expr| self.lower_expression(expr)),
                    self.var_bindings.clone(),
                    self.global_properties.clone(),
                )
            })
            .collect::<Vec<_>>();

        let mut cases = Vec::with_capacity(switch.cases().len());
        let mut result_kind: Option<ValueKind> = None;
        let mut merged_vars = before_vars.clone();
        let mut merged_globals = before_globals.clone();

        for (case, (condition, case_vars, case_globals)) in
            switch.cases().iter().zip(case_conditions.into_iter())
        {
            self.var_bindings = case_vars;
            self.global_properties = case_globals;
            let body = self.lower_statement_items_without_function_initialization(
                case.body().statements(),
                &mut scope,
            );
            merged_vars = self.merge_var_bindings(&merged_vars, &self.var_bindings);
            merged_globals = self.merge_global_properties(&merged_globals, &self.global_properties);
            if let Some(kind) = result_kind {
                if kind != body.result_kind {
                    result_kind = Some(ValueKind::Undefined);
                }
            } else {
                result_kind = Some(body.result_kind);
            }
            cases.push(SwitchCaseIr { condition, body });
        }

        self.breakable_depth -= 1;
        scope.finish(self);
        self.var_bindings = merged_vars;
        self.global_properties = merged_globals;

        let statement = StatementIr::Switch {
            discriminant,
            lexical_environment: self.lower_materialized_lexical_environment(
                self.analysis
                    .switch_environment_ids
                    .get(&(switch as *const AstSwitch as usize))
                    .copied(),
            ),
            lexical_declarations,
            cases,
        };
        // Case selection and switch breaks have no continuation-state owner.
        // A child's exit cannot represent a break that bypasses that child.
        if crate::ir::statement_contains_async_while_condition(&statement) {
            self.unsupported("awaited while condition inside a switch case");
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        (statement, result_kind.unwrap_or(ValueKind::Undefined))
    }
}
